## What is a .docpack?

A .docpack is an **immutable, content-addressed, reproducible bundle** containing:

- The semantic understanding of a codebase (graph, embeddings, AST/metadata)
- The human-readable documentation generated from that understanding
- Enough structure for future tools to query it without re-running the pipeline

## File layout

Everything inside the .docpack lives under a flat directory:

```
docpack/
├── manifest.json
├── graph.json
├── nodes.json
├── clusters.json
├── embeddings.bin         (optional)
├── symbol_contexts.json   (optional)
└── source_map.json
```

**Note the lack of folders**. From my experience, folders introduce schema drift; flat keeps it stable and diffable.

## File purposes

### manifest.json (Authoritative Metadata)
This is the root of truth. Everything else is subordinate to it.

```json
{
  "schema_version": "docpack/1.0",
  "docpack_id": "sha256:abc123...",
  "created_at": "2025-11-22T22:14:00Z",
  "generator": {
    "version": "doctown-packer/1.0.0",
    "pipeline_version": "v5.0"
  },
  "source": {
    "repo_url": "https://github.com/pyke/ort",
    "git_ref": "main",
    "commit_hash": "deadbeef"
  },
  "statistics": {
    "file_count": 42,
    "symbol_count": 128,
    "cluster_count": 12,
    "embedding_dimensions": 384
  },
  "checksum": {
    "algorithm": "sha256",
    "value": "def456..."
  },
  "optional": {
    "has_embeddings": true,
    "has_symbol_contexts": true
  }
}
```

#### Notes

- **Mandatory**: `manifest.json` MUST define everything the packer needs to re-verify the pack.
- `docpack_id` = hash of all included files (content-addressing).
- `optional` exists because you may skip embeddings for cost reasons in future tiers.

### graph.json (Global Semantic Graph)

```json
{
  "nodes": [],     // node_id list (metadata in nodes.json)
  "edges": [],     // adjacency list
  "metrics": {     // global graph metrics
    "density": 0.42,
    "avg_degree": 3.7
  }
}
```

#### Notes

- Graph files must remain small, indexable, and fast to load in the UI.
- Keep heavy fields in `nodes.json`.

### nodes.json (Symbol Table + Documentation)

```json
{
  "symbols": [
    {
      "id": "sym_main_fn",
      "name": "main",
      "kind": "function",
      "language": "rust",

      "file_path": "src/main.rs",
      "byte_range": [0, 200],

      "signature": "fn main()",
      "calls": ["sym_parse_args"],
      "called_by": [],
      "imports": ["std::env"],

      "cluster_id": "cluster_auth",
      "centrality": 0.84,

      "documentation": {
        "summary": "This function initializes the application...",
        "details": null
      }
    }
  ]
}
```

#### Notes

- `documentation.summary` is required.
- `documentation.details` optional for future "expanded doc mode."
- No embeddings here (keep them in `embeddings.bin`).
- `cluster_id` and graph properties help the UI.

### clusters.json

These are “semantic buckets” — used for navigation and UI.

```json
{
  "clusters": [
    {
      "cluster_id": "cluster_auth",
      "label": "authentication",
      "member_count": 12
    }
  ]
}
```

#### Notes

- Maybe add "representative_symbol": "sym_auth_login_handler" to help UI pick a title?

### embeddings.bin (Optional)

Binary, compact representation of embeddings. \
Why binary? Because JSON vectors explode file size by 4–10x.

```
// Header:
uint32: num_vectors
uint32: dimensions

// Followed by:
float32[dimensions] * num_vectors
```

And a separate source_map.json entry mapping chunk_id → offset.

### symbol_contexts.json (Optional)

This file only exists for:
- regeneration without recomputing graph
- debugging the LLM layer
- enterprise usage

```json
{
  "contexts": {
    "sym_main_fn": {
      "language": "rust",
      "signature": "fn main()",
      "imports": [...],
      "calls": [...],
      "called_by": [...],
      "cluster_label": "authentication",
      "related_symbols": ["sym_x", "sym_y"],
      "centrality": 0.84,
      "raw_prompt_text": "You are documenting a Rust codebase..."
    }
  }
}
```

#### Notes

- This whole file should be optional
- `raw_prompt_text` optional but insanely useful for deterministic regeneration.

### source_map.json

This is stable and important. It connects internal docpack structure back to file paths and byte ranges.

```json
{
  "files": [
    {
      "file_path": "src/main.rs",
      "language": "rust",
      "chunks": [
        {
          "chunk_id": "chunk_abc",
          "byte_range": [0, 200],
          "symbol_ids": ["sym_main_fn"]
        }
      ]
    }
  ]
}
```

This allows:
- UI highlighting
- mapping docpack → original source
- incremental rebuilds later

Don't fuck with this lol.