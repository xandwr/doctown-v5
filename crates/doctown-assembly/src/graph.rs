//! Graph construction and metrics for code understanding.

use doctown_common::types::{Call, Import};
use std::collections::HashMap;

/// A node in the code graph representing a symbol.
#[derive(Debug, Clone)]
pub struct Node {
    /// Unique identifier for the node (symbol_id).
    pub id: String,
    /// Metadata about the symbol (e.g., name, kind, file_path, signature).
    pub metadata: HashMap<String, String>,
}

impl Node {
    /// Create a new node with the given ID and metadata.
    pub fn new(id: String, metadata: HashMap<String, String>) -> Self {
        Self { id, metadata }
    }
}

/// Kind of edge in the graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeKind {
    /// Function/method call relationship.
    Calls,
    /// Import relationship.
    Imports,
    /// Semantic similarity relationship.
    Related,
}

/// An edge between two nodes in the graph.
#[derive(Debug, Clone)]
pub struct Edge {
    /// Source node ID.
    pub source: String,
    /// Target node ID.
    pub target: String,
    /// Kind of relationship.
    pub kind: EdgeKind,
    /// Optional weight/score.
    pub weight: Option<f32>,
}

/// Graph structure for code understanding.
#[derive(Debug, Default)]
pub struct Graph {
    /// All nodes in the graph.
    pub nodes: Vec<Node>,
    /// All edges in the graph.
    pub edges: Vec<Edge>,
}

impl Graph {
    /// Create a new empty graph.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    /// Add an edge to the graph.
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    /// Get a node by its ID.
    pub fn get_node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Compute in-degree for a node.
    pub fn in_degree(&self, node_id: &str) -> usize {
        self.edges.iter().filter(|e| e.target == node_id).count()
    }

    /// Compute out-degree for a node.
    pub fn out_degree(&self, node_id: &str) -> usize {
        self.edges.iter().filter(|e| e.source == node_id).count()
    }

    /// Compute graph density.
    pub fn density(&self) -> f64 {
        let n = self.nodes.len() as f64;
        let e = self.edges.len() as f64;
        if n <= 1.0 {
            return 0.0;
        }
        e / (n * (n - 1.0))
    }

    /// Compute degree centrality for a node.
    ///
    /// Degree centrality is the fraction of nodes a node is connected to.
    /// Returns a value between 0.0 and 1.0.
    pub fn degree_centrality(&self, node_id: &str) -> f64 {
        let n = self.nodes.len();
        if n <= 1 {
            return 0.0;
        }

        let degree = self.in_degree(node_id) + self.out_degree(node_id);
        degree as f64 / (n - 1) as f64
    }

    /// Compute degree centrality for all nodes.
    ///
    /// Returns a map from node_id to centrality score (0.0-1.0).
    pub fn all_degree_centralities(&self) -> HashMap<String, f64> {
        let mut centralities = HashMap::new();
        for node in &self.nodes {
            centralities.insert(node.id.clone(), self.degree_centrality(&node.id));
        }
        centralities
    }

    /// Get the total degree (in + out) for a node.
    pub fn degree(&self, node_id: &str) -> usize {
        self.in_degree(node_id) + self.out_degree(node_id)
    }
}

/// Input data for building a graph node.
#[derive(Debug, Clone)]
pub struct SymbolData {
    /// Unique identifier for the symbol.
    pub symbol_id: String,
    /// Name of the symbol.
    pub name: String,
    /// Kind of symbol (function, class, etc.).
    pub kind: String,
    /// File path where the symbol is defined.
    pub file_path: String,
    /// Full signature (for functions/methods).
    pub signature: Option<String>,
}

/// Builder for constructing graphs from symbols, calls, and imports.
#[derive(Debug, Default)]
pub struct GraphBuilder {
    graph: Graph,
    /// Map symbol_id -> node index for quick lookup.
    symbol_index: HashMap<String, usize>,
}

impl GraphBuilder {
    /// Create a new graph builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Build nodes from symbol data.
    ///
    /// Creates a node for each symbol with metadata about the symbol.
    pub fn build_nodes(&mut self, symbols: &[SymbolData]) {
        for symbol in symbols {
            let mut metadata = HashMap::new();
            metadata.insert("name".to_string(), symbol.name.clone());
            metadata.insert("kind".to_string(), symbol.kind.clone());
            metadata.insert("file_path".to_string(), symbol.file_path.clone());

            if let Some(sig) = &symbol.signature {
                metadata.insert("signature".to_string(), sig.clone());
            }

            let node = Node::new(symbol.symbol_id.clone(), metadata);
            let node_idx = self.graph.nodes.len();
            self.graph.add_node(node);
            self.symbol_index.insert(symbol.symbol_id.clone(), node_idx);
        }
    }

