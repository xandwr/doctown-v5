//! GitHub URL parsing and API client.

use doctown_common::DocError;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use reqwest::StatusCode;
use serde::Deserialize;
use std::fmt;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use url::Url;

/// A parsed GitHub repository URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubUrl {
    /// Repository owner (user or organization).
    pub owner: String,

    /// Repository name.
    pub repo: String,

    /// Optional git ref (branch, tag, or commit).
    pub git_ref: Option<String>,
}

impl GitHubUrl {
    /// Parses a GitHub URL.
    ///
    /// Accepts formats:
    /// - `https://github.com/owner/repo`
    /// - `https://github.com/owner/repo/tree/branch`
    /// - `https://github.com/owner/repo/commit/sha`
    /// - `github.com/owner/repo`
    pub fn parse(input: &str) -> Result<Self, DocError> {
        // Normalize the URL
        let normalized = if input.starts_with("http://") || input.starts_with("https://") {
            input.to_string()
        } else if input.starts_with("github.com") {
            format!("https://{}", input)
        } else {
            return Err(DocError::InvalidUrl("URL must be a GitHub URL".to_string()));
        };

        let url = Url::parse(&normalized)?;

        // Verify it's GitHub
        if url.host_str() != Some("github.com") {
            return Err(DocError::InvalidUrl("URL must be a GitHub URL".to_string()));
        }

        // Parse path segments
        let segments: Vec<&str> = url.path_segments().map(|s| s.collect()).unwrap_or_default();

        if segments.len() < 2 {
            return Err(DocError::InvalidUrl(
                "URL must include owner and repository name".to_string(),
            ));
        }

        let owner = segments[0].to_string();
        let repo = segments[1].trim_end_matches(".git").to_string();

        if owner.is_empty() || repo.is_empty() {
            return Err(DocError::InvalidUrl(
                "Owner and repository name cannot be empty".to_string(),
            ));
        }

        // Parse optional ref
        let git_ref = if segments.len() >= 4 {
            match segments[2] {
                "tree" | "commit" | "blob" => Some(segments[3..].join("/")),
                _ => None,
            }
        } else {
            None
        };

        Ok(Self {
            owner,
            repo,
            git_ref,
        })
    }

    /// Returns the URL for downloading the repository as a ZIP archive.
    pub fn archive_url(&self) -> String {
        let git_ref = self.git_ref.as_deref().unwrap_or("HEAD");
        format!(
            "https://github.com/{}/{}/archive/{}.zip",
            self.owner, self.repo, git_ref
        )
    }

    /// Returns the API URL for the repository.
    pub fn api_url(&self) -> String {
        format!("https://api.github.com/repos/{}/{}", self.owner, self.repo)
    }

    /// Returns the canonical GitHub URL.
    pub fn canonical_url(&self) -> String {
        format!("https://github.com/{}/{}", self.owner, self.repo)
    }
}

/// Metadata about a GitHub repository from the API.
#[derive(Debug, Clone, Deserialize)]
pub struct RepoMetadata {
    /// Repository size in kilobytes.
    pub size: u64,
    /// Default branch name (e.g., "main" or "master").
    pub default_branch: String,
    /// Whether the repository is private.
    pub private: bool,
    /// Full repository name (owner/repo).
    pub full_name: String,
}

/// Information about a resolved git ref.
#[derive(Debug, Clone, Deserialize)]
pub struct RefInfo {
    /// The commit SHA this ref points to.
    #[serde(rename = "sha")]
    pub commit_sha: String,
}

/// Rate limit information from GitHub API headers.
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    /// Maximum requests per hour.
    pub limit: u32,
    /// Remaining requests in the current window.
    pub remaining: u32,
    /// Unix timestamp when the rate limit resets.
    pub reset: u64,
}

impl RateLimitInfo {
    /// Parses rate limit info from response headers.
    fn from_headers(headers: &HeaderMap) -> Option<Self> {
        let limit = headers
            .get("x-ratelimit-limit")?
            .to_str()
            .ok()?
            .parse()
            .ok()?;
        let remaining = headers
            .get("x-ratelimit-remaining")?
            .to_str()
            .ok()?
            .parse()
            .ok()?;
        let reset = headers
            .get("x-ratelimit-reset")?
            .to_str()
            .ok()?
            .parse()
            .ok()?;
        Some(Self {
            limit,
            remaining,
            reset,
        })
    }
}

/// A client for interacting with the GitHub API.
pub struct GitHubClient {
    client: reqwest::Client,
}

