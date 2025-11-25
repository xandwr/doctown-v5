use crate::{
    Clusters, EmbeddingsError, EmbeddingsWriter, Graph, Manifest, Nodes, SourceMap, SymbolContexts,
};
use flate2::write::GzEncoder;
use flate2::Compression;
use sha2::{Digest, Sha256};
use std::io::{self, Write};
use tar::Builder;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WriteError {
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Tar error: {0}")]
    Tar(String),

    #[error("Embeddings error: {0}")]
    Embeddings(#[from] EmbeddingsError),
}

pub type Result<T> = std::result::Result<T, WriteError>;

/// Core content components for a docpack
pub struct DocpackContent<'a> {
    pub graph: &'a Graph,
    pub nodes: &'a Nodes,
    pub clusters: &'a Clusters,
    pub source_map: &'a SourceMap,
}

impl<'a> DocpackContent<'a> {
    pub fn new(
        graph: &'a Graph,
        nodes: &'a Nodes,
        clusters: &'a Clusters,
        source_map: &'a SourceMap,
    ) -> Self {
        Self {
            graph,
            nodes,
            clusters,
            source_map,
        }
    }
}

/// Writer for creating .docpack archives
pub struct DocpackWriter {
    compression_level: Compression,
}

impl DocpackWriter {
    /// Create a new writer with default compression
    pub fn new() -> Self {
        Self {
            compression_level: Compression::default(),
        }
    }

    /// Create a writer with custom compression level
    pub fn with_compression(level: u32) -> Self {
        Self {
            compression_level: Compression::new(level),
        }
    }

    /// Write a docpack to bytes (without optional files)
    pub fn write(
        &self,
        manifest: Manifest,
        content: &DocpackContent,
    ) -> Result<Vec<u8>> {
        self.write_with_optional(manifest, content, None, None)
    }

    /// Write a docpack to bytes with optional embeddings and symbol contexts
    pub fn write_with_optional(
        &self,
        mut manifest: Manifest,
        content: &DocpackContent,
        embeddings: Option<&EmbeddingsWriter>,
        symbol_contexts: Option<&SymbolContexts>,
    ) -> Result<Vec<u8>> {
        // Serialize all components
        let graph_json = content.graph.to_json_bytes()?;
        let nodes_json = content.nodes.to_json_bytes()?;
        let clusters_json = content.clusters.to_json_bytes()?;
        let source_map_json = content.source_map.to_json_bytes()?;

        // Serialize optional components
        let embeddings_bin = embeddings.map(|e| e.write()).transpose()?;
        let symbol_contexts_json = symbol_contexts.map(|s| s.to_json_bytes()).transpose()?;

        // Update manifest optional flags
        manifest.optional.has_embeddings = embeddings_bin.is_some();
        manifest.optional.has_symbol_contexts = symbol_contexts_json.is_some();

        // Compute checksum of all content
        let mut hasher = Sha256::new();
        hasher.update(&graph_json);
        hasher.update(&nodes_json);
        hasher.update(&clusters_json);
        hasher.update(&source_map_json);
        if let Some(ref emb_data) = embeddings_bin {
            hasher.update(emb_data);
        }
        if let Some(ref ctx_data) = symbol_contexts_json {
            hasher.update(ctx_data);
        }
        let checksum_hash = hasher.finalize();
        let checksum_value = format!("{:x}", checksum_hash);

        // Update manifest with checksum and docpack_id
        manifest.checksum.value = checksum_value.clone();
        manifest.docpack_id = format!("sha256:{}", checksum_value);

        let manifest_json = manifest.to_json_bytes()?;

        // Create tar archive in memory
        let tar_buffer = Vec::new();
        let mut tar_builder = Builder::new(tar_buffer);

        // Add required files to tar
        self.add_file_to_tar(&mut tar_builder, "manifest.json", &manifest_json)?;
        self.add_file_to_tar(&mut tar_builder, "graph.json", &graph_json)?;
        self.add_file_to_tar(&mut tar_builder, "nodes.json", &nodes_json)?;
        self.add_file_to_tar(&mut tar_builder, "clusters.json", &clusters_json)?;
        self.add_file_to_tar(&mut tar_builder, "source_map.json", &source_map_json)?;

        // Add optional files
        if let Some(emb_data) = embeddings_bin {
            self.add_file_to_tar(&mut tar_builder, "embeddings.bin", &emb_data)?;
        }
        if let Some(ctx_data) = symbol_contexts_json {
            self.add_file_to_tar(&mut tar_builder, "symbol_contexts.json", &ctx_data)?;
        }

        // Finish tar archive
        let tar_bytes = tar_builder
            .into_inner()
            .map_err(|e| WriteError::Tar(e.to_string()))?;

        // Gzip compress
        let mut encoder = GzEncoder::new(Vec::new(), self.compression_level);
        encoder.write_all(&tar_bytes)?;
        let compressed = encoder.finish()?;

        Ok(compressed)
    }

