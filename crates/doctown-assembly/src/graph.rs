//! Graph construction and metrics for code understanding.

use std::collections::HashMap;

/// A node in the code graph representing a symbol.
#[derive(Debug, Clone)]
pub struct Node {
    /// Unique identifier for the node.
    pub id: String,
    /// Metadata about the symbol.
    pub metadata: HashMap<String, String>,
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
}
