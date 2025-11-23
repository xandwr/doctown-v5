//! Ingest pipeline orchestration.

use crate::archive::{extract_zip, process_extracted_files};
use crate::github::{GitHubClient, GitHubUrl};
use doctown_common::DocError;
use tempfile::tempdir;

pub async fn run_pipeline(github_url: &GitHubUrl) -> Result<(), DocError> {
    let client = GitHubClient::new();
    let dir = tempdir()?;
    let zip_path = dir.path().join("repo.zip");

    // 1. Download the repository
    client.download_repo(github_url, &zip_path).await?;

    // 2. Unzip the repository
    let extract_dir = dir.path().join("extracted");
    extract_zip(&zip_path, &extract_dir)?;

    // 3. Process the extracted files
    process_extracted_files(&extract_dir);

    dir.close()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_pipeline() {
        let url = GitHubUrl::parse("https://github.com/supabase/etl").unwrap();
        let result = run_pipeline(&url).await;
        assert!(result.is_ok());
    }
}