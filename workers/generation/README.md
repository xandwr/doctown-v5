# Doctown Generation Worker

OpenAI-based documentation generation service using gpt-5-nano with structured output for generating concise, high-quality symbol documentation.

## Features

- **Structured Output**: Uses OpenAI's structured output API to ensure consistent documentation format
- **Token Counting**: Accurate token counting with tiktoken for cost tracking
- **Smart Truncation**: Automatically truncates prompts that exceed token limits
- **Concurrent Generation**: Processes multiple symbols in parallel (default: 10 concurrent requests)
- **Retry Logic**: Exponential backoff retry for rate limits and transient errors
- **Event Emission**: Emits structured events for progress tracking
- **Cost Tracking**: Calculates and reports token usage and costs

## Setup

### Prerequisites

- Python 3.10+
- OpenAI API key

### Installation

```bash
./setup.sh
```

This will:
1. Create a virtual environment
2. Install all dependencies

### Configuration

Set the required environment variable:

```bash
export OPENAI_API_KEY=your-openai-api-key-here
```

Optional environment variables:

```bash
export MODEL_NAME=gpt-5-nano              # Default: gpt-5-nano
export MAX_CONCURRENT_REQUESTS=10         # Default: 10
export PORT=8003                          # Default: 8003
export HOST=0.0.0.0                       # Default: 0.0.0.0
```

## Run

```bash
./run.sh
```

Or with custom settings:

```bash
MODEL_NAME=gpt-4o-mini PORT=8080 ./run.sh
```

## API

### Health Check

```
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "model": "gpt-5-nano",
  "ready": true
}
```

### Generate Documentation

```
POST /generate
```

**Request:**
```json
{
  "job_id": "job_123",
  "symbols": [
    {
      "symbol_id": "sym_1",
      "context": {
        "symbol_id": "sym_1",
        "name": "calculate_total",
        "kind": "function",
        "language": "python",
        "file_path": "src/utils.py",
        "signature": "def calculate_total(items: list[int]) -> int",
        "calls": ["sum", "len"],
        "called_by": ["main"],
        "imports": ["typing"],
        "related_symbols": ["validate_items"],
        "cluster_label": "math utilities",
        "centrality": 0.75
      }
    }
  ]
}
```

**Response:**
```json
{
  "documented_symbols": [
    {
      "symbol_id": "sym_1",
      "summary": "Calculates the total sum of integer items in a list.",
      "tokens_used": 87
    }
  ],
  "total_tokens": 87,
  "total_cost": 0.000013
}
```

## Model

This worker uses **gpt-5-nano** (or the configured model) with structured output for documentation generation.

### Pricing (gpt-5-nano)

- Input: $0.15 per 1M tokens
- Output: $0.60 per 1M tokens

### Prompt Structure

The worker constructs prompts with:
- Programming language context
- Symbol metadata (name, kind, signature, file path)
- Relational information (calls, called_by)
- Semantic clustering (cluster label)
- Importance score (centrality)
- Clear instructions for 1-2 sentence documentation

Prompts are automatically truncated if they exceed 2000 tokens.

## Testing

```bash
# Run all tests
pytest

# Run specific test file
pytest tests/test_token_counter.py

# Run with coverage
pytest --cov=app --cov-report=html
```

## Docker

### Build

```bash
docker build -t doctown-generation-worker .
```

### Run

```bash
docker run -p 8003:8003 \
  -e OPENAI_API_KEY=your-key \
  doctown-generation-worker
```

### Docker Compose

```bash
# Set your API key
export OPENAI_API_KEY=your-key

# Run
docker-compose up
```

## Events

The worker emits events to stdout in JSON format:

### generation.started.v1
```json
{
  "type": "generation.started.v1",
  "payload": {
    "symbol_count": 65,
    "estimated_tokens": 15000
  }
}
```

### generation.symbol_documented.v1
```json
{
  "type": "generation.symbol_documented.v1",
  "payload": {
    "symbol_id": "sym_main_fn",
    "token_count": 87
  }
}
```

### generation.completed.v1
```json
{
  "type": "generation.completed.v1",
  "status": "success",
  "payload": {
    "total_tokens": 14200,
    "total_cost": 0.0028,
    "duration_ms": 12000,
    "warnings": []
  }
}
```

## Architecture

- **token_counter.py**: Token counting and cost calculation using tiktoken
- **prompt_builder.py**: Prompt construction with smart truncation
- **openai_client.py**: OpenAI API wrapper with structured output and retry logic
- **generator.py**: Batch processing with concurrency control
- **main.py**: FastAPI application with /health and /generate endpoints
- **events.py**: Event emission for progress tracking
- **schemas.py**: Pydantic models for request/response validation
- **config.py**: Configuration management from environment variables

## Development

### Code Style

The project follows standard Python conventions. Format with:

```bash
black app tests
```

### Adding New Features

1. Add implementation to appropriate module
2. Add tests to corresponding test file
3. Update README with new functionality
4. Test locally before deploying

## Troubleshooting

### Rate Limits

The worker automatically retries on rate limit errors (429) with exponential backoff. If you consistently hit rate limits, consider:

- Reducing `MAX_CONCURRENT_REQUESTS`
- Adding delays between batches
- Upgrading your OpenAI API tier

### Token Limit Exceeded

If prompts exceed token limits, the worker automatically truncates signatures. If this is too aggressive:

- Increase `MAX_PROMPT_TOKENS` in config
- Reduce the amount of relational data included

### Cost Optimization

To reduce costs:

- Use a smaller model (if gpt-5-nano has alternatives)
- Reduce `MAX_CONCURRENT_REQUESTS` to batch requests
- Pre-filter symbols to only document high-centrality ones
