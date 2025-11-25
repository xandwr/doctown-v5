use serde::{Deserialize, Serialize};

/// Root metadata for a docpack
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Manifest {
    pub schema_version: String,
    pub docpack_id: String,
    pub created_at: String,
    pub generator: Generator,
    pub source: Source,
    pub statistics: Statistics,
    pub checksum: Checksum,
    pub optional: OptionalFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Generator {
    pub version: String,
    pub pipeline_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Source {
    pub repo_url: String,
    pub git_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Statistics {
    pub file_count: usize,
    pub symbol_count: usize,
    pub cluster_count: usize,
    pub embedding_dimensions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Checksum {
    pub algorithm: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OptionalFeatures {
    pub has_embeddings: bool,
    pub has_symbol_contexts: bool,
}

impl Manifest {
    /// Create a new manifest with default values
    pub fn new(
        repo_url: String,
        git_ref: String,
        commit_hash: Option<String>,
        file_count: usize,
        symbol_count: usize,
        cluster_count: usize,
    ) -> Self {
        let created_at = chrono::Utc::now().to_rfc3339();

        Self {
            schema_version: "docpack/1.0".to_string(),
            docpack_id: String::new(), // Will be set after computing checksum
            created_at,
            generator: Generator {
                version: "doctown-packer/1.0.0".to_string(),
                pipeline_version: "v5.0".to_string(),
            },
            source: Source {
                repo_url,
                git_ref,
                commit_hash,
            },
            statistics: Statistics {
                file_count,
                symbol_count,
                cluster_count,
                embedding_dimensions: 384,
            },
            checksum: Checksum {
                algorithm: "sha256".to_string(),
                value: String::new(), // Will be set after computing checksum
            },
            optional: OptionalFeatures {
                has_embeddings: false,
                has_symbol_contexts: false,
            },
        }
    }

    /// Create a new manifest with a deterministic timestamp (for testing/reproducibility)
    pub fn new_deterministic(
        repo_url: String,
        git_ref: String,
        commit_hash: Option<String>,
        file_count: usize,
        symbol_count: usize,
        cluster_count: usize,
        created_at: String,
    ) -> Self {
        Self {
            schema_version: "docpack/1.0".to_string(),
            docpack_id: String::new(),
            created_at,
            generator: Generator {
                version: "doctown-packer/1.0.0".to_string(),
                pipeline_version: "v5.0".to_string(),
            },
            source: Source {
                repo_url,
                git_ref,
                commit_hash,
            },
            statistics: Statistics {
                file_count,
                symbol_count,
                cluster_count,
                embedding_dimensions: 384,
            },
            checksum: Checksum {
                algorithm: "sha256".to_string(),
                value: String::new(),
            },
            optional: OptionalFeatures {
                has_embeddings: false,
                has_symbol_contexts: false,
            },
        }
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Serialize to JSON bytes
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec_pretty(self)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Deserialize from JSON bytes
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let manifest = Manifest::new(
            "https://github.com/test/repo".to_string(),
            "main".to_string(),
            Some("abc123".to_string()),
            10,
            50,
            5,
        );

        assert_eq!(manifest.schema_version, "docpack/1.0");
        assert_eq!(manifest.source.repo_url, "https://github.com/test/repo");
        assert_eq!(manifest.source.git_ref, "main");
        assert_eq!(manifest.source.commit_hash, Some("abc123".to_string()));
        assert_eq!(manifest.statistics.file_count, 10);
        assert_eq!(manifest.statistics.symbol_count, 50);
        assert_eq!(manifest.statistics.cluster_count, 5);
        assert_eq!(manifest.statistics.embedding_dimensions, 384);
    }

    #[test]
    fn test_manifest_json_roundtrip() {
        let manifest = Manifest::new(
            "https://github.com/test/repo".to_string(),
            "main".to_string(),
            None,
            10,
            50,
            5,
        );

        let json = manifest.to_json().unwrap();
        let parsed = Manifest::from_json(&json).unwrap();

        assert_eq!(manifest.schema_version, parsed.schema_version);
        assert_eq!(manifest.source.repo_url, parsed.source.repo_url);
        assert_eq!(manifest.statistics.file_count, parsed.statistics.file_count);
    }

    #[test]
    fn test_manifest_json_format() {
        let manifest = Manifest::new(
            "https://github.com/test/repo".to_string(),
            "main".to_string(),
            Some("deadbeef".to_string()),
            42,
            128,
            12,
        );

        let json = manifest.to_json().unwrap();

        // Check that key fields are present
        assert!(json.contains("\"schema_version\": \"docpack/1.0\""));
        assert!(json.contains("\"repo_url\": \"https://github.com/test/repo\""));
        assert!(json.contains("\"git_ref\": \"main\""));
        assert!(json.contains("\"commit_hash\": \"deadbeef\""));
        assert!(json.contains("\"file_count\": 42"));
        assert!(json.contains("\"symbol_count\": 128"));
        assert!(json.contains("\"cluster_count\": 12"));
        assert!(json.contains("\"embedding_dimensions\": 384"));
    }
}
