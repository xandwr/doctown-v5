# Doctown Production Architecture v5.0

## Overview

Doctown is an automated documentation generation platform. Users provide a GitHub repository URL, and the system produces a `.docpack` file
containing semantically-rich, LLM-generated documentation derived from static analysis.

The core insight: **don't ask the LLM to understand code—tell it what you already computed, and ask it to verbalize.**

```
AST parsing + embeddings + graph analysis = complete understanding
LLM = verbalization layer with zero guesswork
```

---

## System Components

### 1. Website (Vercel)

**Role:** User interface and job orchestration client.

**Responsibilities:**
- User authentication (GitHub OAuth)
- Repository URL input and validation
- Job submission to Coordinator
- Real-time progress streaming via SSE/WebSocket
- Docpack browsing, downloading, and visualization
- User library management
- Stripe payment integration for premium tiers

**Does NOT:**
- Talk directly to workers
- Store job state (stateless frontend)
- Process any pipeline logic

**Tech stack:** SvelteKit, deployed on Vercel.

---

### 2. Coordinator (RunPod Serverless - CPU)

**Role:** Central orchestrator. The only stateful component.

**Responsibilities:**
- Receive job requests from Website
- Validate repository accessibility
- Estimate cost before work begins
- Assign workers to stages
- Track job state and progress
- Aggregate events from all workers
- Stream progress back to Website
- Handle retries and failures
- Emit final `job.completed.v1` event

**Communication:**
- Inbound: HTTPS from Website
- Outbound: Message queue to workers
- Events: Receives all worker events, forwards to Website

**State storage:** Redis or PostgreSQL for active job tracking.

**Scaling:** Single coordinator instance per job. Coordinator is lightweight—most compute happens in workers.

---

### 3. Ingest Worker (RunPod Serverless - CPU)

**Role:** Download repository and produce semantic chunks.

**Responsibilities:**
- Clone or download repository ZIP from GitHub
- Extract files to memory (no disk persistence)
- Detect file types and languages
- Parse AST for supported languages (Rust, Python, TypeScript, Go, etc.)
- Split code into semantic chunks (functions, classes, modules)
- Normalize paths and metadata
- Stream `chunk.created` events as soon as chunks are ready

**Key behavior:** Streaming output. Does NOT wait for full repo processing before emitting chunks. This enables pipeline parallelism.

**Input:**
```json
{
  "repo_url": "https://github.com/user/repo",
  "git_ref": "main",
  "job_id": "job_abc123"
}
```

**Output events:**
- `ingest.started.v1`
- `ingest.file_detected.v1` (per file)
- `ingest.file_skipped.v1` (per skipped file)
- `ingest.chunk_created.v1` (per chunk, streamed)
- `ingest.completed.v1`

**Tech stack:** Rust binary for performance. Actix-web for HTTP. Tree-sitter for AST parsing.

---

### 4. Embedding Worker Pool (RunPod Serverless - GPU)

**Role:** Convert text chunks into vector embeddings.

**Responsibilities:**
- Receive batches of chunks from message queue
- Group into large mega-batches for GPU efficiency
- Generate embeddings using ONNX runtime or sentence-transformers
- Return vectors with chunk IDs
- Die after job completion (stateless, no GPU hoarding)

**Key behavior:** Batched processing. Waits for enough chunks to fill a batch before processing. Balances latency vs throughput.

**Batch strategy:**
- Minimum batch size: 16 chunks
- Maximum batch size: 256 chunks
- Timeout: 500ms (process whatever is available)

**Input:**
```json
{
  "batch_id": "batch_001",
  "chunks": [
    { "chunk_id": "chunk_a", "content": "fn main() { ... }" },
    { "chunk_id": "chunk_b", "content": "def hello(): ..." }
  ]
}
```

**Output events:**
- `embedding.started.v1`
- `embedding.batch_started.v1`
- `embedding.chunk_vector.v1` (per chunk)
- `embedding.batch_completed.v1`
- `embedding.completed.v1`

**Model:** `all-MiniLM-L6-v2` (384 dimensions) or `INSTRUCTOR` for instruction-tuned embeddings.

**Tech stack:** Python with ONNX Runtime (CUDA) or sentence-transformers. RunPod GPU serverless (RTX 3090 or A4000).

