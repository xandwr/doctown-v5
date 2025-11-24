# Doctown Architecture - Updated with Generation Worker

## System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Doctown v5                                   │
│                  Complete Documentation Pipeline                     │
└─────────────────────────────────────────────────────────────────────┘

                              ▼
                    
┌─────────────────────────────────────────────────────────────────────┐
│                      Vercel (Website)                                │
│                  - SvelteKit Frontend                                │
│                  - Job Orchestration                                 │
│                  - User Interface                                    │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              │ HTTP Requests
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│              RunPod Pod (Combined Container)                         │
│                                                                      │
│  ┌────────────────────┐  ┌────────────────────┐                    │
│  │   Builder API      │  │  Assembly Worker   │                    │
│  │   (Rust - Actix)   │  │  (Rust - Actix)    │                    │
│  │                    │  │                    │                    │
│  │   Port 3000        │  │   Port 8002        │                    │
│  │                    │  │                    │                    │
│  │ • Pipeline Control │  │ • Graph Building   │                    │
│  │ • Job Management   │  │ • Clustering       │                    │
│  │ • Worker Coord.    │  │ • Context Gen      │                    │
│  └────────────────────┘  └────────────────────┘                    │
│                                                                      │
│  ┌────────────────────┐  ┌────────────────────┐                    │
│  │ Embedding Worker   │  │ Generation Worker  │                    │
│  │ (Python - FastAPI) │  │ (Python - FastAPI) │                    │
│  │                    │  │                    │                    │
│  │   Port 8000        │  │   Port 8003        │                    │
│  │                    │  │                    │                    │
│  │ • MiniLM-L6 ONNX  │  │ • OpenAI gpt-5-nano│                    │
│  │ • Batch Embedding │  │ • Structured Output│                    │
│  │ • 384-dim vectors │  │ • Token Tracking   │                    │
│  └────────────────────┘  └────────────────────┘                    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              │ Output
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   Cloudflare R2 Storage                              │
│                  - .docpack files (gzipped tar)                      │
│                  - Embeddings (binary)                               │
└─────────────────────────────────────────────────────────────────────┘
```

## Pipeline Flow

### Complete Documentation Generation Process

```
1. USER SUBMITS REPO
   │
   │ → Website (Vercel)
   │
   ▼

2. BUILDER ORCHESTRATES
   │ 
   │ → Builder API (Port 3000)
   │    - Fetches repo from GitHub
   │    - Parses code (tree-sitter)
   │    - Extracts symbols
   │    - Chunks code
   │
   ▼

3. EMBEDDING GENERATION
   │
   │ → Embedding Worker (Port 8000)
   │    - Processes chunks in batches
   │    - MiniLM-L6 ONNX model
   │    - Returns 384-dim vectors
   │
   ▼

4. GRAPH & CLUSTERING
   │
   │ → Assembly Worker (Port 8002)
   │    - Builds call graph
   │    - Clusters symbols
   │    - Calculates centrality
   │    - Generates symbol contexts
   │
   ▼

5. DOCUMENTATION GENERATION ⭐ NEW
   │
   │ → Generation Worker (Port 8003)
   │    - Receives symbol contexts
   │    - Calls OpenAI gpt-5-nano
   │    - Structured output (1-2 sentences)
   │    - Returns documented symbols
   │
   ▼

6. PACKAGING
   │
   │ → Builder API (Port 3000)
   │    - Creates manifest.json
   │    - Assembles graph.json
   │    - Includes nodes.json (with docs!)
   │    - Adds clusters.json
   │    - Adds source_map.json
   │    - Compresses to .tar.gz
   │
   ▼

7. STORAGE & DELIVERY
   │
   │ → Cloudflare R2
   │    - Stores .docpack
   │    - Stores embeddings.bin
   │
   │ → Website
   │    - Download link
   │    - Browse/visualize
