//! # doctown-docpack
//!
//! Rust implementation of the `.docpack` format - an immutable, content-addressed,
//! reproducible bundle containing semantic understanding of codebases.
//!
//! ## Format
//!
//! A `.docpack` is a gzipped tar archive containing:
//! - `manifest.json` - Metadata and checksums
//! - `graph.json` - Global semantic graph
//! - `nodes.json` - Symbol table with documentation
//! - `clusters.json` - Semantic buckets for navigation
//! - `source_map.json` - Maps internal structure to source files
//!
//! Optional files (not yet implemented):
//! - `embeddings.bin` - Binary embedding vectors
//! - `symbol_contexts.json` - Regeneration contexts
//!
//! ## Usage
//!
//! ### Creating a docpack
//!
//! ```rust
//! use doctown_docpack::{
//!     DocpackWriter, Manifest, Graph, Nodes, Clusters, SourceMap, Symbol, Cluster,
//!     SourceMapFile, SourceMapChunk, Edge
//! };
//!
//! // Create components
//! let manifest = Manifest::new(
//!     "https://github.com/user/repo".to_string(),
//!     "main".to_string(),
//!     Some("abc123".to_string()),
//!     1,  // file_count
//!     2,  // symbol_count
//!     1,  // cluster_count
//! );
//!
//! let graph = Graph::new(
//!     vec!["sym_a".to_string(), "sym_b".to_string()],
//!     vec![Edge::calls("sym_a".to_string(), "sym_b".to_string())],
//! );
//!
//! let nodes = Nodes::new(vec![
//!     Symbol::new(
//!         "sym_a".to_string(),
//!         "func_a".to_string(),
//!         "function".to_string(),
//!         "rust".to_string(),
//!         "src/lib.rs".to_string(),
//!         (0, 100),
//!         "cluster_1".to_string(),
//!         "A test function".to_string(),
//!     ),
//! ]);
//!
//! let clusters = Clusters::new(vec![
//!     Cluster::new("cluster_1".to_string(), "test".to_string(), 1),
//! ]);
//!
//! let source_map = SourceMap::new(vec![
//!     SourceMapFile::new(
//!         "src/lib.rs".to_string(),
//!         "rust".to_string(),
//!         vec![SourceMapChunk::new(
//!             "chunk_1".to_string(),
//!             (0, 100),
//!             vec!["sym_a".to_string()],
//!         )],
//!     ),
//! ]);
//!
//! // Write to bytes
//! let writer = DocpackWriter::new();
//! let bytes = writer.write(manifest, &graph, &nodes, &clusters, &source_map).unwrap();
//! ```
//!
//! ### Reading a docpack
//!
//! ```rust
//! use doctown_docpack::DocpackReader;
//! # use doctown_docpack::{DocpackWriter, Manifest, Graph, Nodes, Clusters, SourceMap, Symbol, Cluster, SourceMapFile, SourceMapChunk, Edge};
//! # let manifest = Manifest::new("https://github.com/user/repo".to_string(), "main".to_string(), None, 0, 0, 0);
//! # let graph = Graph::empty();
//! # let nodes = Nodes::empty();
//! # let clusters = Clusters::empty();
//! # let source_map = SourceMap::empty();
//! # let writer = DocpackWriter::new();
//! # let bytes = writer.write(manifest, &graph, &nodes, &clusters, &source_map).unwrap();
//!
//! let reader = DocpackReader::read(&bytes).unwrap();
//! let manifest = reader.manifest();
//! let nodes = reader.nodes();
//! ```

mod clusters;
mod graph;
mod manifest;
mod nodes;
mod reader;
mod source_map;
mod writer;

