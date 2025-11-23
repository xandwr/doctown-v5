//! Event envelope structure matching the Doctown spec.

use chrono::{DateTime, Utc};
use doctown_common::{EventId, JobId, TraceId};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// Thread-safe sequence number generator.
static SEQUENCE: AtomicU64 = AtomicU64::new(0);

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
}
