# Embedding Worker - Quick Start Guide

## Prerequisites

- Python 3.10 or higher
- The ONNX model files in `../../models/minilm-l6/`

## 5-Minute Setup

### 1. Setup (First Time Only)
```bash
cd workers/embedding
./setup.sh
```

This will:
- Create a virtual environment (`.venv`)
- Install all dependencies
- Set up the development environment

### 2. Run the Server
```bash
./run.sh
```

Or manually:
```bash
source .venv/bin/activate
uvicorn app.main:app --host 0.0.0.0 --port 8000 --reload
```

The server will start on http://localhost:8000

### 3. Test It

#### Health Check
```bash
curl http://localhost:8000/health
```

Expected response:
```json
{
  "status": "healthy",
  "model_loaded": true,
  "embedding_dim": 384
}
```

#### Embed Some Code
```bash
curl -X POST http://localhost:8000/embed \
  -H "Content-Type: application/json" \
  -d '{
    "batch_id": "test_001",
    "chunks": [
      {
        "chunk_id": "chunk_1",
        "content": "function calculateSum(numbers) { return numbers.reduce((a, b) => a + b, 0); }"
      },
      {
        "chunk_id": "chunk_2", 
        "content": "class DataProcessor { constructor() { this.data = []; } process() { return this.data.filter(x => x > 0); } }"
      }
    ]
  }'
```

You should get back two 384-dimensional vectors!

### 4. Run Tests
```bash
source .venv/bin/activate
pytest
```

Or with verbose output:
```bash
pytest -v
```

Or run specific tests:
```bash
pytest tests/test_model.py -v
pytest tests/test_api.py -v
```

## API Documentation

Once the server is running, visit:
- Swagger UI: http://localhost:8000/docs
- ReDoc: http://localhost:8000/redoc

## Configuration

Environment variables (all optional):
- `EMBEDDING_HOST`: Server host (default: 0.0.0.0)
- `EMBEDDING_PORT`: Server port (default: 8000)
- `EMBEDDING_MODEL_PATH`: Path to model directory (default: ../../models/minilm-l6)
- `EMBEDDING_MIN_BATCH_SIZE`: Min batch size (default: 16)
- `EMBEDDING_MAX_BATCH_SIZE`: Max batch size (default: 256)
- `EMBEDDING_BATCH_TIMEOUT_MS`: Batch timeout (default: 500)
- `EMBEDDING_ONNX_THREADS`: ONNX thread count (default: 4)

Example:
```bash
EMBEDDING_PORT=9000 EMBEDDING_ONNX_THREADS=8 uvicorn app.main:app
```

## Docker

### Build and Run
```bash
docker-compose up
```

### Build Only
```bash
docker build -t doctown-embedding .
```

### Run Container
```bash
docker run -p 8000:8000 -v $(pwd)/../../models/minilm-l6:/app/models/minilm-l6:ro doctown-embedding
```

## Troubleshooting

### Model Not Found
Ensure the model files exist:
```bash
ls -la ../../models/minilm-l6/
# Should show: model.onnx, tokenizer.json
```

### Port Already in Use
Change the port:
```bash
uvicorn app.main:app --port 8001
```

### Dependencies Won't Install
Make sure you have Python 3.10+:
```bash
python3 --version
```

Upgrade pip:
```bash
pip install --upgrade pip
```

### Import Errors
Make sure you're in the virtual environment:
```bash
source .venv/bin/activate
which python
# Should show: /path/to/workers/embedding/.venv/bin/python
```

## Performance Tips

1. **Batch Size**: Use 16-256 chunks per batch for optimal throughput
2. **Thread Count**: Set `EMBEDDING_ONNX_THREADS` to match your CPU cores
3. **Content Length**: Shorter texts embed faster (model truncates at 512 tokens)

## Example Client (Python)

```python
import httpx

async def embed_chunks(chunks):
    async with httpx.AsyncClient() as client:
        response = await client.post(
            "http://localhost:8000/embed",
            json={
                "batch_id": "my_batch",
                "chunks": [
                    {"chunk_id": f"c{i}", "content": chunk}
                    for i, chunk in enumerate(chunks)
                ]
            }
        )
        return response.json()

# Usage
chunks = ["def hello(): pass", "class Foo: pass"]
result = await embed_chunks(chunks)
print(f"Got {len(result['vectors'])} vectors")
```

## Next Steps

- Integrate with the ingest pipeline (M2.3)
- Connect to vector database
- Deploy to RunPod
- Add monitoring and metrics
