# Doctown Embedding Worker

CPU-optimized embedding service using all-MiniLM-L6-v2 ONNX model with intelligent memory management to prevent crashes on large codebases.

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
- **Runtime**: CPU-only with sequential execution for memory efficiency
- **Batch size**: Min 8, Max 64 (adaptive based on memory pressure)
- **Memory management**: Monitors RAM usage and adjusts batch size dynamically
- **Max memory**: 70% of system RAM (configurable)

## Features

### Intelligent Memory Management
- **Adaptive batching**: Automatically adjusts batch size based on available memory
- **Memory monitoring**: Tracks process memory usage in real-time
- **Aggressive GC**: Forces garbage collection between large batches
- **Graceful degradation**: Reduces batch size instead of crashing

### Large Codebase Support
The worker can handle giant repositories (numpy, ort, etc.) by:
1. Splitting large batches into smaller chunks
2. Processing sequentially to avoid memory spikes
3. Monitoring memory between chunks
4. Collecting garbage every 10 chunks
5. Dynamically adjusting batch size if memory pressure detected

## Configuration

Environment variables:
- `EMBEDDING_MIN_BATCH_SIZE=8` - Minimum batch size
- `EMBEDDING_MAX_BATCH_SIZE=64` - Maximum batch size
- `EMBEDDING_MAX_MEMORY_PERCENT=70.0` - Max RAM usage (%)
- `EMBEDDING_ADAPTIVE_BATCHING=true` - Enable adaptive batching
- `EMBEDDING_ONNX_THREADS=8` - Number of CPU threads (capped at 8)

## Docker

```bash
# Build
docker build -t doctown-embedding-worker .

# Run (CPU-only)
docker run -p 8000:8000 doctown-embedding-worker

# Run with custom memory limit
docker run -p 8000:8000 -e EMBEDDING_MAX_MEMORY_PERCENT=60.0 doctown-embedding-worker
```
