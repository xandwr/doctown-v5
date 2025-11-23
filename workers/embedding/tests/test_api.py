"""Tests for API endpoints."""

import pytest
from fastapi.testclient import TestClient

from app.main import app


@pytest.fixture
def client():
    """Create a test client."""
    return TestClient(app)


def test_health_endpoint_returns_healthy(client):
    """Test that health endpoint returns healthy status."""
    response = client.get("/health")
    
    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "healthy"
    assert data["model_loaded"] is True
    assert data["embedding_dim"] == 384


def test_embed_endpoint_accepts_valid_request(client):
    """Test that embed endpoint processes valid requests."""
    request = {
        "batch_id": "test_batch_001",
        "chunks": [
            {
                "chunk_id": "chunk_1",
                "content": "function hello() { return 'world'; }"
            },
            {
                "chunk_id": "chunk_2",
                "content": "class Parser { parse() {} }"
            }
        ]
    }
    
    response = client.post("/embed", json=request)
    
    assert response.status_code == 200
    data = response.json()
    assert data["batch_id"] == "test_batch_001"
    assert len(data["vectors"]) == 2
    assert data["vectors"][0]["chunk_id"] == "chunk_1"
    assert data["vectors"][1]["chunk_id"] == "chunk_2"
    assert len(data["vectors"][0]["vector"]) == 384
    assert len(data["vectors"][1]["vector"]) == 384


def test_embed_endpoint_rejects_empty_chunks(client):
    """Test that embed endpoint rejects empty chunk list."""
    request = {
        "batch_id": "test_batch_002",
        "chunks": []
    }
    
    response = client.post("/embed", json=request)
    
    assert response.status_code == 422  # Validation error


def test_embed_endpoint_handles_single_chunk(client):
    """Test that embed endpoint handles single chunk correctly."""
    request = {
        "batch_id": "test_batch_003",
        "chunks": [
            {
                "chunk_id": "single_chunk",
                "content": "def process(): pass"
            }
        ]
    }
    
    response = client.post("/embed", json=request)
    
    assert response.status_code == 200
    data = response.json()
    assert len(data["vectors"]) == 1
    assert data["vectors"][0]["chunk_id"] == "single_chunk"


def test_embed_endpoint_handles_large_batch(client):
    """Test that embed endpoint handles larger batches."""
    chunks = [
        {
            "chunk_id": f"chunk_{i}",
            "content": f"function test{i}() {{ return {i}; }}"
        }
        for i in range(50)
    ]
    
    request = {
        "batch_id": "test_batch_large",
        "chunks": chunks
    }
    
    response = client.post("/embed", json=request)
    
    assert response.status_code == 200
    data = response.json()
    assert len(data["vectors"]) == 50
    for i, vector in enumerate(data["vectors"]):
        assert vector["chunk_id"] == f"chunk_{i}"


def test_embed_endpoint_requires_batch_id(client):
    """Test that batch_id is required."""
    request = {
        "chunks": [
            {"chunk_id": "c1", "content": "test"}
        ]
    }
    
    response = client.post("/embed", json=request)
    
    assert response.status_code == 422


def test_embed_endpoint_requires_chunk_id(client):
    """Test that chunk_id is required for each chunk."""
    request = {
        "batch_id": "test_batch_004",
        "chunks": [
            {"content": "test"}
        ]
    }
    
    response = client.post("/embed", json=request)
    
    assert response.status_code == 422


def test_embed_endpoint_requires_content(client):
    """Test that content is required for each chunk."""
    request = {
        "batch_id": "test_batch_005",
        "chunks": [
            {"chunk_id": "c1"}
        ]
    }
    
    response = client.post("/embed", json=request)
    
    assert response.status_code == 422


def test_cors_headers_present(client):
    """Test that CORS headers are properly set."""
    response = client.options("/health")
    
    assert "access-control-allow-origin" in response.headers
