# doctown-docpack

Rust implementation of the `.docpack` format - an immutable, content-addressed, reproducible bundle containing semantic understanding of codebases.

## Format

A `.docpack` is a gzipped tar archive containing:
- `manifest.json` - Metadata and checksums
- `graph.json` - Global semantic graph
- `nodes.json` - Symbol table with documentation
- `clusters.json` - Semantic buckets for navigation
- `source_map.json` - Maps internal structure to source files

Optional files:
- `embeddings.bin` - Binary embedding vectors
- `symbol_contexts.json` - Regeneration contexts

See `specs/docpack.md` for full specification.

## Usage

### Creating a basic docpack

```rust
use doctown_docpack::{DocpackWriter, Manifest, Graph, Nodes, Clusters, SourceMap};

let writer = DocpackWriter::new();
let bytes = writer.write(manifest, &graph, &nodes, &clusters, &source_map)?;
```

### Creating a docpack with embeddings

```rust
use doctown_docpack::{DocpackWriter, EmbeddingsWriter, Manifest};

// Create embeddings
let mut embeddings = EmbeddingsWriter::new(384);
embeddings.add_vector("chunk_1".to_string(), vec![0.1; 384])?;

// Write docpack with embeddings
let writer = DocpackWriter::new();
let bytes = writer.write_with_optional(
    manifest,
    &graph,
    &nodes,
    &clusters,
    &source_map,
    Some(&embeddings),
    None,
)?;
```

### Reading a docpack

```rust
use doctown_docpack::DocpackReader;

let reader = DocpackReader::read(&bytes)?;
let manifest = reader.manifest();
let nodes = reader.nodes();

// Access optional files
if reader.has_embeddings() {
    let embeddings = reader.embeddings().unwrap();
    let vector = embeddings.get_vector("chunk_1")?;
}

if reader.has_symbol_contexts() {
    let contexts = reader.symbol_contexts().unwrap();
    let context = contexts.get_context("sym_a");
}
```
