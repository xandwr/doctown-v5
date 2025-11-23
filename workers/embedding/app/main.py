"""FastAPI application for embedding worker."""

import logging
import time
from contextlib import asynccontextmanager

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware

from .config import settings
from .events import emit_batch_completed, emit_batch_started
from .model import get_model
from .schemas import (
    ChunkVector,
    EmbedRequest,
    EmbedResponse,
    HealthResponse,
)

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Lifecycle manager for the application."""
    # Startup
    logger.info("Starting embedding worker...")
    try:
        model = get_model()
        logger.info("Model loaded successfully")
    except Exception as e:
        logger.error(f"Failed to load model: {e}")
        raise
    
    yield
    
    # Shutdown
    logger.info("Shutting down embedding worker...")


app = FastAPI(
    title="Doctown Embedding Worker",
    description="CPU-optimized embedding service using all-MiniLM-L6-v2 ONNX model with intelligent batching",
    version="0.1.0",
    lifespan=lifespan,
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/health", response_model=HealthResponse)
async def health_check():
    """Health check endpoint."""
    try:
        model = get_model()
        return HealthResponse(
            status="healthy",
            model_loaded=True,
            embedding_dim=settings.embedding_dim,
        )
    except Exception as e:
        logger.error(f"Health check failed: {e}")
        return HealthResponse(
            status="unhealthy",
            model_loaded=False,
            embedding_dim=settings.embedding_dim,
        )


@app.post("/embed", response_model=EmbedResponse)
async def embed_chunks(request: EmbedRequest):
    """Embed a batch of chunks.
    
    Args:
        request: Batch of chunks to embed
        
    Returns:
        Response containing embedding vectors for each chunk
    """
    try:
        # Emit batch started event
        emit_batch_started(request.batch_id, len(request.chunks))
        
        # Start timer
        start_time = time.perf_counter()
        
        # Get model
        model = get_model()
        
        # Extract texts
        texts = [chunk.content for chunk in request.chunks]
        
        # Embed texts
        embeddings = model.embed(texts)
        
        # Build response
        vectors = [
            ChunkVector(
                chunk_id=chunk.chunk_id,
                vector=embeddings[i].tolist()
            )
            for i, chunk in enumerate(request.chunks)
        ]
        
        # Calculate duration
        duration_ms = (time.perf_counter() - start_time) * 1000
        
        # Emit batch completed event
        emit_batch_completed(request.batch_id, len(request.chunks), duration_ms)
        
        logger.info(
            f"Embedded batch {request.batch_id}: {len(request.chunks)} chunks "
            f"in {duration_ms:.2f}ms ({len(request.chunks) / (duration_ms / 1000):.1f} chunks/sec)"
        )
        
        return EmbedResponse(
            batch_id=request.batch_id,
            vectors=vectors
        )
        
    except Exception as e:
        logger.error(f"Error embedding batch {request.batch_id}: {e}")
        raise HTTPException(status_code=500, detail=str(e))


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(
        "app.main:app",
        host=settings.host,
        port=settings.port,
        reload=True
    )
