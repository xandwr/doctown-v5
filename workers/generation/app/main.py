"""FastAPI application for generation worker."""

import logging
import time
from contextlib import asynccontextmanager

from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware

from .config import settings
from .schemas import GenerateRequest, GenerateResponse, HealthResponse
from .openai_client import OpenAIClient
from .token_counter import TokenCounter
from .generator import DocumentationGenerator
from .events import emit_generation_started, emit_generation_completed

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)


# Global instances
openai_client: OpenAIClient = None
token_counter: TokenCounter = None
generator: DocumentationGenerator = None


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Lifecycle manager for the application."""
    global openai_client, token_counter, generator
    
    # Startup
    logger.info("Starting generation worker...")
    logger.info(f"Model: {settings.model_name}")
    logger.info(f"Max concurrent requests: {settings.max_concurrent_requests}")
    
    try:
        token_counter = TokenCounter(settings.model_name)
        openai_client = OpenAIClient(
            api_key=settings.openai_api_key,
            model_name=settings.model_name,
            max_retries=settings.max_retries,
        )
        generator = DocumentationGenerator(
            openai_client=openai_client,
            token_counter=token_counter,
            max_concurrent=settings.max_concurrent_requests,
        )
        logger.info("Generation worker initialized successfully")
    except Exception as e:
        logger.error(f"Failed to initialize worker: {e}")
        raise
    
    yield
    
    # Shutdown
    logger.info("Shutting down generation worker...")


app = FastAPI(
    title="Doctown Generation Worker",
    description="OpenAI-based documentation generation service using gpt-5-nano with structured output",
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
        # Check if client is initialized
        if openai_client is None:
            return HealthResponse(
                status="initializing",
                model=settings.model_name,
                ready=False,
            )
        
        # Try a simple API health check
        is_ready = await openai_client.check_health()
        
        return HealthResponse(
            status="healthy" if is_ready else "degraded",
            model=settings.model_name,
            ready=is_ready,
        )
    except Exception as e:
        logger.error(f"Health check failed: {e}")
        return HealthResponse(
            status="unhealthy",
            model=settings.model_name,
            ready=False,
        )


@app.post("/generate", response_model=GenerateResponse)
async def generate_documentation(request: GenerateRequest):
    """Generate documentation for a batch of symbols.
    
    Args:
        request: Generation request with job_id and symbols
        
    Returns:
        Response containing documented symbols with token usage and cost
    """
    if generator is None:
        raise HTTPException(status_code=503, detail="Generator not initialized")
    
    try:
        logger.info(f"Generation request for job {request.job_id} with {len(request.symbols)} symbols")
        
        # Extract contexts from request
        contexts = [symbol.context for symbol in request.symbols]
        
        # Estimate total tokens
        estimated_tokens = sum(
            token_counter.estimate_prompt_tokens(
                f"{ctx.signature} {ctx.name}"
            ) for ctx in contexts
        )
        
        # Emit started event
        emit_generation_started(len(contexts), estimated_tokens)
        
        # Generate documentation
        documented_symbols, input_tokens, output_tokens, duration_ms, warnings = \
            await generator.generate_batch(contexts)
        
        # Calculate cost
        total_tokens = input_tokens + output_tokens
        total_cost = token_counter.calculate_cost(
            input_tokens,
            output_tokens,
            settings.input_token_price,
            settings.output_token_price,
        )
        
        # Emit completed event
        status = "success" if len(warnings) == 0 else "success"  # Still success with partial failures
        emit_generation_completed(
            total_tokens=total_tokens,
            total_cost=total_cost,
            duration_ms=duration_ms,
            status=status,
            warnings=warnings,
        )
        
        logger.info(
            f"Generation complete for job {request.job_id}: "
            f"{len(documented_symbols)} symbols, "
            f"{total_tokens} tokens, "
            f"${total_cost:.4f}"
        )
        
        return GenerateResponse(
            documented_symbols=documented_symbols,
            total_tokens=total_tokens,
            total_cost=total_cost,
        )
        
    except Exception as e:
        logger.error(f"Generation failed for job {request.job_id}: {e}")
        emit_generation_completed(
            total_tokens=0,
            total_cost=0.0,
            duration_ms=0,
            status="failed",
            warnings=[str(e)],
        )
        raise HTTPException(status_code=500, detail=str(e))


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(
        app,
        host=settings.host,
        port=settings.port,
        log_level="info",
    )
