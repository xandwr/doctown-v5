"""Prompt construction for symbol documentation."""

import logging
from typing import Optional

from .schemas import SymbolContext
from .token_counter import TokenCounter

logger = logging.getLogger(__name__)


class PromptBuilder:
    """Build prompts for symbol documentation generation."""
    
    def __init__(self, token_counter: Optional[TokenCounter] = None, max_tokens: int = 2000):
        """Initialize prompt builder.
        
        Args:
            token_counter: Token counter instance
            max_tokens: Maximum tokens allowed in prompt
        """
        self.token_counter = token_counter or TokenCounter()
        self.max_tokens = max_tokens
    
    def build_prompt(self, context: SymbolContext) -> str:
        """Build documentation prompt from symbol context.
        
        Args:
            context: Symbol context with metadata
            
        Returns:
            Formatted prompt string
        """
        # Build the full prompt
        prompt_parts = [
            f"You are documenting a {context.language} codebase.",
            "",
            f"Symbol: {context.name}",
            f"Kind: {context.kind}",
            f"File: {context.file_path}",
            f"Signature: {context.signature}",
        ]
        
        # Add relational information
        if context.calls:
            calls_str = ", ".join(context.calls[:10])  # Limit to 10
            prompt_parts.append(f"Calls: {calls_str}")
        
        if context.called_by:
            called_by_str = ", ".join(context.called_by[:10])  # Limit to 10
            prompt_parts.append(f"Called by: {called_by_str}")
        
        if context.cluster_label:
            prompt_parts.append(f"Related to: {context.cluster_label}")
        
        prompt_parts.append(f"Importance: {context.centrality:.2f} (0-1 scale)")
        
        # Add instructions
        prompt_parts.extend([
            "",
            "Write 1-2 sentences describing what this symbol does.",
            "Be concise and precise. Focus on purpose, not implementation.",
        ])
        
        prompt = "\n".join(prompt_parts)
        
        # Check token count and truncate if necessary
        token_count = self.token_counter.count_tokens(prompt)
        
        if token_count > self.max_tokens:
            logger.warning(
                f"Prompt for {context.symbol_id} exceeds {self.max_tokens} tokens "
                f"({token_count} tokens). Truncating signature."
            )
            # Truncate the signature which is often the longest part
            prompt = self._build_truncated_prompt(context)
        
        return prompt
    
    def _build_truncated_prompt(self, context: SymbolContext) -> str:
        """Build prompt with truncated signature to fit token limit.
        
        Args:
            context: Symbol context
            
        Returns:
            Truncated prompt string
        """
        # Calculate base prompt size without signature
        base_parts = [
            f"You are documenting a {context.language} codebase.",
            "",
            f"Symbol: {context.name}",
            f"Kind: {context.kind}",
            f"File: {context.file_path}",
        ]
        
        # Add smaller metadata
        metadata_parts = []
        if context.calls:
            calls_str = ", ".join(context.calls[:5])  # Reduced to 5
            metadata_parts.append(f"Calls: {calls_str}")
        
        if context.called_by:
            called_by_str = ", ".join(context.called_by[:5])  # Reduced to 5
            metadata_parts.append(f"Called by: {called_by_str}")
        
        if context.cluster_label:
            metadata_parts.append(f"Related to: {context.cluster_label}")
        
        metadata_parts.append(f"Importance: {context.centrality:.2f}")
        
        instructions = [
            "",
            "Write 1-2 sentences describing what this symbol does.",
            "Be concise and precise. Focus on purpose, not implementation.",
        ]
        
        # Calculate tokens available for signature
        base_prompt = "\n".join(base_parts + metadata_parts + instructions)
        base_tokens = self.token_counter.count_tokens(base_prompt)
        
        # Reserve space for "Signature: " prefix
        signature_prefix = "Signature: "
        prefix_tokens = self.token_counter.count_tokens(signature_prefix)
        
        available_tokens = self.max_tokens - base_tokens - prefix_tokens - 10  # Buffer
        
        if available_tokens > 0:
            truncated_signature = self.token_counter.truncate_to_token_limit(
                context.signature,
                available_tokens,
                suffix="..."
            )
        else:
            truncated_signature = "..."
        
        # Build final prompt
        signature_part = f"Signature: {truncated_signature}"
        all_parts = base_parts + [signature_part] + metadata_parts + instructions
        
        return "\n".join(all_parts)
    
    def build_messages(self, context: SymbolContext) -> list[dict]:
        """Build OpenAI chat messages from symbol context.
        
        Args:
            context: Symbol context
            
        Returns:
            List of message dicts for OpenAI API
        """
        prompt = self.build_prompt(context)
        
        return [
            {
                "role": "system",
                "content": "You are a technical documentation expert. Generate concise, accurate documentation for code symbols."
            },
            {
                "role": "user",
                "content": prompt
            }
        ]
