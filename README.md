# Doctown v5

Intelligent code documentation and understanding platform.

## Quick Start

Start the complete development environment:

```bash
./dev.sh
```

This will start:
- Backend API server on http://127.0.0.1:3000
- Frontend dev server on http://localhost:5173

For detailed setup instructions, see [DEV_SETUP.md](DEV_SETUP.md).

## Stopping Services

```bash
./stop.sh
```

Or press `Ctrl+C` when running `dev.sh`.

## Roadmap Overview

- [TODO.md](TODO.md)

|Milestone|User Feels|What Ships|
|---------|----------|----------|
|M1|"It understands my code"|Ingest worker, AST parsing, file tree UI
|M2|"It understands my code semantically"|Embeddings, clusters, graph explorer
|M3|"It explains my code"|LLM summaries, downloadable .docpack
|M4|"It integrates into workflows"|Full docpack spec, stable format
|M5|"It scales to real usage"|Coordinator, queues, distributed workers
|M6|"It's a real business"|Auth, payments, library, private repos

---