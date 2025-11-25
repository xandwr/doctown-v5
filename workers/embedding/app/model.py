"""Embedding model using ONNX Runtime with memory-aware batching.

Supports both CPU and GPU inference via ONNX Runtime providers.
"""

import gc
import json
import logging
import os
import time
from pathlib import Path
from typing import List, Tuple

import numpy as np
import onnxruntime as ort
import psutil
from transformers import AutoTokenizer

from .config import settings

logger = logging.getLogger(__name__)

# Check for GPU support
ONNX_USE_GPU = os.environ.get("ONNX_USE_GPU", "false").lower() in ("true", "1", "yes")
AVAILABLE_PROVIDERS = ort.get_available_providers()
HAS_CUDA = "CUDAExecutionProvider" in AVAILABLE_PROVIDERS


class EmbeddingModel:
    """ONNX-based embedding model with intelligent memory management for CPU inference."""
    
    def __init__(self, model_path: str):
        """Initialize the model.
        
        Args:
            model_path: Path to directory containing model.onnx and tokenizer.json
        """
        self.model_path = Path(model_path)
        self.session = None
        self.tokenizer = None
        self.embedding_dim = settings.embedding_dim
        self.process = psutil.Process()
        self.max_memory_bytes = psutil.virtual_memory().total * (settings.max_memory_percent / 100.0)
        self.current_batch_size = settings.min_batch_size
        logger.info(f"Memory limit set to {self.max_memory_bytes / (1024**3):.2f} GB ({settings.max_memory_percent}% of system RAM)")
        
    def load(self):
        """Load the ONNX model and tokenizer."""
        logger.info(f"Loading model from {self.model_path}")
        
        # Load tokenizer
        tokenizer_path = self.model_path / "tokenizer.json"
        if not tokenizer_path.exists():
            raise FileNotFoundError(f"Tokenizer not found at {tokenizer_path}")
        
        # Use the sentence-transformers model name for compatibility
        self.tokenizer = AutoTokenizer.from_pretrained(
            "sentence-transformers/all-MiniLM-L6-v2"
        )
        logger.info("Tokenizer loaded")
        
        # Load ONNX model
        model_file = self.model_path / "model.onnx"
        if not model_file.exists():
            raise FileNotFoundError(f"Model not found at {model_file}")
        
        sess_options = ort.SessionOptions()
        sess_options.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL
        
        # Determine execution providers
        if ONNX_USE_GPU and HAS_CUDA:
            # GPU mode - use CUDA with fallback to CPU
            providers = [
                ('CUDAExecutionProvider', {
                    'device_id': 0,
                    'arena_extend_strategy': 'kNextPowerOfTwo',
                    'gpu_mem_limit': 2 * 1024 * 1024 * 1024,  # 2GB limit
                    'cudnn_conv_algo_search': 'EXHAUSTIVE',
                    'do_copy_in_default_stream': True,
                }),
                'CPUExecutionProvider'
            ]
            logger.info("Using CUDA GPU for inference")
        else:
            # CPU mode - optimize for sequential processing
            sess_options.intra_op_num_threads = settings.onnx_threads
            sess_options.inter_op_num_threads = settings.onnx_threads
            sess_options.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
            sess_options.enable_mem_pattern = True
            sess_options.enable_cpu_mem_arena = True
            providers = ['CPUExecutionProvider']
            logger.info(f"Using CPU for inference with {settings.onnx_threads} threads")
        
        self.session = ort.InferenceSession(
            str(model_file),
            sess_options=sess_options,
            providers=providers
        )
        
        # Log actual providers being used
        actual_providers = self.session.get_providers()
        logger.info(f"ONNX model loaded with providers: {actual_providers}")
        
    def warmup(self):
        """Warm up the model with a dummy inference."""
        if self.session is None or self.tokenizer is None:
            raise RuntimeError("Model not loaded. Call load() first.")
        
        logger.info("Warming up model...")
        dummy_text = "This is a warmup text to initialize the model."
        _ = self._embed_batch([dummy_text])
        logger.info("Model warmup complete")
    
    def _check_memory(self) -> Tuple[float, bool]:
        """Check current memory usage.
        
        Returns:
            (current_usage_bytes, is_safe)
        """
        mem_info = self.process.memory_info()
        current_usage = mem_info.rss
        is_safe = current_usage < self.max_memory_bytes
        return current_usage, is_safe
    
    def _adjust_batch_size(self, success: bool):
        """Adjust batch size based on success/failure.
        
        Args:
            success: Whether the last batch succeeded without memory issues
        """
        if not settings.adaptive_batching:
            return
        
        if success and self.current_batch_size < settings.max_batch_size:
            # Gradually increase batch size
            self.current_batch_size = min(
                self.current_batch_size + 4,
                settings.max_batch_size
            )
            logger.debug(f"Increased batch size to {self.current_batch_size}")
        elif not success and self.current_batch_size > settings.min_batch_size:
            # Reduce batch size on memory pressure
            self.current_batch_size = max(
                self.current_batch_size // 2,
                settings.min_batch_size
            )
            logger.warning(f"Reduced batch size to {self.current_batch_size} due to memory pressure")
    
    def _embed_batch(self, texts: List[str]) -> np.ndarray:
        """Embed a single batch of texts (internal method).
        
        Args:
            texts: List of text strings to embed
            
        Returns:
            numpy array of shape (len(texts), embedding_dim)
        """
        
        # Tokenize with optimized settings
        encoded = self.tokenizer(
            texts,
            padding=True,
            truncation=True,
            max_length=512,
            return_tensors="np",
            return_token_type_ids=False  # Don't generate if not needed
        )
        
        # Prepare inputs - avoid unnecessary copies
        input_ids = encoded["input_ids"]
        attention_mask = encoded["attention_mask"]
        
        # Convert to int64 in-place if needed
        if input_ids.dtype != np.int64:
            input_ids = input_ids.astype(np.int64)
        if attention_mask.dtype != np.int64:
            attention_mask = attention_mask.astype(np.int64)
        
        ort_inputs = {
            "input_ids": input_ids,
            "attention_mask": attention_mask,
        }
        
        # Add token_type_ids if the model expects it
        if "token_type_ids" in [inp.name for inp in self.session.get_inputs()]:
            ort_inputs["token_type_ids"] = np.zeros_like(input_ids, dtype=np.int64)
        
        # Run inference
        outputs = self.session.run(None, ort_inputs)
        
        # Get embeddings from the last hidden state
        # Shape: (batch_size, seq_len, hidden_dim)
        last_hidden_state = outputs[0]
        
        # Optimized mean pooling - avoid expand_dims
        attention_mask_float = attention_mask.astype(np.float32)
        attention_mask_expanded = attention_mask_float[:, :, np.newaxis]
        
        # Sum embeddings weighted by attention mask
        sum_embeddings = np.sum(last_hidden_state * attention_mask_expanded, axis=1)
        sum_mask = np.maximum(attention_mask_expanded.sum(axis=1), 1e-9)
        embeddings = sum_embeddings / sum_mask
        
        # L2 normalize - optimized
        norms = np.sqrt(np.sum(embeddings ** 2, axis=1, keepdims=True))
        embeddings = embeddings / np.maximum(norms, 1e-9)
        
        return embeddings
    
    def embed(self, texts: List[str]) -> np.ndarray:
        """Embed texts with intelligent chunking to prevent memory exhaustion.
        
        For large batches, this splits them into smaller sub-batches and processes
        them sequentially, with memory monitoring and garbage collection between batches.
        
        Args:
            texts: List of text strings to embed
            
        Returns:
            numpy array of shape (len(texts), embedding_dim)
        """
        if self.session is None or self.tokenizer is None:
            raise RuntimeError("Model not loaded. Call load() first.")
        
        total_texts = len(texts)
        
        # For small batches, just process directly
        if total_texts <= self.current_batch_size:
            try:
                current_mem, is_safe = self._check_memory()
                if not is_safe:
                    logger.warning(f"Memory usage high ({current_mem / (1024**3):.2f} GB) before processing")
                    gc.collect()
                
                result = self._embed_batch(texts)
                self._adjust_batch_size(success=True)
                return result
            except Exception as e:
                logger.error(f"Failed to embed batch: {e}")
                self._adjust_batch_size(success=False)
                # Try again with smaller batch if possible
                if self.current_batch_size > settings.min_batch_size and total_texts > settings.min_batch_size:
                    logger.info("Retrying with reduced batch size...")
                    return self.embed(texts)
                raise
        
        # For large batches, process in chunks
        logger.info(f"Processing {total_texts} texts in chunks of {self.current_batch_size}")
        all_embeddings = []
        
        for i in range(0, total_texts, self.current_batch_size):
            chunk = texts[i:i + self.current_batch_size]
            chunk_num = i // self.current_batch_size + 1
            total_chunks = (total_texts + self.current_batch_size - 1) // self.current_batch_size
            
            try:
                # Check memory before processing
                current_mem, is_safe = self._check_memory()
                if not is_safe:
                    logger.warning(f"Memory pressure detected ({current_mem / (1024**3):.2f} GB), forcing GC")
                    gc.collect()
                    # Check again after GC
                    current_mem, is_safe = self._check_memory()
                    if not is_safe:
                        # Reduce batch size and retry this chunk
                        self._adjust_batch_size(success=False)
                        logger.warning(f"Retrying chunk {chunk_num}/{total_chunks} with smaller batch size")
                        # Recursively process this chunk with new batch size
                        chunk_embeddings = self.embed(chunk)
                        all_embeddings.append(chunk_embeddings)
                        continue
                
                logger.debug(f"Processing chunk {chunk_num}/{total_chunks} ({len(chunk)} texts)")
                chunk_embeddings = self._embed_batch(chunk)
                all_embeddings.append(chunk_embeddings)
                
                # Aggressive memory management
                if chunk_num % 10 == 0:  # GC every 10 chunks
                    gc.collect()
                    current_mem, _ = self._check_memory()
                    logger.debug(f"Memory after chunk {chunk_num}: {current_mem / (1024**3):.2f} GB")
                
                self._adjust_batch_size(success=True)
                
            except Exception as e:
                logger.error(f"Failed to process chunk {chunk_num}/{total_chunks}: {e}")
                self._adjust_batch_size(success=False)
                gc.collect()
                # Try to continue with remaining chunks if possible
                raise
        
        # Concatenate all embeddings
        logger.info(f"Successfully processed {total_texts} texts in {len(all_embeddings)} chunks")
        result = np.vstack(all_embeddings)
        
        # Final cleanup
        gc.collect()
        
        return result
    
    def embed_single(self, text: str) -> np.ndarray:
        """Embed a single text.
        
        Args:
            text: Text string to embed
            
        Returns:
            numpy array of shape (embedding_dim,)
        """
        embeddings = self.embed([text])
        return embeddings[0]


# Global model instance
_model_instance = None


def get_model() -> EmbeddingModel:
    """Get the global model instance."""
    global _model_instance
    if _model_instance is None:
        _model_instance = EmbeddingModel(settings.model_path)
        _model_instance.load()
        _model_instance.warmup()
    return _model_instance
