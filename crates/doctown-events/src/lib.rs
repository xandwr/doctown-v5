//! Event envelope and serialization for Doctown.
//!
//! This crate provides the event system used for streaming updates during processing.

pub mod envelope;
pub mod ingest;
pub mod assembly;

pub use envelope::{Context, Envelope, EventType, Meta, Status, ValidationError};
pub use ingest::*;
pub use assembly::*;
