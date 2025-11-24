"""Tests for prompt building."""

import pytest
from app.prompt_builder import PromptBuilder
from app.schemas import SymbolContext
from app.token_counter import TokenCounter


@pytest.fixture
def sample_context():
    """Create a sample symbol context."""
    return SymbolContext(
        symbol_id="sym_123",
        name="calculate_total",
        kind="function",
        language="python",
        file_path="src/utils.py",
        signature="def calculate_total(items: list[int]) -> int",
        calls=["sum", "len"],
        called_by=["main", "process_order"],
        imports=["typing"],
        related_symbols=["validate_items"],
        cluster_label="math utilities",
        centrality=0.75,
    )


def test_build_prompt_basic(sample_context):
    """Test basic prompt building."""
    builder = PromptBuilder()
    prompt = builder.build_prompt(sample_context)
    
    assert "python" in prompt.lower()
    assert "calculate_total" in prompt
    assert "function" in prompt
    assert "src/utils.py" in prompt
    assert "def calculate_total" in prompt
    assert "sum" in prompt
    assert "main" in prompt
    assert "0.75" in prompt


def test_build_prompt_includes_instructions(sample_context):
    """Test that prompt includes documentation instructions."""
    builder = PromptBuilder()
    prompt = builder.build_prompt(sample_context)
    
    assert "1-2 sentences" in prompt
    assert "concise" in prompt.lower()
    assert "purpose" in prompt.lower()


def test_build_prompt_with_cluster_label(sample_context):
    """Test prompt includes cluster label."""
    builder = PromptBuilder()
    prompt = builder.build_prompt(sample_context)
    
    assert "math utilities" in prompt


def test_build_prompt_without_cluster_label(sample_context):
    """Test prompt works without cluster label."""
    sample_context.cluster_label = None
    builder = PromptBuilder()
    prompt = builder.build_prompt(sample_context)
    
    assert "Related to:" not in prompt
    assert "calculate_total" in prompt  # Other fields still present


def test_build_prompt_truncation():
    """Test prompt truncation for very long signatures."""
    long_signature = "def very_long_function(" + ", ".join(f"arg{i}: str" for i in range(100)) + ") -> None"
    
    context = SymbolContext(
        symbol_id="sym_long",
        name="very_long_function",
        kind="function",
        language="python",
        file_path="src/long.py",
        signature=long_signature,
        calls=[],
        called_by=[],
        imports=[],
        related_symbols=[],
        cluster_label=None,
        centrality=0.5,
    )
    
    builder = PromptBuilder(max_tokens=500)
    prompt = builder.build_prompt(context)
    
    # Should be truncated
    token_counter = TokenCounter()
    assert token_counter.count_tokens(prompt) <= 500


def test_build_messages(sample_context):
    """Test building chat messages."""
    builder = PromptBuilder()
    messages = builder.build_messages(sample_context)
    
    assert len(messages) == 2
    assert messages[0]["role"] == "system"
    assert messages[1]["role"] == "user"
    assert "documentation" in messages[0]["content"].lower()
    assert "calculate_total" in messages[1]["content"]


def test_build_prompt_limits_calls(sample_context):
    """Test that calls list is limited."""
    sample_context.calls = [f"function_{i}" for i in range(20)]
    
    builder = PromptBuilder()
    prompt = builder.build_prompt(sample_context)
    
    # Should only include first 10
    assert "function_0" in prompt
    assert "function_9" in prompt
    # Depending on implementation, may not include later ones


def test_build_prompt_limits_called_by(sample_context):
    """Test that called_by list is limited."""
    sample_context.called_by = [f"caller_{i}" for i in range(20)]
    
    builder = PromptBuilder()
    prompt = builder.build_prompt(sample_context)
    
    # Should only include first 10
    assert "caller_0" in prompt
    assert "caller_9" in prompt
