"""Configuration for the generation worker."""

from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    """Application settings loaded from environment variables."""
    
    # OpenAI Configuration
    openai_api_key: str
    model_name: str = "gpt-5-nano"
    
    # Token pricing (per 1M tokens)
    input_token_price: float = 0.15  # $0.15 per 1M input tokens
    output_token_price: float = 0.60  # $0.60 per 1M output tokens
    
    # Rate limiting
    max_concurrent_requests: int = 10
    max_retries: int = 3
    retry_min_wait: float = 1.0  # seconds
    retry_max_wait: float = 60.0  # seconds
    
    # Prompt settings
    max_prompt_tokens: int = 2000
    
    # Server settings
    host: str = "0.0.0.0"
    port: int = 8003
    
    class Config:
        env_file = ".env"
        env_file_encoding = "utf-8"


settings = Settings()