impl Default for GitHubClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GitHubClient {
    /// Creates a new GitHub client.
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("doctown/0.1"));
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to build HTTP client");

        Self { client }
    }

    /// Checks if a repository exists (HEAD request).
    ///
    /// Returns `Ok(true)` if the repo exists, `Ok(false)` if not found,
    /// or an error for other failures (rate limited, network issues, etc.).
    pub async fn repo_exists(&self, url: &GitHubUrl) -> Result<bool, DocError> {
        let api_url = url.api_url();
        let response = self.client.head(&api_url).send().await?;

        self.check_rate_limit(&response)?;

        match response.status() {
            StatusCode::OK => Ok(true),
            StatusCode::NOT_FOUND => Ok(false),
            StatusCode::FORBIDDEN => {
                // Could be rate limited or private repo
                if let Some(rate_info) = RateLimitInfo::from_headers(response.headers()) {
                    if rate_info.remaining == 0 {
                        return Err(DocError::RateLimited(format!(
                            "GitHub API rate limit exceeded. Resets at {}",
                            rate_info.reset
                        )));
                    }
                }
                Err(DocError::Http(
                    "Access forbidden: repository may be private".to_string(),
                ))
            }
            status => Err(DocError::Http(format!(
                "Unexpected status checking repository: {}",
                status
            ))),
        }
    }

    /// Fetches repository metadata from the GitHub API.
    pub async fn fetch_metadata(&self, url: &GitHubUrl) -> Result<RepoMetadata, DocError> {
        let api_url = url.api_url();
        let response = self.client.get(&api_url).send().await?;

        self.check_rate_limit(&response)?;

        match response.status() {
            StatusCode::OK => {
                let metadata: RepoMetadata = response.json().await?;
                Ok(metadata)
            }
            StatusCode::NOT_FOUND => Err(DocError::NotFound(format!(
                "Repository not found: {}/{}",
                url.owner, url.repo
            ))),
            StatusCode::FORBIDDEN => Err(DocError::RateLimited(
                "GitHub API rate limit exceeded".to_string(),
            )),
            status => Err(DocError::Http(format!(
                "Failed to fetch repository metadata: {}",
                status
            ))),
        }
    }

    /// Resolves a branch/tag name to a commit SHA.
    ///
    /// If the ref is already a 40-character hex string (SHA), returns it as-is.
    pub async fn resolve_ref(&self, url: &GitHubUrl, git_ref: &str) -> Result<String, DocError> {
        // If it looks like a SHA already, return it
        if git_ref.len() == 40 && git_ref.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(git_ref.to_string());
        }

        // Try as a branch first
        let branch_url = format!(
            "https://api.github.com/repos/{}/{}/branches/{}",
            url.owner, url.repo, git_ref
        );

        let response = self.client.get(&branch_url).send().await?;
        self.check_rate_limit(&response)?;

        if response.status() == StatusCode::OK {
            #[derive(Deserialize)]
            struct BranchResponse {
                commit: RefInfo,
            }
            let branch: BranchResponse = response.json().await?;
            return Ok(branch.commit.commit_sha);
        }

        // Try as a tag
        let tag_url = format!(
            "https://api.github.com/repos/{}/{}/git/refs/tags/{}",
            url.owner, url.repo, git_ref
        );

        let response = self.client.get(&tag_url).send().await?;
        self.check_rate_limit(&response)?;

        if response.status() == StatusCode::OK {
            #[derive(Deserialize)]
            struct TagRef {
                object: RefInfo,
            }
            let tag: TagRef = response.json().await?;
            return Ok(tag.object.commit_sha);
        }

        Err(DocError::NotFound(format!(
            "Could not resolve ref '{}' to a commit",
            git_ref
        )))
    }

    /// Downloads a repository archive to the specified path.
    pub async fn download_repo(&self, url: &GitHubUrl, dest: &Path) -> Result<(), DocError> {
        let archive_url = url.archive_url();
        let response = self.client.get(&archive_url).send().await?;

        self.check_rate_limit(&response)?;

        if !response.status().is_success() {
            return Err(DocError::Http(format!(
                "Failed to download repository: {}",
                response.status()
            )));
        }

        let content = response.bytes().await?;

        let mut file = File::create(dest).await?;
        file.write_all(&content).await?;
        file.sync_all().await?;

        Ok(())
    }

    /// Downloads a repository archive with streaming (for large repos).
    ///
    /// Returns the total number of bytes downloaded.
    pub async fn download_repo_streaming(
        &self,
        url: &GitHubUrl,
        dest: &Path,
        max_size: u64,
    ) -> Result<u64, DocError> {
        use futures_util::StreamExt;
        use tokio::io::AsyncWriteExt;

        let archive_url = url.archive_url();
        let response = self.client.get(&archive_url).send().await?;

        self.check_rate_limit(&response)?;

        if !response.status().is_success() {
            return Err(DocError::Http(format!(
                "Failed to download repository: {}",
                response.status()
            )));
        }

        // Check content-length if available
        if let Some(content_length) = response.content_length() {
            if content_length > max_size {
                return Err(DocError::Validation(format!(
                    "Repository archive too large: {} bytes (max: {} bytes)",
                    content_length, max_size
                )));
            }
        }

        let mut file = File::create(dest).await?;
        let mut stream = response.bytes_stream();
        let mut total_bytes: u64 = 0;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            total_bytes += chunk.len() as u64;

            if total_bytes > max_size {
                // Clean up partial file
                drop(file);
                let _ = tokio::fs::remove_file(dest).await;
                return Err(DocError::Validation(format!(
                    "Repository archive exceeded size limit during download: {} bytes (max: {} bytes)",
                    total_bytes, max_size
                )));
            }

            file.write_all(&chunk).await?;
        }

        file.sync_all().await?;
        Ok(total_bytes)
    }

    /// Checks response headers for rate limiting and returns an error if exceeded.
    fn check_rate_limit(&self, response: &reqwest::Response) -> Result<(), DocError> {
        if let Some(rate_info) = RateLimitInfo::from_headers(response.headers()) {
            if rate_info.remaining == 0 && !response.status().is_success() {
                return Err(DocError::RateLimited(format!(
                    "GitHub API rate limit exceeded. Limit: {}, resets at Unix timestamp: {}",
                    rate_info.limit, rate_info.reset
                )));
            }
        }
        Ok(())
    }
}

