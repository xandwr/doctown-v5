//! Packer module for assembling complete .docpack files.
//!
//! This module takes all the assembled artifacts (clusters, graph, nodes, source map,
//! embeddings, symbol contexts) and packages them into a reproducible .docpack file.

use doctown_docpack::{
    Cluster, Clusters, DocpackWriter, Edge, Graph, Manifest, Nodes, SourceMap, SourceMapChunk,
    SourceMapFile, Symbol,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::context::SymbolContext;

/// Packer for assembling docpack files
pub struct Packer {
    // Stateless for now
}

/// Request to pack a docpack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackRequest {
    /// Repository metadata
    pub repo_url: String,
    pub git_ref: String,
    pub commit_hash: Option<String>,

    /// Source files and chunks from ingest
    pub source_files: Vec<SourceFileInfo>,

    /// Cluster assignments from assembly
    pub cluster_assignments: HashMap<String, String>, // symbol_id -> cluster_id
    pub cluster_labels: HashMap<String, String>, // cluster_id -> label

    /// Graph from assembly
    pub nodes: Vec<NodeInfo>,
    pub edges: Vec<EdgeInfo>,

    /// Optional: embeddings data
    pub embeddings: Option<EmbeddingData>,

    /// Optional: symbol contexts for reproducibility
    pub symbol_contexts: Option<Vec<SymbolContext>>,

    /// Optional: deterministic timestamp for reproducibility (testing only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deterministic_timestamp: Option<String>,
}

/// Information about a source file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceFileInfo {
    pub file_path: String,
    pub language: String,
    pub chunks: Vec<ChunkInfo>,
}

/// Information about a chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInfo {
    pub chunk_id: String,
    pub byte_range: (usize, usize),
    pub symbol_ids: Vec<String>,
}

/// Node information from assembly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub symbol_id: String,
    pub name: String,
    pub kind: String,
    pub language: String,
    pub file_path: String,
    pub byte_range: (usize, usize),
    pub signature: Option<String>,
    pub calls: Vec<String>,
    pub called_by: Vec<String>,
    pub imports: Vec<String>,
    pub centrality: f64,
    pub documentation_summary: String,
    pub documentation_details: Option<String>,
}

/// Edge information from assembly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeInfo {
    pub from: String,
    pub to: String,
    pub kind: String,
}

/// Embedding data (optional)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    pub dimensions: usize,
    pub vectors: HashMap<String, Vec<f32>>, // chunk_id -> vector
}

/// Response from packing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackResponse {
    pub docpack_id: String,
    pub docpack_bytes: Vec<u8>,
    pub statistics: PackStatistics,
}

/// Statistics about the packed docpack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackStatistics {
    pub file_count: usize,
    pub symbol_count: usize,
    pub cluster_count: usize,
    pub embedding_dimensions: Option<usize>,
    pub has_embeddings: bool,
    pub has_symbol_contexts: bool,
}

impl Packer {
    /// Create a new packer
    pub fn new() -> Self {
        Self {}
    }

    /// Pack all artifacts into a .docpack file
    pub fn pack(&self, request: PackRequest) -> Result<PackResponse, String> {
        // M4.2.1: Collect artifacts
        let clusters = self.build_clusters(&request)?;
        let source_map = self.build_source_map(&request)?;
        let graph = self.build_graph(&request)?;
        let nodes = self.build_nodes(&request)?;

        // Statistics
        let file_count = request.source_files.len();
        let symbol_count = request.nodes.len();
        let cluster_count = request.cluster_labels.len();
        let embedding_dimensions = request.embeddings.as_ref().map(|e| e.dimensions);
        let has_embeddings = request.embeddings.is_some();
        let has_symbol_contexts = request.symbol_contexts.is_some();

        // M4.2.2: Build manifest (use deterministic timestamp if provided for testing)
        let manifest = if let Some(timestamp) = request.deterministic_timestamp.clone() {
            Manifest::new_deterministic(
                request.repo_url.clone(),
                request.git_ref.clone(),
                request.commit_hash.clone(),
                file_count,
                symbol_count,
                cluster_count,
                timestamp,
            )
        } else {
            Manifest::new(
                request.repo_url.clone(),
                request.git_ref.clone(),
                request.commit_hash.clone(),
                file_count,
                symbol_count,
                cluster_count,
            )
        };

        // M4.2.3: Write docpack (reproducible)
        let writer = DocpackWriter::new();
        let docpack_bytes = writer
            .write(manifest.clone(), &graph, &nodes, &clusters, &source_map)
            .map_err(|e| format!("Failed to write docpack: {}", e))?;

        // Compute docpack_id (content-addressed)
        let docpack_id = self.compute_docpack_id(&docpack_bytes);

        Ok(PackResponse {
            docpack_id,
            docpack_bytes,
            statistics: PackStatistics {
                file_count,
                symbol_count,
                cluster_count,
                embedding_dimensions,
                has_embeddings,
                has_symbol_contexts,
            },
        })
    }