---

### 5. Semantic Assembly Worker (RunPod Serverless - CPU)

**Role:** Build the docpack graph from AST + embeddings.

**Responsibilities:**
- Receive all embeddings and AST metadata
- Cluster embeddings using HDBSCAN or k-means
- Label clusters based on content similarity
- Build the docpack graph:
  - Nodes: symbols (functions, classes, modules, files)
  - Edges: calls, imports, type references, trait implementations
- Compute graph metrics:
  - Centrality (PageRank or eigenvector)
  - Complexity scores
  - Module coupling
- Generate relatedness edges from embedding similarity
- Produce structured context for each symbol (for LLM stage)

**Key behavior:** Waits for ALL embeddings before processing. This is the sync point in the pipeline.

**Output artifacts:**
- `graph.json` - Full docpack graph
- `nodes.json` - Symbol metadata
- `clusters.json` - Semantic clusters
- `symbol_contexts.json` - Pre-computed LLM prompts per symbol

**Output events:**
- `assembly.started.v1`
- `assembly.cluster_created.v1` (per cluster)
- `assembly.graph_completed.v1`
- `assembly.completed.v1`

**Tech stack:** Rust or Python. numpy/scipy for clustering. petgraph or networkx for graph operations.

---

### 6. Generation Worker (RunPod Serverless - CPU)

**Role:** Generate human-readable documentation via LLM.

**Responsibilities:**
- Receive symbol contexts from Assembly
- Batch symbols into OpenAI API requests
- Generate 1-2 sentence descriptions per symbol
- Generate module overviews and architecture summaries
- Apply user-provided templates or style guides (premium)
- Track token usage and costs

**Key behavior:** Massively parallel. Each symbol is independent once context is computed. Fire many requests concurrently.

**Prompt structure:**
```
You are documenting a {language} codebase.

Symbol: {symbol_name}
Kind: {function|class|module|...}
File: {file_path}
Signature: {signature}

Calls: {list of functions this calls}
Called by: {list of functions that call this}
Imports: {imports used}
Cluster: {semantic cluster label}
Centrality: {importance score}

Write 1-2 sentences describing what this symbol does.
```

**Output events:**
- `generation.started.v1`
- `generation.symbol_documented.v1` (per symbol)
- `generation.completed.v1`

**Tech stack:** Python. OpenAI API with batch endpoint. tiktoken for token counting.

**Cost control:** Token estimates computed before generation. User can approve or cancel.

---

### 7. Packer Worker (RunPod Serverless - CPU)

**Role:** Assemble final `.docpack` artifact.

**Responsibilities:**
- Collect all artifacts from previous stages
- Validate schema versions
- Compress into `.docpack` format (gzipped JSON or MessagePack)
- Compute checksum (SHA-256)
- Upload to R2 storage
- Write index entry to database
- Return final URL

**Key behavior:** Deterministic and reproducible. Same inputs = same outputs = same checksum.

**Docpack structure:**
```
docpack/
├── manifest.json       # Version, checksum, metadata
├── graph.json          # Full symbol graph
├── nodes.json          # Symbol data with documentation
├── clusters.json       # Semantic clusters
├── embeddings.bin      # Raw vectors (optional, for search)
└── source_map.json     # File paths and byte ranges
```

**Output events:**
- `pack.started.v1`
- `pack.completed.v1`

**Tech stack:** Rust or Python. zstd or gzip compression. Cloudflare R2 for storage.

---

### 8. Storage Layer

#### R2 (Cloudflare)

**Role:** Docpack artifact storage.

**Contents:**
- `.docpack` files (immutable, content-addressed)
- User uploads (private repos, premium)

**Access:** Pre-signed URLs for downloads. No direct public access.

#### PostgreSQL (Neon or Supabase)

**Role:** Persistent metadata storage.

**Tables:**
- `users` - Auth, plan tier, Stripe customer ID
- `jobs` - Job history, status, costs
- `docpacks` - Index of generated docpacks
- `repos` - Linked GitHub repositories

#### Redis (Upstash)

**Role:** Ephemeral job state and caching.

**Contents:**
- Active job state (events, progress)
- Rate limiting
- Session tokens