    /// Build "calls" edges from call graph data.
    ///
    /// Creates edges between symbols that call each other. Only creates edges
    /// when both the caller and callee symbols exist in the graph.
    ///
    /// # Arguments
    /// * `calls` - List of calls with (caller_symbol_id, Call) tuples
    pub fn build_calls_edges(&mut self, calls: &[(String, Call)]) {
        for (caller_id, call) in calls {
            // Only create edges for resolved calls
            if !call.is_resolved {
                continue;
            }

            // For now, we assume the call name is the target symbol ID
            // In a real implementation, this would use proper symbol resolution
            let target_id = &call.name;

            // Check if both caller and target exist in the graph
            if self.symbol_index.contains_key(caller_id)
                && self.symbol_index.contains_key(target_id)
            {
                let edge = Edge {
                    source: caller_id.clone(),
                    target: target_id.clone(),
                    kind: EdgeKind::Calls,
                    weight: None,
                };
                self.graph.add_edge(edge);
            }
        }
    }

    /// Build "imports" edges from import data.
    ///
    /// Creates edges between files/modules that import from each other.
    ///
    /// # Arguments
    /// * `imports` - List of imports with (importer_symbol_id, Import) tuples
    pub fn build_imports_edges(&mut self, imports: &[(String, Import)]) {
        for (importer_id, import) in imports {
            // For imports with specific items, create an edge for each item
            if let Some(items) = &import.imported_items {
                for item in items {
                    // Try to find the imported symbol in the graph
                    // In a real implementation, this would use proper module resolution
                    if self.symbol_index.contains_key(importer_id)
                        && self.symbol_index.contains_key(item)
                    {
                        let edge = Edge {
                            source: importer_id.clone(),
                            target: item.clone(),
                            kind: EdgeKind::Imports,
                            weight: None,
                        };
                        self.graph.add_edge(edge);
                    }
                }
            }
            // For wildcard imports or module-level imports, we could create
            // edges to a module-level node, but we skip that for now
        }
    }

    /// Build "related" edges based on semantic similarity.
    ///
    /// Computes pairwise cosine similarity between all nodes and creates
    /// "related" edges for pairs with similarity > threshold. Limits to
    /// top-k most similar nodes per node.
    ///
    /// # Arguments
    /// * `embeddings` - Map from symbol_id to embedding vector
    /// * `threshold` - Minimum similarity score (0.0-1.0) to create an edge
    /// * `top_k` - Maximum number of related edges per node
    pub fn build_similarity_edges(
        &mut self,
        embeddings: &HashMap<String, Vec<f32>>,
        threshold: f32,
        top_k: usize,
    ) {
        use ndarray::Array1;

        // Build list of (symbol_id, embedding) pairs for nodes that have embeddings
        let mut node_embeddings: Vec<(String, Array1<f32>)> = Vec::new();
        for node in &self.graph.nodes {
            if let Some(embedding) = embeddings.get(&node.id) {
                node_embeddings.push((node.id.clone(), Array1::from_vec(embedding.clone())));
            }
        }

        if node_embeddings.len() < 2 {
            return; // Need at least 2 nodes to compare
        }

        // For each node, compute similarities to all other nodes
        let mut similarities: HashMap<String, Vec<(String, f32)>> = HashMap::new();

        for i in 0..node_embeddings.len() {
            let (id_i, emb_i) = &node_embeddings[i];
            let mut sims: Vec<(String, f32)> = Vec::new();

            for j in 0..node_embeddings.len() {
                if i == j {
                    continue; // Skip self-similarity
                }

                let (id_j, emb_j) = &node_embeddings[j];
                let similarity = cosine_similarity(emb_i, emb_j);

                if similarity > threshold {
                    sims.push((id_j.clone(), similarity));
                }
            }

            // Sort by similarity (descending) and keep top-k
            sims.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            sims.truncate(top_k);

            similarities.insert(id_i.clone(), sims);
        }

        // Create edges from the similarity map
        for (source_id, sims) in similarities {
            for (target_id, similarity) in sims {
                let edge = Edge {
                    source: source_id.clone(),
                    target: target_id,
                    kind: EdgeKind::Related,
                    weight: Some(similarity),
                };
                self.graph.add_edge(edge);
            }
        }
    }

