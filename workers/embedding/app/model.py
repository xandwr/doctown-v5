"""Embedding model using ONNX Runtime."""

import json
import logging
import time
from pathlib import Path
from typing import List

import numpy as np
import onnxruntime as ort
from transformers import AutoTokenizer

from .config import settings

logger = logging.getLogger(__name__)


class EmbeddingModel:
    """ONNX-based embedding model for CPU inference."""
    
    def __init__(self, model_path: str):
        """Initialize the model.
        
        Args:
            model_path: Path to directory containing model.onnx and tokenizer.json
        """
        self.model_path = Path(model_path)
        self.session = None
        self.tokenizer = None
        self.embedding_dim = settings.embedding_dim
        
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
        sess_options.intra_op_num_threads = settings.onnx_threads
        sess_options.inter_op_num_threads = settings.onnx_threads
        sess_options.graph_optimization_level = ort.GraphOptimizationLevel.ORT_ENABLE_ALL
        sess_options.execution_mode = ort.ExecutionMode.ORT_PARALLEL
        
        # Enable memory pattern optimization
        sess_options.enable_mem_pattern = True
        sess_options.enable_cpu_mem_arena = True
        
        self.session = ort.InferenceSession(
            str(model_file),
            sess_options=sess_options,
            providers=['CPUExecutionProvider']
        )
        logger.info(f"ONNX model loaded with {settings.onnx_threads} threads")
        
    def warmup(self):
        """Warm up the model with a dummy inference."""
        if self.session is None or self.tokenizer is None:
            raise RuntimeError("Model not loaded. Call load() first.")
        
        logger.info("Warming up model...")
        dummy_text = "This is a warmup text to initialize the model."
        _ = self.embed([dummy_text])
        logger.info("Model warmup complete")
    
    def embed(self, texts: List[str]) -> np.ndarray:
        """Embed a batch of texts.
        
        Args:
            texts: List of text strings to embed
            
        Returns:
            numpy array of shape (len(texts), embedding_dim)
        """
        if self.session is None or self.tokenizer is None:
            raise RuntimeError("Model not loaded. Call load() first.")
        
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
