"""Configuration for the embedding worker."""

import os

try:
    from pydantic_settings import BaseSettings
except ImportError:
    from pydantic import BaseSettings


class Settings(BaseSettings):
    """Application settings."""
    
    # Server
    host: str = "0.0.0.0"
    port: int = 8000
    
    # Model
    model_path: str = "../../models/minilm-l6"
    embedding_dim: int = 384
    
    # Batching - small batches optimized for CPU sequential processing
    min_batch_size: int = 1
    max_batch_size: int = 8
    batch_timeout_ms: int = 100
    
    # Memory management
    max_memory_percent: float = 70.0  # Don't exceed 70% of available RAM
    adaptive_batching: bool = True  # Adjust batch size based on memory
    
    # ONNX Runtime - optimized for CPU
    onnx_threads: int = min(os.cpu_count() or 4, 8)  # Cap at 8 threads to avoid thrashing
    
    class Config:
        env_prefix = "EMBEDDING_"


settings = Settings()
