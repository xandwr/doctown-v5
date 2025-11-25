#!/usr/bin/env python3
"""
RunPod Serverless Handler for GPU-Accelerated Embeddings

This handler wraps the ONNX embedding model for serverless GPU inference.
Designed to be called by the Builder serverless worker.
"""

import os
import sys
import time
import logging
from typing import Any, Dict, List

import runpod

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)

# Global model reference (loaded once, reused across invocations)
_model = None


def get_model():
    """Get or load the embedding model (singleton pattern for serverless)."""
    global _model
    if _model is None:
        logger.info("Loading embedding model...")
        from app.model import EmbeddingModel
        _model = EmbeddingModel()
        logger.info("Model loaded successfully")
    return _model


def handler(job: Dict[str, Any]) -> Dict[str, Any]:
    """
    RunPod serverless handler for embedding chunks.
    
    Expected input:
    {
        "batch_id": "batch_001",
        "chunks": [
            {"chunk_id": "c1", "content": "function hello() { ... }"},
            {"chunk_id": "c2", "content": "class Parser { ... }"}
        ]
    }
    
    Returns:
    {
        "batch_id": "batch_001",
        "vectors": [
            {"chunk_id": "c1", "vector": [0.1, 0.2, ...]},
            {"chunk_id": "c2", "vector": [0.3, 0.4, ...]}
        ],
        "duration_ms": 123
    }
    """
    start_time = time.perf_counter()
    
    # Validate input
    if "input" not in job:
        return {"error": "Missing 'input' field"}
    
    input_data = job["input"]
    batch_id = input_data.get("batch_id", "unknown")
    chunks = input_data.get("chunks", [])
    
    if not chunks:
        return {
            "batch_id": batch_id,
            "vectors": [],
            "duration_ms": 0
        }
    
    logger.info(f"Processing batch {batch_id} with {len(chunks)} chunks")
    
    try:
        # Load model (cached after first call)
        model = get_model()
        
        # Extract texts
        texts = [chunk["content"] for chunk in chunks]
        chunk_ids = [chunk["chunk_id"] for chunk in chunks]
        
        # Generate embeddings
        embeddings = model.embed(texts)
        
        # Build response
        vectors = [
            {
                "chunk_id": chunk_id,
                "vector": embedding.tolist()
            }
            for chunk_id, embedding in zip(chunk_ids, embeddings)
        ]
        
        duration_ms = (time.perf_counter() - start_time) * 1000
        
        logger.info(
            f"Embedded batch {batch_id}: {len(chunks)} chunks "
            f"in {duration_ms:.2f}ms ({len(chunks) / (duration_ms / 1000):.1f} chunks/sec)"
        )
        
        return {
            "batch_id": batch_id,
            "vectors": vectors,
            "duration_ms": duration_ms
        }
        
    except Exception as e:
        logger.error(f"Error embedding batch {batch_id}: {e}")
        return {
            "error": str(e),
            "batch_id": batch_id
        }


# Warm up the model on cold start
logger.info("Warming up model on cold start...")
try:
    model = get_model()
    # Run a test embedding to warm up CUDA/ONNX
    _ = model.embed(["warmup test"])
    logger.info("Model warmup complete")
except Exception as e:
    logger.error(f"Model warmup failed: {e}")

# Start the serverless handler
if __name__ == "__main__":
    logger.info("Starting RunPod serverless handler for embeddings")
    runpod.serverless.start({"handler": handler})