    /// Build clusters from request
    fn build_clusters(&self, request: &PackRequest) -> Result<Clusters, String> {
        // Count members per cluster
        let mut member_counts: HashMap<String, usize> = HashMap::new();
        for cluster_id in request.cluster_assignments.values() {
            *member_counts.entry(cluster_id.clone()).or_insert(0) += 1;
        }

        // Build cluster objects
        let mut clusters_vec = Vec::new();
        for (cluster_id, label) in &request.cluster_labels {
            let member_count = member_counts.get(cluster_id).copied().unwrap_or(0);
            clusters_vec.push(Cluster::new(
                cluster_id.clone(),
                label.clone(),
                member_count,
            ));
        }

        // Sort for reproducibility
        clusters_vec.sort_by(|a, b| a.cluster_id.cmp(&b.cluster_id));

        Ok(Clusters::new(clusters_vec))
    }

    /// Build source map from request
    fn build_source_map(&self, request: &PackRequest) -> Result<SourceMap, String> {
        let mut files = Vec::new();

        for source_file in &request.source_files {
            let chunks: Vec<SourceMapChunk> = source_file
                .chunks
                .iter()
                .map(|c| {
                    SourceMapChunk::new(c.chunk_id.clone(), c.byte_range, c.symbol_ids.clone())
                })
                .collect();

            files.push(SourceMapFile::new(
                source_file.file_path.clone(),
                source_file.language.clone(),
                chunks,
            ));
        }

        // Sort for reproducibility
        files.sort_by(|a, b| a.file_path.cmp(&b.file_path));

        Ok(SourceMap::new(files))
    }

    /// Build graph from request
    fn build_graph(&self, request: &PackRequest) -> Result<Graph, String> {
        // Collect all node IDs
        let node_ids: Vec<String> = request.nodes.iter().map(|n| n.symbol_id.clone()).collect();

        // Build edges
        let edges: Vec<Edge> = request
            .edges
            .iter()
            .map(|e| Edge {
                from: e.from.clone(),
                to: e.to.clone(),
                kind: e.kind.clone(),
            })
            .collect();

        Ok(Graph::new(node_ids, edges))
    }

    /// Build nodes from request
    fn build_nodes(&self, request: &PackRequest) -> Result<Nodes, String> {
        let mut symbols = Vec::new();

        for node in &request.nodes {
            // Get cluster_id for this symbol
            let cluster_id = request
                .cluster_assignments
                .get(&node.symbol_id)
                .cloned()
                .unwrap_or_else(|| "unclustered".to_string());

            let mut symbol = Symbol::new(
                node.symbol_id.clone(),
                node.name.clone(),
                node.kind.clone(),
                node.language.clone(),
                node.file_path.clone(),
                node.byte_range,
                cluster_id,
                node.documentation_summary.clone(),
            );

            if let Some(sig) = &node.signature {
                symbol = symbol.with_signature(sig.clone());
            }

            symbol = symbol
                .with_calls(node.calls.clone())
                .with_called_by(node.called_by.clone())
                .with_imports(node.imports.clone())
                .with_centrality(node.centrality);

            symbols.push(symbol);
        }

        // Sort for reproducibility
        symbols.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(Nodes::new(symbols))
    }

    /// Compute content-addressed docpack_id
    fn compute_docpack_id(&self, bytes: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let result = hasher.finalize();
        format!("sha256:{}", ::hex::encode(result))
    }
}

