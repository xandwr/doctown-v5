//! Assembly event payloads.
//!
//! Events emitted during the semantic assembly phase (clustering, labeling, graph construction).

use serde::{Deserialize, Serialize};

/// Payload for `assembly.started.v1` event.
///
/// Emitted when the assembly worker begins processing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssemblyStartedPayload {
    /// Number of chunks to process.
    pub chunk_count: usize,
    /// Number of symbols to process.
    pub symbol_count: usize,
}

/// Payload for `assembly.cluster_created.v1` event.
///
/// Emitted for each cluster created during assembly.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssemblyClusterCreatedPayload {
    /// Unique identifier for this cluster.
    pub cluster_id: String,
    /// Human-readable label for this cluster.
    pub label: String,
    /// Number of symbols in this cluster.
    pub member_count: usize,
}

/// Payload for `assembly.graph_completed.v1` event.
///
/// Emitted when the graph construction is complete.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssemblyGraphCompletedPayload {
    /// Total number of nodes in the graph.
    pub node_count: usize,
    /// Total number of edges in the graph.
    pub edge_count: usize,
    /// Breakdown of edges by type.
    pub edge_types: EdgeTypeBreakdown,
}

/// Breakdown of edges by type in the graph.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EdgeTypeBreakdown {
    /// Number of "calls" edges.
    pub calls: usize,
    /// Number of "imports" edges.
    pub imports: usize,
    /// Number of "related" (similarity) edges.
    pub related: usize,
}

/// Payload for `assembly.completed.v1` event.
///
/// Emitted when the assembly worker completes processing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AssemblyCompletedPayload {
    /// Total number of clusters created.
    pub cluster_count: usize,
    /// Total number of nodes in the graph.
    pub node_count: usize,
    /// Total number of edges in the graph.
    pub edge_count: usize,
    /// Total processing duration in milliseconds.
    pub duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assembly_started_serialization() {
        let payload = AssemblyStartedPayload {
            chunk_count: 100,
            symbol_count: 50,
        };
        let json = serde_json::to_string(&payload).unwrap();
        let deserialized: AssemblyStartedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(payload, deserialized);
    }

    #[test]
    fn test_cluster_created_serialization() {
        let payload = AssemblyClusterCreatedPayload {
            cluster_id: "cluster_0".to_string(),
            label: "API handlers".to_string(),
            member_count: 15,
        };
        let json = serde_json::to_string(&payload).unwrap();
        let deserialized: AssemblyClusterCreatedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(payload, deserialized);
    }

    #[test]
    fn test_graph_completed_serialization() {
        let payload = AssemblyGraphCompletedPayload {
            node_count: 50,
            edge_count: 120,
            edge_types: EdgeTypeBreakdown {
                calls: 80,
                imports: 30,
                related: 10,
            },
        };
        let json = serde_json::to_string(&payload).unwrap();
        let deserialized: AssemblyGraphCompletedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(payload, deserialized);
    }

    #[test]
    fn test_assembly_completed_serialization() {
        let payload = AssemblyCompletedPayload {
            cluster_count: 5,
            node_count: 50,
            edge_count: 120,
            duration_ms: 1500,
        };
        let json = serde_json::to_string(&payload).unwrap();
        let deserialized: AssemblyCompletedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(payload, deserialized);
    }
}