// Re-export public types
pub use clusters::{Cluster, Clusters};
pub use graph::{Edge, Graph, GraphMetrics};
pub use manifest::{Checksum, Generator, Manifest, OptionalFeatures, Source, Statistics};
pub use nodes::{Documentation, Nodes, Symbol};
pub use reader::{DocpackReader, ReadError};
pub use source_map::{SourceMap, SourceMapChunk, SourceMapFile};
pub use writer::{DocpackWriter, WriteError};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_roundtrip() {
        // Create a complete docpack
        let manifest = Manifest::new(
            "https://github.com/test/repo".to_string(),
            "main".to_string(),
            Some("deadbeef".to_string()),
            2,
            4,
            2,
        );

        let graph = Graph::new(
            vec![
                "sym_main".to_string(),
                "sym_helper".to_string(),
                "sym_util".to_string(),
            ],
            vec![
                Edge::calls("sym_main".to_string(), "sym_helper".to_string()),
                Edge::calls("sym_helper".to_string(), "sym_util".to_string()),
            ],
        );

        let nodes = Nodes::new(vec![
            Symbol::new(
                "sym_main".to_string(),
                "main".to_string(),
                "function".to_string(),
                "rust".to_string(),
                "src/main.rs".to_string(),
                (0, 100),
                "cluster_init".to_string(),
                "Main entry point".to_string(),
            )
            .with_signature("fn main()".to_string())
            .with_centrality(0.9),
            Symbol::new(
                "sym_helper".to_string(),
                "helper".to_string(),
                "function".to_string(),
                "rust".to_string(),
                "src/lib.rs".to_string(),
                (0, 50),
                "cluster_utils".to_string(),
                "Helper function".to_string(),
            ),
        ]);

        let clusters = Clusters::new(vec![
            Cluster::new("cluster_init".to_string(), "initialization".to_string(), 1),
            Cluster::new("cluster_utils".to_string(), "utilities".to_string(), 2),
        ]);

        let source_map = SourceMap::new(vec![
            SourceMapFile::new(
                "src/main.rs".to_string(),
                "rust".to_string(),
                vec![SourceMapChunk::new(
                    "chunk_main".to_string(),
                    (0, 100),
                    vec!["sym_main".to_string()],
                )],
            ),
            SourceMapFile::new(
                "src/lib.rs".to_string(),
                "rust".to_string(),
                vec![SourceMapChunk::new(
                    "chunk_lib".to_string(),
                    (0, 100),
                    vec!["sym_helper".to_string(), "sym_util".to_string()],
                )],
            ),
        ]);

        // Write
        let writer = DocpackWriter::new();
        let bytes = writer
            .write(manifest.clone(), &graph, &nodes, &clusters, &source_map)
            .expect("Failed to write docpack");

        // Read
        let reader = DocpackReader::read(&bytes).expect("Failed to read docpack");

        // Verify manifest
        assert_eq!(reader.manifest().source.repo_url, "https://github.com/test/repo");
        assert_eq!(reader.manifest().source.git_ref, "main");
        assert_eq!(reader.manifest().source.commit_hash, Some("deadbeef".to_string()));
        assert_eq!(reader.manifest().statistics.file_count, 2);
        assert_eq!(reader.manifest().statistics.symbol_count, 4);
        assert_eq!(reader.manifest().statistics.cluster_count, 2);

        // Verify graph
        assert_eq!(reader.graph().node_count(), 3);
        assert_eq!(reader.graph().edge_count(), 2);

        // Verify nodes
        assert_eq!(reader.nodes().len(), 2);
        assert_eq!(reader.nodes().symbols[0].name, "main");
        assert_eq!(reader.nodes().symbols[0].signature, Some("fn main()".to_string()));
        assert_eq!(reader.nodes().symbols[0].centrality, 0.9);

        // Verify clusters
        assert_eq!(reader.clusters().len(), 2);
        assert_eq!(reader.clusters().clusters[0].label, "initialization");

        // Verify source map
        assert_eq!(reader.source_map().file_count(), 2);
        assert_eq!(reader.source_map().files[0].file_path, "src/main.rs");
    }

    #[test]
    fn test_minimal_docpack() {
        // Create the smallest valid docpack
        let manifest = Manifest::new(
            "https://github.com/test/minimal".to_string(),
            "main".to_string(),
            None,
            0,
            0,
            0,
        );

        let graph = Graph::empty();
        let nodes = Nodes::empty();
        let clusters = Clusters::empty();
        let source_map = SourceMap::empty();

        let writer = DocpackWriter::new();
        let bytes = writer
            .write(manifest, &graph, &nodes, &clusters, &source_map)
            .expect("Failed to write minimal docpack");

        let reader = DocpackReader::read(&bytes).expect("Failed to read minimal docpack");

        assert_eq!(reader.nodes().len(), 0);
        assert_eq!(reader.graph().node_count(), 0);
        assert_eq!(reader.clusters().len(), 0);
        assert_eq!(reader.source_map().file_count(), 0);
    }
}