impl fmt::Display for GitHubUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.canonical_url())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderMap;
    use tempfile::tempdir;

    // ==========================================================================
    // URL Parsing Tests
    // ==========================================================================

    #[test]
    fn test_parse_simple_url() {
        let url = GitHubUrl::parse("https://github.com/rust-lang/rust").unwrap();
        assert_eq!(url.owner, "rust-lang");
        assert_eq!(url.repo, "rust");
        assert_eq!(url.git_ref, None);
    }

    #[test]
    fn test_parse_url_with_branch() {
        let url = GitHubUrl::parse("https://github.com/rust-lang/rust/tree/master").unwrap();
        assert_eq!(url.owner, "rust-lang");
        assert_eq!(url.repo, "rust");
        assert_eq!(url.git_ref, Some("master".to_string()));
    }

    #[test]
    fn test_parse_url_with_nested_path() {
        let url =
            GitHubUrl::parse("https://github.com/owner/repo/tree/feature/nested/branch").unwrap();
        assert_eq!(url.git_ref, Some("feature/nested/branch".to_string()));
    }

    #[test]
    fn test_parse_url_without_https() {
        let url = GitHubUrl::parse("github.com/owner/repo").unwrap();
        assert_eq!(url.owner, "owner");
        assert_eq!(url.repo, "repo");
    }

    #[test]
    fn test_parse_url_with_git_suffix() {
        let url = GitHubUrl::parse("https://github.com/owner/repo.git").unwrap();
        assert_eq!(url.repo, "repo");
    }

    #[test]
    fn test_parse_url_with_commit() {
        let url = GitHubUrl::parse(
            "https://github.com/owner/repo/commit/abc123def456789012345678901234567890abcd",
        )
        .unwrap();
        assert_eq!(url.owner, "owner");
        assert_eq!(url.repo, "repo");
        assert_eq!(
            url.git_ref,
            Some("abc123def456789012345678901234567890abcd".to_string())
        );
    }

    #[test]
    fn test_parse_invalid_host() {
        let result = GitHubUrl::parse("https://gitlab.com/owner/repo");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_repo() {
        let result = GitHubUrl::parse("https://github.com/owner");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_owner() {
        let result = GitHubUrl::parse("https://github.com//repo");
        assert!(result.is_err());
    }

    #[test]
    fn test_archive_url() {
        let url = GitHubUrl::parse("https://github.com/owner/repo").unwrap();
        assert_eq!(
            url.archive_url(),
            "https://github.com/owner/repo/archive/HEAD.zip"
        );
    }

    #[test]
    fn test_archive_url_with_ref() {
        let url = GitHubUrl::parse("https://github.com/owner/repo/tree/main").unwrap();
        assert_eq!(
            url.archive_url(),
            "https://github.com/owner/repo/archive/main.zip"
        );
    }

    #[test]
    fn test_api_url() {
        let url = GitHubUrl::parse("https://github.com/owner/repo").unwrap();
        assert_eq!(url.api_url(), "https://api.github.com/repos/owner/repo");
    }

    #[test]
    fn test_canonical_url() {
        let url = GitHubUrl::parse("https://github.com/owner/repo/tree/main").unwrap();
        assert_eq!(url.canonical_url(), "https://github.com/owner/repo");
    }

    #[test]
    fn test_display_trait() {
        let url = GitHubUrl::parse("https://github.com/owner/repo").unwrap();
        assert_eq!(format!("{}", url), "https://github.com/owner/repo");
    }

    // ==========================================================================
    // Rate Limit Parsing Tests
    // ==========================================================================

    #[test]
    fn test_rate_limit_from_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("x-ratelimit-limit", "60".parse().unwrap());
        headers.insert("x-ratelimit-remaining", "58".parse().unwrap());
        headers.insert("x-ratelimit-reset", "1700000000".parse().unwrap());

        let info = RateLimitInfo::from_headers(&headers).unwrap();
        assert_eq!(info.limit, 60);
        assert_eq!(info.remaining, 58);
        assert_eq!(info.reset, 1700000000);
    }

    #[test]
    fn test_rate_limit_missing_headers() {
        let headers = HeaderMap::new();
        assert!(RateLimitInfo::from_headers(&headers).is_none());
    }

    #[test]
    fn test_rate_limit_partial_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("x-ratelimit-limit", "60".parse().unwrap());
        // Missing remaining and reset
        assert!(RateLimitInfo::from_headers(&headers).is_none());
    }

    // ==========================================================================
    // Integration Tests (require network, gated by env var)
    // ==========================================================================

    #[tokio::test]
    async fn test_download_repo() {
        // Use a small, stable repo for testing
        let url = GitHubUrl::parse("https://github.com/supabase/etl").unwrap();
        let dir = tempdir().unwrap();
        let dest = dir.path().join("repo.zip");
        let client = GitHubClient::new();
        let result = client.download_repo(&url, &dest).await;
        assert!(result.is_ok());
        assert!(dest.exists());
        dir.close().unwrap();
    }

    #[tokio::test]
    async fn test_download_repo_streaming() {
        let url = GitHubUrl::parse("https://github.com/supabase/etl").unwrap();
        let dir = tempdir().unwrap();
        let dest = dir.path().join("repo.zip");
        let client = GitHubClient::new();
        // Allow up to 50MB for this test
        let result = client
            .download_repo_streaming(&url, &dest, 50 * 1024 * 1024)
            .await;
        assert!(result.is_ok());
        let bytes_downloaded = result.unwrap();
        assert!(bytes_downloaded > 0);
        assert!(dest.exists());
        dir.close().unwrap();
    }

    #[tokio::test]
    async fn test_repo_exists_public() {
        let url = GitHubUrl::parse("https://github.com/rust-lang/rust").unwrap();
        let client = GitHubClient::new();
        let result = client.repo_exists(&url).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_repo_exists_nonexistent() {
        let url =
            GitHubUrl::parse("https://github.com/this-owner-does-not-exist-12345/no-such-repo")
                .unwrap();
        let client = GitHubClient::new();
        let result = client.repo_exists(&url).await;
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn test_fetch_metadata() {
        let url = GitHubUrl::parse("https://github.com/rust-lang/rust").unwrap();
        let client = GitHubClient::new();
        let result = client.fetch_metadata(&url).await;
        assert!(result.is_ok());
        let metadata = result.unwrap();
        assert_eq!(metadata.full_name, "rust-lang/rust");
        assert!(!metadata.default_branch.is_empty());
    }

    #[tokio::test]
    async fn test_fetch_metadata_not_found() {
        let url =
            GitHubUrl::parse("https://github.com/this-owner-does-not-exist-12345/no-such-repo")
                .unwrap();
        let client = GitHubClient::new();
        let result = client.fetch_metadata(&url).await;
        assert!(matches!(result, Err(doctown_common::DocError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_resolve_ref_branch() {
        let url = GitHubUrl::parse("https://github.com/rust-lang/rust").unwrap();
        let client = GitHubClient::new();
        let result = client.resolve_ref(&url, "master").await;
        assert!(result.is_ok());
        let sha = result.unwrap();
        // SHA should be 40 hex characters
        assert_eq!(sha.len(), 40);
        assert!(sha.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[tokio::test]
    async fn test_resolve_ref_already_sha() {
        let url = GitHubUrl::parse("https://github.com/rust-lang/rust").unwrap();
        let client = GitHubClient::new();
        let sha = "a1b2c3d4e5f6789012345678901234567890abcd";
        let result = client.resolve_ref(&url, sha).await;
        assert!(result.is_ok());
        // Should return the same SHA without API call
        assert_eq!(result.unwrap(), sha);
    }

    #[tokio::test]
    async fn test_resolve_ref_not_found() {
        let url = GitHubUrl::parse("https://github.com/rust-lang/rust").unwrap();
        let client = GitHubClient::new();
        let result = client
            .resolve_ref(&url, "this-branch-definitely-does-not-exist-12345")
            .await;
        assert!(matches!(result, Err(doctown_common::DocError::NotFound(_))));
    }
}
