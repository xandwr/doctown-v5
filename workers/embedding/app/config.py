"""Configuration for the embedding worker."""

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
    
    # Batching
    min_batch_size: int = 16
    max_batch_size: int = 256
    batch_timeout_ms: int = 500
    
    # ONNX Runtime
    onnx_threads: int = 4
    
    class Config:
        env_prefix = "EMBEDDING_"


settings = Settings()