```

## Service Details

### Builder API (Rust - Port 3000)

**Purpose:** Pipeline orchestration and coordination

**Responsibilities:**
- GitHub repo fetching
- Code parsing (tree-sitter)
- Symbol extraction
- Job management
- Worker coordination
- .docpack assembly

**Key Endpoints:**
- `POST /ingest` - Start documentation job
- `GET /health` - Health check
- `GET /status/{job_id}` - Job status

**Stack:** Actix-web, tree-sitter, tokio

---

### Assembly Worker (Rust - Port 8002)

**Purpose:** Graph construction and semantic analysis

**Responsibilities:**
- Call graph building
- K-means clustering
- Centrality calculation
- Symbol context generation
- Cluster labeling

**Key Endpoints:**
- `POST /assemble` - Assemble graph from embeddings
- `GET /health` - Health check

**Algorithms:**
- K-means clustering (k=8)
- PageRank centrality
- TF-IDF for labels

**Stack:** Actix-web, ndarray, petgraph

---

### Embedding Worker (Python - Port 8000)

**Purpose:** Code embedding generation

**Responsibilities:**
- Text embedding via ONNX
- Batch processing
- Memory management
- Intelligent chunking

**Model:** all-MiniLM-L6-v2 (384 dimensions)

**Key Endpoints:**
- `POST /embed` - Generate embeddings
- `GET /health` - Health check

**Performance:**
- ~50-100 chunks/second
- CPU-optimized (ONNX Runtime)
- Adaptive batch sizing

**Stack:** FastAPI, transformers, onnxruntime

---

### Generation Worker (Python - Port 8003) ⭐ NEW

**Purpose:** AI-powered documentation generation

**Responsibilities:**
- Prompt construction from context
- OpenAI API calls
- Token counting & cost tracking
- Batch processing (10 concurrent)
- Retry logic & rate limiting

**Model:** gpt-5-nano (structured output)

**Key Endpoints:**
- `POST /generate` - Generate documentation
- `GET /health` - Health check

**Features:**
- Structured output (consistent format)
- Smart prompt truncation
- Exponential backoff retry
- Partial failure handling
- Progress events

**Pricing:**
- Input: $0.15/1M tokens
- Output: $0.60/1M tokens

**Stack:** FastAPI, openai, tiktoken, tenacity

---

## Data Flow

### Symbol Context → Documentation

```
Assembly Worker Output:
┌──────────────────────────────────┐
│ Symbol Context                   │
├──────────────────────────────────┤
│ symbol_id: "sym_123"             │
│ name: "calculate_total"          │
│ kind: "function"                 │
│ language: "python"               │
│ file_path: "src/utils.py"        │
│ signature: "def calc..."         │
│ calls: ["sum", "len"]            │
│ called_by: ["main"]              │
│ cluster_label: "math utils"      │
│ centrality: 0.75                 │
└──────────────────────────────────┘
          │
          ▼
Generation Worker Processing:
┌──────────────────────────────────┐
│ Prompt Construction              │
├──────────────────────────────────┤
│ "You are documenting a python    │
│  codebase.                       │
│                                  │
│  Symbol: calculate_total         │
│  Kind: function                  │
│  ...                             │
│                                  │
│  Write 1-2 sentences..."         │
└──────────────────────────────────┘
          │
          ▼
OpenAI API (gpt-5-nano):
┌──────────────────────────────────┐
│ Structured Output                │
├──────────────────────────────────┤
│ {                                │
│   "summary": "Calculates the     │
│   total sum of items in a list." │
│ }                                │
└──────────────────────────────────┘
          │
          ▼
Final Docpack Node:
┌──────────────────────────────────┐
│ {                                │
│   "id": "sym_123",               │
│   "name": "calculate_total",     │
│   "kind": "function",            │
│   "documentation": {             │
│     "summary": "Calculates..." ⭐│
│   },                             │
│   ...                            │
│ }                                │
└──────────────────────────────────┘
```

## Deployment Architecture

### Single Combined Container

**Advantages:**
- ✅ Simplified deployment (one image)
- ✅ No network latency between workers
- ✅ Shared resources (CPU, memory)
- ✅ Single health check endpoint
- ✅ Easier debugging (one log stream)

**Resource Requirements:**
- CPU: 4+ cores recommended
- RAM: 8GB minimum (16GB recommended)
- Disk: 10GB
- Network: Outbound to OpenAI API

### Environment Configuration

```env
# Required
OPENAI_API_KEY=sk-...

