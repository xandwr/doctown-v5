//! Shared types, errors, and utilities for Doctown.
//!
//! This crate provides the foundational types used across all Doctown components.

pub mod error;
pub mod ids;
pub mod types;

pub use error::DocError;
pub use ids::{ChunkId, EventId, JobId, SymbolId, TraceId};
pub use types::{ByteRange, Language, SymbolKind, Visibility};
