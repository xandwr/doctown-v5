"""OpenAI API client wrapper with retry logic and rate limiting."""

import logging
import asyncio
from typing import Optional

from openai import AsyncOpenAI, RateLimitError, APIError
from tenacity import (
    retry,
    stop_after_attempt,
    wait_exponential,
    retry_if_exception_type,
    before_sleep_log,
)

from .config import settings
from .schemas import SymbolContext, DocumentationOutput
from .prompt_builder import PromptBuilder
from .token_counter import TokenCounter

logger = logging.getLogger(__name__)


class OpenAIClient:
    """Wrapper for OpenAI API with structured output support."""
    
    def __init__(
        self,
        api_key: Optional[str] = None,
        model_name: Optional[str] = None,
        max_retries: int = 3,
    ):
        """Initialize OpenAI client.
        
        Args:
            api_key: OpenAI API key
            model_name: Model to use for generation
            max_retries: Maximum number of retries for failed requests
        """
        self.api_key = api_key or settings.openai_api_key
        self.model_name = model_name or settings.model_name
        self.max_retries = max_retries
        
        self.client = AsyncOpenAI(api_key=self.api_key)
        self.prompt_builder = PromptBuilder()
        self.token_counter = TokenCounter(self.model_name)
    
    @retry(
        retry=retry_if_exception_type((RateLimitError, APIError)),
        stop=stop_after_attempt(3),
        wait=wait_exponential(
            multiplier=settings.retry_min_wait,
            max=settings.retry_max_wait
        ),
        before_sleep=before_sleep_log(logger, logging.WARNING),
    )
    async def generate_documentation(
        self,
        context: SymbolContext
    ) -> tuple[str, int, int]:
        """Generate documentation for a symbol using structured output.
        
        Args:
            context: Symbol context
            
        Returns:
            Tuple of (summary, input_tokens, output_tokens)
            
        Raises:
            RateLimitError: If rate limit is exceeded after retries
            APIError: If API request fails after retries
        """
        try:
            # Build messages
            messages = self.prompt_builder.build_messages(context)
            
            # Estimate input tokens
            input_tokens_estimate = self.token_counter.count_message_tokens(messages)
            
            logger.info(
                f"Generating docs for {context.symbol_id} "
                f"(estimated {input_tokens_estimate} input tokens)"
            )
            
            # Call OpenAI API with structured output
            response = await self.client.beta.chat.completions.parse(
                model=self.model_name,
                messages=messages,
                response_format=DocumentationOutput,
                temperature=0.3,  # Lower temperature for more consistent output
                max_tokens=150,  # 1-2 sentences shouldn't need more
            )
            
            # Extract structured output
            output = response.choices[0].message.parsed
            summary = output.summary
            
            # Get actual token usage
            usage = response.usage
            input_tokens = usage.prompt_tokens
            output_tokens = usage.completion_tokens
            
            logger.info(
                f"Generated docs for {context.symbol_id}: "
                f"{input_tokens} input + {output_tokens} output = "
                f"{input_tokens + output_tokens} total tokens"
            )
            
            return summary, input_tokens, output_tokens
            
        except RateLimitError as e:
            logger.warning(f"Rate limit hit for {context.symbol_id}: {e}")
            raise
        except APIError as e:
            logger.error(f"API error for {context.symbol_id}: {e}")
            raise
        except Exception as e:
            logger.error(f"Unexpected error for {context.symbol_id}: {e}")
            # Re-raise as APIError for retry logic
            raise APIError(f"Unexpected error: {e}") from e
    
    async def check_health(self) -> bool:
        """Check if the OpenAI API is accessible.
        
        Returns:
            True if API is accessible, False otherwise
        """
        try:
            # Make a minimal API call to check connectivity
            response = await self.client.chat.completions.create(
                model=self.model_name,
                messages=[{"role": "user", "content": "test"}],
                max_tokens=1,
            )
            return True
        except Exception as e:
            logger.error(f"Health check failed: {e}")
            return False