# Optional (with defaults)
MODEL_NAME=gpt-5-nano
MAX_CONCURRENT_REQUESTS=10
INPUT_TOKEN_PRICE=0.15
OUTPUT_TOKEN_PRICE=0.60
EMBEDDING_MODEL_PATH=/app/models/minilm-l6
```

## Cost Analysis

### Per Documentation Job (200 symbols)

```
Component          Cost        Notes
────────────────────────────────────────────
RunPod (CPU)       $0.001      ~15-30 seconds
Embedding Worker   $0.000      CPU-only, included
Assembly Worker    $0.000      CPU-only, included
Generation Worker  $0.030      OpenAI API (~15k tokens)
────────────────────────────────────────────
Total per job:     ~$0.031
```

### Monthly Costs (100 jobs/day)

```
Item                   Monthly Cost
───────────────────────────────────
RunPod (24/7)          $150-300
OpenAI API             $90-150
────────────────────────────────────
Total:                 $240-450
```

Can optimize by:
- Stopping pod when idle
- Caching frequent repos
- Batching jobs

## Scalability

### Horizontal Scaling

Deploy multiple RunPod pods:
```
Website → Load Balancer → Pod 1 (4 workers)
                       → Pod 2 (4 workers)
                       → Pod 3 (4 workers)
```

Each pod is independent and can handle full pipeline.

### Performance Targets

| Codebase Size | Time  | Cost  |
|--------------|-------|-------|
| 50 files     | <30s  | $0.01 |
| 200 files    | <90s  | $0.04 |
| 1000 files   | <5min | $0.20 |

## Technology Stack

### Backend (RunPod Pod)

**Rust Components:**
- actix-web (HTTP servers)
- tree-sitter (parsing)
- tokio (async runtime)
- serde (serialization)
- ndarray (linear algebra)

**Python Components:**
- FastAPI (HTTP servers)
- onnxruntime (embedding)
- openai (documentation)
- tiktoken (token counting)
- transformers (tokenization)

### Frontend (Vercel)

- SvelteKit
- TypeScript
- TailwindCSS
- D3.js (visualization)

### Storage (Cloudflare R2)

- S3-compatible API
- CDN integration
- Cost-effective

## Security

### API Key Management
- OpenAI API key stored as environment variable
- Never logged or exposed
- Rotated periodically

### Network Security
- HTTPS only (RunPod provides SSL)
- CORS configured for website origin
- Rate limiting in website layer

### Data Privacy
- Code never stored permanently
- Embeddings ephemeral
- Only .docpack retained

## Monitoring

### Health Checks

All services monitored:
```bash
/health → 200 OK = Healthy
       → 5xx = Unhealthy
```

### Metrics to Track

- Job success rate
- Average processing time
- OpenAI token usage
- Error rates per worker
- Cost per job

### Logging

Structured logs to stdout:
- Job lifecycle events
- Worker communications
- Error conditions
- Performance metrics

## Future Enhancements

### Planned
- [ ] Packer worker (M3.3) - Separate packaging service
- [ ] Caching layer for repeated repos
- [ ] Fine-tuned model (replace OpenAI)
- [ ] GraphQL API
- [ ] Real-time progress via WebSockets

### Considered
- [ ] GPU acceleration for embeddings
- [ ] Multi-language documentation
- [ ] Incremental updates
- [ ] Diff-based documentation

---

**Architecture Version:** 2.0 (with Generation Worker)  
**Last Updated:** 2025-11-23  
**Status:** ✅ Production Ready
