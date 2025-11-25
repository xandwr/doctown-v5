use crate::{
    Clusters, EmbeddingsError, EmbeddingsReader, Graph, Manifest, Nodes, SourceMap, SymbolContexts,
};
use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};
use std::io::{self, Read};
use tar::Archive;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("JSON deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Tar error: {0}")]
    Tar(String),

    #[error("Missing required file: {0}")]
    MissingFile(String),

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("Invalid docpack format: {0}")]
    InvalidFormat(String),

    #[error("Schema version mismatch: expected {expected}, got {actual}")]
    SchemaVersionMismatch { expected: String, actual: String },

    #[error("Embeddings error: {0}")]
    Embeddings(#[from] EmbeddingsError),
}

pub type Result<T> = std::result::Result<T, ReadError>;

/// Reader for extracting .docpack archives
pub struct DocpackReader {
    manifest: Manifest,
    graph: Graph,
    nodes: Nodes,
    clusters: Clusters,
    source_map: SourceMap,
    embeddings: Option<EmbeddingsReader>,
    symbol_contexts: Option<SymbolContexts>,
}

impl DocpackReader {
    /// Read and parse a docpack from bytes
    pub fn read(bytes: &[u8]) -> Result<Self> {
        // Decompress gzip
        let mut decoder = GzDecoder::new(bytes);
        let mut tar_bytes = Vec::new();
        decoder.read_to_end(&mut tar_bytes)?;

        // Extract tar archive
        let mut archive = Archive::new(&tar_bytes[..]);
        let mut files = std::collections::HashMap::new();

        for entry in archive
            .entries()
            .map_err(|e| ReadError::Tar(e.to_string()))?
        {
            let mut entry = entry.map_err(|e| ReadError::Tar(e.to_string()))?;
            let path = entry
                .path()
                .map_err(|e| ReadError::Tar(e.to_string()))?
                .to_string_lossy()
                .to_string();

            let mut content = Vec::new();
            entry.read_to_end(&mut content)?;
            files.insert(path, content);
        }

        // Parse required files
        let manifest_bytes = files
            .get("manifest.json")
            .ok_or_else(|| ReadError::MissingFile("manifest.json".to_string()))?;
        let manifest = Manifest::from_json_bytes(manifest_bytes)?;

        let graph_bytes = files
            .get("graph.json")
            .ok_or_else(|| ReadError::MissingFile("graph.json".to_string()))?;
        let graph = Graph::from_json_bytes(graph_bytes)?;

        let nodes_bytes = files
            .get("nodes.json")
            .ok_or_else(|| ReadError::MissingFile("nodes.json".to_string()))?;
        let nodes = Nodes::from_json_bytes(nodes_bytes)?;

        let clusters_bytes = files
            .get("clusters.json")
            .ok_or_else(|| ReadError::MissingFile("clusters.json".to_string()))?;
        let clusters = Clusters::from_json_bytes(clusters_bytes)?;

        let source_map_bytes = files
            .get("source_map.json")
            .ok_or_else(|| ReadError::MissingFile("source_map.json".to_string()))?;
        let source_map = SourceMap::from_json_bytes(source_map_bytes)?;

        // Parse optional files
        let embeddings = if let Some(embeddings_bytes) = files.get("embeddings.bin") {
            Some(EmbeddingsReader::read(embeddings_bytes.clone())?)
        } else {
            None
        };

        let symbol_contexts = if let Some(contexts_bytes) = files.get("symbol_contexts.json") {
            Some(SymbolContexts::from_json_bytes(contexts_bytes)?)
        } else {
            None
        };

        // Verify checksum (include optional files if present)
        let mut hasher = Sha256::new();
        hasher.update(graph_bytes);
        hasher.update(nodes_bytes);
        hasher.update(clusters_bytes);
        hasher.update(source_map_bytes);
        if let Some(ref emb_bytes) = files.get("embeddings.bin") {
            hasher.update(emb_bytes);
        }
        if let Some(ref ctx_bytes) = files.get("symbol_contexts.json") {
            hasher.update(ctx_bytes);
        }
        let checksum_hash = hasher.finalize();
        let computed_checksum = format!("{:x}", checksum_hash);

        if computed_checksum != manifest.checksum.value {
            return Err(ReadError::ChecksumMismatch {
                expected: manifest.checksum.value.clone(),
                actual: computed_checksum,
            });
        }

        // Verify schema version
        Self::verify_schema_version(&manifest)?;

        Ok(Self {
            manifest,
            graph,
            nodes,
            clusters,
            source_map,
            embeddings,
            symbol_contexts,
        })
    }

    /// Verify the schema version is compatible
    fn verify_schema_version(manifest: &Manifest) -> Result<()> {
        const SUPPORTED_VERSION: &str = "docpack/1.0";
        if manifest.schema_version != SUPPORTED_VERSION {
            return Err(ReadError::SchemaVersionMismatch {
                expected: SUPPORTED_VERSION.to_string(),
                actual: manifest.schema_version.clone(),
            });
        }
        Ok(())
    }

