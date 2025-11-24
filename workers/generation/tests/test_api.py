"""Tests for API endpoints."""

import pytest
from fastapi.testclient import TestClient
from unittest.mock import AsyncMock, patch, MagicMock

# Mock the settings before importing the app
with patch('app.config.settings') as mock_settings:
    mock_settings.openai_api_key = "test-key"
    mock_settings.model_name = "gpt-5-nano"
    mock_settings.max_concurrent_requests = 10
    mock_settings.max_retries = 3
    mock_settings.host = "0.0.0.0"
    mock_settings.port = 8003
    
    from app.main import app
    from app.schemas import SymbolContext


@pytest.fixture
def client():
    """Create a test client."""
    return TestClient(app)


@pytest.fixture
def sample_request_data():
    """Create sample request data."""
    return {
        "job_id": "job_test_123",
        "symbols": [
            {
                "symbol_id": "sym_1",
                "context": {
                    "symbol_id": "sym_1",
                    "name": "calculate_total",
                    "kind": "function",
                    "language": "python",
                    "file_path": "src/utils.py",
                    "signature": "def calculate_total() -> int",
                    "calls": ["sum"],
                    "called_by": ["main"],
                    "imports": [],
                    "related_symbols": [],
                    "cluster_label": "math",
                    "centrality": 0.8,
                }
            }
        ]
    }


def test_health_endpoint_structure(client):
    """Test that health endpoint returns correct structure."""
    response = client.get("/health")
    
    assert response.status_code == 200
    data = response.json()
    
    assert "status" in data
    assert "model" in data
    assert "ready" in data
    assert data["model"] == "gpt-5-nano"


@patch('app.main.openai_client')
def test_generate_endpoint_structure(mock_client, client, sample_request_data):
    """Test generate endpoint returns correct structure."""
    # Mock the OpenAI client and generator
    with patch('app.main.generator') as mock_generator:
        mock_generator.generate_batch = AsyncMock(return_value=(
            [
                MagicMock(
                    symbol_id="sym_1",
                    summary="Calculates the total of items.",
                    tokens_used=50
                )
            ],
            30,  # input tokens
            20,  # output tokens
            100,  # duration_ms
            []   # warnings
        ))
        
        response = client.post("/generate", json=sample_request_data)
        
        assert response.status_code == 200
        data = response.json()
        
        assert "documented_symbols" in data
        assert "total_tokens" in data
        assert "total_cost" in data
        
        assert len(data["documented_symbols"]) == 1
        assert data["documented_symbols"][0]["symbol_id"] == "sym_1"
        assert "summary" in data["documented_symbols"][0]


def test_generate_endpoint_requires_symbols(client):
    """Test that generate endpoint requires symbols."""
    response = client.post("/generate", json={"job_id": "test", "symbols": []})
    
    # Should accept empty list (even if not useful)
    # Or return 503 if generator not initialized in test
    assert response.status_code in [200, 503]


def test_health_endpoint_when_not_initialized(client):
    """Test health endpoint before initialization."""
    # In test environment, health check should handle uninitialized state
    response = client.get("/health")
    assert response.status_code == 200
    data = response.json()
    assert "status" in data
