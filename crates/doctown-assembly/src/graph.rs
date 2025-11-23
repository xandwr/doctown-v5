//! Graph construction and metrics for code understanding.

use std::collections::HashMap;
use doctown_common::types::{Call, Import};

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
        self.edges.iter()
            .filter(|e| e.target == node_id)
            .count()
    }

    /// Compute out-degree for a node.
    pub fn out_degree(&self, node_id: &str) -> usize {
        self.edges.iter()
            .filter(|e| e.source == node_id)
            .count()
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
                && self.symbol_index.contains_key(target_id) {
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
                        && self.symbol_index.contains_key(item) {
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

    /// Consume the builder and return the constructed graph.
    pub fn build(self) -> Graph {
        self.graph
    }

    /// Get a reference to the current graph state.
    pub fn graph(&self) -> &Graph {
        &self.graph
    }
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
        use doctown_common::types::{Call, CallKind, ByteRange};
        
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
        let calls = vec![
            (
                "fn_main".to_string(),
                Call {
                    name: "fn_helper".to_string(),
                    range: ByteRange::new(100, 110),
                    kind: CallKind::Function,
                    is_resolved: true,
                },
            ),
        ];
        builder.build_calls_edges(&calls);
        
        assert_eq!(builder.graph().edges.len(), 1);
        
        let edge = &builder.graph().edges[0];
        assert_eq!(edge.source, "fn_main");
        assert_eq!(edge.target, "fn_helper");
        assert_eq!(edge.kind, EdgeKind::Calls);
    }

    #[test]
    fn test_graph_builder_calls_edges_unresolved_skipped() {
        use doctown_common::types::{Call, CallKind, ByteRange};
        
        let mut builder = GraphBuilder::new();
        
        let symbols = vec![
            SymbolData {
                symbol_id: "fn_main".to_string(),
                name: "main".to_string(),
                kind: "function".to_string(),
                file_path: "src/main.rs".to_string(),
                signature: None,
            },
        ];
        builder.build_nodes(&symbols);
        
        // Add unresolved call (should be skipped)
        let calls = vec![
            (
                "fn_main".to_string(),
                Call {
                    name: "external_function".to_string(),
                    range: ByteRange::new(100, 110),
                    kind: CallKind::Function,
                    is_resolved: false,
                },
            ),
        ];
        builder.build_calls_edges(&calls);
        
        // No edges should be created for unresolved calls
        assert_eq!(builder.graph().edges.len(), 0);
    }

    #[test]
    fn test_graph_builder_imports_edges() {
        use doctown_common::types::{Import, ByteRange};
        
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
        let imports = vec![
            (
                "mod_main".to_string(),
                Import {
                    module_path: "std::collections".to_string(),
                    imported_items: Some(vec!["HashMap".to_string()]),
                    alias: None,
                    range: ByteRange::new(0, 30),
                    is_wildcard: false,
                },
            ),
        ];
        builder.build_imports_edges(&imports);
        
        assert_eq!(builder.graph().edges.len(), 1);
        
        let edge = &builder.graph().edges[0];
        assert_eq!(edge.source, "mod_main");
        assert_eq!(edge.target, "HashMap");
        assert_eq!(edge.kind, EdgeKind::Imports);
    }

    #[test]
    fn test_graph_builder_edge_types_correct() {
        use doctown_common::types::{Call, CallKind, Import, ByteRange};
        
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
        let calls = vec![
            (
                "fn_a".to_string(),
                Call {
                    name: "fn_b".to_string(),
                    range: ByteRange::new(10, 20),
                    kind: CallKind::Function,
                    is_resolved: true,
                },
            ),
        ];
        builder.build_calls_edges(&calls);
        
        // Add an import edge
        let imports = vec![
            (
                "fn_a".to_string(),
                Import {
                    module_path: "b".to_string(),
                    imported_items: Some(vec!["fn_b".to_string()]),
                    alias: None,
                    range: ByteRange::new(0, 10),
                    is_wildcard: false,
                },
            ),
        ];
        builder.build_imports_edges(&imports);
        
        assert_eq!(builder.graph().edges.len(), 2);
        
        // Verify edge types
        let calls_edges: Vec<_> = builder.graph().edges.iter()
            .filter(|e| e.kind == EdgeKind::Calls)
            .collect();
        let imports_edges: Vec<_> = builder.graph().edges.iter()
            .filter(|e| e.kind == EdgeKind::Imports)
            .collect();
        
        assert_eq!(calls_edges.len(), 1);
        assert_eq!(imports_edges.len(), 1);
    }
}
