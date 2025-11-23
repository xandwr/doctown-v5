//! Ingest worker for Doctown.
//!
//! This crate handles:
//! - Fetching repositories from GitHub
//! - Parsing source code using tree-sitter
//! - Extracting symbols and creating chunks
//! - Streaming events via SSE

pub mod archive;
pub mod chunk;
pub mod filter;
pub mod github;
pub mod language;
pub mod parsing;
pub mod pipeline;
pub mod symbol;
pub mod traversal;
pub mod api;

pub use archive::{extract_zip, process_extracted_files};
pub use chunk::{create_chunks, Chunk, ChunkMetadata, ChunkingConfig};
pub use filter::{
    normalize_archive_path, FileFilter, FilterResult, SkipReason as FilterSkipReason,
    MAX_FILE_SIZE, MAX_REPO_SIZE,
};
pub use github::{GitHubClient, GitHubUrl, RateLimitInfo, RefInfo, RepoMetadata};
pub use language::detect_language;
pub use parsing::{parse, Parser};
pub use pipeline::run_pipeline;
pub use symbol::{extract_symbols, Symbol};
pub use traversal::{
    ancestors, child_by_field, child_text, collect_named_children_text, find_ancestor_by_kind,
    find_child_by_kind, find_children_by_kind, find_nodes_by_kind, find_nodes_by_kinds, has_error,
    is_error, is_missing, is_named, matches_any_kind, matches_kind, node_byte_range,
    node_end_position, node_line_count, node_start_position, node_text, node_text_owned,
    text_from_range, DfsIterator, TreeCursor,
};
