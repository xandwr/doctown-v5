//! Semantic Assembly: Clustering, labeling, and graph construction for code understanding.
//!
//! This crate takes embedded chunks and builds semantic structures:
//! - Vector clustering (k-means)
//! - Cluster labeling (TF-IDF based)
//! - Graph construction (calls, imports, similarity edges)
//! - Graph metrics (centrality, density)

pub mod cluster;
pub mod graph;
pub mod label;

pub use cluster::Clusterer;
pub use graph::{Graph, Node, Edge, EdgeKind, GraphBuilder, SymbolData};
pub use label::ClusterLabeler;
