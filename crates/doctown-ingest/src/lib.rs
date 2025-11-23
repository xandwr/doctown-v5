//! Ingest worker for Doctown.
//!
//! This crate handles:
//! - Fetching repositories from GitHub
//! - Parsing source code using tree-sitter
//! - Extracting symbols and creating chunks
//! - Streaming events via SSE

pub mod github;
pub mod language;

pub use github::GitHubUrl;
pub use language::detect_language;