---

## Data Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                  WEBSITE                                     │
│                                 (Vercel)                                     │
└─────────────────────────────────────┬───────────────────────────────────────┘
                                      │
                                      │ HTTPS: Submit job, poll status
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                               COORDINATOR                                    │
│                           (RunPod CPU Serverless)                            │
│                                                                              │
│  • Validates repo                                                            │
│  • Estimates cost                                                            │
│  • Dispatches to workers                                                     │
│  • Aggregates events                                                         │
│  • Streams progress to Website                                               │
└───────┬─────────────────────────────────────────────────────────────────────┘
        │
        │ Message Queue (Redis pub/sub or SQS)
        │
        ▼
┌───────────────────┐     ┌───────────────────┐     ┌───────────────────┐
│   INGEST WORKER   │     │  EMBEDDING POOL   │     │ ASSEMBLY WORKER   │
│   (RunPod CPU)    │────▶│   (RunPod GPU)    │────▶│   (RunPod CPU)    │
│                   │     │                   │     │                   │
│ • Clone repo      │     │ • Batch chunks    │     │ • Cluster vectors │
│ • Parse AST       │     │ • Generate vectors│     │ • Build graph     │
│ • Stream chunks   │     │ • GPU acceleration│     │ • Compute metrics │
└───────────────────┘     └───────────────────┘     └─────────┬─────────┘
                                                              │
                                                              ▼
                          ┌───────────────────┐     ┌───────────────────┐
                          │  PACKER WORKER    │◀────│ GENERATION WORKER │
                          │   (RunPod CPU)    │     │   (RunPod CPU)    │
                          │                   │     │                   │
                          │ • Compress        │     │ • OpenAI API      │
                          │ • Checksum        │     │ • Batch requests  │
                          │ • Upload to R2    │     │ • Token tracking  │
                          └─────────┬─────────┘     └───────────────────┘
                                    │
                                    ▼
                          ┌───────────────────┐
                          │        R2         │
                          │   (Cloudflare)    │
                          │                   │
                          │ • .docpack files  │
                          │ • Pre-signed URLs │
                          └───────────────────┘
```

---

## Pipeline Parallelism

The key optimization is **streaming between stages**:

### Without streaming (v1-v4):
```
Ingest ALL ──────────────────▶ Embed ALL ──────────────────▶ Assemble
     [5s wait]                      [30s wait]                  [2s]

Total: 37s, GPU idle for 5s at start
```

### With streaming (v5):
```
Ingest chunk 1 ─▶ Embed chunk 1 ─┐
Ingest chunk 2 ─▶ Embed chunk 2 ─┤
Ingest chunk 3 ─▶ Embed chunk 3 ─┼──▶ Assemble (when all done)
...                              │
Ingest chunk N ─▶ Embed chunk N ─┘

Total: ~15s, GPU saturated throughout
```

**Sync points:**
1. Assembly waits for ALL embeddings (must have complete picture)
2. Generation waits for Assembly (needs graph context)
3. Packer waits for Generation (needs documentation)

**Parallelism:**
1. Ingest → Embedding: streaming (no wait)
2. Generation: parallel API calls per symbol
3. Multiple jobs: independent, can run concurrently

---

## Message Queue Design

Workers communicate via events (see [events.v1.md](events.v1.md)).

**Queue topology:**
```
coordinator.jobs          # New job requests
ingest.tasks              # Ingest work items
embedding.batches         # Chunk batches for GPU
assembly.tasks            # Assembly work items
generation.tasks          # Generation work items
pack.tasks                # Packing work items

