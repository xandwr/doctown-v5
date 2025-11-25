use serde::{Deserialize, Serialize};

/// Container for semantic clusters
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Clusters {
    pub clusters: Vec<Cluster>,
}

/// A semantic cluster for navigation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Cluster {
    pub cluster_id: String,
    pub label: String,
    pub member_count: usize,
}

impl Clusters {
    /// Create a new Clusters container
    pub fn new(clusters: Vec<Cluster>) -> Self {
        Self { clusters }
    }

    /// Create an empty Clusters container
    pub fn empty() -> Self {
        Self {
            clusters: Vec::new(),
        }
    }

    /// Add a cluster
    pub fn add_cluster(&mut self, cluster: Cluster) {
        self.clusters.push(cluster);
    }

    /// Get the number of clusters
    pub fn len(&self) -> usize {
        self.clusters.len()
    }

    /// Check if there are no clusters
    pub fn is_empty(&self) -> bool {
        self.clusters.is_empty()
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

impl Cluster {
    /// Create a new cluster
    pub fn new(cluster_id: String, label: String, member_count: usize) -> Self {
        Self {
            cluster_id,
            label,
            member_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_creation() {
        let cluster = Cluster::new("cluster_auth".to_string(), "authentication".to_string(), 12);

        assert_eq!(cluster.cluster_id, "cluster_auth");
        assert_eq!(cluster.label, "authentication");
        assert_eq!(cluster.member_count, 12);
    }

    #[test]
    fn test_clusters_creation() {
        let cluster1 = Cluster::new("cluster_auth".to_string(), "authentication".to_string(), 12);
        let cluster2 = Cluster::new("cluster_db".to_string(), "database".to_string(), 8);

        let clusters = Clusters::new(vec![cluster1, cluster2]);

        assert_eq!(clusters.len(), 2);
        assert!(!clusters.is_empty());
        assert_eq!(clusters.clusters[0].cluster_id, "cluster_auth");
        assert_eq!(clusters.clusters[1].cluster_id, "cluster_db");
    }

    #[test]
    fn test_empty_clusters() {
        let clusters = Clusters::empty();

        assert_eq!(clusters.len(), 0);
        assert!(clusters.is_empty());
    }

    #[test]
    fn test_clusters_json_roundtrip() {
        let cluster = Cluster::new("cluster_auth".to_string(), "authentication".to_string(), 12);
        let clusters = Clusters::new(vec![cluster]);

        let json = clusters.to_json().unwrap();
        let parsed = Clusters::from_json(&json).unwrap();

        assert_eq!(clusters.len(), parsed.len());
        assert_eq!(
            clusters.clusters[0].cluster_id,
            parsed.clusters[0].cluster_id
        );
        assert_eq!(clusters.clusters[0].label, parsed.clusters[0].label);
        assert_eq!(
            clusters.clusters[0].member_count,
            parsed.clusters[0].member_count
        );
    }

    #[test]
    fn test_clusters_json_format() {
        let cluster = Cluster::new("cluster_auth".to_string(), "authentication".to_string(), 12);
        let clusters = Clusters::new(vec![cluster]);

        let json = clusters.to_json().unwrap();

        assert!(json.contains("\"cluster_id\": \"cluster_auth\""));
        assert!(json.contains("\"label\": \"authentication\""));
        assert!(json.contains("\"member_count\": 12"));
    }
}
