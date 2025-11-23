//! Ingest event types for Milestone 1.

use doctown_common::{ByteRange, ChunkId, Language, SymbolKind};
use serde::{Deserialize, Serialize};

/// Payload for `ingest.started.v1` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestStartedPayload {
    /// The repository URL being ingested.
    pub repo_url: String,

    /// The git ref being processed.
    pub git_ref: String,

    /// Resolved commit SHA (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_sha: Option<String>,
}

impl IngestStartedPayload {
    pub fn new(repo_url: impl Into<String>, git_ref: impl Into<String>) -> Self {
        Self {
            repo_url: repo_url.into(),
            git_ref: git_ref.into(),
            commit_sha: None,
        }
    }

    pub fn with_commit(mut self, sha: impl Into<String>) -> Self {
        self.commit_sha = Some(sha.into());
        self
    }
}

/// Payload for `ingest.file_detected.v1` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestFileDetectedPayload {
    /// Path to the file relative to repo root.
    pub file_path: String,

    /// Detected language.
    pub language: Language,

    /// File size in bytes.
    pub size_bytes: usize,
}

impl IngestFileDetectedPayload {
    pub fn new(file_path: impl Into<String>, language: Language, size_bytes: usize) -> Self {
        Self {
            file_path: file_path.into(),
            language,
            size_bytes,
        }
    }
}

/// Payload for `ingest.file_skipped.v1` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestFileSkippedPayload {
    /// Path to the file relative to repo root.
    pub file_path: String,

    /// Reason the file was skipped.
    pub reason: SkipReason,
}

impl IngestFileSkippedPayload {
    pub fn new(file_path: impl Into<String>, reason: SkipReason) -> Self {
        Self {
            file_path: file_path.into(),
            reason,
        }
    }
}

/// Reasons a file might be skipped during ingest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkipReason {
    /// File is binary (contains null bytes).
    Binary,

    /// File exceeds size limit.
    TooLarge,

    /// Unsupported file type/language.
    UnsupportedLanguage,

    /// File matches ignore pattern.
    IgnorePattern,

    /// Failed to parse the file.
    ParseError,
}

/// Payload for `ingest.chunk_created.v1` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestChunkCreatedPayload {
    /// Unique identifier for this chunk.
    pub chunk_id: ChunkId,

    /// Path to the source file.
    pub file_path: String,

    /// The language of the chunk.
    pub language: Language,

    /// Byte range in the source file.
    pub byte_range: ByteRange,

    /// The kind of symbol this chunk represents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_kind: Option<SymbolKind>,

    /// The name of the symbol (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_name: Option<String>,

    /// The content of the chunk.
    pub content: String,
}

impl IngestChunkCreatedPayload {
    pub fn new(
        chunk_id: ChunkId,
        file_path: impl Into<String>,
        language: Language,
        byte_range: ByteRange,
        content: impl Into<String>,
    ) -> Self {
        Self {
            chunk_id,
            file_path: file_path.into(),
            language,
            byte_range,
            symbol_kind: None,
            symbol_name: None,
            content: content.into(),
        }
    }

    pub fn with_symbol(mut self, kind: SymbolKind, name: impl Into<String>) -> Self {
        self.symbol_kind = Some(kind);
        self.symbol_name = Some(name.into());
        self
    }
}

/// Payload for `ingest.completed.v1` event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestCompletedPayload {
    /// Total number of files processed.
    pub files_processed: usize,

    /// Number of files skipped.
    pub files_skipped: usize,

    /// Total number of chunks created.
    pub chunks_created: usize,

    /// Processing duration in milliseconds.
    pub duration_ms: u64,

    /// Breakdown by language.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub language_breakdown: Vec<LanguageCount>,

    /// Error message if failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl IngestCompletedPayload {
    pub fn success(
        files_processed: usize,
        files_skipped: usize,
        chunks_created: usize,
        duration_ms: u64,
    ) -> Self {
        Self {
            files_processed,
            files_skipped,
            chunks_created,
            duration_ms,
            language_breakdown: Vec::new(),
            error: None,
        }
    }

    pub fn failed(error: impl Into<String>, duration_ms: u64) -> Self {
        Self {
            files_processed: 0,
            files_skipped: 0,
            chunks_created: 0,
            duration_ms,
            language_breakdown: Vec::new(),
            error: Some(error.into()),
        }
    }

    pub fn with_breakdown(mut self, breakdown: Vec<LanguageCount>) -> Self {
        self.language_breakdown = breakdown;
        self
    }
}

