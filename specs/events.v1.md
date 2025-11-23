# Doctown Event Specification v1.0

## Envelope Schema

Every event in the Doctown pipeline uses this envelope structure:

```json
{
  "envelope_version": "1.0",
  "timestamp": "2025-11-22T18:16:04Z",
  "job_id": "job_abc123",
  "event_id": "evt_9f81c",
  "parent_event_id": null,
  "source": "ingest-worker-1",
  "type": "ingest.chunk_created.v1",
  "status": null,
  "sequence": 42,
  "payload": {},
  "context": {
    "repo_url": "https://github.com/pyke/ort",
    "git_ref": "main",
    "user_id": "user_123",
    "plan_tier": "creator"
  },
  "meta": {
    "producer_version": "doctown-ingest/1.2.1",
    "trace_id": "trace_8f0340",
    "idempotency_key": "chunk_main.rs_0_abc123",
    "tags": ["repo:ort-main"]
  }
}
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `envelope_version` | string | yes | Always `"1.0"` for this spec |
| `timestamp` | string (ISO 8601) | yes | When the event was created |
| `job_id` | string | yes | Unique identifier for the pipeline job |
| `event_id` | string | yes | Unique identifier for this event |
| `parent_event_id` | string | no | Event that triggered this one (causality) |
| `source` | string | yes | Worker/service that emitted the event |
| `type` | string | yes | Event type with `.v1` suffix |
| `status` | string | no | Only on terminal events: `success`, `failed`, `partial`, `cancelled` |
| `sequence` | integer | yes | Ordering within this job (monotonic) |
| `payload` | object | yes | Event-specific data |
| `context` | object | no | Job-level context, copied across workers |
| `meta` | object | no | Operational metadata |

### Context Object

Optional but recommended. Set by coordinator, copied by workers.

| Field | Type | Description |
|-------|------|-------------|
| `repo_url` | string | Repository being processed |
| `git_ref` | string | Branch/tag/commit |
| `user_id` | string | User who initiated the job |
| `plan_tier` | string | `free`, `creator`, `team`, `enterprise` |

### Meta Object

| Field | Type | Description |
|-------|------|-------------|
| `producer_version` | string | Software version that emitted this event |
| `trace_id` | string | Distributed tracing ID |
| `idempotency_key` | string | For deduplication on retries |
| `tags` | string[] | Arbitrary labels for filtering |

---

## Event Types

### Rules

1. All event types end with `.v1` suffix
2. `status` only set on terminal events (`.completed`)
3. `warnings[]` included on every completion event
4. `partial` status only valid on `pack.completed.v1`
5. Pattern: `{stage}.started` → `{stage}.{progress}` → `{stage}.completed`

---

### Job (Coordinator)

#### `job.created.v1`

Job has been accepted by the coordinator.

```json
{
  "type": "job.created.v1",
  "payload": {
    "estimated_cost": 0.0042,
    "estimated_duration_ms": 45000
  }
}
```

#### `job.validated.v1`

Repository has been validated and analyzed for scope.

```json
{
  "type": "job.validated.v1",
  "payload": {
    "file_count": 42,
    "size_bytes": 102400,
    "languages": ["rust", "python"],
    "estimated_cost_refined": 0.0038
  }
}
```

#### `job.started.v1`

Workers have been assigned and pipeline is running.

```json
{
  "type": "job.started.v1",
  "payload": {
    "worker_assignments": [
      { "stage": "ingest", "worker_id": "ingest-worker-1" },
      { "stage": "embedding", "worker_id": "gpu-pool-3" }
    ]
  }
}
```

#### `job.completed.v1`

Job has finished. Check `status` for outcome.

```json
{
  "type": "job.completed.v1",
  "status": "success",
  "payload": {
    "docpack_url": "https://r2.doctown.dev/packs/abc123.docpack",
    "duration_ms": 42000,
    "final_cost": 0.0035,
    "warnings": []
  }
}
```

**Status values:** `success`, `failed`, `cancelled`

---

### Ingest

#### `ingest.started.v1`

Ingest worker has begun processing.

```json
{
  "type": "ingest.started.v1",
  "payload": {}
}
```

#### `ingest.file_detected.v1`

A file has been discovered in the repository.

```json
{
  "type": "ingest.file_detected.v1",
  "payload": {
    "file_path": "src/main.rs",
    "size_bytes": 4096
  }
}
```

#### `ingest.file_skipped.v1`

A file was skipped (binary, too large, unsupported, etc.).

```json
{
  "type": "ingest.file_skipped.v1",
  "payload": {
    "file_path": "assets/logo.png",
    "reason": "binary_file"
  }
}
```

#### `ingest.chunk_created.v1`

A semantic chunk has been extracted and is ready for embedding.

```json
{
  "type": "ingest.chunk_created.v1",
  "payload": {
    "chunk_id": "chunk_abc123",
    "file_path": "src/main.rs",
    "language": "rust",
    "content_hash": "sha256:9f86d08...",
    "byte_range": [0, 2048]
  }
}
```

#### `ingest.completed.v1`

Ingest stage finished.

```json
{
  "type": "ingest.completed.v1",
  "status": "success",
  "payload": {
    "total_files": 42,
    "total_chunks": 128,
    "languages": ["rust", "python"],
    "warnings": []
  }
}
```

**Status values:** `success`, `failed`

---

### Embedding

#### `embedding.started.v1`

Embedding worker has begun processing chunks.

```json
{
  "type": "embedding.started.v1",
  "payload": {
    "total_chunks": 128
  }
}
```

#### `embedding.batch_started.v1`

A batch of chunks is being embedded.

```json
{
  "type": "embedding.batch_started.v1",
  "payload": {
    "batch_id": "batch_001",
    "chunk_ids": ["chunk_a", "chunk_b", "chunk_c"],
    "batch_size": 64,
    "attempt": 1
  }
}
```

#### `embedding.chunk_vector.v1`

A single chunk has been embedded.

```json
{
  "type": "embedding.chunk_vector.v1",
  "payload": {
    "chunk_id": "chunk_abc123",
    "vector_hash": "sha256:a1b2c3...",
    "dimensions": 384
  }
}
```

#### `embedding.batch_completed.v1`

A batch has finished embedding.

```json
{
  "type": "embedding.batch_completed.v1",
  "status": "success",
  "payload": {
    "batch_id": "batch_001",
    "vector_count": 64,
    "duration_ms": 230,
    "latency_ms": 12,
    "warnings": []
  }
}
```

**Status values:** `success`, `failed`

#### `embedding.completed.v1`

All embeddings finished.

```json
{
  "type": "embedding.completed.v1",
  "status": "success",
  "payload": {
    "total_vectors": 128,
    "duration_ms": 8500,
    "warnings": []
  }
}
```

**Status values:** `success`, `failed`

---

### Assembly

#### `assembly.started.v1`

Semantic assembly has begun.

```json
{
  "type": "assembly.started.v1",
  "payload": {
    "total_embeddings": 128,
    "total_symbols": 65
  }
}
```

#### `assembly.cluster_created.v1`

A semantic cluster has been identified.

```json
{
  "type": "assembly.cluster_created.v1",
  "payload": {
    "cluster_id": "cluster_auth",
    "label": "authentication",
    "member_count": 12
  }
}
```

#### `assembly.graph_completed.v1`

The docpack graph structure is complete.

```json
{
  "type": "assembly.graph_completed.v1",
  "payload": {
    "node_count": 65,
    "edge_count": 171
  }
}
```

#### `assembly.completed.v1`

Semantic assembly finished.

```json
{
  "type": "assembly.completed.v1",
  "status": "success",
  "payload": {
    "clusters": [
      { "cluster_id": "cluster_auth", "label": "authentication", "member_count": 12 }
    ],
    "metrics": {
      "avg_cluster_size": 8.5,
      "graph_density": 0.42
    },
    "warnings": []
  }
}
```

**Status values:** `success`, `failed`

---

### Generation

#### `generation.started.v1`

LLM documentation generation has begun.

```json
{
  "type": "generation.started.v1",
  "payload": {
    "symbol_count": 65,
    "estimated_tokens": 15000
  }
}
```

#### `generation.symbol_documented.v1`

A single symbol has been documented.

```json
{
  "type": "generation.symbol_documented.v1",
  "payload": {
    "symbol_id": "sym_main_fn",
    "token_count": 87
  }
}
```

#### `generation.completed.v1`

All documentation generated.

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

**Status values:** `success`, `failed`

---

### Pack

#### `pack.started.v1`

Packer has begun assembling the final docpack.

```json
{
  "type": "pack.started.v1",
  "payload": {
    "artifact_count": 5
  }
}
```

#### `pack.completed.v1`

Docpack has been created and uploaded.

```json
{
  "type": "pack.completed.v1",
  "status": "success",
  "payload": {
    "docpack_url": "https://r2.doctown.dev/packs/abc123.docpack",
    "size_bytes": 524288,
    "checksum": "sha256:def456...",
    "schema_version": "docpack/1.0",
    "skipped_files": [],
    "warnings": []
  }
}
```

**Status values:** `success`, `failed`, `partial`

---

## Status Values Summary

| Status | Meaning | Valid On |
|--------|---------|----------|
| `success` | Completed without errors | All `.completed` events |
| `failed` | Stage failed, cannot continue | All `.completed` events |
| `partial` | Completed with some content skipped | `pack.completed.v1` only |
| `cancelled` | User or system cancelled | `job.completed.v1` only |
