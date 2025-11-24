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
mod embeddings;
mod graph;
mod manifest;
mod nodes;
mod reader;
mod source_map;
mod symbol_contexts;
mod writer;

// Re-export public types
pub use clusters::{Cluster, Clusters};
pub use embeddings::{EmbeddingsError, EmbeddingsHeader, EmbeddingsReader, EmbeddingsWriter};
pub use graph::{Edge, Graph, GraphMetrics};
pub use manifest::{Checksum, Generator, Manifest, OptionalFeatures, Source, Statistics};
pub use nodes::{Documentation, Nodes, Symbol};
pub use reader::{DocpackReader, ReadError};
pub use source_map::{SourceMap, SourceMapChunk, SourceMapFile};
pub use symbol_contexts::{SymbolContext, SymbolContexts};
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
        assert!(!reader.has_embeddings());
        assert!(!reader.has_symbol_contexts());
    }

    #[test]
    fn test_docpack_with_embeddings() {
        // Create docpack with embeddings
        let manifest = Manifest::new(
            "https://github.com/test/repo".to_string(),
            "main".to_string(),
            None,
            1,
            2,
            1,
        );

        let graph = Graph::new(vec!["sym_a".to_string()], vec![]);
        let nodes = Nodes::new(vec![Symbol::new(
            "sym_a".to_string(),
            "test".to_string(),
            "function".to_string(),
            "rust".to_string(),
            "src/lib.rs".to_string(),
            (0, 100),
            "cluster_1".to_string(),
            "Test".to_string(),
        )]);
        let clusters = Clusters::new(vec![Cluster::new(
            "cluster_1".to_string(),
            "test".to_string(),
            1,
        )]);
        let source_map = SourceMap::new(vec![SourceMapFile::new(
            "src/lib.rs".to_string(),
            "rust".to_string(),
            vec![SourceMapChunk::new(
                "chunk_1".to_string(),
                (0, 100),
                vec!["sym_a".to_string()],
            )],
        )]);

        // Create embeddings
        let mut embeddings_writer = EmbeddingsWriter::new(384);
        let vector: Vec<f32> = (0..384).map(|i| i as f32 * 0.01).collect();
        embeddings_writer
            .add_vector("chunk_1".to_string(), vector.clone())
            .unwrap();

        let writer = DocpackWriter::new();
        let bytes = writer
            .write_with_optional(
                manifest,
                &graph,
                &nodes,
                &clusters,
                &source_map,
                Some(&embeddings_writer),
                None,
            )
            .expect("Failed to write docpack with embeddings");

        let reader = DocpackReader::read(&bytes).expect("Failed to read docpack with embeddings");

        assert!(reader.has_embeddings());
        assert!(!reader.has_symbol_contexts());

        let embeddings = reader.embeddings().unwrap();
        assert_eq!(embeddings.dimensions(), 384);
        assert_eq!(embeddings.num_vectors(), 1);

        let retrieved_vector = embeddings.get_vector("chunk_1").unwrap();
        assert_eq!(retrieved_vector, vector);
    }

    #[test]
    fn test_docpack_with_symbol_contexts() {
        // Create docpack with symbol contexts
        let manifest = Manifest::new(
            "https://github.com/test/repo".to_string(),
            "main".to_string(),
            None,
            1,
            1,
            1,
        );

        let graph = Graph::new(vec!["sym_a".to_string()], vec![]);
        let nodes = Nodes::new(vec![Symbol::new(
            "sym_a".to_string(),
            "test".to_string(),
            "function".to_string(),
            "rust".to_string(),
            "src/lib.rs".to_string(),
            (0, 100),
            "cluster_1".to_string(),
            "Test".to_string(),
        )]);
        let clusters = Clusters::new(vec![Cluster::new(
            "cluster_1".to_string(),
            "test".to_string(),
            1,
        )]);
        let source_map = SourceMap::empty();

        // Create symbol contexts
        let symbol_contexts = SymbolContexts::new(vec![SymbolContext::new(
            "sym_a".to_string(),
            "Explain this function".to_string(),
        )
        .with_model("gpt-4".to_string())
        .with_temperature(0.7)]);

        let writer = DocpackWriter::new();
        let bytes = writer
            .write_with_optional(
                manifest,
                &graph,
                &nodes,
                &clusters,
                &source_map,
                None,
                Some(&symbol_contexts),
            )
            .expect("Failed to write docpack with contexts");

        let reader = DocpackReader::read(&bytes).expect("Failed to read docpack with contexts");

        assert!(!reader.has_embeddings());
        assert!(reader.has_symbol_contexts());

        let contexts = reader.symbol_contexts().unwrap();
        assert_eq!(contexts.len(), 1);

        let context = contexts.get_context("sym_a").unwrap();
        assert_eq!(context.prompt, "Explain this function");
        assert_eq!(context.model, Some("gpt-4".to_string()));
        assert_eq!(context.temperature, Some(0.7));
    }

    #[test]
    fn test_docpack_with_all_optional_files() {
        // Create docpack with both embeddings and symbol contexts
        let manifest = Manifest::new(
            "https://github.com/test/repo".to_string(),
            "main".to_string(),
            None,
            1,
            1,
            1,
        );

        let graph = Graph::new(vec!["sym_a".to_string()], vec![]);
        let nodes = Nodes::new(vec![Symbol::new(
            "sym_a".to_string(),
            "test".to_string(),
            "function".to_string(),
            "rust".to_string(),
            "src/lib.rs".to_string(),
            (0, 100),
            "cluster_1".to_string(),
            "Test".to_string(),
        )]);
        let clusters = Clusters::new(vec![Cluster::new(
            "cluster_1".to_string(),
            "test".to_string(),
            1,
        )]);
        let source_map = SourceMap::new(vec![SourceMapFile::new(
            "src/lib.rs".to_string(),
            "rust".to_string(),
            vec![SourceMapChunk::new(
                "chunk_1".to_string(),
                (0, 100),
                vec!["sym_a".to_string()],
            )],
        )]);

        // Create embeddings
        let mut embeddings_writer = EmbeddingsWriter::new(384);
        let vector: Vec<f32> = (0..384).map(|i| i as f32 * 0.01).collect();
        embeddings_writer
            .add_vector("chunk_1".to_string(), vector.clone())
            .unwrap();

        // Create symbol contexts
        let symbol_contexts = SymbolContexts::new(vec![SymbolContext::new(
            "sym_a".to_string(),
            "Test prompt".to_string(),
        )]);

        let writer = DocpackWriter::new();
        let bytes = writer
            .write_with_optional(
                manifest,
                &graph,
                &nodes,
                &clusters,
                &source_map,
                Some(&embeddings_writer),
                Some(&symbol_contexts),
            )
            .expect("Failed to write docpack");

        let reader = DocpackReader::read(&bytes).expect("Failed to read docpack");

        assert!(reader.has_embeddings());
        assert!(reader.has_symbol_contexts());
        assert_eq!(reader.manifest().optional.has_embeddings, true);
        assert_eq!(reader.manifest().optional.has_symbol_contexts, true);
    }

    #[test]
    fn test_schema_version_validation() {
        // Create a docpack
        let manifest = Manifest::new(
            "https://github.com/test/repo".to_string(),
            "main".to_string(),
            None,
            0,
            0,
            0,
        );

        let writer = DocpackWriter::new();
        let bytes = writer
            .write(
                manifest,
                &Graph::empty(),
                &Nodes::empty(),
                &Clusters::empty(),
                &SourceMap::empty(),
            )
            .unwrap();

        // Should succeed with correct version
        let reader = DocpackReader::read(&bytes);
        assert!(reader.is_ok());
        assert_eq!(reader.unwrap().manifest().schema_version, "docpack/1.0");
    }
}