/// Count of files/chunks per language.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageCount {
    pub language: Language,
    pub file_count: usize,
    pub chunk_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_ingest_started_serialization() {
        let payload =
            IngestStartedPayload::new("https://github.com/user/repo", "main").with_commit("abc123");

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["repo_url"], "https://github.com/user/repo");
        assert_eq!(json["git_ref"], "main");
        assert_eq!(json["commit_sha"], "abc123");
    }

    #[test]
    fn test_file_detected_serialization() {
        let payload = IngestFileDetectedPayload::new("src/main.rs", Language::Rust, 1024);

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["file_path"], "src/main.rs");
        assert_eq!(json["language"], "rust");
        assert_eq!(json["size_bytes"], 1024);
    }

    #[test]
    fn test_file_skipped_serialization() {
        let payload = IngestFileSkippedPayload::new("binary.exe", SkipReason::Binary);

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["file_path"], "binary.exe");
        assert_eq!(json["reason"], "binary");
    }

    #[test]
    fn test_chunk_created_with_symbol() {
        let chunk_id = ChunkId::generate();
        let payload = IngestChunkCreatedPayload::new(
            chunk_id,
            "src/lib.rs",
            Language::Rust,
            ByteRange::new(0, 100),
            "fn main() {}",
        )
        .with_symbol(SymbolKind::Function, "main");

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["symbol_kind"], "function");
        assert_eq!(json["symbol_name"], "main");
    }

    #[test]
    fn test_ingest_completed_success() {
        let payload = IngestCompletedPayload::success(10, 2, 50, 1234);

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["files_processed"], 10);
        assert_eq!(json["files_skipped"], 2);
        assert_eq!(json["chunks_created"], 50);
        assert!(json.get("error").is_none());
    }

    #[test]
    fn test_ingest_completed_failed() {
        let payload = IngestCompletedPayload::failed("Download failed", 500);

        let json = serde_json::to_value(&payload).unwrap();
        assert_eq!(json["error"], "Download failed");
    }

    // --- Snapshot tests ---

    #[test]
    fn test_ingest_started_snapshot() {
        let payload = IngestStartedPayload::new("https://github.com/example/awesome-project", "main")
            .with_commit("abc123def456789");

        insta::assert_json_snapshot!("ingest_started_payload", payload);
    }

    #[test]
    fn test_ingest_file_detected_snapshot() {
        let payload = IngestFileDetectedPayload::new("src/lib.rs", Language::Rust, 2048);

        insta::assert_json_snapshot!("ingest_file_detected_payload", payload);
    }

    #[test]
    fn test_ingest_file_skipped_snapshot() {
        let payload = IngestFileSkippedPayload::new("data/large-binary.bin", SkipReason::Binary);

        insta::assert_json_snapshot!("ingest_file_skipped_payload", payload);
    }

    #[test]
    fn test_ingest_chunk_created_snapshot() {
        let chunk_id = ChunkId::new("chunk_deterministic1").unwrap();
        let payload = IngestChunkCreatedPayload::new(
            chunk_id,
            "src/main.rs",
            Language::Rust,
            ByteRange::new(0, 150),
            "fn main() {\n    println!(\"Hello, world!\");\n}",
        )
        .with_symbol(SymbolKind::Function, "main");

        insta::assert_json_snapshot!("ingest_chunk_created_payload", payload);
    }

    #[test]
    fn test_ingest_completed_success_snapshot() {
        let payload = IngestCompletedPayload::success(25, 5, 100, 3500).with_breakdown(vec![
            LanguageCount {
                language: Language::Rust,
                file_count: 15,
                chunk_count: 60,
            },
            LanguageCount {
                language: Language::Python,
                file_count: 10,
                chunk_count: 40,
            },
        ]);

        insta::assert_json_snapshot!("ingest_completed_success_payload", payload);
    }

    #[test]
    fn test_ingest_completed_failed_snapshot() {
        let payload = IngestCompletedPayload::failed("Repository not found: 404", 250);

        insta::assert_json_snapshot!("ingest_completed_failed_payload", payload);
    }
}