    /// Consume the builder and return the constructed graph.
    pub fn build(self) -> Graph {
        self.graph
    }

    /// Get a reference to the current graph state.
    pub fn graph(&self) -> &Graph {
        &self.graph
    }
}

/// Compute cosine similarity between two vectors.
///
/// Returns a value between -1.0 and 1.0, where:
/// - 1.0 = identical direction
/// - 0.0 = orthogonal
/// - -1.0 = opposite direction
fn cosine_similarity(a: &ndarray::Array1<f32>, b: &ndarray::Array1<f32>) -> f32 {
    let dot_product = a.dot(b);
    let norm_a = a.dot(a).sqrt();
    let norm_b = b.dot(b).sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_creation() {
        let graph = Graph::new();
        assert_eq!(graph.nodes.len(), 0);
        assert_eq!(graph.edges.len(), 0);
    }

    #[test]
    fn test_add_node() {
        let mut graph = Graph::new();
        graph.add_node(Node {
            id: "node1".to_string(),
            metadata: HashMap::new(),
        });
        assert_eq!(graph.nodes.len(), 1);
    }

    #[test]
    fn test_degree() {
        let mut graph = Graph::new();
        graph.add_edge(Edge {
            source: "a".to_string(),
            target: "b".to_string(),
            kind: EdgeKind::Calls,
            weight: None,
        });
        assert_eq!(graph.out_degree("a"), 1);
        assert_eq!(graph.in_degree("b"), 1);
        assert_eq!(graph.in_degree("a"), 0);
    }

    #[test]
    fn test_graph_builder_nodes() {
        let mut builder = GraphBuilder::new();

        let symbols = vec![
            SymbolData {
                symbol_id: "fn_foo".to_string(),
                name: "foo".to_string(),
                kind: "function".to_string(),
                file_path: "src/main.rs".to_string(),
                signature: Some("fn foo() -> i32".to_string()),
            },
            SymbolData {
                symbol_id: "fn_bar".to_string(),
                name: "bar".to_string(),
                kind: "function".to_string(),
                file_path: "src/lib.rs".to_string(),
                signature: Some("fn bar(x: i32)".to_string()),
            },
        ];

        builder.build_nodes(&symbols);

        assert_eq!(builder.graph().nodes.len(), 2);

        let node1 = builder.graph().get_node("fn_foo").unwrap();
        assert_eq!(node1.metadata.get("name").unwrap(), "foo");
        assert_eq!(node1.metadata.get("kind").unwrap(), "function");
        assert_eq!(node1.metadata.get("file_path").unwrap(), "src/main.rs");
        assert_eq!(node1.metadata.get("signature").unwrap(), "fn foo() -> i32");

        let node2 = builder.graph().get_node("fn_bar").unwrap();
        assert_eq!(node2.metadata.get("name").unwrap(), "bar");
    }

    #[test]
    fn test_graph_builder_calls_edges() {
        use doctown_common::types::{ByteRange, Call, CallKind};

        let mut builder = GraphBuilder::new();

        // First add nodes
        let symbols = vec![
            SymbolData {
                symbol_id: "fn_main".to_string(),
                name: "main".to_string(),
                kind: "function".to_string(),
                file_path: "src/main.rs".to_string(),
                signature: None,
            },
            SymbolData {
                symbol_id: "fn_helper".to_string(),
                name: "helper".to_string(),
                kind: "function".to_string(),
                file_path: "src/lib.rs".to_string(),
                signature: None,
            },
        ];
        builder.build_nodes(&symbols);

        // Add calls edges
        let calls = vec![(
            "fn_main".to_string(),
            Call {
                name: "fn_helper".to_string(),
                range: ByteRange::new(100, 110),
                kind: CallKind::Function,
                is_resolved: true,
            },
        )];
        builder.build_calls_edges(&calls);

        assert_eq!(builder.graph().edges.len(), 1);

        let edge = &builder.graph().edges[0];
        assert_eq!(edge.source, "fn_main");
        assert_eq!(edge.target, "fn_helper");
        assert_eq!(edge.kind, EdgeKind::Calls);
    }

    #[test]
    fn test_graph_builder_calls_edges_unresolved_skipped() {
        use doctown_common::types::{ByteRange, Call, CallKind};

        let mut builder = GraphBuilder::new();

        let symbols = vec![SymbolData {
            symbol_id: "fn_main".to_string(),
            name: "main".to_string(),
            kind: "function".to_string(),
            file_path: "src/main.rs".to_string(),
            signature: None,
        }];
        builder.build_nodes(&symbols);

        // Add unresolved call (should be skipped)
        let calls = vec![(
            "fn_main".to_string(),
            Call {
                name: "external_function".to_string(),
                range: ByteRange::new(100, 110),
                kind: CallKind::Function,
                is_resolved: false,
            },
        )];
        builder.build_calls_edges(&calls);

        // No edges should be created for unresolved calls
        assert_eq!(builder.graph().edges.len(), 0);
    }

    #[test]
    fn test_graph_builder_imports_edges() {
        use doctown_common::types::{ByteRange, Import};

        let mut builder = GraphBuilder::new();

        // Add nodes
        let symbols = vec![
            SymbolData {
                symbol_id: "mod_main".to_string(),
                name: "main".to_string(),
                kind: "module".to_string(),
                file_path: "src/main.rs".to_string(),
                signature: None,
            },
            SymbolData {
                symbol_id: "HashMap".to_string(),
                name: "HashMap".to_string(),
                kind: "struct".to_string(),
                file_path: "std/collections/mod.rs".to_string(),
                signature: None,
            },
        ];
        builder.build_nodes(&symbols);

        // Add imports edges
        let imports = vec![(
            "mod_main".to_string(),
            Import {
                module_path: "std::collections".to_string(),
                imported_items: Some(vec!["HashMap".to_string()]),
                alias: None,
                range: ByteRange::new(0, 30),
                is_wildcard: false,
            },
        )];
        builder.build_imports_edges(&imports);

        assert_eq!(builder.graph().edges.len(), 1);

        let edge = &builder.graph().edges[0];
        assert_eq!(edge.source, "mod_main");
        assert_eq!(edge.target, "HashMap");
        assert_eq!(edge.kind, EdgeKind::Imports);
    }

    #[test]
    fn test_graph_builder_edge_types_correct() {
        use doctown_common::types::{ByteRange, Call, CallKind, Import};

        let mut builder = GraphBuilder::new();

        // Add nodes
        let symbols = vec![
            SymbolData {
                symbol_id: "fn_a".to_string(),
                name: "a".to_string(),
                kind: "function".to_string(),
                file_path: "src/a.rs".to_string(),
                signature: None,
            },
            SymbolData {
                symbol_id: "fn_b".to_string(),
                name: "b".to_string(),
                kind: "function".to_string(),
                file_path: "src/b.rs".to_string(),
                signature: None,
            },
        ];
        builder.build_nodes(&symbols);

        // Add a call edge
        let calls = vec![(
            "fn_a".to_string(),
            Call {
                name: "fn_b".to_string(),
                range: ByteRange::new(10, 20),
                kind: CallKind::Function,
                is_resolved: true,
            },
        )];
        builder.build_calls_edges(&calls);

        // Add an import edge
        let imports = vec![(
            "fn_a".to_string(),
            Import {
                module_path: "b".to_string(),
                imported_items: Some(vec!["fn_b".to_string()]),
                alias: None,
                range: ByteRange::new(0, 10),
                is_wildcard: false,
            },
        )];
        builder.build_imports_edges(&imports);

        assert_eq!(builder.graph().edges.len(), 2);

        // Verify edge types
        let calls_edges: Vec<_> = builder
            .graph()
            .edges
            .iter()
            .filter(|e| e.kind == EdgeKind::Calls)
            .collect();
        let imports_edges: Vec<_> = builder
            .graph()
            .edges
            .iter()
            .filter(|e| e.kind == EdgeKind::Imports)
            .collect();

        assert_eq!(calls_edges.len(), 1);
        assert_eq!(imports_edges.len(), 1);
    }

    #[test]
    fn test_cosine_similarity() {
        use ndarray::Array1;

        // Test identical vectors
        let v1 = Array1::from_vec(vec![1.0, 0.0, 0.0]);
        let v2 = Array1::from_vec(vec![1.0, 0.0, 0.0]);
        assert!((cosine_similarity(&v1, &v2) - 1.0).abs() < 0.0001);

        // Test orthogonal vectors
        let v3 = Array1::from_vec(vec![1.0, 0.0, 0.0]);
        let v4 = Array1::from_vec(vec![0.0, 1.0, 0.0]);
        assert!(cosine_similarity(&v3, &v4).abs() < 0.0001);

        // Test opposite vectors
        let v5 = Array1::from_vec(vec![1.0, 0.0, 0.0]);
        let v6 = Array1::from_vec(vec![-1.0, 0.0, 0.0]);
        assert!((cosine_similarity(&v5, &v6) + 1.0).abs() < 0.0001);

        // Test similar but not identical vectors
        let v7 = Array1::from_vec(vec![1.0, 2.0, 3.0]);
        let v8 = Array1::from_vec(vec![1.5, 2.0, 3.5]);
        let sim = cosine_similarity(&v7, &v8);
        assert!(sim > 0.99); // Should be very similar
    }

    #[test]
    fn test_similarity_edges() {
        let mut builder = GraphBuilder::new();

        // Create 4 nodes
        let symbols = vec![
            SymbolData {
                symbol_id: "node_a".to_string(),
                name: "a".to_string(),
                kind: "function".to_string(),
                file_path: "a.rs".to_string(),
                signature: None,
            },
            SymbolData {
                symbol_id: "node_b".to_string(),
                name: "b".to_string(),
                kind: "function".to_string(),
                file_path: "b.rs".to_string(),
                signature: None,
            },
            SymbolData {
                symbol_id: "node_c".to_string(),
                name: "c".to_string(),
                kind: "function".to_string(),
                file_path: "c.rs".to_string(),
                signature: None,
            },
            SymbolData {
                symbol_id: "node_d".to_string(),
                name: "d".to_string(),
                kind: "function".to_string(),
                file_path: "d.rs".to_string(),
                signature: None,
            },
        ];
        builder.build_nodes(&symbols);

        // Create embeddings where:
        // - a and b are very similar
        // - c is somewhat similar to a and b
        // - d is dissimilar
        let mut embeddings = HashMap::new();
        embeddings.insert("node_a".to_string(), vec![1.0, 0.0, 0.0]);
        embeddings.insert("node_b".to_string(), vec![0.95, 0.1, 0.0]);
        embeddings.insert("node_c".to_string(), vec![0.7, 0.5, 0.0]);
        embeddings.insert("node_d".to_string(), vec![0.0, 0.0, 1.0]);

        // Build similarity edges with threshold 0.7 and top-k 2
        builder.build_similarity_edges(&embeddings, 0.7, 2);

        // Count related edges
        let related_edges: Vec<_> = builder
            .graph()
            .edges
            .iter()
            .filter(|e| e.kind == EdgeKind::Related)
            .collect();

        // Should have created some related edges
        assert!(related_edges.len() > 0);

        // Verify that edges have weights
        for edge in &related_edges {
            assert!(edge.weight.is_some());
            let weight = edge.weight.unwrap();
            assert!(weight > 0.7); // Above threshold
            assert!(weight <= 1.0); // Similarity is at most 1.0
        }

        // Verify no node has more than top_k=2 outgoing related edges
        for node_id in &["node_a", "node_b", "node_c", "node_d"] {
            let outgoing_related = related_edges
                .iter()
                .filter(|e| e.source == *node_id)
                .count();
            assert!(outgoing_related <= 2);
        }
    }

    #[test]
    fn test_similarity_edges_top_k_limit() {
        let mut builder = GraphBuilder::new();

        // Create 5 nodes
        for i in 0..5 {
            builder.build_nodes(&[SymbolData {
                symbol_id: format!("node_{}", i),
                name: format!("node_{}", i),
                kind: "function".to_string(),
                file_path: "test.rs".to_string(),
                signature: None,
            }]);
        }

        // Create embeddings where all nodes are somewhat similar
        let mut embeddings = HashMap::new();
        for i in 0..5 {
            // All vectors point roughly in same direction with slight variations
            embeddings.insert(format!("node_{}", i), vec![1.0, i as f32 * 0.1, 0.0]);
        }

        // Build similarity edges with top_k = 2
        builder.build_similarity_edges(&embeddings, 0.5, 2);

        // Each node should have at most 2 outgoing related edges
        let related_edges: Vec<_> = builder
            .graph()
            .edges
            .iter()
            .filter(|e| e.kind == EdgeKind::Related)
            .collect();

        for i in 0..5 {
            let node_id = format!("node_{}", i);
            let outgoing = related_edges.iter().filter(|e| e.source == node_id).count();
            assert!(
                outgoing <= 2,
                "Node {} has {} outgoing edges, expected <= 2",
                node_id,
                outgoing
            );
        }
    }

    #[test]
    fn test_graph_density() {
        let mut graph = Graph::new();

        // Empty graph has density 0
        assert_eq!(graph.density(), 0.0);

        // Single node has density 0
        graph.add_node(Node::new("a".to_string(), HashMap::new()));
        assert_eq!(graph.density(), 0.0);

        // Add 3 nodes total
        graph.add_node(Node::new("b".to_string(), HashMap::new()));
        graph.add_node(Node::new("c".to_string(), HashMap::new()));

        // With 3 nodes, max edges = 3 * 2 = 6 (directed)
        // Add 2 edges
        graph.add_edge(Edge {
            source: "a".to_string(),
            target: "b".to_string(),
            kind: EdgeKind::Calls,
            weight: None,
        });
        graph.add_edge(Edge {
            source: "b".to_string(),
            target: "c".to_string(),
            kind: EdgeKind::Calls,
            weight: None,
        });

        // Density = 2 / 6 = 0.333...
        assert!((graph.density() - 0.3333).abs() < 0.01);
    }

    #[test]
    fn test_degree_centrality() {
        let mut graph = Graph::new();

        // Add 4 nodes
        for id in &["a", "b", "c", "d"] {
            graph.add_node(Node::new(id.to_string(), HashMap::new()));
        }

        // Create a star topology: a connects to all others
        for target in &["b", "c", "d"] {
            graph.add_edge(Edge {
                source: "a".to_string(),
                target: target.to_string(),
                kind: EdgeKind::Calls,
                weight: None,
            });
        }

        // Node a has degree 3 (connects to all other 3 nodes)
        // Centrality = 3 / 3 = 1.0 (perfectly central)
        assert!((graph.degree_centrality("a") - 1.0).abs() < 0.0001);

        // Nodes b, c, d each have degree 1 (connected only to a)
        // Centrality = 1 / 3 = 0.333...
        for node_id in &["b", "c", "d"] {
            assert!((graph.degree_centrality(node_id) - 0.3333).abs() < 0.01);
        }
    }

    #[test]
    fn test_all_degree_centralities() {
        let mut graph = Graph::new();

        // Add 3 nodes
        for id in &["a", "b", "c"] {
            graph.add_node(Node::new(id.to_string(), HashMap::new()));
        }

        // Add edges
        graph.add_edge(Edge {
            source: "a".to_string(),
            target: "b".to_string(),
            kind: EdgeKind::Calls,
            weight: None,
        });
        graph.add_edge(Edge {
            source: "b".to_string(),
            target: "c".to_string(),
            kind: EdgeKind::Calls,
            weight: None,
        });

        let centralities = graph.all_degree_centralities();

        assert_eq!(centralities.len(), 3);
        assert!(centralities.contains_key("a"));
        assert!(centralities.contains_key("b"));
        assert!(centralities.contains_key("c"));

        // All centrality values should be between 0 and 1
        for (_, centrality) in centralities {
            assert!(centrality >= 0.0);
            assert!(centrality <= 1.0);
        }
    }

    #[test]
    fn test_total_degree() {
        let mut graph = Graph::new();

        graph.add_edge(Edge {
            source: "a".to_string(),
            target: "b".to_string(),
            kind: EdgeKind::Calls,
            weight: None,
        });
        graph.add_edge(Edge {
            source: "c".to_string(),
            target: "b".to_string(),
            kind: EdgeKind::Calls,
            weight: None,
        });

        // Node b has in-degree 2, out-degree 0, total degree 2
        assert_eq!(graph.degree("b"), 2);
        assert_eq!(graph.in_degree("b"), 2);
        assert_eq!(graph.out_degree("b"), 0);

        // Node a has in-degree 0, out-degree 1, total degree 1
        assert_eq!(graph.degree("a"), 1);
        assert_eq!(graph.in_degree("a"), 0);
        assert_eq!(graph.out_degree("a"), 1);
    }
}
