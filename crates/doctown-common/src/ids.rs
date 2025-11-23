//! Identifier types for Doctown entities.

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// A unique identifier for a job.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JobId(String);

impl JobId {
    /// Creates a new JobId with validation.
    ///
    /// Job IDs must:
    /// - Start with "job_"
    /// - Be between 8 and 64 characters
    /// - Contain only alphanumeric characters and underscores
    pub fn new(id: impl Into<String>) -> Result<Self, &'static str> {
        let id = id.into();
        Self::validate(&id)?;
        Ok(Self(id))
    }

    /// Creates a new random JobId.
    pub fn generate() -> Self {
        let uuid = Uuid::new_v4().to_string().replace('-', "");
        Self(format!("job_{}", &uuid[..16]))
    }

    fn validate(id: &str) -> Result<(), &'static str> {
        if !id.starts_with("job_") {
            return Err("JobId must start with 'job_'");
        }
        if id.len() < 8 || id.len() > 64 {
            return Err("JobId must be between 8 and 64 characters");
        }
        if !id.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err("JobId must contain only alphanumeric characters and underscores");
        }
        Ok(())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for JobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A unique identifier for a chunk.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ChunkId(String);

impl ChunkId {
    /// Creates a new ChunkId with validation.
    pub fn new(id: impl Into<String>) -> Result<Self, &'static str> {
        let id = id.into();
        Self::validate(&id)?;
        Ok(Self(id))
    }

    /// Creates a new random ChunkId.
    pub fn generate() -> Self {
        let uuid = Uuid::new_v4().to_string().replace('-', "");
        Self(format!("chunk_{}", &uuid[..16]))
    }

    fn validate(id: &str) -> Result<(), &'static str> {
        if !id.starts_with("chunk_") {
            return Err("ChunkId must start with 'chunk_'");
        }
        if id.len() < 10 || id.len() > 64 {
            return Err("ChunkId must be between 10 and 64 characters");
        }
        Ok(())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ChunkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A unique identifier for a symbol.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SymbolId(String);

impl SymbolId {
    /// Creates a new SymbolId with validation.
    pub fn new(id: impl Into<String>) -> Result<Self, &'static str> {
        let id = id.into();
        Self::validate(&id)?;
        Ok(Self(id))
    }

    /// Creates a new random SymbolId.
    pub fn generate() -> Self {
        let uuid = Uuid::new_v4().to_string().replace('-', "");
        Self(format!("sym_{}", &uuid[..16]))
    }

    fn validate(id: &str) -> Result<(), &'static str> {
        if !id.starts_with("sym_") {
            return Err("SymbolId must start with 'sym_'");
        }
        if id.len() < 8 || id.len() > 64 {
            return Err("SymbolId must be between 8 and 64 characters");
        }
        Ok(())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SymbolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A unique identifier for an event.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventId(String);

impl EventId {
    /// Creates a new EventId with validation.
    pub fn new(id: impl Into<String>) -> Result<Self, &'static str> {
        let id = id.into();
        Self::validate(&id)?;
        Ok(Self(id))
    }

    /// Creates a new random EventId using UUID v4.
    pub fn generate() -> Self {
        Self(format!("evt_{}", Uuid::new_v4()))
    }

    fn validate(id: &str) -> Result<(), &'static str> {
        if !id.starts_with("evt_") {
            return Err("EventId must start with 'evt_'");
        }
        if id.len() < 8 {
            return Err("EventId must be at least 8 characters");
        }
        Ok(())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A trace identifier for distributed tracing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TraceId(String);

impl TraceId {
    /// Creates a new TraceId with validation.
    pub fn new(id: impl Into<String>) -> Result<Self, &'static str> {
        let id = id.into();
        Self::validate(&id)?;
        Ok(Self(id))
    }

    /// Creates a new random TraceId.
    pub fn generate() -> Self {
        Self(format!("trace_{}", Uuid::new_v4()))
    }

    fn validate(id: &str) -> Result<(), &'static str> {
        if !id.starts_with("trace_") {
            return Err("TraceId must start with 'trace_'");
        }
        if id.len() < 10 {
            return Err("TraceId must be at least 10 characters");
        }
        Ok(())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TraceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_id_valid() {
        let id = JobId::new("job_abc123def").unwrap();
        assert_eq!(id.as_str(), "job_abc123def");
    }

    #[test]
    fn test_job_id_invalid_prefix() {
        let result = JobId::new("invalid_123");
        assert!(result.is_err());
    }

    #[test]
    fn test_job_id_generate() {
        let id = JobId::generate();
        assert!(id.as_str().starts_with("job_"));
    }

    #[test]
    fn test_chunk_id_valid() {
        let id = ChunkId::new("chunk_abc123def").unwrap();
        assert_eq!(id.as_str(), "chunk_abc123def");
    }

    #[test]
    fn test_chunk_id_invalid_prefix() {
        let result = ChunkId::new("invalid_123");
        assert!(result.is_err());
    }

    #[test]
    fn test_symbol_id_valid() {
        let id = SymbolId::new("sym_abc123def").unwrap();
        assert_eq!(id.as_str(), "sym_abc123def");
    }

    #[test]
    fn test_event_id_generate() {
        let id = EventId::generate();
        assert!(id.as_str().starts_with("evt_"));
    }

    #[test]
    fn test_trace_id_generate() {
        let id = TraceId::generate();
        assert!(id.as_str().starts_with("trace_"));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let job_id = JobId::generate();
        let json = serde_json::to_string(&job_id).unwrap();
        let deserialized: JobId = serde_json::from_str(&json).unwrap();
        assert_eq!(job_id, deserialized);
    }
}