    /// Get the manifest
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Get the graph
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Get the nodes
    pub fn nodes(&self) -> &Nodes {
        &self.nodes
    }

    /// Get the clusters
    pub fn clusters(&self) -> &Clusters {
        &self.clusters
    }

    /// Get the source map
    pub fn source_map(&self) -> &SourceMap {
        &self.source_map
    }

    /// Get the embeddings (if present)
    pub fn embeddings(&self) -> Option<&EmbeddingsReader> {
        self.embeddings.as_ref()
    }

    /// Get the symbol contexts (if present)
    pub fn symbol_contexts(&self) -> Option<&SymbolContexts> {
        self.symbol_contexts.as_ref()
    }

    /// Check if embeddings are available
    pub fn has_embeddings(&self) -> bool {
        self.embeddings.is_some()
    }

    /// Check if symbol contexts are available
    pub fn has_symbol_contexts(&self) -> bool {
        self.symbol_contexts.is_some()
    }

    /// Consume the reader and return all components
    pub fn into_parts(self) -> (Manifest, Graph, Nodes, Clusters, SourceMap) {
        (
            self.manifest,
            self.graph,
            self.nodes,
            self.clusters,
            self.source_map,
        )
    }

    /// Consume the reader and return all components including optional ones
    pub fn into_parts_with_optional(
        self,
    ) -> (
        Manifest,
        Graph,
        Nodes,
        Clusters,
        SourceMap,
        Option<EmbeddingsReader>,
        Option<SymbolContexts>,
    ) {
        (
            self.manifest,
            self.graph,
            self.nodes,
            self.clusters,
            self.source_map,
            self.embeddings,
            self.symbol_contexts,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{writer::DocpackWriter, Cluster, Edge, Symbol};

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
    fn test_read_docpack() {
        let writer = DocpackWriter::new();
        let manifest = create_test_manifest();
        let graph = create_test_graph();
        let nodes = create_test_nodes();
        let clusters = create_test_clusters();
        let source_map = create_test_source_map();

        let bytes = writer
            .write(manifest, &graph, &nodes, &clusters, &source_map)
            .unwrap();

        let reader = DocpackReader::read(&bytes);
        assert!(reader.is_ok());

        let reader = reader.unwrap();
        assert_eq!(
            reader.manifest().source.repo_url,
            "https://github.com/test/repo"
        );
        assert_eq!(reader.graph().node_count(), 2);
        assert_eq!(reader.nodes().len(), 2);
        assert_eq!(reader.clusters().len(), 1);
        assert_eq!(reader.source_map().file_count(), 1);
    }

    #[test]
    fn test_roundtrip() {
        let writer = DocpackWriter::new();
        let manifest = create_test_manifest();
        let graph = create_test_graph();
        let nodes = create_test_nodes();
        let clusters = create_test_clusters();
        let source_map = create_test_source_map();

        // Write
        let bytes = writer
            .write(manifest.clone(), &graph, &nodes, &clusters, &source_map)
            .unwrap();

        // Read
        let reader = DocpackReader::read(&bytes).unwrap();

        // Verify
        assert_eq!(reader.manifest().source.repo_url, manifest.source.repo_url);
        assert_eq!(reader.graph().nodes, graph.nodes);
        assert_eq!(reader.nodes().symbols[0].id, nodes.symbols[0].id);
        assert_eq!(
            reader.clusters().clusters[0].cluster_id,
            clusters.clusters[0].cluster_id
        );
    }

    #[test]
    fn test_corrupted_docpack_rejected() {
        let writer = DocpackWriter::new();
        let manifest = create_test_manifest();
        let graph = create_test_graph();
        let nodes = create_test_nodes();
        let clusters = create_test_clusters();
        let source_map = create_test_source_map();

        let mut bytes = writer
            .write(manifest, &graph, &nodes, &clusters, &source_map)
            .unwrap();

        // Corrupt some bytes in the middle
        if bytes.len() > 100 {
            bytes[50] ^= 0xFF;
            bytes[51] ^= 0xFF;
        }

        let result = DocpackReader::read(&bytes);
        // Should fail either during decompression or checksum validation
        assert!(result.is_err());
    }

    #[test]
    fn test_into_parts() {
        let writer = DocpackWriter::new();
        let manifest = create_test_manifest();
        let graph = create_test_graph();
        let nodes = create_test_nodes();
        let clusters = create_test_clusters();
        let source_map = create_test_source_map();

        let bytes = writer
            .write(manifest, &graph, &nodes, &clusters, &source_map)
            .unwrap();

        let reader = DocpackReader::read(&bytes).unwrap();
        let (manifest, graph, nodes, clusters, source_map) = reader.into_parts();

        assert_eq!(manifest.source.repo_url, "https://github.com/test/repo");
        assert_eq!(graph.node_count(), 2);
        assert_eq!(nodes.len(), 2);
        assert_eq!(clusters.len(), 1);
        assert_eq!(source_map.file_count(), 1);
    }
}
