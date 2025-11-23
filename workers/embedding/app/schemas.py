"""Request and response schemas."""

from typing import List
from pydantic import BaseModel, Field


class ChunkInput(BaseModel):
    """A single chunk to embed."""
    chunk_id: str = Field(..., description="Unique identifier for the chunk")
    content: str = Field(..., description="Text content to embed")


class EmbedRequest(BaseModel):
    """Request to embed a batch of chunks."""
    batch_id: str = Field(..., description="Unique identifier for this batch")
    chunks: List[ChunkInput] = Field(..., description="Chunks to embed", min_length=1)


class ChunkVector(BaseModel):
    """A chunk with its embedding vector."""
    chunk_id: str = Field(..., description="Unique identifier for the chunk")
    vector: List[float] = Field(..., description="384-dimensional embedding vector")


class EmbedResponse(BaseModel):
    """Response containing embedded chunks."""
    batch_id: str = Field(..., description="Unique identifier for this batch")
    vectors: List[ChunkVector] = Field(..., description="Embedded chunks")


class HealthResponse(BaseModel):
    """Health check response."""
    status: str = Field(..., description="Health status")
    model_loaded: bool = Field(..., description="Whether the model is loaded")
    embedding_dim: int = Field(..., description="Embedding dimension")
