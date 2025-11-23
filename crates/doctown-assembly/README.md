# Doctown Assembly Worker

Semantic assembly service that clusters embeddings, labels clusters, and constructs a code graph with semantic relationships.

## Features

- **Vector Clustering**: K-means clustering of embedding vectors
- **Cluster Labeling**: TF-IDF based label generation for clusters
- **Graph Construction**: Builds code graph with:
  - Call edges (function/method calls)
  - Import edges (module imports)
  - Similarity edges (semantic relationships)
- **Centrality Metrics**: Computes degree centrality for all nodes
- **Event Streaming**: Emits events for progress tracking

## API Endpoints

### Health Check

```
GET /health
```

Returns service status and version.

### Assembly

```
POST /assemble
```

**Request Body:**
```json
{
  "job_id": "job_123",
  "repo_url": "https://github.com/user/repo",
  "git_ref": "main",
  "chunks": [
    {
      "chunk_id": "chunk_1",
      "vector": [0.1, 0.2, ...],  // 384 dimensions
      "content": "function code..."
    }
  ],
  "symbols": [
    {
      "symbol_id": "sym_1",
      "name": "calculate_total",
      "kind": "function",
      "file_path": "src/math.rs",
      "signature": "fn calculate_total() -> i32",
      "chunk_ids": ["chunk_1"],
      "calls": ["sum"],
      "imports": ["std::collections"]
    }
  ]
}
```

**Response:**
```json
{
  "job_id": "job_123",
  "clusters": [
    {
      "cluster_id": "cluster_0",
      "label": "math utilities",
      "members": ["sym_1", "sym_2"]
    }
  ],
  "nodes": [
    {
      "id": "sym_1",
      "metadata": { "name": "calculate_total", ... },
      "cluster_id": "cluster_0",
      "centrality": 0.75
    }
  ],
  "edges": [
    {
      "source": "sym_1",
      "target": "sym_2",
      "kind": "calls"
    }
  ],
  "stats": {
    "cluster_count": 5,
    "node_count": 50,
    "edge_count": 120,
    "duration_ms": 1500
  },
  "events": [...]
}
```

## Running the Server

### Development

```bash
cd crates/doctown-assembly
cargo run --bin assembly-server
```

Server runs on `http://0.0.0.0:3001` by default.

### Environment Variables

- `HOST`: Server host (default: `0.0.0.0`)
- `PORT`: Server port (default: `3001`)

### Example

```bash
HOST=127.0.0.1 PORT=8080 cargo run --bin assembly-server
```

## Testing

```bash
# Run all tests
cargo test -p doctown-assembly

# Run only unit tests
cargo test -p doctown-assembly --lib

# Run only integration tests
cargo test -p doctown-assembly --test integration_test
```

## Events

The assembly worker emits the following events during processing:

1. **assembly.started.v1**: Emitted when assembly begins
2. **assembly.cluster_created.v1**: Emitted for each cluster created
3. **assembly.graph_completed.v1**: Emitted when graph construction is complete
4. **assembly.completed.v1**: Emitted when assembly is finished (terminal event with status)

All events follow the Doctown event envelope format and are included in the response.

## Architecture

```
Request
  ↓
Clustering (k-means)
  ↓
Labeling (TF-IDF)
  ↓
Graph Building
  ├── Nodes (from symbols)
  ├── Call edges (from call data)
  ├── Import edges (from import data)
  └── Similarity edges (from embeddings)
  ↓
Centrality Computation
  ↓
Response
```

## Dependencies

- **actix-web**: HTTP server
- **ndarray**: Vector operations
- **doctown-events**: Event system
- **doctown-common**: Shared types
