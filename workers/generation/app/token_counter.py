"""Token counting and cost calculation for OpenAI models."""

import logging
import tiktoken
from typing import Optional

logger = logging.getLogger(__name__)


class TokenCounter:
    """Token counter for OpenAI models using tiktoken."""
    
    def __init__(self, model_name: str = "gpt-5-nano"):
        """Initialize token counter.
        
        Args:
            model_name: Name of the OpenAI model
        """
        self.model_name = model_name
        
        # Use gpt-4 encoding as base for newer models
        # gpt-5-nano would use the same encoding family
        try:
            self.encoding = tiktoken.encoding_for_model("gpt-4")
        except KeyError:
            logger.warning(f"No encoding found for {model_name}, using cl100k_base")
            self.encoding = tiktoken.get_encoding("cl100k_base")
    
    def count_tokens(self, text: str) -> int:
        """Count tokens in a text string.
        
        Args:
            text: Text to count tokens for
            
        Returns:
            Number of tokens
        """
        if not text:
            return 0
        return len(self.encoding.encode(text))
    
    def count_message_tokens(self, messages: list[dict]) -> int:
        """Count tokens in a list of messages.
        
        Args:
            messages: List of message dicts with 'role' and 'content'
            
        Returns:
            Total number of tokens including message formatting overhead
        """
        # Approximate overhead per message for chat completion format
        tokens_per_message = 3
        tokens_per_name = 1
        
        num_tokens = 0
        for message in messages:
            num_tokens += tokens_per_message
            for key, value in message.items():
                if isinstance(value, str):
                    num_tokens += self.count_tokens(value)
                if key == "name":
                    num_tokens += tokens_per_name
        
        # Add overhead for reply priming
        num_tokens += 3
        
        return num_tokens
    
    def estimate_prompt_tokens(self, prompt: str) -> int:
        """Estimate tokens for a prompt string.
        
        This is a convenience method that wraps count_tokens.
        
        Args:
            prompt: Prompt text
            
        Returns:
            Estimated token count
        """
        return self.count_tokens(prompt)
    
    def calculate_cost(
        self,
        input_tokens: int,
        output_tokens: int,
        input_price_per_million: float = 0.15,
        output_price_per_million: float = 0.60
    ) -> float:
        """Calculate cost for token usage.
        
        Args:
            input_tokens: Number of input tokens
            output_tokens: Number of output tokens
            input_price_per_million: Price per 1M input tokens in dollars
            output_price_per_million: Price per 1M output tokens in dollars
            
        Returns:
            Total cost in dollars
        """
        input_cost = (input_tokens / 1_000_000) * input_price_per_million
        output_cost = (output_tokens / 1_000_000) * output_price_per_million
        return input_cost + output_cost
    
    def truncate_to_token_limit(
        self,
        text: str,
        max_tokens: int,
        suffix: str = "..."
    ) -> str:
        """Truncate text to fit within a token limit.
        
        Args:
            text: Text to truncate
            max_tokens: Maximum number of tokens
            suffix: Suffix to add to truncated text
            
        Returns:
            Truncated text
        """
        if not text:
            return text
        
        tokens = self.encoding.encode(text)
        
        if len(tokens) <= max_tokens:
            return text
        
        # Reserve tokens for suffix
        suffix_tokens = self.count_tokens(suffix)
        available_tokens = max_tokens - suffix_tokens
        
        if available_tokens <= 0:
            return suffix
        
        # Truncate and decode
        truncated_tokens = tokens[:available_tokens]
        truncated_text = self.encoding.decode(truncated_tokens)
        
        return truncated_text + suffix
