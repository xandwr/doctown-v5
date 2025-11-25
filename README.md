# Doctown v5

Intelligent code documentation and understanding platform.

## Architecture

Doctown uses a **serverless-first architecture** for cost efficiency:

```
                     ┌─────────────────┐
                     │   Website       │
                     │   (Vercel)      │
                     └────────┬────────┘
                              │
              ┌───────────────┴───────────────┐
              │                               │
              ▼                               ▼
┌─────────────────────────┐     ┌─────────────────────────┐
│   Builder (Serverless)  │     │  Embedder (Serverless)  │
│   RunPod CPU            │     │  RunPod GPU             │
│                         │     │                         │
│   • GitHub fetch        │     │  • ONNX MiniLM-L6      │
│   • Tree-sitter parsing │     │  • Batch embedding      │
│   • Symbol extraction   │────▶│  • 384-dim vectors      │
│   • Chunking            │     │                         │
│   • Graph assembly      │◀────│                         │
│   • Docpack packaging   │     └─────────────────────────┘
└────────────┬────────────┘
             │
             ▼
┌─────────────────────────┐
│   Cloudflare R2         │
│   .docpack storage      │
└─────────────────────────┘
```

### Pipeline Flow

1. **Ingest** - Builder fetches repo, parses code, extracts symbols, creates chunks
2. **Embed** - Embedder generates 384-dim vectors for all chunks (GPU-accelerated)
3. **Assemble** - Builder clusters, labels, builds graph, packages .docpack
4. **Upload** - .docpack uploaded to R2 for download

## Project Structure

```
crates/
├── doctown-common/     # Shared types, IDs, errors
├── doctown-events/     # Event system for streaming
├── doctown-ingest/     # Parsing, symbols, chunking, GitHub client
├── doctown-assembly/   # Clustering, graph building, docpack packing
└── doctown-docpack/    # Docpack format utilities

workers/
├── embedding/          # ONNX embedding worker (Python/FastAPI)
└── generation/         # LLM doc generation worker (Python/FastAPI)

builder/                # Main API server entry point (Rust/Actix)
website/                # SvelteKit frontend
specs/                  # Architecture and format specifications
models/                 # ONNX model files
```

## Local Development

```bash
./dev.sh        # Start backend + frontend
./stop.sh       # Stop services
```

## Supported Languages

- Rust
- Python
- TypeScript/JavaScript
- Go
