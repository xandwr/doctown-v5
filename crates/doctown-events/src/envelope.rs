//! Event envelope structure matching the Doctown spec.

use chrono::{DateTime, Utc};
use doctown_common::{EventId, JobId, TraceId};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

/// Thread-safe sequence number generator.
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// All valid event types in the Doctown system.
///
/// Each variant corresponds to a specific event in the processing pipeline.
/// The string representation follows the format: `{domain}.{action}.v{version}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    // Ingest events
    /// Emitted when ingest pipeline starts.
    #[serde(rename = "ingest.started.v1")]
    IngestStarted,

    /// Emitted when a processable file is detected.
    #[serde(rename = "ingest.file_detected.v1")]
    IngestFileDetected,

    /// Emitted when a file is skipped.
    #[serde(rename = "ingest.file_skipped.v1")]
    IngestFileSkipped,

    /// Emitted when a chunk is created.
    #[serde(rename = "ingest.chunk_created.v1")]
    IngestChunkCreated,

    /// Emitted when ingest pipeline completes.
    #[serde(rename = "ingest.completed.v1")]
    IngestCompleted,
}

impl EventType {
    /// Returns the string representation of this event type.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::IngestStarted => "ingest.started.v1",
            Self::IngestFileDetected => "ingest.file_detected.v1",
            Self::IngestFileSkipped => "ingest.file_skipped.v1",
            Self::IngestChunkCreated => "ingest.chunk_created.v1",
            Self::IngestCompleted => "ingest.completed.v1",
        }
    }

    /// Returns true if this event type is a terminal event (requires status).
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::IngestCompleted)
    }

    /// Attempts to parse an event type from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "ingest.started.v1" => Some(Self::IngestStarted),
            "ingest.file_detected.v1" => Some(Self::IngestFileDetected),
            "ingest.file_skipped.v1" => Some(Self::IngestFileSkipped),
            "ingest.chunk_created.v1" => Some(Self::IngestChunkCreated),
            "ingest.completed.v1" => Some(Self::IngestCompleted),
            _ => None,
        }
    }
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Validation error for event envelopes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// The event_id field is empty or invalid.
    InvalidEventId,
    /// The event_type field is empty or unrecognized.
    InvalidEventType(String),
    /// The repo_url field is empty.
    EmptyRepoUrl,
    /// Terminal event is missing status field.
    MissingStatus,
    /// Non-terminal event has status field (should only be on .completed).
    UnexpectedStatus,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidEventId => write!(f, "event_id is invalid"),
            Self::InvalidEventType(t) => write!(f, "unrecognized event_type: {t}"),
            Self::EmptyRepoUrl => write!(f, "repo_url is empty"),
            Self::MissingStatus => write!(f, "terminal event is missing status field"),
            Self::UnexpectedStatus => {
                write!(f, "non-terminal event should not have status field")
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// The event envelope that wraps all Doctown events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T> {
    /// Unique identifier for this event.
    pub event_id: EventId,

    /// The event type (e.g., "ingest.started.v1").
    pub event_type: String,

    /// ISO 8601 timestamp when the event was created.
    pub timestamp: DateTime<Utc>,

    /// Sequence number for ordering events within a job.
    pub sequence: u64,

    /// Context about the job.
    pub context: Context,

    /// Additional metadata.
    pub meta: Meta,

    /// The event payload.
    pub payload: T,

    /// Terminal status (only present on .completed events).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
}

impl<T> Envelope<T> {
    /// Creates a new event envelope.
    pub fn new(event_type: impl Into<String>, context: Context, payload: T) -> Self {
        Self {
            event_id: EventId::generate(),
            event_type: event_type.into(),
            timestamp: Utc::now(),
            sequence: SEQUENCE.fetch_add(1, Ordering::SeqCst),
            context,
            meta: Meta::default(),
            payload,
            status: None,
        }
    }

    /// Creates a new envelope with a parent event for causality chains.
    pub fn with_parent(
        event_type: impl Into<String>,
        context: Context,
        payload: T,
        parent_id: &EventId,
    ) -> Self {
        let mut envelope = Self::new(event_type, context, payload);
        envelope.meta.parent_event_id = Some(parent_id.clone());
        envelope
    }

    /// Sets the terminal status on this envelope.
    pub fn with_status(mut self, status: Status) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets the trace ID on this envelope.
    pub fn with_trace(mut self, trace_id: TraceId) -> Self {
        self.meta.trace_id = Some(trace_id);
        self
    }

    /// Creates a new envelope with a typed event type.
    pub fn typed(event_type: EventType, context: Context, payload: T) -> Self {
        Self::new(event_type.as_str(), context, payload)
    }

    /// Validates the envelope according to the Doctown spec.
    ///
    /// Checks:
    /// - event_id starts with "evt_"
    /// - event_type is a recognized type
    /// - repo_url is not empty
    /// - terminal events have status, non-terminal events do not
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Validate event_id format
        if !self.event_id.as_str().starts_with("evt_") {
            return Err(ValidationError::InvalidEventId);
        }

