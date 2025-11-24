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

### Creating a docpack

```rust
use doctown_docpack::{DocpackWriter, Manifest, Graph, Nodes, Clusters, SourceMap};

let writer = DocpackWriter::new();
let bytes = writer.write(manifest, graph, nodes, clusters, source_map)?;
```

### Reading a docpack

```rust
use doctown_docpack::DocpackReader;

let reader = DocpackReader::new(&bytes)?;
let manifest = reader.manifest();
let nodes = reader.nodes();
```
