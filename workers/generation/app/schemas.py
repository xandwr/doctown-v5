"""Pydantic schemas for request/response models."""

from typing import List, Optional
from pydantic import BaseModel, Field


class SymbolContext(BaseModel):
    """Context about a symbol for documentation generation."""
    symbol_id: str
    name: str
    kind: str
    language: str
    file_path: str
    signature: str
    calls: List[str] = Field(default_factory=list)
    called_by: List[str] = Field(default_factory=list)
    imports: List[str] = Field(default_factory=list)
    related_symbols: List[str] = Field(default_factory=list)
    cluster_label: Optional[str] = None
    centrality: float = Field(ge=0.0, le=1.0)


class SymbolInput(BaseModel):
    """Input for a single symbol to document."""
    symbol_id: str
    context: SymbolContext


class GenerateRequest(BaseModel):
    """Request schema for /generate endpoint."""
    job_id: str
    symbols: List[SymbolInput]


class DocumentedSymbol(BaseModel):
    """A symbol with its generated documentation."""
    symbol_id: str
    summary: str
    tokens_used: int


class GenerateResponse(BaseModel):
    """Response schema for /generate endpoint."""
    documented_symbols: List[DocumentedSymbol]
    total_tokens: int
    total_cost: float


class HealthResponse(BaseModel):
    """Response schema for /health endpoint."""
    status: str
    model: str
    ready: bool


class DocumentationOutput(BaseModel):
    """Structured output schema for OpenAI API."""
    summary: str = Field(
        description="1-2 sentences describing what this symbol does. Be concise and precise. Focus on purpose, not implementation."
    )
