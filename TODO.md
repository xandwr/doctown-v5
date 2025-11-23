# Doctown v5 - Complete Implementation TODO

> **Philosophy:** Bottom-up, test-first, tiny tasks. Every layer is stable before building on top of it.
>
> **Shipping Strategy:** Six milestones, each a deployable product. Ship early, validate often.

---

## Legend

- `[ ]` Not started
- `[~]` In progress
- `[x]` Complete
- `[!]` Blocked
- `[T]` Test task
- `[S]` Ship gate (must pass before deploying milestone)

---

# ═══════════════════════════════════════════════════════════════════════════════
# MILESTONE 1: "Ingest-Only Doctown"
# ═══════════════════════════════════════════════════════════════════════════════
#
# User sees: Paste a repo URL → See all detected files, languages, and symbols
# User feels: "Whoa, it understands my repo?"
#
# Why ship here? This is a developer toy. Immediate validation that it works.
# Already more insight than GitHub's own UI.
# ═══════════════════════════════════════════════════════════════════════════════

## M1.0: Project Scaffolding

### M1.0.1 Repository Setup
- [x] Initialize Cargo workspace at root
- [x] Create `Cargo.toml` with workspace members
- [x] Create `.gitignore` for Rust/Node artifacts
- [x] Create `rust-toolchain.toml` (pin stable version)
- [x] Set up `clippy.toml` with strict lints
- [x] Set up `rustfmt.toml` for consistent formatting

### M1.0.2 Workspace Structure
- [x] Create `crates/` directory for all Rust crates
- [x] Create `crates/doctown-common/` (shared types, errors, utils)
- [x] Create `crates/doctown-events/` (event envelope, serialization)
- [x] Create `crates/doctown-ingest/` (ingest worker)
- [x] Create `website/` directory for SvelteKit app

### M1.0.3 CI Foundation
- [x] Create `.github/workflows/rust.yml` for Rust CI
- [x] Add `cargo fmt --check` step
- [x] Add `cargo clippy -- -D warnings` step
- [x] Add `cargo test` step
- [x] Add `cargo build --release` step

---

## M1.1: Core Types (`doctown-common`)

### M1.1.1 Identifier Types
- [x] Define `JobId` newtype with validation
- [x] Define `ChunkId` newtype with validation
- [x] Define `SymbolId` newtype with validation
- [x] Define `EventId` newtype with UUID generation
- [x] Define `TraceId` newtype
- [x] [T] Unit tests for all newtype validation
- [x] [T] Unit tests for serialization roundtrip (serde)

### M1.1.2 Domain Types
- [x] Define `ByteRange` struct (start, end)
- [x] Define `Language` enum (Rust, Python, TypeScript, Go, JavaScript)
- [x] Define `SymbolKind` enum (Function, Class, Module, Struct, Trait, Enum, Method, Const)
- [x] [T] Unit tests for Language from file extension
- [x] [T] Unit tests for SymbolKind display

### M1.1.3 Error Types
- [x] Define `DocError` enum with variants for M1 errors
- [x] Implement `std::error::Error` for `DocError`
- [x] Implement `From` conversions for io::Error, reqwest::Error
- [x] [T] Unit tests for error display formatting

---

## M1.2: Event System (`doctown-events`)

### M1.2.1 Event Envelope
- [x] Define `Envelope` struct matching spec
- [x] Define `Context` struct (repo_url, git_ref, user_id)
- [x] Define `Meta` struct (producer_version, trace_id, idempotency_key, tags)
- [x] Implement `Envelope::new()` with auto-generated fields
- [x] Implement `Envelope::with_parent()` for causality chains
- [x] Implement timestamp generation (ISO 8601)
- [x] Implement sequence number tracking per job
- [x] [T] Unit tests for envelope creation
- [x] [T] Unit tests for JSON serialization matches spec exactly
- [x] [T] Snapshot test comparing output to spec example

### M1.2.2 Ingest Event Types (M1 only)
- [x] Define `EventType` enum (ingest events only for now)
- [x] Define `IngestStartedPayload` struct
- [x] Define `IngestFileDetectedPayload` struct
- [x] Define `IngestFileSkippedPayload` struct
- [x] Define `IngestChunkCreatedPayload` struct
- [x] Define `IngestCompletedPayload` struct
- [x] Define `Status` enum (Success, Failed)
- [x] Implement typed event constructors for each type
- [x] [T] Unit tests for each payload serialization
- [x] [T] Snapshot tests comparing output to spec examples

### M1.2.3 Event Validation
- [x] Implement `Envelope::validate()` method
- [x] Validate required fields are present
- [x] Validate status only on terminal events (.completed)
- [x] [T] Unit tests for valid envelopes pass
- [x] [T] Unit tests for invalid cases rejected

---

## M1.3: Language Detection (`doctown-ingest`)

### M1.3.1 File Extension Mapping
- [x] Implement extension → Language mapping
- [x] Handle .rs → Rust
- [x] Handle .py → Python
- [x] Handle .ts, .tsx → TypeScript
- [x] Handle .js, .jsx → JavaScript
- [x] Handle .go → Go
- [x] Handle ambiguous extensions (.h → None, let tree-sitter decide)
- [x] [T] Unit tests for all supported extensions
- [x] [T] Unit tests for unknown extensions return None

### M1.3.2 Shebang Detection
- [x] Parse first line for #! patterns
- [x] Detect python, python3 → Python
- [x] Detect node, deno → JavaScript/TypeScript
- [x] [T] Unit tests for shebang parsing

---

## M1.4: Tree-sitter Setup (`doctown-ingest`)

### M1.4.1 Grammar Integration
- [ ] Add tree-sitter dependency
- [ ] Add tree-sitter-rust grammar
- [ ] Add tree-sitter-python grammar
- [ ] Create `Parser` struct that selects grammar by Language
- [ ] Implement parser pooling (reuse parsers)
- [ ] [T] Unit test: parse simple Rust file successfully
- [ ] [T] Unit test: parse simple Python file successfully
- [ ] [T] Unit test: parse invalid syntax returns partial tree

### M1.4.2 AST Traversal Utilities
- [ ] Implement tree cursor wrapper
- [ ] Implement node type matching helpers
- [ ] Implement text extraction from node
- [ ] [T] Unit tests for traversal helpers

---

## M1.5: Symbol Extraction - Rust (`doctown-ingest`)

### M1.5.1 Function Extraction
- [ ] Extract `fn` definitions
- [ ] Extract `async fn` definitions
- [ ] Capture function name
- [ ] Capture full signature (params + return type)
- [ ] Capture byte range (start, end)
- [ ] Capture visibility (pub, pub(crate), private)
- [ ] [T] Unit test: extract simple function
- [ ] [T] Unit test: extract async function
- [ ] [T] Unit test: extract generic function
- [ ] [T] Unit test: extract function with lifetime params

### M1.5.2 Struct Extraction
- [ ] Extract `struct` definitions
- [ ] Capture struct name
- [ ] Capture generics
- [ ] Capture field names (for context)
- [ ] Capture byte range
- [ ] Capture visibility
- [ ] [T] Unit test: extract simple struct
- [ ] [T] Unit test: extract tuple struct
- [ ] [T] Unit test: extract generic struct