impl Default for Packer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// M4.2.1: Test artifact collection
    #[test]
    fn test_artifact_collection() {
        let packer = Packer::new();

        // Minimal request with clusters and source map
        let request = PackRequest {
            repo_url: "https://github.com/test/repo".to_string(),
            git_ref: "main".to_string(),
            commit_hash: Some("abc123".to_string()),
            source_files: vec![SourceFileInfo {
                file_path: "src/main.rs".to_string(),
                language: "rust".to_string(),
                chunks: vec![ChunkInfo {
                    chunk_id: "chunk_1".to_string(),
                    byte_range: (0, 100),
                    symbol_ids: vec!["sym_1".to_string()],
                }],
            }],
            cluster_assignments: {
                let mut map = HashMap::new();
                map.insert("sym_1".to_string(), "cluster_1".to_string());
                map
            },
            cluster_labels: {
                let mut map = HashMap::new();
                map.insert("cluster_1".to_string(), "main".to_string());
                map
            },
            nodes: vec![NodeInfo {
                symbol_id: "sym_1".to_string(),
                name: "main".to_string(),
                kind: "function".to_string(),
                language: "rust".to_string(),
                file_path: "src/main.rs".to_string(),
                byte_range: (0, 100),
                signature: Some("fn main()".to_string()),
                calls: vec![],
                called_by: vec![],
                imports: vec![],
                centrality: 0.8,
                documentation_summary: "Main entry point".to_string(),
                documentation_details: None,
            }],
            edges: vec![],
            embeddings: None,
            symbol_contexts: None,
            deterministic_timestamp: None,
        };

        // Build clusters
        let clusters = packer.build_clusters(&request).unwrap();
        assert_eq!(clusters.clusters.len(), 1);
        assert_eq!(clusters.clusters[0].cluster_id, "cluster_1");

        // Build source map
        let source_map = packer.build_source_map(&request).unwrap();
        assert_eq!(source_map.files.len(), 1);

        // Build nodes
        let nodes = packer.build_nodes(&request).unwrap();
        assert_eq!(nodes.symbols.len(), 1);
        assert_eq!(nodes.symbols[0].id, "sym_1");

        // Build graph
        let graph = packer.build_graph(&request).unwrap();
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.edges.len(), 0);
    }

    /// M4.2.2: Test full docpack assembly
    #[test]
    fn test_full_assembly() {
        let packer = Packer::new();

        let request = PackRequest {
            repo_url: "https://github.com/test/repo".to_string(),
            git_ref: "main".to_string(),
            commit_hash: Some("deadbeef".to_string()),
            source_files: vec![SourceFileInfo {
                file_path: "src/lib.rs".to_string(),
                language: "rust".to_string(),
                chunks: vec![ChunkInfo {
                    chunk_id: "chunk_abc".to_string(),
                    byte_range: (0, 200),
                    symbol_ids: vec!["sym_helper".to_string()],
                }],
            }],
            cluster_assignments: {
                let mut map = HashMap::new();
                map.insert("sym_helper".to_string(), "cluster_utils".to_string());
                map
            },
            cluster_labels: {
                let mut map = HashMap::new();
                map.insert("cluster_utils".to_string(), "utilities".to_string());
                map
            },
            nodes: vec![NodeInfo {
                symbol_id: "sym_helper".to_string(),
                name: "helper".to_string(),
                kind: "function".to_string(),
                language: "rust".to_string(),
                file_path: "src/lib.rs".to_string(),
                byte_range: (0, 200),
                signature: Some("fn helper() -> i32".to_string()),
                calls: vec![],
                called_by: vec![],
                imports: vec!["std::fmt".to_string()],
                centrality: 0.5,
                documentation_summary: "A helper function for testing".to_string(),
                documentation_details: Some("Returns a fixed value".to_string()),
            }],
            edges: vec![],
            embeddings: Some(EmbeddingData {
                dimensions: 384,
                vectors: {
                    let mut map = HashMap::new();
                    map.insert("chunk_abc".to_string(), vec![0.1; 384]);
                    map
                },
            }),
            symbol_contexts: None,
            deterministic_timestamp: None,
        };

        // Pack should succeed
        let response = packer.pack(request).unwrap();

        // Check statistics
        assert_eq!(response.statistics.file_count, 1);
        assert_eq!(response.statistics.symbol_count, 1);
        assert_eq!(response.statistics.cluster_count, 1);
        assert_eq!(response.statistics.embedding_dimensions, Some(384));
        assert!(response.statistics.has_embeddings);
        assert!(!response.statistics.has_symbol_contexts);

        // Check docpack_id format
        assert!(response.docpack_id.starts_with("sha256:"));

        // Check bytes are not empty
        assert!(!response.docpack_bytes.is_empty());
    }

    /// M4.2.3: Test reproducibility
    #[test]
    fn test_reproducibility() {
        let packer = Packer::new();

        let request = PackRequest {
            repo_url: "https://github.com/test/repo".to_string(),
            git_ref: "main".to_string(),
            commit_hash: Some("deadbeef".to_string()),
            source_files: vec![SourceFileInfo {
                file_path: "src/test.rs".to_string(),
                language: "rust".to_string(),
                chunks: vec![ChunkInfo {
                    chunk_id: "chunk_1".to_string(),
                    byte_range: (0, 50),
                    symbol_ids: vec!["sym_test".to_string()],
                }],
            }],
            cluster_assignments: {
                let mut map = HashMap::new();
                map.insert("sym_test".to_string(), "cluster_test".to_string());
                map
            },
            cluster_labels: {
                let mut map = HashMap::new();
                map.insert("cluster_test".to_string(), "testing".to_string());
                map
            },
            nodes: vec![NodeInfo {
                symbol_id: "sym_test".to_string(),
                name: "test_fn".to_string(),
                kind: "function".to_string(),
                language: "rust".to_string(),
                file_path: "src/test.rs".to_string(),
                byte_range: (0, 50),
                signature: None,
                calls: vec![],
                called_by: vec![],
                imports: vec![],
                centrality: 0.3,
                documentation_summary: "Test function".to_string(),
                documentation_details: None,
            }],
            edges: vec![],
            embeddings: None,
            symbol_contexts: None,
            deterministic_timestamp: Some("2025-01-01T00:00:00Z".to_string()),
        };

        // Pack twice
        let response1 = packer.pack(request.clone()).unwrap();
        let response2 = packer.pack(request).unwrap();

        // Same inputs should produce same outputs
        assert_eq!(response1.docpack_id, response2.docpack_id);
        assert_eq!(response1.docpack_bytes, response2.docpack_bytes);
    }
}
