//! Ingest pipeline orchestration.

use crate::archive::{extract_zip, process_extracted_files};
use crate::embedding::EmbeddingClient;
use crate::github::{GitHubClient, GitHubUrl};
use doctown_common::{DocError, JobId};
use doctown_events::{Context, Envelope, IngestCompletedPayload, IngestStartedPayload, Status};
use serde_json;
use std::env;
use tempfile::tempdir;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

/// Type alias for the event sender.
pub type EventSender = mpsc::Sender<Envelope<serde_json::Value>>;

pub async fn run_pipeline(
    job_id: JobId,
    github_url: &GitHubUrl,
    sender: EventSender,
    cancel: CancellationToken,
) -> Result<(), DocError> {
    let started_at = std::time::Instant::now();
    let client = GitHubClient::new();
    let dir = tempdir()?;
    let zip_path = dir.path().join("repo.zip");

    let context = Context::new(job_id.clone(), github_url.canonical_url()).with_git_ref(
        github_url
            .git_ref
            .clone()
            .unwrap_or_else(|| "HEAD".to_string()),
    );

    // Emit IngestStarted event
    sender
        .send(Envelope::new(
            "ingest.started.v1",
            context.clone(),
            serde_json::to_value(IngestStartedPayload::new(
                github_url.canonical_url(),
                github_url
                    .git_ref
                    .clone()
                    .unwrap_or_else(|| "HEAD".to_string()),
            ))?,
        ))
        .await
        .map_err(|e| DocError::Internal(format!("Failed to send event: {}", e)))?;

    let result = tokio::select! {
        _ = cancel.cancelled() => {
            Err(DocError::Internal("Ingest cancelled".to_string()))
        }
        res = async {
            // 1. Download the repository
            client.download_repo(github_url, &zip_path).await?;

            // 2. Unzip the repository
            let extract_dir = dir.path().join("extracted");
            extract_zip(&zip_path, &extract_dir)?;

            // 3. Process the extracted files
            let (files_processed, files_skipped, chunks_created, collected_chunks) =
                process_extracted_files(&extract_dir, context.clone(), sender.clone()).await?;

            // 4. Embed the chunks in batches (sequentially to avoid model contention)
            let chunks_embedded = if !collected_chunks.is_empty() {
                let embedding_url = env::var("EMBEDDING_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
                let embedding_client = EmbeddingClient::new(embedding_url);

                // Larger batch size for better throughput (1024 chunks per batch)
                const BATCH_SIZE: usize = 1024;
                let mut total_embedded = 0;

                // Process batches sequentially to avoid ONNX model contention
                for (batch_num, chunk_batch) in collected_chunks.chunks(BATCH_SIZE).enumerate() {
                    let batch_id = format!("job_{}_batch_{}", context.job_id, batch_num);
                    
                    match embedding_client.embed_batch(batch_id, chunk_batch.to_vec()).await {
                        Ok((vectors, duration_ms)) => {
                            total_embedded += vectors.len();
                            let chunks_per_sec = if duration_ms > 0 {
                                (vectors.len() as f64 / (duration_ms as f64 / 1000.0)) as usize
                            } else {
                                0
                            };
                            eprintln!("Embedded batch {}: {} chunks in {}ms (~{} chunks/sec)",
                                batch_num + 1, vectors.len(), duration_ms, chunks_per_sec);
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to embed batch {}: {}", batch_num + 1, e);
                        }
                    }
                }

                total_embedded
            } else {
                0
            };

            dir.close()?;
            Ok((files_processed, files_skipped, chunks_created, chunks_embedded))
        } => res,
    };

    let duration_ms = started_at.elapsed().as_millis() as u64;

    match result {
        Ok((files_processed, files_skipped, chunks_created, chunks_embedded)) => {
            let payload = IngestCompletedPayload::success(
                files_processed,
                files_skipped,
                chunks_created,
                duration_ms,
            );

            let payload = if chunks_embedded > 0 {
                payload.with_embeddings(chunks_embedded)
            } else {
                payload
            };

            sender
                .send(
                    Envelope::new(
                        "ingest.completed.v1",
                        context,
                        serde_json::to_value(payload)?,
                    )
                    .with_status(Status::Success),
                )
                .await
                .map_err(|e| DocError::Internal(format!("Failed to send event: {}", e)))?;
            Ok(())
        }
        Err(e) => {
            sender
                .send(
                    Envelope::new(
                        "ingest.completed.v1",
                        context,
                        serde_json::to_value(IngestCompletedPayload::failed(
                            e.to_string(),
                            duration_ms,
                        ))?,
                    )
                    .with_status(Status::Failed),
                )
                .await
                .map_err(|send_err| {
                    DocError::Internal(format!("Failed to send event: {}", send_err))
                })?;
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doctown_common::JobId;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_run_pipeline() {
        let (sender, mut receiver) = mpsc::channel(100);
        let job_id = JobId::generate();
        let cancel_token = CancellationToken::new();
        let url = GitHubUrl::parse("https://github.com/xandwr/localdoc").unwrap();
        let result = run_pipeline(job_id.clone(), &url, sender, cancel_token).await;
        assert!(result.is_ok());

        // Verify events - close the receiver to drain remaining messages
        receiver.close();
        let mut events = Vec::new();
        while let Some(event) = receiver.recv().await {
            events.push(event);
        }

        assert!(!events.is_empty());
        let started_event = events
            .iter()
            .find(|e| e.event_type == "ingest.started.v1")
            .unwrap();
        let completed_event = events
            .iter()
            .find(|e| e.event_type == "ingest.completed.v1")
            .unwrap();

        assert_eq!(started_event.context.job_id, job_id);
        assert_eq!(completed_event.context.job_id, job_id);
        assert_eq!(completed_event.status, Some(Status::Success));
    }

    #[tokio::test]
    async fn test_run_pipeline_cancellation() {
        let (sender, _receiver) = mpsc::channel(100);
        let job_id = JobId::generate();
        let cancel_token = CancellationToken::new();
        let url = GitHubUrl::parse("https://github.com/xandwr/localdoc").unwrap();

        // Cancel immediately
        cancel_token.cancel();

        let result = run_pipeline(job_id.clone(), &url, sender, cancel_token).await;
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Internal error: Ingest cancelled"
        );
    }
}