events.{job_id}           # Per-job event stream (fan-in to coordinator)
```

**Delivery guarantees:**
- At-least-once delivery
- Idempotency via `idempotency_key` in event envelope
- Retry with exponential backoff

**Implementation options:**
- Redis Streams (simple, good for MVP)
- AWS SQS + SNS (managed, scalable)
- NATS (high performance, self-hosted)

---

## Failure Handling

### Worker failures

Workers are stateless. If a worker dies:
1. Coordinator detects timeout (no events received)
2. Coordinator re-dispatches work to new worker
3. New worker starts fresh (idempotent)

### Partial failures

If some chunks fail to embed:
1. `embedding.batch_completed.v1` with `status: failed`
2. Coordinator retries failed batch (up to 3 attempts)
3. If still failing, continue without those chunks
4. `pack.completed.v1` with `status: partial` and `skipped_files[]`

### Unrecoverable failures

If critical stage fails completely:
1. `{stage}.completed.v1` with `status: failed`
2. Coordinator emits `job.completed.v1` with `status: failed`
3. User notified, no charge applied

---

## Cost Model

### Compute costs (RunPod)

| Worker | Instance | Cost/sec | Typical duration |
|--------|----------|----------|------------------|
| Coordinator | CPU | $0.0002 | 30-60s |
| Ingest | CPU | $0.0002 | 2-5s |
| Embedding | GPU (3090) | $0.0019 | 5-15s |
| Assembly | CPU | $0.0002 | 2-5s |
| Generation | CPU | $0.0002 | 10-30s |
| Packer | CPU | $0.0002 | 1-2s |

### External costs

| Service | Cost |
|---------|------|
| OpenAI (GPT-4o-mini) | ~$0.001 per 1K tokens |
| R2 Storage | $0.015/GB/month |
| R2 Egress | Free (within limits) |

### Example job (ORT repo, ~50 files)

| Stage | Duration | Cost |
|-------|----------|------|
| Ingest | 2s | $0.0004 |
| Embedding | 8s | $0.0152 |
| Assembly | 3s | $0.0006 |
| Generation | 15s | $0.0030 + ~$0.003 OpenAI |
| Packer | 1s | $0.0002 |
| **Total** | **29s** | **~$0.02** |

Target margin: 10x → charge $0.20 per job or include in subscription.

---

## Plan Tiers

| Tier | Price | Jobs/month | Features |
|------|-------|------------|----------|
| Free | $0 | 5 | Public repos only, basic docs |
| Creator | $12/mo | 100 | Private repos, full docs, priority queue |
| Team | $49/mo | 500 | Team sharing, custom templates, API access |
| Enterprise | Custom | Unlimited | Self-hosted, SLA, dedicated support |

---

## Security

### Repository access

- Public repos: No auth required
- Private repos: GitHub OAuth token (user grants access)
- Tokens stored encrypted, scoped to repo read-only

### Artifact access

- Docpacks are private by default
- Access via pre-signed R2 URLs (expire in 1 hour)
- Users can make docpacks public (shareable link)

### Worker isolation

- Each job runs in isolated container
- No persistent storage between jobs
- Workers cannot access other jobs' data

---

## Monitoring

### Metrics to track

| Metric | Source | Alert threshold |
|--------|--------|-----------------|
| Job success rate | Coordinator | < 95% |
| Avg job duration | Coordinator | > 60s |
| GPU utilization | Embedding worker | < 50% (inefficient batching) |
| OpenAI error rate | Generation worker | > 5% |
| Queue depth | Message queue | > 100 (backlog) |

### Logging

- All events stored in time-series DB (InfluxDB or TimescaleDB)
- `trace_id` links all events for a single job
- Searchable by `job_id`, `user_id`, `repo_url`

---

## Future Enhancements (v2+)

### Near-term
- `assembly.node_created.v1` event for finer progress tracking
- `snapshot_id` on generation events for LLM output caching
- Webhook notifications on job completion

### Medium-term
- Fine-tuned Doctown generation model (replace OpenAI)
- Incremental updates (re-process only changed files)
- GitHub App integration (auto-trigger on push)

### Long-term
- Self-hosted enterprise deployment
- Multi-language support beyond code (Markdown, comments)
- Interactive documentation explorer (WebGL graph viz)

---

## Summary

The Doctown pipeline is:

1. **Message-based** - Every stage communicates via events
2. **Streaming** - Chunks flow through pipeline without blocking
3. **Stateless workers** - Die after job, no resource hoarding
4. **GPU-efficient** - Batched embeddings, minimal idle time
5. **Cost-predictable** - Estimates before work begins
6. **Reproducible** - Same inputs = same docpack = same checksum
7. **Horizontally scalable** - Add workers, not bigger machines

Target performance: **Sub-30 seconds** for medium repos (50-100 files).

Target cost: **< $0.02** per job at scale.
