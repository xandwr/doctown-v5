"""Batch documentation generation with concurrency control."""

import logging
import asyncio
import time
from typing import List, Tuple

from .config import settings
from .schemas import SymbolContext, DocumentedSymbol
from .openai_client import OpenAIClient
from .token_counter import TokenCounter
from .events import emit_symbol_documented

logger = logging.getLogger(__name__)


class DocumentationGenerator:
    """Generate documentation for multiple symbols with concurrency control."""
    
    def __init__(
        self,
        openai_client: OpenAIClient,
        token_counter: TokenCounter,
        max_concurrent: int = 10,
    ):
        """Initialize documentation generator.
        
        Args:
            openai_client: OpenAI client for API calls
            token_counter: Token counter for cost calculation
            max_concurrent: Maximum concurrent requests
        """
        self.openai_client = openai_client
        self.token_counter = token_counter
        self.max_concurrent = max_concurrent
        self.semaphore = asyncio.Semaphore(max_concurrent)
    
    async def generate_single(
        self,
        context: SymbolContext
    ) -> Tuple[DocumentedSymbol, int, int]:
        """Generate documentation for a single symbol.
        
        Args:
            context: Symbol context
            
        Returns:
            Tuple of (documented_symbol, input_tokens, output_tokens)
        """
        async with self.semaphore:
            try:
                summary, input_tokens, output_tokens = await self.openai_client.generate_documentation(context)
                
                documented = DocumentedSymbol(
                    symbol_id=context.symbol_id,
                    summary=summary,
                    tokens_used=input_tokens + output_tokens,
                )
                
                # Emit event for this symbol
                emit_symbol_documented(context.symbol_id, input_tokens + output_tokens)
                
                return documented, input_tokens, output_tokens
                
            except Exception as e:
                logger.error(f"Failed to document {context.symbol_id}: {e}")
                # Return a placeholder with error message
                documented = DocumentedSymbol(
                    symbol_id=context.symbol_id,
                    summary=f"[Documentation generation failed: {str(e)[:100]}]",
                    tokens_used=0,
                )
                return documented, 0, 0
    
    async def generate_batch(
        self,
        contexts: List[SymbolContext],
        progress_callback=None
    ) -> Tuple[List[DocumentedSymbol], int, int, int, List[str]]:
        """Generate documentation for multiple symbols concurrently.
        
        Args:
            contexts: List of symbol contexts
            progress_callback: Optional callback for progress updates
            
        Returns:
            Tuple of (documented_symbols, total_input_tokens, total_output_tokens, duration_ms, warnings)
        """
        start_time = time.perf_counter()
        
        logger.info(f"Starting batch documentation for {len(contexts)} symbols")
        
        # Create tasks for all symbols
        tasks = [self.generate_single(context) for context in contexts]
        
        # Execute with progress tracking
        documented_symbols = []
        total_input_tokens = 0
        total_output_tokens = 0
        warnings = []
        
        completed = 0
        for coro in asyncio.as_completed(tasks):
            try:
                documented, input_tokens, output_tokens = await coro
                documented_symbols.append(documented)
                total_input_tokens += input_tokens
                total_output_tokens += output_tokens
                
                # Check for failures
                if documented.summary.startswith("[Documentation generation failed"):
                    warnings.append(f"Failed to document {documented.symbol_id}")
                
                completed += 1
                
                if progress_callback:
                    progress_callback(completed, len(contexts))
                
                if completed % 10 == 0:
                    logger.info(f"Progress: {completed}/{len(contexts)} symbols documented")
                    
            except Exception as e:
                logger.error(f"Task failed unexpectedly: {e}")
                warnings.append(f"Task failed: {str(e)[:100]}")
        
        duration_ms = int((time.perf_counter() - start_time) * 1000)
        
        logger.info(
            f"Batch complete: {len(documented_symbols)} symbols, "
            f"{total_input_tokens + total_output_tokens} tokens, "
            f"{duration_ms}ms"
        )
        
        return documented_symbols, total_input_tokens, total_output_tokens, duration_ms, warnings
