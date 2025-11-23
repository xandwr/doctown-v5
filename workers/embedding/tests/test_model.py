"""Tests for embedding model."""

import pytest
import numpy as np
from pathlib import Path

from app.model import EmbeddingModel
from app.config import settings


@pytest.fixture
def model():
    """Create and load a model instance."""
    m = EmbeddingModel(settings.model_path)
    m.load()
    return m


def test_model_loads_successfully(model):
    """Test that the model loads without errors."""
    assert model.session is not None
    assert model.tokenizer is not None


def test_embed_single_returns_correct_shape(model):
    """Test that embedding a single text returns a 384-dim vector."""
    text = "This is a test function that processes data."
    vector = model.embed_single(text)
    
    assert isinstance(vector, np.ndarray)
    assert vector.shape == (settings.embedding_dim,)
    assert vector.dtype == np.float32 or vector.dtype == np.float64


def test_embed_batch_returns_correct_shape(model):
    """Test that embedding a batch returns correct shape."""
    texts = [
        "function calculateTotal(items) { return items.reduce((a, b) => a + b, 0); }",
        "class DataProcessor { process(data) { return data.filter(x => x > 0); } }",
        "def parse_config(path): with open(path) as f: return json.load(f)",
    ]
    
    vectors = model.embed(texts)
    
    assert isinstance(vectors, np.ndarray)
    assert vectors.shape == (len(texts), settings.embedding_dim)
    assert vectors.dtype == np.float32 or vectors.dtype == np.float64


def test_embeddings_are_normalized(model):
    """Test that embeddings are normalized (L2 norm ≈ 1)."""
    text = "Normalize this embedding vector."
    vector = model.embed_single(text)
    
    norm = np.linalg.norm(vector)
    assert abs(norm - 1.0) < 0.01  # Should be very close to 1


def test_similar_texts_have_similar_embeddings(model):
    """Test that similar texts produce similar embeddings."""
    text1 = "Calculate the sum of numbers"
    text2 = "Compute the total of values"
    text3 = "Parse JSON configuration file"
    
    v1 = model.embed_single(text1)
    v2 = model.embed_single(text2)
    v3 = model.embed_single(text3)
    
    # Cosine similarity
    sim_12 = np.dot(v1, v2)
    sim_13 = np.dot(v1, v3)
    
    # Similar texts should be more similar than dissimilar ones
    assert sim_12 > sim_13


def test_batch_embedding_matches_individual(model):
    """Test that batch embedding produces same results as individual embeddings."""
    texts = [
        "First test text",
        "Second test text",
        "Third test text"
    ]
    
    # Batch embedding
    batch_vectors = model.embed(texts)
    
    # Individual embeddings
    individual_vectors = np.array([model.embed_single(text) for text in texts])
    
    # Should be very close (allowing for minor floating point differences)
    assert np.allclose(batch_vectors, individual_vectors, atol=1e-5)


def test_empty_text_handles_gracefully(model):
    """Test that empty text is handled without crashing."""
    text = ""
    vector = model.embed_single(text)
    
    assert isinstance(vector, np.ndarray)
    assert vector.shape == (settings.embedding_dim,)


def test_long_text_is_truncated(model):
    """Test that very long text is properly handled."""
    # Create a text longer than max token length (512 tokens)
    text = " ".join(["word"] * 1000)
    vector = model.embed_single(text)
    
    assert isinstance(vector, np.ndarray)
    assert vector.shape == (settings.embedding_dim,)


@pytest.mark.benchmark
def test_embedding_throughput(model, benchmark):
    """Benchmark embedding throughput."""
    texts = [
        f"This is test text number {i} with some code content."
        for i in range(32)
    ]
    
    result = benchmark(model.embed, texts)
    
    assert result.shape == (32, settings.embedding_dim)