### M1.5.3 Enum Extraction
- [ ] Extract `enum` definitions
- [ ] Capture enum name
- [ ] Capture variant names
- [ ] Capture byte range
- [ ] [T] Unit test: extract simple enum
- [ ] [T] Unit test: extract enum with data variants

### M1.5.4 Trait Extraction
- [ ] Extract `trait` definitions
- [ ] Capture trait name
- [ ] Capture method signatures
- [ ] Capture byte range
- [ ] [T] Unit test: extract trait with methods

### M1.5.5 Impl Block Extraction
- [ ] Extract `impl` blocks
- [ ] Capture target type
- [ ] Capture trait being implemented (if any)
- [ ] Capture methods within impl
- [ ] Capture byte range
- [ ] [T] Unit test: extract inherent impl
- [ ] [T] Unit test: extract trait impl

### M1.5.6 Module Extraction
- [ ] Extract `mod` declarations
- [ ] Capture module name
- [ ] Distinguish inline vs file modules
- [ ] Capture byte range
- [ ] [T] Unit test: extract inline module
- [ ] [T] Unit test: extract file module declaration

### M1.5.7 Other Items
- [ ] Extract `const` items
- [ ] Extract `static` items
- [ ] Extract `type` aliases
- [ ] Extract `macro_rules!` definitions
- [ ] [T] Unit tests for each item type

### M1.5.8 Rust Integration Test
- [ ] [T] Integration test: parse real Rust file (e.g., from std lib)
- [ ] [T] Verify all expected symbols extracted
- [ ] [T] Verify byte ranges are accurate

---

## M1.6: Symbol Extraction - Python (`doctown-ingest`)

### M1.6.1 Function Extraction
- [ ] Extract `def` function definitions
- [ ] Extract `async def` async functions
- [ ] Capture function name
- [ ] Capture parameters (with type hints if present)
- [ ] Capture return type hint if present
- [ ] Capture byte range
- [ ] Capture decorators
- [ ] [T] Unit test: extract simple function
- [ ] [T] Unit test: extract async function
- [ ] [T] Unit test: extract decorated function
- [ ] [T] Unit test: extract function with type hints

### M1.6.2 Class Extraction
- [ ] Extract `class` definitions
- [ ] Capture class name
- [ ] Capture base classes
- [ ] Capture method definitions within class
- [ ] Capture `__init__` specially
- [ ] Capture byte range
- [ ] Capture decorators (dataclass, etc.)
- [ ] [T] Unit test: extract simple class
- [ ] [T] Unit test: extract class with inheritance
- [ ] [T] Unit test: extract dataclass

### M1.6.3 Module-level Items
- [ ] Extract module-level variable assignments (NAME = ...)
- [ ] Extract `__all__` definition if present
- [ ] [T] Unit test: extract module constants

### M1.6.4 Python Integration Test
- [ ] [T] Integration test: parse real Python file
- [ ] [T] Verify all expected symbols extracted

---

## M1.7: Chunk Creation (`doctown-ingest`)

### M1.7.1 Chunk Structure
- [ ] Define `Chunk` struct (id, content, file_path, language, byte_range, symbol_kind)
- [ ] Define `ChunkMetadata` for additional context
- [ ] Implement `Chunk::new()` constructor
- [ ] [T] Unit test: chunk creation

### M1.7.2 Chunking Strategy
- [ ] Implement symbol-based chunking (one chunk per symbol)
- [ ] Handle nested symbols (methods inside class → separate chunks)
- [ ] Handle overlapping byte ranges (prefer smaller, more specific)
- [ ] [T] Unit test: chunking produces expected chunks
- [ ] [T] Unit test: nested symbols handled correctly

