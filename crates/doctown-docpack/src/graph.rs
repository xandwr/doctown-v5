use serde::{Deserialize, Serialize};

/// Global semantic graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Graph {
    pub nodes: Vec<String>,
    pub edges: Vec<Edge>,
    pub metrics: GraphMetrics,
}

/// An edge in the graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub kind: String,
}

/// Global graph metrics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphMetrics {
    pub density: f64,
    pub avg_degree: f64,
}

impl Graph {
    /// Create a new graph
    pub fn new(nodes: Vec<String>, edges: Vec<Edge>) -> Self {
        let metrics = Self::calculate_metrics(&nodes, &edges);
        Self {
            nodes,
            edges,
            metrics,
        }
    }

    /// Create an empty graph
    pub fn empty() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            metrics: GraphMetrics {
                density: 0.0,
                avg_degree: 0.0,
            },
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node_id: String) {
        self.nodes.push(node_id);
        self.update_metrics();
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
        self.update_metrics();
    }

    /// Calculate graph metrics
    fn calculate_metrics(nodes: &[String], edges: &[Edge]) -> GraphMetrics {
        let node_count = nodes.len();
        let edge_count = edges.len();

        let density = if node_count > 1 {
            let max_edges = node_count * (node_count - 1);
            if max_edges > 0 {
                (edge_count as f64) / (max_edges as f64)
            } else {
                0.0
            }
        } else {
            0.0
        };

        let avg_degree = if node_count > 0 {
            (2.0 * edge_count as f64) / (node_count as f64)
        } else {
            0.0
        };

        GraphMetrics {
            density,
            avg_degree,
        }
    }

    /// Update metrics after modifications
    fn update_metrics(&mut self) {
        self.metrics = Self::calculate_metrics(&self.nodes, &self.edges);
    }

    /// Get the number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of edges
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Serialize to JSON bytes
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec_pretty(self)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Deserialize from JSON bytes
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

impl Edge {
    /// Create a new edge
    pub fn new(from: String, to: String, kind: String) -> Self {
        Self { from, to, kind }
    }

    /// Create a "calls" edge
    pub fn calls(from: String, to: String) -> Self {
        Self::new(from, to, "calls".to_string())
    }

    /// Create an "imports" edge
    pub fn imports(from: String, to: String) -> Self {
        Self::new(from, to, "imports".to_string())
    }

    /// Create a "contains" edge
    pub fn contains(from: String, to: String) -> Self {
        Self::new(from, to, "contains".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_creation() {
        let edge = Edge::new(
            "sym_a".to_string(),
            "sym_b".to_string(),
            "calls".to_string(),
        );

        assert_eq!(edge.from, "sym_a");
        assert_eq!(edge.to, "sym_b");
        assert_eq!(edge.kind, "calls");
    }

    #[test]
    fn test_edge_helpers() {
        let calls = Edge::calls("sym_a".to_string(), "sym_b".to_string());
        assert_eq!(calls.kind, "calls");

        let imports = Edge::imports("sym_a".to_string(), "sym_b".to_string());
        assert_eq!(imports.kind, "imports");

        let contains = Edge::contains("sym_a".to_string(), "sym_b".to_string());
        assert_eq!(contains.kind, "contains");
    }

    #[test]
    fn test_empty_graph() {
        let graph = Graph::empty();

        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
        assert_eq!(graph.metrics.density, 0.0);
        assert_eq!(graph.metrics.avg_degree, 0.0);
    }

    #[test]
    fn test_graph_creation() {
        let nodes = vec!["sym_a".to_string(), "sym_b".to_string(), "sym_c".to_string()];
        let edges = vec![
            Edge::calls("sym_a".to_string(), "sym_b".to_string()),
            Edge::calls("sym_b".to_string(), "sym_c".to_string()),
        ];

        let graph = Graph::new(nodes, edges);

        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 2);
        
        // For 3 nodes, max edges = 3 * 2 = 6
        // Actual edges = 2
        // Density = 2/6 = 0.333...
        assert!((graph.metrics.density - 0.333).abs() < 0.01);
        
        // Avg degree = 2 * edges / nodes = 2 * 2 / 3 = 1.333...
        assert!((graph.metrics.avg_degree - 1.333).abs() < 0.01);
    }

    #[test]
    fn test_graph_mutations() {
        let mut graph = Graph::empty();

        graph.add_node("sym_a".to_string());
        graph.add_node("sym_b".to_string());
        assert_eq!(graph.node_count(), 2);

        graph.add_edge(Edge::calls("sym_a".to_string(), "sym_b".to_string()));
        assert_eq!(graph.edge_count(), 1);

        // For 2 nodes, max edges = 2 * 1 = 2
        // Actual edges = 1
        // Density = 1/2 = 0.5
        assert!((graph.metrics.density - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_graph_json_roundtrip() {
        let nodes = vec!["sym_a".to_string(), "sym_b".to_string()];
        let edges = vec![Edge::calls("sym_a".to_string(), "sym_b".to_string())];
        let graph = Graph::new(nodes, edges);

        let json = graph.to_json().unwrap();
        let parsed = Graph::from_json(&json).unwrap();

        assert_eq!(graph.node_count(), parsed.node_count());
        assert_eq!(graph.edge_count(), parsed.edge_count());
        assert_eq!(graph.nodes, parsed.nodes);
        assert_eq!(graph.edges, parsed.edges);
    }

    #[test]
    fn test_graph_json_format() {
        let nodes = vec!["sym_a".to_string(), "sym_b".to_string()];
        let edges = vec![
            Edge::calls("sym_a".to_string(), "sym_b".to_string()),
        ];
        let graph = Graph::new(nodes, edges);

        let json = graph.to_json().unwrap();

        assert!(json.contains("\"nodes\""));
        assert!(json.contains("\"edges\""));
        assert!(json.contains("\"metrics\""));
        assert!(json.contains("\"density\""));
        assert!(json.contains("\"avg_degree\""));
        assert!(json.contains("\"from\": \"sym_a\""));
        assert!(json.contains("\"to\": \"sym_b\""));
        assert!(json.contains("\"kind\": \"calls\""));
    }
}
