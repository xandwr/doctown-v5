//! Semantic Assembly: Clustering, labeling, and graph construction for code understanding.
//!
//! This crate takes embedded chunks and builds semantic structures:
//! - Vector clustering (k-means)
//! - Cluster labeling (TF-IDF based)
//! - Graph construction (calls, imports, similarity edges)
//! - Graph metrics (centrality, density)
//! - Symbol context generation for LLM documentation
//! - Docpack packing (assembling complete .docpack files)

pub mod api;
pub mod cluster;
pub mod context;
pub mod graph;
pub mod label;
pub mod packer;

pub use api::{start_server, AssembleRequest, AssembleResponse};
pub use cluster::Clusterer;
pub use context::{ContextGenerator, SymbolContext};
pub use graph::{Edge, EdgeKind, Graph, GraphBuilder, Node, SymbolData};
pub use label::ClusterLabeler;
pub use packer::{ChunkInfo, EmbeddingData, PackRequest, PackResponse, Packer, SourceFileInfo};