### M1.7.3 Large Symbol Handling
- [ ] Detect symbols exceeding size threshold (e.g., 4KB)
- [ ] Implement splitting with overlap for large symbols
- [ ] Maintain semantic boundaries (don't split mid-statement)
- [ ] [T] Unit test: large function split correctly
- [ ] [T] Unit test: overlap preserved

### M1.7.4 File-level Fallback
- [ ] Handle files with no extractable symbols
- [ ] Create single file-level chunk
- [ ] [T] Unit test: file without symbols gets file chunk

### M1.7.5 Chunk ID Generation
- [ ] Generate stable chunk IDs (hash of content + path + range)
- [ ] Ensure same input → same ID (deterministic)
- [ ] [T] Unit test: chunk ID stability
- [ ] [T] Unit test: different content → different ID

---

## M1.8: Repository Fetching (`doctown-ingest`)

### M1.8.1 GitHub URL Parsing
- [x] Parse GitHub URLs (https://github.com/user/repo)
- [x] Extract owner and repo name
- [x] Handle URLs with branch/tag/commit refs
- [x] Handle URLs with path (strip it)
- [x] [T] Unit tests for various URL formats
- [x] [T] Unit tests for invalid URLs rejected

### M1.8.2 GitHub API Client
- [ ] Implement basic GitHub API client
- [ ] Implement repository existence check (HEAD request)
- [ ] Implement repository metadata fetch (size, default branch)
- [ ] Implement ref resolution (branch name → commit SHA)
- [ ] Handle rate limiting (check X-RateLimit headers)
- [ ] [T] Unit tests with mock HTTP responses
- [ ] [T] Integration test with real public repo (gated by env var)

### M1.8.3 Repository Download
- [ ] Implement ZIP archive download (codeload.github.com)
- [ ] Implement streaming extraction (don't load full ZIP in memory)
- [ ] Handle nested directories correctly
- [ ] Normalize paths (remove "repo-branch/" prefix)
- [ ] [T] Unit tests with sample ZIP file
- [ ] [T] Integration test: download real repo

### M1.8.4 File Filtering
- [ ] Implement binary file detection (check for null bytes)
- [ ] Define default ignore patterns:
  - [ ] `.git/`
  - [ ] `node_modules/`
  - [ ] `target/` (Rust)
  - [ ] `__pycache__/`
  - [ ] `.venv/`, `venv/`
  - [ ] `dist/`, `build/`
  - [ ] `*.lock` files (Cargo.lock, package-lock.json, etc.)
- [ ] Implement max file size limit (skip files > 1MB)
- [ ] Implement total repo size limit (abort if > 100MB)
- [ ] [T] Unit test: binary detection
- [ ] [T] Unit test: ignore patterns match
- [ ] [T] Unit test: size limits enforced

---

## M1.9: Ingest Worker HTTP API (`doctown-ingest`)

### M1.9.1 Server Setup
- [ ] Set up Actix-web server
- [ ] Configure CORS for website origin
- [ ] Configure request body size limits
- [ ] Implement graceful shutdown
- [ ] [T] Unit test: server starts and stops cleanly

### M1.9.2 Health Endpoint
- [ ] Implement `GET /health` endpoint
- [ ] Return `{"status": "ok", "version": "..."}`
- [ ] [T] Integration test: health endpoint responds

### M1.9.3 Ingest Endpoint
- [ ] Implement `POST /ingest` endpoint
- [ ] Define request schema:
  ```json
  {
    "repo_url": "https://github.com/user/repo",
    "git_ref": "main",
    "job_id": "job_abc123"
  }
  ```
- [ ] Implement request validation
- [ ] Return SSE stream of events
- [ ] [T] Unit test: request validation
- [ ] [T] Integration test: valid request returns SSE stream

### M1.9.4 Server-Sent Events Streaming
- [ ] Implement SSE response with correct content-type
- [ ] Implement event channel (mpsc) for internal → SSE bridge
- [ ] Format events as `data: {json}\n\n`
- [ ] Handle client disconnect gracefully (stop processing)
- [ ] Send keepalive comments every 15s
- [ ] [T] Unit test: SSE encoding correct
- [ ] [T] Integration test: events stream to client

---

## M1.10: Ingest Pipeline (`doctown-ingest`)

- [x] M1.10.1: Unzip downloaded repository archive
- [x] M1.10.2: Walk through extracted files and process them
- [x] M1.10.3: Extract symbols for each file

### M1.10.1 Pipeline Orchestration
- [x] Wire up: receive request → validate → download → parse → stream
- [x] Create event emitter channel
- [x] Run pipeline in spawned task
- [x] Handle cancellation on client disconnect
- [x] [T] Unit test: pipeline runs to completion

### M1.10.2 Event Emission
- [ ] Emit `ingest.started.v1` at pipeline start
- [ ] Emit `ingest.file_detected.v1` for each processable file
- [ ] Emit `ingest.file_skipped.v1` for each skipped file (with reason)
- [ ] Emit `ingest.chunk_created.v1` for each chunk (streamed immediately)
- [ ] Emit `ingest.completed.v1` at end with totals
- [ ] [T] Verify event sequence is valid (started → ... → completed)
- [ ] [T] Verify event payloads match spec

### M1.10.3 Streaming Behavior
- [ ] Emit chunks as soon as each file is parsed (don't batch)
- [ ] Process files concurrently (bounded parallelism)
- [ ] Keep memory usage bounded (don't hold all chunks)
- [ ] [T] Unit test: chunks stream incrementally
- [ ] [T] Benchmark: memory stays under 256MB for 1000-file repo

### M1.10.4 Error Handling
- [ ] Handle download failures → emit failed status
- [ ] Handle parse errors → skip file, emit warning, continue
- [ ] Handle timeout → emit failed status
- [ ] Always emit completed event (even on failure)
- [ ] [T] Unit test: download failure handled
- [ ] [T] Unit test: parse error doesn't abort pipeline

---

## M1.11: Website - Ingest UI (`website/`)

### M1.11.1 Project Setup
- [ ] Initialize SvelteKit project
- [ ] Set up TypeScript
- [ ] Set up Tailwind CSS
- [ ] Set up ESLint + Prettier
- [ ] [T] Verify dev server runs

### M1.11.2 Repository Input
- [ ] Create repo URL input component
- [ ] Implement URL validation (GitHub URL pattern)
- [ ] Show validation errors inline
- [ ] Create submit button
- [ ] Show loading state during submission
- [ ] [T] Component test: URL validation

### M1.11.3 SSE Client
- [ ] Implement EventSource wrapper
- [ ] Parse incoming JSON events
- [ ] Handle connection errors
- [ ] Handle reconnection
- [ ] [T] Unit test: event parsing

### M1.11.4 Progress Display
- [ ] Create progress panel component
- [ ] Show "Ingesting..." status
- [ ] Show file count (detected / skipped)
- [ ] Show chunk count (incrementing live)
- [ ] Show language breakdown
- [ ] Show current file being processed
- [ ] [T] Component test: progress updates

### M1.11.5 Results Display
- [ ] Create file tree component
- [ ] Show all detected files with language icons
- [ ] Show symbol count per file
- [ ] Create symbol list component
- [ ] Show symbols grouped by file
- [ ] Show symbol kind (fn, struct, class, etc.)
- [ ] Show symbol signature
- [ ] [T] Component test: file tree rendering
- [ ] [T] Component test: symbol list rendering

### M1.11.6 Error Handling
- [ ] Show error state if ingest fails
- [ ] Display error message from event
- [ ] Allow retry

---

## M1.12: Deployment - Milestone 1

### M1.12.1 Ingest Worker Deployment
- [ ] Create Dockerfile for ingest worker
- [ ] Build and test Docker image locally
- [ ] Set up RunPod serverless endpoint
- [ ] Deploy ingest worker to RunPod
- [ ] Verify `/health` endpoint accessible
- [ ] [T] Smoke test: ingest small repo via deployed endpoint

### M1.12.2 Website Deployment
- [ ] Configure Vercel project
- [ ] Set environment variables (ingest worker URL)
- [ ] Deploy to Vercel
- [ ] Verify website accessible
- [ ] [T] Smoke test: submit repo via deployed website

### M1.12.3 Ship Gates
- [ ] [S] Ingest worker health check passes
- [ ] [S] Website loads without errors
- [ ] [S] Can submit a real GitHub repo URL
- [ ] [S] See streaming file detection events
- [ ] [S] See streaming chunk creation events
- [ ] [S] See final file tree with symbols
- [ ] [S] Total time < 10s for 50-file repo
- [ ] [S] No console errors in browser

---

# ═══════════════════════════════════════════════════════════════════════════════
# MILESTONE 2: "Semantic Doc Explorer"
# ═══════════════════════════════════════════════════════════════════════════════
#
# User sees: After ingest, clusters and a graph. Explore code semantically.
# User feels: "I can understand this codebase at a glance."
#
# Backend: Embeddings working, Assembly worker, graph + clusters in UI
# Still no LLM.
#
# Why ship here? This is already a product. A semantic code explorer.
# Way better than Sourcegraph Lite. No AI tokens required.
# You will have actual paying users at this stage.
# ═══════════════════════════════════════════════════════════════════════════════

## M2.0: Additional Languages (Optional but valuable)

### M2.0.1 TypeScript/JavaScript Support
- [ ] Add tree-sitter-typescript grammar
- [ ] Add tree-sitter-javascript grammar (for .js files)
- [ ] Extract function declarations
- [ ] Extract arrow function assignments (`const foo = () => {}`)
- [ ] Extract class declarations
- [ ] Extract interface declarations
- [ ] Extract type alias declarations
- [ ] Extract method definitions
- [ ] Capture export status
- [ ] [T] Unit tests for TS symbol extraction
- [ ] [T] Integration test: parse real TS file

### M2.0.2 Go Support
- [ ] Add tree-sitter-go grammar
- [ ] Extract function declarations
- [ ] Extract method declarations (with receiver)
- [ ] Extract struct declarations
- [ ] Extract interface declarations
- [ ] Extract type declarations
- [ ] [T] Unit tests for Go symbol extraction
- [ ] [T] Integration test: parse real Go file

---

## M2.1: Call Graph Extraction (`doctown-ingest`)

### M2.1.1 Call Detection - Rust
- [ ] Extract function calls within function bodies
- [ ] Extract method calls (including self.method())
- [ ] Extract associated function calls (Type::function())
- [ ] Track call target (resolved symbol ID or unresolved name)
- [ ] [T] Unit test: detect direct function calls
- [ ] [T] Unit test: detect method calls
- [ ] [T] Unit test: detect chained calls

### M2.1.2 Call Detection - Python
- [ ] Extract function calls
- [ ] Extract method calls
- [ ] Extract class instantiation (constructor calls)
- [ ] [T] Unit tests for Python call detection

### M2.1.3 Symbol Resolution
- [ ] Build symbol table during extraction
- [ ] Resolve local calls to symbol IDs
- [ ] Mark external calls as unresolved (store name only)
- [ ] Handle imports for resolution
- [ ] [T] Unit test: local calls resolved
- [ ] [T] Unit test: external calls marked unresolved

### M2.1.4 Import Extraction
- [ ] Extract Rust `use` statements
- [ ] Extract Python `import` and `from...import`
- [ ] Normalize import paths
- [ ] Associate imports with file/module
- [ ] [T] Unit tests for import extraction

---

## M2.2: Embedding Worker (`workers/embedding/`)

### M2.2.1 Python Project Setup
- [ ] Create `workers/embedding/` directory
- [ ] Create `pyproject.toml` with dependencies
- [ ] Add sentence-transformers
- [ ] Add FastAPI
- [ ] Add pydantic
- [ ] Add uvicorn
- [ ] Set up pytest
- [ ] Create virtual environment
- [ ] [T] Verify dependencies install

### M2.2.2 Embedding Model
- [ ] Implement model loader (all-MiniLM-L6-v2)
- [ ] Implement model warmup on startup
- [ ] Implement single text embedding function
- [ ] Implement batch embedding function
- [ ] Handle GPU if available, fallback to CPU
- [ ] [T] Unit test: embed single text returns 384-dim vector
- [ ] [T] Unit test: embed batch returns correct shape
- [ ] [T] Benchmark: throughput on GPU vs CPU

### M2.2.3 Batch Strategy
- [ ] Implement batch accumulator
- [ ] Configure min batch size (16)
- [ ] Configure max batch size (256)
- [ ] Configure timeout (500ms)
- [ ] Flush on timeout even if min not reached
- [ ] [T] Unit test: batching accumulates correctly
- [ ] [T] Unit test: timeout triggers flush

### M2.2.4 Worker HTTP API
- [ ] Set up FastAPI server
- [ ] Implement `GET /health` endpoint
- [ ] Implement `POST /embed` endpoint
- [ ] Define request schema:
  ```json
  {
    "batch_id": "batch_001",
    "chunks": [
      {"chunk_id": "c1", "content": "..."},
      {"chunk_id": "c2", "content": "..."}
    ]
  }
  ```
- [ ] Define response schema:
  ```json
  {
    "batch_id": "batch_001",
    "vectors": [
      {"chunk_id": "c1", "vector": [0.1, 0.2, ...]},
      {"chunk_id": "c2", "vector": [0.3, 0.4, ...]}
    ]
  }
  ```
- [ ] [T] Integration test: embed batch via HTTP

### M2.2.5 Event Emission
- [ ] Emit `embedding.batch_started.v1` when batch begins
- [ ] Emit `embedding.batch_completed.v1` when batch done
- [ ] Include duration_ms in completed event
- [ ] [T] Unit tests for event payloads match spec

---

## M2.3: Semantic Assembly (`doctown-assembly`)

### M2.3.1 Crate Setup
- [ ] Create `crates/doctown-assembly/`
- [ ] Add dependencies: ndarray, linfa (for clustering)
- [ ] Set up module structure

### M2.3.2 Vector Clustering
- [ ] Implement k-means clustering
- [ ] Implement cluster count heuristic (sqrt(n/2) as starting point)
- [ ] Implement cluster assignment for each vector
- [ ] Compute cluster centroids
- [ ] [T] Unit test: clustering on synthetic data
- [ ] [T] Unit test: correct number of clusters

### M2.3.3 Cluster Labeling
- [ ] Extract common terms from cluster members
- [ ] Use TF-IDF or simple frequency for term importance
- [ ] Generate 1-2 word label per cluster
- [ ] [T] Unit test: label generation
- [ ] [T] Manual verification: labels make sense

### M2.3.4 Graph Construction
- [ ] Define `Node` struct (symbol_id, metadata)
- [ ] Define `Edge` struct (source, target, kind)
- [ ] Build nodes from symbols
- [ ] Build "calls" edges from call graph
- [ ] Build "imports" edges from import data
- [ ] [T] Unit test: graph construction
- [ ] [T] Unit test: edge types correct

### M2.3.5 Similarity Edges
- [ ] Compute pairwise cosine similarity
- [ ] Add "related" edges for similarity > threshold (0.7)
- [ ] Limit to top-k related per node (5)
- [ ] [T] Unit test: similarity computation
- [ ] [T] Unit test: edge count reasonable

### M2.3.6 Graph Metrics
- [ ] Implement in-degree / out-degree calculation
- [ ] Implement simple centrality (degree-based for now)
- [ ] Compute graph density
- [ ] Assign centrality scores to nodes
- [ ] [T] Unit tests for each metric

### M2.3.7 Assembly Worker API
- [ ] Set up Actix-web server
- [ ] Implement `GET /health` endpoint
- [ ] Implement `POST /assemble` endpoint
- [ ] Define request schema (chunks, embeddings, symbol metadata)
- [ ] Define response schema (graph, clusters, nodes with centrality)
- [ ] [T] Integration test: full assembly pipeline

### M2.3.8 Assembly Events
- [ ] Emit `assembly.started.v1`
- [ ] Emit `assembly.cluster_created.v1` per cluster
- [ ] Emit `assembly.graph_completed.v1`
- [ ] Emit `assembly.completed.v1`
- [ ] [T] Unit tests for event payloads

---

## M2.4: Pipeline Integration (Ingest → Embed → Assemble)

### M2.4.1 Temporary Orchestration (Website-side)
- [ ] Website calls ingest, collects all chunks
- [ ] Website batches chunks, calls embedding worker
- [ ] Website calls assembly with chunks + embeddings
- [ ] Display results
- [ ] (Note: This is inefficient but ships M2 without Coordinator)

### M2.4.2 Data Flow
- [ ] Store chunks in memory during pipeline
- [ ] Store embeddings as they return
- [ ] Pass everything to assembly at end
- [ ] [T] Integration test: full pipeline works

---

## M2.5: Website - Semantic Explorer (`website/`)

### M2.5.1 Graph Visualization
- [ ] Add graph visualization library (e.g., cytoscape.js or d3-force)
- [ ] Render nodes as circles, colored by cluster
- [ ] Render edges as lines (different styles per edge kind)
- [ ] Implement zoom and pan
- [ ] Implement node click → select
- [ ] [T] Component test: graph renders

### M2.5.2 Cluster Navigation
- [ ] Create cluster list sidebar
- [ ] Show cluster label and member count
- [ ] Click cluster → highlight all nodes in cluster
- [ ] Click cluster → zoom to fit cluster
- [ ] [T] Component test: cluster selection

### M2.5.3 Symbol Detail Panel
- [ ] Create symbol detail panel (shows on node click)
- [ ] Show symbol name, kind, file path
- [ ] Show signature
- [ ] Show "calls" list (outgoing edges)
- [ ] Show "called by" list (incoming edges)
- [ ] Show "related" symbols (similarity edges)
- [ ] Show centrality score (as importance indicator)
- [ ] [T] Component test: detail panel content

### M2.5.4 Search
- [ ] Implement symbol search (filter by name)
- [ ] Highlight matching nodes in graph
- [ ] Show search results list
- [ ] Click result → zoom to node
- [ ] [T] Component test: search works

### M2.5.5 Polish
- [ ] Add loading states for each pipeline stage
- [ ] Show stage progress (Ingesting → Embedding → Assembling)
- [ ] Handle errors gracefully
- [ ] Mobile-responsive layout (or graceful degradation)

---

## M2.6: Deployment - Milestone 2

### M2.6.1 Embedding Worker Deployment
- [ ] Create Dockerfile for embedding worker
- [ ] Configure for GPU (CUDA base image)
- [ ] Build and test locally
- [ ] Deploy to RunPod (GPU serverless)
- [ ] Verify `/health` responds
- [ ] [T] Smoke test: embed batch via deployed endpoint

### M2.6.2 Assembly Worker Deployment
- [ ] Create Dockerfile for assembly worker
- [ ] Build and test locally
- [ ] Deploy to RunPod (CPU serverless)
- [ ] Verify `/health` responds
- [ ] [T] Smoke test: assemble via deployed endpoint

### M2.6.3 Website Update
- [ ] Update environment variables (embedding + assembly URLs)
- [ ] Deploy updated website
- [ ] [T] Full flow smoke test

### M2.6.4 Ship Gates
- [ ] [S] All three workers healthy
- [ ] [S] Submit repo → see graph within 30s
- [ ] [S] Clusters have meaningful labels
- [ ] [S] Can click node and see details
- [ ] [S] Can search for symbol
- [ ] [S] Graph looks reasonable (not a hairball)
- [ ] [S] Works on 100-file repo without timeout

---

# ═══════════════════════════════════════════════════════════════════════════════
# MILESTONE 3: "LLM Summaries (Minimum Usable Doctown)"
# ═══════════════════════════════════════════════════════════════════════════════
#
# User sees: 1-2 sentence summaries for each symbol. Downloadable .docpack.
# User feels: "This is magical auto-documentation."
#
# Backend: Generation worker, simple prompts, minimal docpack format.
#
# Why ship here? This is the first viable SaaS version.
# $10/mo subscriptions become viable today.
# ═══════════════════════════════════════════════════════════════════════════════

## M3.1: Symbol Context Generation (`doctown-assembly`)

### M3.1.1 Context Structure
- [ ] Define `SymbolContext` struct
- [ ] Include: symbol name, kind, language
- [ ] Include: file path, signature
- [ ] Include: calls list (names)
- [ ] Include: called_by list (names)
- [ ] Include: imports used
- [ ] Include: cluster label
- [ ] Include: centrality score
- [ ] [T] Unit test: context creation

### M3.1.2 Context Generation
- [ ] Generate context for each symbol after graph built
- [ ] Include top 3 related symbols
- [ ] Truncate long lists (max 10 items)
- [ ] [T] Unit test: context generation

### M3.1.3 Assembly API Update
- [ ] Include symbol_contexts in assembly response
- [ ] [T] Verify contexts included in response

---

## M3.2: Generation Worker (`workers/generation/`)

### M3.2.1 Python Project Setup
- [ ] Create `workers/generation/` directory
- [ ] Create `pyproject.toml`
- [ ] Add openai dependency
- [ ] Add tiktoken
- [ ] Add FastAPI, pydantic, uvicorn
- [ ] Set up pytest
- [ ] [T] Verify dependencies install

### M3.2.2 Token Counting
- [ ] Implement token counter for gpt-4o-mini
- [ ] Implement prompt token estimation
- [ ] Implement cost calculation ($0.15/1M input, $0.60/1M output)
- [ ] [T] Unit test: token counting accuracy
- [ ] [T] Unit test: cost calculation

### M3.2.3 Prompt Construction
- [ ] Define prompt template:
  ```
  You are documenting a {language} codebase.

  Symbol: {name}
  Kind: {kind}
  File: {file_path}
  Signature: {signature}

  Calls: {calls}
  Called by: {called_by}
  Related to: {cluster_label}
  Importance: {centrality} (0-1 scale)

  Write 1-2 sentences describing what this symbol does.
  Be concise and precise. Focus on purpose, not implementation.
  ```
- [ ] Implement prompt builder from SymbolContext
- [ ] Implement prompt truncation if > 2000 tokens
- [ ] [T] Unit test: prompt construction
- [ ] [T] Snapshot test: prompt format

### M3.2.4 OpenAI Integration
- [ ] Implement OpenAI client wrapper
- [ ] Use gpt-4o-mini model
- [ ] Implement retry with exponential backoff (3 attempts)
- [ ] Implement rate limit handling (429 responses)
- [ ] Track tokens used per request
- [ ] [T] Unit test with mocked responses
- [ ] [T] Integration test with real API (gated)

### M3.2.5 Batch Processing
- [ ] Implement concurrent symbol documentation (max 10 parallel)
- [ ] Implement progress tracking
- [ ] Handle partial failures (continue on individual errors)
- [ ] Collect all results
- [ ] [T] Unit test: batch processing
- [ ] [T] Unit test: partial failure handling

### M3.2.6 Worker HTTP API
- [ ] Set up FastAPI server
- [ ] Implement `GET /health` endpoint
- [ ] Implement `POST /generate` endpoint
- [ ] Define request schema:
  ```json
  {
    "job_id": "...",
    "symbols": [
      {"symbol_id": "...", "context": {...}}
    ]
  }
  ```
- [ ] Define response schema:
  ```json
  {
    "documented_symbols": [
      {"symbol_id": "...", "summary": "..."}
    ],
    "total_tokens": 1234,
    "total_cost": 0.002
  }
  ```
- [ ] [T] Integration test: generate docs via HTTP

### M3.2.7 Generation Events
- [ ] Emit `generation.started.v1`
- [ ] Emit `generation.symbol_documented.v1` per symbol
- [ ] Emit `generation.completed.v1` with totals
- [ ] [T] Unit tests for event payloads

---

## M3.3: Minimal Docpack Format (`doctown-docpack`)

### M3.3.1 Crate Setup
- [ ] Create `crates/doctown-docpack/`
- [ ] Add dependencies: serde, serde_json, flate2 (gzip)
- [ ] Set up module structure

### M3.3.2 Manifest
- [ ] Define `Manifest` struct (minimal version)
- [ ] Include: schema_version ("docpack/1.0")
- [ ] Include: docpack_id (SHA-256 of contents)
- [ ] Include: created_at
- [ ] Include: source (repo_url, git_ref, commit_hash)
- [ ] Include: statistics (file_count, symbol_count)
- [ ] Implement JSON serialization
- [ ] [T] Unit test: manifest creation
- [ ] [T] Snapshot test: manifest JSON

### M3.3.3 Nodes (Symbols + Docs)
- [ ] Define `Nodes` struct
- [ ] Define `Symbol` struct with documentation field
- [ ] Define `Documentation` struct (summary only for M3)
- [ ] Implement JSON serialization
- [ ] [T] Unit test: nodes creation
- [ ] [T] Snapshot test: nodes JSON

### M3.3.4 Graph
- [ ] Define `Graph` struct (nodes list, edges list)
- [ ] Define `Edge` struct (source, target, kind)
- [ ] Implement JSON serialization
- [ ] [T] Unit test: graph creation
- [ ] [T] Snapshot test: graph JSON

### M3.3.5 Docpack Writer
- [ ] Define `DocpackWriter` struct
- [ ] Implement `write()` method
- [ ] Create in-memory tar archive
- [ ] Add manifest.json
- [ ] Add nodes.json
- [ ] Add graph.json
- [ ] Gzip compress the archive
- [ ] Compute SHA-256 checksum
- [ ] Return bytes + checksum
- [ ] [T] Unit test: write docpack
- [ ] [T] Unit test: checksum is deterministic

### M3.3.6 Docpack Reader
- [ ] Define `DocpackReader` struct
- [ ] Implement `read()` method
- [ ] Decompress gzip
- [ ] Extract tar archive
- [ ] Parse manifest.json
- [ ] Verify checksum matches
- [ ] Parse nodes.json
- [ ] Parse graph.json
- [ ] [T] Unit test: read docpack
- [ ] [T] Unit test: roundtrip (write → read)
- [ ] [T] Unit test: corrupted docpack rejected

---

## M3.4: Packer Worker (Minimal) (`doctown-packer`)

### M3.4.1 Crate Setup
- [ ] Create `crates/doctown-packer/`
- [ ] Add doctown-docpack dependency
- [ ] Set up HTTP server

### M3.4.2 Artifact Collection
- [ ] Accept symbols with documentation
- [ ] Accept graph structure
- [ ] Accept job metadata
- [ ] Validate all required data present
- [ ] [T] Unit test: validation

### M3.4.3 Docpack Assembly
- [ ] Build manifest from metadata
- [ ] Build nodes from symbols + docs
- [ ] Build graph from edges
- [ ] Call DocpackWriter
- [ ] [T] Unit test: assembly

### M3.4.4 Packer Worker API
- [ ] Set up Actix-web server
- [ ] Implement `GET /health`
- [ ] Implement `POST /pack`
- [ ] Return docpack bytes + metadata
- [ ] [T] Integration test: pack via HTTP

### M3.4.5 Packer Events
- [ ] Emit `pack.started.v1`
- [ ] Emit `pack.completed.v1`
- [ ] [T] Unit tests for event payloads

---

## M3.5: Website - Documentation View (`website/`)

### M3.5.1 Symbol Documentation Display
- [ ] Update symbol detail panel
- [ ] Show LLM-generated summary prominently
- [ ] Style summary distinctly (quote style or highlight)
- [ ] [T] Component test: summary displays

### M3.5.2 Docpack Download
- [ ] Add "Download .docpack" button
- [ ] Trigger download with proper filename
- [ ] Show file size before download
- [ ] [T] Test: download works

### M3.5.3 Pipeline Updates
- [ ] Add generation stage to pipeline
- [ ] Add packing stage to pipeline
- [ ] Show progress through all stages
- [ ] Display total cost after generation
- [ ] [T] Integration test: full pipeline

---

## M3.6: Deployment - Milestone 3

### M3.6.1 Generation Worker Deployment
- [ ] Create Dockerfile
- [ ] Configure OpenAI API key as secret
- [ ] Deploy to RunPod (CPU)
- [ ] Verify `/health`
- [ ] [T] Smoke test: generate docs

### M3.6.2 Packer Worker Deployment
- [ ] Create Dockerfile
- [ ] Deploy to RunPod (CPU)
- [ ] Verify `/health`
- [ ] [T] Smoke test: pack docpack

### M3.6.3 Website Update
- [ ] Add generation + packer URLs
- [ ] Deploy
- [ ] [T] Full flow smoke test

### M3.6.4 Ship Gates
- [ ] [S] All workers healthy
- [ ] [S] Submit repo → get documented symbols
- [ ] [S] Summaries are coherent and accurate
- [ ] [S] Can download .docpack file
- [ ] [S] .docpack opens correctly (verify with reader)
- [ ] [S] Total cost displayed accurately
- [ ] [S] Full pipeline < 60s for 50-file repo

---

# ═══════════════════════════════════════════════════════════════════════════════
# MILESTONE 4: "Complete v1 Docpack (Stable Format)"
# ═══════════════════════════════════════════════════════════════════════════════
#
# User sees: Full docpack with clusters, source map, optional embeddings.
# User feels: "This is a real artifact I can integrate into my workflow."
#
# Backend: Full docpack spec implemented. Schema is stable and versioned.
#
# Why ship here? The format becomes portable. Other tools can consume it.
# This is where Doctown becomes a format, not just a product.
# ═══════════════════════════════════════════════════════════════════════════════

## M4.1: Complete Docpack Format (`doctown-docpack`)

### M4.1.1 Clusters File
- [ ] Define `Clusters` struct
- [ ] Define `Cluster` struct (cluster_id, label, member_count)
- [ ] Implement JSON serialization
- [ ] [T] Unit test: clusters
- [ ] [T] Snapshot test: clusters.json

### M4.1.2 Source Map File
- [ ] Define `SourceMap` struct
- [ ] Define `FileEntry` struct (file_path, language, chunks)
- [ ] Define `ChunkEntry` struct (chunk_id, byte_range, symbol_ids)
- [ ] Implement JSON serialization
- [ ] [T] Unit test: source map
- [ ] [T] Snapshot test: source_map.json

### M4.1.3 Embeddings Binary Format
- [ ] Define binary header (uint32 num_vectors, uint32 dimensions)
- [ ] Implement `EmbeddingsWriter`
- [ ] Write header
- [ ] Write float32 vectors sequentially
- [ ] Create index mapping chunk_id → byte offset
- [ ] Implement `EmbeddingsReader`
- [ ] Read header
- [ ] Random access by chunk_id
- [ ] [T] Unit test: write/read roundtrip
- [ ] [T] Unit test: random access works
- [ ] [T] Property test: various sizes

### M4.1.4 Symbol Contexts (Optional)
- [ ] Define `SymbolContexts` struct
- [ ] Include raw prompt text for reproducibility
- [ ] Mark as optional in manifest
- [ ] [T] Unit test: contexts

### M4.1.5 Updated Manifest
- [ ] Add `statistics.cluster_count`
- [ ] Add `statistics.embedding_dimensions`
- [ ] Add `optional.has_embeddings`
- [ ] Add `optional.has_symbol_contexts`
- [ ] Add `checksum` field (algorithm + value)
- [ ] [T] Unit test: updated manifest

### M4.1.6 Updated Writer
- [ ] Add clusters.json to archive
- [ ] Add source_map.json to archive
- [ ] Optionally add embeddings.bin
- [ ] Optionally add symbol_contexts.json
- [ ] Update checksum calculation to include all files
- [ ] [T] Unit test: full docpack

### M4.1.7 Updated Reader
- [ ] Read all new files
- [ ] Handle optional files gracefully
- [ ] Provide access to embeddings by chunk_id
- [ ] [T] Unit test: read full docpack
- [ ] [T] Unit test: read minimal docpack (M3 format)

### M4.1.8 Schema Versioning
- [ ] Define schema version "docpack/1.0"
- [ ] Implement version checking in reader
- [ ] Reject incompatible versions with clear error
- [ ] [T] Unit test: version validation

---

## M4.2: Packer Updates (`doctown-packer`)

### M4.2.1 Artifact Collection
- [ ] Accept clusters from assembly
- [ ] Accept source map from ingest
- [ ] Accept embeddings (optional)
- [ ] Accept symbol contexts (optional)
- [ ] [T] Unit test: collection

### M4.2.2 Full Docpack Assembly
- [ ] Build all artifacts
- [ ] Compute content-addressed docpack_id
- [ ] Include embeddings if requested
- [ ] Include symbol contexts if requested
- [ ] [T] Unit test: full assembly

### M4.2.3 Reproducibility
- [ ] Ensure same inputs → same checksum
- [ ] Sort all JSON keys
- [ ] Use consistent float formatting
- [ ] [T] Unit test: reproducibility

---

## M4.3: Website - Enhanced Viewer (`website/`)

### M4.3.1 Cluster Browser
- [ ] Dedicated cluster view
- [ ] Show all symbols in cluster
- [ ] Show cluster statistics
- [ ] [T] Component test

### M4.3.2 Source Map Integration
- [ ] Show byte ranges in symbol detail
- [ ] Future: link to GitHub source (line numbers)
- [ ] [T] Component test

### M4.3.3 Embeddings Search (stretch)
- [ ] Implement client-side similarity search
- [ ] "Find similar symbols" feature
- [ ] [T] Component test

---

## M4.4: Deployment - Milestone 4

### M4.4.1 Update All Workers
- [ ] Update assembly to output clusters, source map
- [ ] Update packer to handle all artifacts
- [ ] Deploy updates
- [ ] [T] Integration test

### M4.4.2 Ship Gates
- [ ] [S] Docpack contains all specified files
- [ ] [S] Manifest has correct statistics
- [ ] [S] Checksum is reproducible
- [ ] [S] Reader can parse all files
- [ ] [S] Embeddings binary accessible
- [ ] [S] Schema version is "docpack/1.0"

---

# ═══════════════════════════════════════════════════════════════════════════════
# MILESTONE 5: "Production Pipeline (Distributed Workers)"
# ═══════════════════════════════════════════════════════════════════════════════
#
# User sees: Faster jobs, handles larger repos, more reliable.
# User feels: "This is production-grade infrastructure."
#
# Backend: Coordinator, message queue, proper distributed workers.
#
# Why ship here? Scalability. Handle more users, bigger repos, concurrent jobs.
# Note: You don't need this for v1. Most ship this at version 4-7.
# ═══════════════════════════════════════════════════════════════════════════════

## M5.1: Message Queue (`doctown-common`)

### M5.1.1 Queue Abstraction
- [ ] Define `Queue` trait
- [ ] Define `Message` struct
- [ ] Define `QueueConfig` struct
- [ ] [T] Unit test: trait design

### M5.1.2 In-Memory Queue (Testing)
- [ ] Implement in-memory queue for tests
- [ ] Implement pub/sub semantics
- [ ] [T] Unit tests

### M5.1.3 Redis Streams Implementation
- [ ] Add redis dependency
- [ ] Implement Redis Streams queue
- [ ] Implement consumer groups
- [ ] Implement acknowledgment
- [ ] Implement dead letter queue
- [ ] [T] Integration tests with Redis

---

## M5.2: Coordinator (`doctown-coordinator`)

### M5.2.1 Crate Setup
- [ ] Create `crates/doctown-coordinator/`
- [ ] Add dependencies
- [ ] Set up module structure

### M5.2.2 Job State Machine
- [ ] Define job states enum
- [ ] Define state transitions
- [ ] Implement state machine
- [ ] [T] Unit tests for transitions

### M5.2.3 Worker Registry
- [ ] Define worker types and URLs
- [ ] Implement health checking
- [ ] Implement worker selection
- [ ] [T] Unit tests

### M5.2.4 Job Orchestration
- [ ] Validate repo → estimate cost
- [ ] Dispatch ingest worker
- [ ] Stream chunks to embedding queue
- [ ] Wait for embeddings complete
- [ ] Dispatch assembly worker
- [ ] Dispatch generation worker
- [ ] Dispatch packer worker
- [ ] Emit job events
- [ ] [T] Integration test with mock workers

### M5.2.5 Event Aggregation
- [ ] Subscribe to worker events
- [ ] Aggregate all events for job
- [ ] Forward to client (SSE)
- [ ] Track sequence numbers
- [ ] [T] Unit tests

### M5.2.6 Retry Logic
- [ ] Implement exponential backoff
- [ ] Implement max retries (3)
- [ ] Implement circuit breaker
- [ ] [T] Unit tests

### M5.2.7 Coordinator API
- [ ] `GET /health`
- [ ] `POST /jobs` - create job
- [ ] `GET /jobs/{id}` - job status
- [ ] `GET /jobs/{id}/events` - SSE stream
- [ ] [T] Integration tests

---

## M5.3: Worker Updates

### M5.3.1 Queue Integration
- [ ] Update ingest to publish chunks to queue
- [ ] Update embedding to consume from queue
- [ ] Update all workers to emit events to queue
- [ ] [T] Integration tests

### M5.3.2 Streaming Pipeline
- [ ] Ingest streams chunks as ready
- [ ] Embedding processes in parallel
- [ ] Assembly waits for completion signal
- [ ] [T] Verify streaming behavior

---

## M5.4: Website Updates

### M5.4.1 Coordinator Integration
- [ ] Point website at coordinator (not workers directly)
- [ ] Use coordinator's SSE endpoint
- [ ] Handle new event flow
- [ ] [T] Integration test

---

## M5.5: Deployment - Milestone 5

### M5.5.1 Infrastructure
- [ ] Set up Redis (Upstash)
- [ ] Configure queue topics
- [ ] [T] Verify connectivity

### M5.5.2 Coordinator Deployment
- [ ] Create Dockerfile
- [ ] Deploy to RunPod
- [ ] Verify health
- [ ] [T] Smoke test

### M5.5.3 Worker Updates
- [ ] Redeploy all workers with queue integration
- [ ] Verify queue flow
- [ ] [T] Full pipeline test

### M5.5.4 Ship Gates
- [ ] [S] Coordinator orchestrates full pipeline
- [ ] [S] Events stream through coordinator
- [ ] [S] Retry logic works
- [ ] [S] Can handle 5 concurrent jobs
- [ ] [S] Larger repo (200 files) completes
- [ ] [S] Pipeline < 30s for 50-file repo

---

# ═══════════════════════════════════════════════════════════════════════════════
# MILESTONE 6: "Commercial Doctown (Payments, Auth, Library)"
# ═══════════════════════════════════════════════════════════════════════════════
#
# User sees: Accounts, job history, docpack library, payments.
# User feels: "This is a real SaaS product."
#
# Backend: Database, auth, Stripe, user management.
#
# Why ship here? This is the business layer, not the product layer.
# People will already pay you before milestone 6.
# This turns the business into something stable and predictable.
# ═══════════════════════════════════════════════════════════════════════════════

## M6.1: Database Schema

### M6.1.1 Schema Design
- [ ] Design `users` table
  - id, email, name, github_id
  - plan_tier, stripe_customer_id
  - created_at, updated_at
- [ ] Design `jobs` table
  - id, user_id, repo_url, git_ref
  - status, cost, duration_ms
  - created_at, completed_at
- [ ] Design `docpacks` table
  - id, job_id, user_id
  - checksum, size_bytes, storage_url
  - is_public, created_at
- [ ] Design `repos` table
  - id, user_id, github_repo_id
  - url, name, is_private
  - last_processed_at

### M6.1.2 Migrations
- [ ] Create migration files (sqlx or diesel)
- [ ] Implement up migrations
- [ ] Implement down migrations
- [ ] [T] Test migrations

---

## M6.2: Database Client (`doctown-coordinator`)

### M6.2.1 Connection Pool
- [ ] Set up sqlx connection pool
- [ ] Configure from environment
- [ ] Implement health check
- [ ] [T] Integration test

### M6.2.2 User Operations
- [ ] Create user
- [ ] Get user by ID
- [ ] Get user by GitHub ID
- [ ] Update user plan tier
- [ ] [T] Unit tests

### M6.2.3 Job Operations
- [ ] Create job
- [ ] Update job status
- [ ] Get jobs by user
- [ ] Get job by ID
- [ ] [T] Unit tests

### M6.2.4 Docpack Operations
- [ ] Create docpack record
- [ ] Get docpacks by user
- [ ] Get docpack by ID
- [ ] Update public status
- [ ] [T] Unit tests

---

## M6.3: R2 Storage Integration

### M6.3.1 R2 Client
- [ ] Implement S3-compatible client for R2
- [ ] Implement upload
- [ ] Implement download
- [ ] Implement pre-signed URL generation
- [ ] [T] Integration tests

### M6.3.2 Packer Integration
- [ ] Upload docpack to R2
- [ ] Return storage URL
- [ ] Store in database
- [ ] [T] Integration test

---

## M6.4: Authentication (`website/`)

### M6.4.1 GitHub OAuth
- [ ] Set up GitHub OAuth app
- [ ] Implement login flow
- [ ] Implement callback handler
- [ ] Store tokens securely
- [ ] [T] E2E test

### M6.4.2 Session Management
- [ ] Implement session creation
- [ ] Implement session validation
- [ ] Implement logout
- [ ] [T] Unit tests

### M6.4.3 Protected Routes
- [ ] Create auth middleware
- [ ] Protect dashboard routes
- [ ] Redirect unauthenticated users
- [ ] [T] Integration tests

---

## M6.5: Stripe Integration

### M6.5.1 Stripe Setup
- [ ] Create Stripe account
- [ ] Define products (Creator, Team tiers)
- [ ] Create price IDs
- [ ] Set up webhook endpoint

### M6.5.2 Checkout Flow
- [ ] Implement checkout session creation
- [ ] Handle successful checkout
- [ ] Update user plan tier
- [ ] [T] Integration test

### M6.5.3 Subscription Management
- [ ] Handle subscription updated webhook
- [ ] Handle subscription cancelled webhook
- [ ] Sync plan tier with Stripe
- [ ] [T] Webhook tests

### M6.5.4 Billing Portal
- [ ] Implement portal session creation
- [ ] Allow users to manage subscription
- [ ] [T] Integration test

---

## M6.6: Website - User Dashboard (`website/`)

### M6.6.1 Dashboard Layout
- [ ] Create dashboard page
- [ ] Show user info
- [ ] Show plan tier
- [ ] [T] Component test

### M6.6.2 Job History
- [ ] Fetch jobs from API
- [ ] Show job list with status
- [ ] Show cost per job
- [ ] Click job → view details
- [ ] [T] Component test

### M6.6.3 Docpack Library
- [ ] Fetch docpacks from API
- [ ] Show docpack list
- [ ] Download docpack
- [ ] View docpack in browser
- [ ] Make public/private toggle
- [ ] Share link for public docpacks
- [ ] [T] Component tests

### M6.6.4 Usage & Billing
- [ ] Show jobs used this month
- [ ] Show jobs remaining
- [ ] Link to Stripe portal
- [ ] Upgrade CTA for free users
- [ ] [T] Component test

---

## M6.7: Rate Limiting & API Keys

### M6.7.1 Rate Limiting
- [ ] Implement rate limiter (Redis-based)
- [ ] Configure limits per tier
- [ ] Return 429 when exceeded
- [ ] [T] Integration tests

### M6.7.2 API Keys (for Team tier)
- [ ] Generate API keys
- [ ] Store hashed keys
- [ ] Validate on requests
- [ ] [T] Unit tests

---

## M6.8: Private Repo Support

### M6.8.1 GitHub Token Storage
- [ ] Store user's GitHub token (encrypted)
- [ ] Use token for private repo access
- [ ] Scope to repo read-only
- [ ] [T] Security review

### M6.8.2 Ingest Worker Update
- [ ] Accept optional auth token
- [ ] Use for authenticated downloads
- [ ] [T] Integration test

---

## M6.9: Deployment - Milestone 6

### M6.9.1 Database Setup
- [ ] Create PostgreSQL database (Neon or Supabase)
- [ ] Run migrations
- [ ] Configure connection string
- [ ] [T] Verify connectivity

### M6.9.2 R2 Setup
- [ ] Create R2 bucket
- [ ] Configure access keys
- [ ] Set up CORS for downloads
- [ ] [T] Verify uploads work

### M6.9.3 Auth & Payments
- [ ] Configure GitHub OAuth secrets
- [ ] Configure Stripe keys
- [ ] Configure webhook secrets
- [ ] [T] E2E auth flow
- [ ] [T] E2E payment flow

### M6.9.4 Ship Gates
- [ ] [S] Can create account via GitHub
- [ ] [S] Can submit job as authenticated user
- [ ] [S] Job history shows in dashboard
- [ ] [S] Docpacks show in library
- [ ] [S] Can download from library
- [ ] [S] Can upgrade to paid plan
- [ ] [S] Rate limiting works
- [ ] [S] Private repos work (for paid users)

---

# ═══════════════════════════════════════════════════════════════════════════════
# POST-LAUNCH: Polish & Iteration
# ═══════════════════════════════════════════════════════════════════════════════

## Post-Launch: Observability

### Logging
- [ ] Structured logging in all workers
- [ ] Log aggregation (Axiom, Logtail, etc.)
- [ ] Searchable by job_id, user_id

### Metrics
- [ ] Job success rate
- [ ] Job duration percentiles
- [ ] Worker utilization
- [ ] OpenAI token usage

### Alerting
- [ ] Alert on high failure rate
- [ ] Alert on slow jobs
- [ ] Alert on worker unhealthy

---

## Post-Launch: Performance

### Optimization
- [ ] Profile hot paths
- [ ] Optimize embedding batching
- [ ] Add caching where beneficial
- [ ] Reduce cold start times

### Benchmarks
- [ ] 50-file repo < 20s
- [ ] 200-file repo < 60s
- [ ] 1000-file repo < 180s

---

## Post-Launch: Additional Languages

### Add More Languages
- [ ] Java support
- [ ] C/C++ support
- [ ] Ruby support
- [ ] PHP support
- [ ] C# support

---

## Post-Launch: Advanced Features

### Incremental Updates
- [ ] Detect changed files
- [ ] Re-process only changes
- [ ] Merge with existing docpack

### GitHub App Integration
- [ ] Create GitHub App
- [ ] Auto-trigger on push
- [ ] Update docpack automatically

### Webhooks
- [ ] Notify on job completion
- [ ] Configurable webhook URLs

---

# ═══════════════════════════════════════════════════════════════════════════════
# COMPLETION CHECKLIST
# ═══════════════════════════════════════════════════════════════════════════════

## Per-Milestone Checklist

Before marking a milestone complete:

- [ ] All unit tests pass (`cargo test`, `pytest`)
- [ ] All integration tests pass
- [ ] All ship gates pass
- [ ] CI pipeline is green
- [ ] Deployed to production
- [ ] Smoke tested in production
- [ ] No critical console errors

## Final v1 Checklist (Post-M6)

- [ ] All 6 milestones shipped
- [ ] Users can sign up and pay
- [ ] Documentation complete
- [ ] Security review completed
- [ ] Performance benchmarks met
- [ ] First 10 paying customers