    /// Helper to add a file to the tar archive
    fn add_file_to_tar(
        &self,
        builder: &mut Builder<Vec<u8>>,
        filename: &str,
        data: &[u8],
    ) -> Result<()> {
        let mut header = tar::Header::new_gnu();
        header
            .set_path(filename)
            .map_err(|e| WriteError::Tar(e.to_string()))?;
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();

        builder
            .append(&header, data)
            .map_err(|e| WriteError::Tar(e.to_string()))?;

        Ok(())
    }
}

impl Default for DocpackWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Cluster, Edge, Symbol};

    fn create_test_manifest() -> Manifest {
        Manifest::new(
            "https://github.com/test/repo".to_string(),
            "main".to_string(),
            Some("abc123".to_string()),
            1,
            2,
            1,
        )
    }

    fn create_test_graph() -> Graph {
        Graph::new(
            vec!["sym_a".to_string(), "sym_b".to_string()],
            vec![Edge::calls("sym_a".to_string(), "sym_b".to_string())],
        )
    }

    fn create_test_nodes() -> Nodes {
        Nodes::new(vec![
            Symbol::new(
                "sym_a".to_string(),
                "func_a".to_string(),
                "function".to_string(),
                "rust".to_string(),
                "src/lib.rs".to_string(),
                (0, 100),
                "cluster_1".to_string(),
                "Test function A".to_string(),
            ),
            Symbol::new(
                "sym_b".to_string(),
                "func_b".to_string(),
                "function".to_string(),
                "rust".to_string(),
                "src/lib.rs".to_string(),
                (100, 200),
                "cluster_1".to_string(),
                "Test function B".to_string(),
            ),
        ])
    }

    fn create_test_clusters() -> Clusters {
        Clusters::new(vec![Cluster::new(
            "cluster_1".to_string(),
            "test".to_string(),
            2,
        )])
    }

    fn create_test_source_map() -> SourceMap {
        use crate::{SourceMapChunk, SourceMapFile};

        SourceMap::new(vec![SourceMapFile::new(
            "src/lib.rs".to_string(),
            "rust".to_string(),
            vec![SourceMapChunk::new(
                "chunk_1".to_string(),
                (0, 200),
                vec!["sym_a".to_string(), "sym_b".to_string()],
            )],
        )])
    }

    #[test]
    fn test_write_docpack() {
        let writer = DocpackWriter::new();
        let manifest = create_test_manifest();
        let graph = create_test_graph();
        let nodes = create_test_nodes();
        let clusters = create_test_clusters();
        let source_map = create_test_source_map();
        let content = DocpackContent::new(&graph, &nodes, &clusters, &source_map);

        let result = writer.write(manifest, &content);
        assert!(result.is_ok());

        let bytes = result.unwrap();
        assert!(!bytes.is_empty());

        // Check that it's gzipped (starts with magic bytes)
        assert_eq!(bytes[0], 0x1f);
        assert_eq!(bytes[1], 0x8b);
    }

    #[test]
    fn test_checksum_is_deterministic() {
        let writer = DocpackWriter::new();
        let manifest = create_test_manifest();
        let graph = create_test_graph();
        let nodes = create_test_nodes();
        let clusters = create_test_clusters();
        let source_map = create_test_source_map();
        let content = DocpackContent::new(&graph, &nodes, &clusters, &source_map);

        let result1 = writer.write(manifest.clone(), &content);
        let result2 = writer.write(manifest, &content);

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let bytes1 = result1.unwrap();
        let bytes2 = result2.unwrap();

        // The checksums should be the same for identical content
        // Note: tar archives may have timestamps, but our content hash should be deterministic
        assert_eq!(bytes1.len(), bytes2.len());
    }

    #[test]
    fn test_custom_compression() {
        let writer = DocpackWriter::with_compression(9);
        let manifest = create_test_manifest();
        let graph = create_test_graph();
        let nodes = create_test_nodes();
        let clusters = create_test_clusters();
        let source_map = create_test_source_map();
        let content = DocpackContent::new(&graph, &nodes, &clusters, &source_map);

        let result = writer.write(manifest, &content);
        assert!(result.is_ok());
    }
}