        // Validate event_type is recognized
        let event_type = EventType::from_str(&self.event_type)
            .ok_or_else(|| ValidationError::InvalidEventType(self.event_type.clone()))?;

        // Validate repo_url is not empty
        if self.context.repo_url.is_empty() {
            return Err(ValidationError::EmptyRepoUrl);
        }

        // Validate status presence based on event type
        if event_type.is_terminal() {
            if self.status.is_none() {
                return Err(ValidationError::MissingStatus);
            }
        } else if self.status.is_some() {
            return Err(ValidationError::UnexpectedStatus);
        }

        Ok(())
    }

    /// Returns the parsed EventType if valid.
    pub fn parsed_event_type(&self) -> Option<EventType> {
        EventType::from_str(&self.event_type)
    }
}

/// Context about the job being processed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    /// The job identifier.
    pub job_id: JobId,

    /// The repository URL being processed.
    pub repo_url: String,

    /// The git ref (branch, tag, or commit).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_ref: Option<String>,

    /// The user who initiated the job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

impl Context {
    /// Creates a new context for a job.
    pub fn new(job_id: JobId, repo_url: impl Into<String>) -> Self {
        Self {
            job_id,
            repo_url: repo_url.into(),
            git_ref: None,
            user_id: None,
        }
    }

    /// Sets the git ref.
    pub fn with_git_ref(mut self, git_ref: impl Into<String>) -> Self {
        self.git_ref = Some(git_ref.into());
        self
    }

    /// Sets the user ID.
    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }
}

