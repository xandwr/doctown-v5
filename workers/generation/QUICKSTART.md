# Generation Worker Quickstart

## 1. Setup

```bash
./setup.sh
```

## 2. Configure

```bash
export OPENAI_API_KEY=your-openai-api-key-here
```

## 3. Run

```bash
./run.sh
```

## 4. Test

```bash
curl http://localhost:8003/health
```

Expected response:
```json
{
  "status": "healthy",
  "model": "gpt-5-nano",
  "ready": true
}
```

## 5. Generate Documentation

```bash
curl -X POST http://localhost:8003/generate \
  -H "Content-Type: application/json" \
  -d '{
    "job_id": "test_job",
    "symbols": [{
      "symbol_id": "sym_1",
      "context": {
        "symbol_id": "sym_1",
        "name": "calculate_total",
        "kind": "function",
        "language": "python",
        "file_path": "src/utils.py",
        "signature": "def calculate_total(items: list[int]) -> int",
        "calls": ["sum"],
        "called_by": ["main"],
        "imports": [],
        "related_symbols": [],
        "cluster_label": "utilities",
        "centrality": 0.8
      }
    }]
  }'
```

## Running Tests

```bash
source .venv/bin/activate
pytest
```

## Docker

```bash
export OPENAI_API_KEY=your-key
docker-compose up
```

That's it! The generation worker is now running on port 8003.
