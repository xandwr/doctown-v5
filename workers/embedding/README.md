# Doctown Embedding Worker

CPU-based embedding service using all-MiniLM-L6-v2 ONNX model.

## Setup

```bash
# Create virtual environment
python -m venv .venv
source .venv/bin/activate

# Install dependencies
pip install -e ".[dev]"
```

## Run

```bash
# Start the server
uvicorn app.main:app --host 0.0.0.0 --port 8000 --reload
```

## Test

```bash
pytest
```

## API

### Health Check
```bash
curl http://localhost:8000/health
```

### Embed Chunks
```bash
curl -X POST http://localhost:8000/embed \
  -H "Content-Type: application/json" \
  -d '{
    "batch_id": "batch_001",
    "chunks": [
      {"chunk_id": "c1", "content": "function hello() { return 'world'; }"},
      {"chunk_id": "c2", "content": "class Parser { parse() {} }"}
    ]
  }'
```

## Model

Uses `sentence-transformers/all-MiniLM-L6-v2` via ONNX Runtime:
- **Dimensions**: 384
- **Runtime**: CPU-optimized ONNX
- **Batch size**: Min 16, Max 256
- **Timeout**: 500ms