/// Metadata about the event.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Meta {
    /// The version of the producer that created this event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub producer_version: Option<String>,

    /// Trace ID for distributed tracing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<TraceId>,

    /// Parent event ID for causality.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_event_id: Option<EventId>,

    /// Idempotency key for deduplication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,

    /// Optional tags for filtering.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// Terminal status for completed events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Success,
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestPayload {
        message: String,
    }

    #[test]
    fn test_envelope_creation() {
        let job_id = JobId::generate();
        let context = Context::new(job_id, "https://github.com/user/repo");
        let payload = TestPayload {
            message: "hello".to_string(),
        };

        let envelope = Envelope::new("test.event.v1", context, payload);

        assert_eq!(envelope.event_type, "test.event.v1");
        assert!(envelope.event_id.as_str().starts_with("evt_"));
        assert!(envelope.status.is_none());
    }

    #[test]
    fn test_envelope_with_status() {
        let job_id = JobId::generate();
        let context = Context::new(job_id, "https://github.com/user/repo");
        let payload = TestPayload {
            message: "done".to_string(),
        };

        let envelope =
            Envelope::new("test.completed.v1", context, payload).with_status(Status::Success);

        assert_eq!(envelope.status, Some(Status::Success));
    }

    #[test]
    fn test_envelope_serialization() {
        let job_id = JobId::new("job_test12345678").unwrap();
        let context = Context::new(job_id, "https://github.com/user/repo").with_git_ref("main");
        let payload = TestPayload {
            message: "test".to_string(),
        };

        let envelope = Envelope::new("test.event.v1", context, payload);
        let json = serde_json::to_value(&envelope).unwrap();

        assert_eq!(json["event_type"], "test.event.v1");
        assert_eq!(json["context"]["repo_url"], "https://github.com/user/repo");
        assert_eq!(json["context"]["git_ref"], "main");
        assert_eq!(json["payload"]["message"], "test");
    }

    #[test]
    fn test_context_with_methods() {
        let job_id = JobId::generate();
        let context = Context::new(job_id.clone(), "https://github.com/user/repo")
            .with_git_ref("develop")
            .with_user("user_123");

        assert_eq!(context.git_ref, Some("develop".to_string()));
        assert_eq!(context.user_id, Some("user_123".to_string()));
    }

    #[test]
    fn test_sequence_increments() {
        let job_id = JobId::generate();
        let context = Context::new(job_id, "https://github.com/user/repo");
        let payload = TestPayload {
            message: "test".to_string(),
        };

        let e1 = Envelope::new("test.v1", context.clone(), payload.clone());
        let e2 = Envelope::new("test.v1", context.clone(), payload.clone());

        assert!(e2.sequence > e1.sequence);
    }

    // --- EventType tests ---

    #[test]
    fn test_event_type_as_str() {
        assert_eq!(EventType::IngestStarted.as_str(), "ingest.started.v1");
        assert_eq!(
            EventType::IngestFileDetected.as_str(),
            "ingest.file_detected.v1"
        );
        assert_eq!(
            EventType::IngestFileSkipped.as_str(),
            "ingest.file_skipped.v1"
        );
        assert_eq!(
            EventType::IngestChunkCreated.as_str(),
            "ingest.chunk_created.v1"
        );
        assert_eq!(EventType::IngestCompleted.as_str(), "ingest.completed.v1");
    }

    #[test]
    fn test_event_type_from_str() {
        assert_eq!(
            EventType::from_str("ingest.started.v1"),
            Some(EventType::IngestStarted)
        );
        assert_eq!(
            EventType::from_str("ingest.completed.v1"),
            Some(EventType::IngestCompleted)
        );
        assert_eq!(EventType::from_str("unknown.event.v1"), None);
    }

    #[test]
    fn test_event_type_is_terminal() {
        assert!(!EventType::IngestStarted.is_terminal());
        assert!(!EventType::IngestFileDetected.is_terminal());
        assert!(!EventType::IngestFileSkipped.is_terminal());
        assert!(!EventType::IngestChunkCreated.is_terminal());
        assert!(EventType::IngestCompleted.is_terminal());
    }

    #[test]
    fn test_event_type_display() {
        assert_eq!(format!("{}", EventType::IngestStarted), "ingest.started.v1");
        assert_eq!(
            format!("{}", EventType::IngestCompleted),
            "ingest.completed.v1"
        );
    }

    #[test]
    fn test_event_type_serialization() {
        let json = serde_json::to_string(&EventType::IngestStarted).unwrap();
        assert_eq!(json, r#""ingest.started.v1""#);

        let parsed: EventType = serde_json::from_str(r#""ingest.completed.v1""#).unwrap();
        assert_eq!(parsed, EventType::IngestCompleted);
    }

    // --- Validation tests ---

    #[test]
    fn test_validate_valid_envelope() {
        let job_id = JobId::generate();
        let context = Context::new(job_id, "https://github.com/user/repo");
        let payload = TestPayload {
            message: "test".to_string(),
        };

        let envelope = Envelope::typed(EventType::IngestStarted, context, payload);
        assert!(envelope.validate().is_ok());
    }

    #[test]
    fn test_validate_terminal_with_status() {
        let job_id = JobId::generate();
        let context = Context::new(job_id, "https://github.com/user/repo");
        let payload = TestPayload {
            message: "done".to_string(),
        };

        let envelope = Envelope::typed(EventType::IngestCompleted, context, payload)
            .with_status(Status::Success);
        assert!(envelope.validate().is_ok());
    }

    #[test]
    fn test_validate_terminal_missing_status() {
        let job_id = JobId::generate();
        let context = Context::new(job_id, "https://github.com/user/repo");
        let payload = TestPayload {
            message: "done".to_string(),
        };

        let envelope = Envelope::typed(EventType::IngestCompleted, context, payload);
        assert_eq!(envelope.validate(), Err(ValidationError::MissingStatus));
    }

    #[test]
    fn test_validate_non_terminal_with_status() {
        let job_id = JobId::generate();
        let context = Context::new(job_id, "https://github.com/user/repo");
        let payload = TestPayload {
            message: "test".to_string(),
        };

        let envelope =
            Envelope::typed(EventType::IngestStarted, context, payload).with_status(Status::Failed);
        assert_eq!(envelope.validate(), Err(ValidationError::UnexpectedStatus));
    }

    #[test]
    fn test_validate_empty_repo_url() {
        let job_id = JobId::generate();
        let context = Context::new(job_id, "");
        let payload = TestPayload {
            message: "test".to_string(),
        };

        let envelope = Envelope::typed(EventType::IngestStarted, context, payload);
        assert_eq!(envelope.validate(), Err(ValidationError::EmptyRepoUrl));
    }

    #[test]
    fn test_validate_unknown_event_type() {
        let job_id = JobId::generate();
        let context = Context::new(job_id, "https://github.com/user/repo");
        let payload = TestPayload {
            message: "test".to_string(),
        };

        let envelope = Envelope::new("unknown.event.v1", context, payload);
        assert_eq!(
            envelope.validate(),
            Err(ValidationError::InvalidEventType("unknown.event.v1".to_string()))
        );
    }

    // --- Snapshot tests ---

    #[test]
    fn test_envelope_snapshot() {
        use chrono::TimeZone;

        let job_id = JobId::new("job_abc123def456").unwrap();
        let event_id = doctown_common::EventId::new("evt_test1234567890").unwrap();
        let context = Context::new(job_id, "https://github.com/example/repo")
            .with_git_ref("main")
            .with_user("user_42");
        let payload = TestPayload {
            message: "Hello, Doctown!".to_string(),
        };

        let mut envelope = Envelope::typed(EventType::IngestStarted, context, payload);
        // Override dynamic fields for deterministic snapshot
        envelope.event_id = event_id;
        envelope.timestamp = Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap();
        envelope.sequence = 42;

        insta::assert_json_snapshot!("envelope_ingest_started", envelope);
    }

    #[test]
    fn test_envelope_completed_snapshot() {
        use chrono::TimeZone;

        let job_id = JobId::new("job_abc123def456").unwrap();
        let event_id = doctown_common::EventId::new("evt_completed123").unwrap();
        let context = Context::new(job_id, "https://github.com/example/repo").with_git_ref("v1.0.0");
        let payload = TestPayload {
            message: "Processing complete".to_string(),
        };

        let mut envelope = Envelope::typed(EventType::IngestCompleted, context, payload)
            .with_status(Status::Success);
        envelope.event_id = event_id;
        envelope.timestamp = Utc.with_ymd_and_hms(2024, 1, 15, 10, 35, 0).unwrap();
        envelope.sequence = 100;

        insta::assert_json_snapshot!("envelope_ingest_completed", envelope);
    }
}
