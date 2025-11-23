//! GitHub URL parsing and validation.

use doctown_common::DocError;
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

/// A client for interacting with the GitHub API.
pub struct GitHubClient {
    client: reqwest::Client,
}

impl GitHubClient {
    /// Creates a new GitHub client.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Downloads a repository archive to the specified path.
    pub async fn download_repo(&self, url: &GitHubUrl, dest: &Path) -> Result<(), DocError> {
        let archive_url = url.archive_url();
        let response = self.client.get(&archive_url).send().await?;
        let content = response.bytes().await?;

        let mut file = File::create(dest).await?;
        file.write_all(&content).await?;

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
    use tempfile::tempdir;

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

    #[tokio::test]
    async fn test_download_repo() {
        let url = GitHubUrl::parse("https://github.com/gemini-testing/lib-hello-gemini-rs").unwrap();
        let dir = tempdir().unwrap();
        let dest = dir.path().join("repo.zip");
        let client = GitHubClient::new();
        let result = client.download_repo(&url, &dest).await;
        assert!(result.is_ok());
        assert!(dest.exists());
        dir.close().unwrap();
    }
}
