"""Tests for token counting."""

import pytest
from app.token_counter import TokenCounter


def test_count_tokens_empty():
    """Test counting tokens in empty string."""
    counter = TokenCounter()
    assert counter.count_tokens("") == 0


def test_count_tokens_simple():
    """Test counting tokens in simple text."""
    counter = TokenCounter()
    text = "Hello world"
    count = counter.count_tokens(text)
    assert count > 0
    assert count < 10  # Should be around 2-3 tokens


def test_count_tokens_code():
    """Test counting tokens in code."""
    counter = TokenCounter()
    code = """
    def calculate_total(items: list) -> int:
        return sum(items)
    """
    count = counter.count_tokens(code)
    assert count > 10
    assert count < 50


def test_calculate_cost():
    """Test cost calculation."""
    counter = TokenCounter()
    
    # Test with default pricing
    cost = counter.calculate_cost(
        input_tokens=1000,
        output_tokens=500,
        input_price_per_million=0.15,
        output_price_per_million=0.60
    )
    
    # 1000 * 0.15 / 1M + 500 * 0.60 / 1M = 0.00015 + 0.0003 = 0.00045
    assert abs(cost - 0.00045) < 0.000001


def test_calculate_cost_large():
    """Test cost calculation with larger numbers."""
    counter = TokenCounter()
    
    cost = counter.calculate_cost(
        input_tokens=100_000,
        output_tokens=50_000,
        input_price_per_million=0.15,
        output_price_per_million=0.60
    )
    
    # 100k * 0.15 / 1M + 50k * 0.60 / 1M = 0.015 + 0.03 = 0.045
    assert abs(cost - 0.045) < 0.000001


def test_truncate_to_token_limit_no_truncation():
    """Test truncation when text is within limit."""
    counter = TokenCounter()
    text = "Short text"
    result = counter.truncate_to_token_limit(text, max_tokens=100)
    assert result == text


def test_truncate_to_token_limit_with_truncation():
    """Test truncation when text exceeds limit."""
    counter = TokenCounter()
    text = "This is a very long text that will definitely exceed the token limit " * 10
    result = counter.truncate_to_token_limit(text, max_tokens=50)
    
    assert len(result) < len(text)
    assert result.endswith("...")
    assert counter.count_tokens(result) <= 50


def test_truncate_empty_text():
    """Test truncating empty text."""
    counter = TokenCounter()
    assert counter.truncate_to_token_limit("", max_tokens=100) == ""


def test_count_message_tokens():
    """Test counting tokens in chat messages."""
    counter = TokenCounter()
    
    messages = [
        {"role": "system", "content": "You are a helpful assistant."},
        {"role": "user", "content": "Hello, how are you?"},
    ]
    
    count = counter.count_message_tokens(messages)
    
    # Should count content plus overhead for message formatting
    assert count > 10
    assert count < 30


def test_estimate_prompt_tokens():
    """Test prompt token estimation."""
    counter = TokenCounter()
    
    prompt = "Generate documentation for this function: def foo(): pass"
    estimated = counter.estimate_prompt_tokens(prompt)
    
    assert estimated > 0
    # Should be similar to count_tokens
    assert abs(estimated - counter.count_tokens(prompt)) < 5
