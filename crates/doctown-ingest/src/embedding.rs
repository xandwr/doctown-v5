//! Embedding client for calling the embedding worker.

use doctown_common::{ChunkId, DocError};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Request to embed a batch of chunks.
#[derive(Debug, Clone, Serialize)]
pub struct EmbedRequest {
    /// Unique identifier for this batch.
    pub batch_id: String,
    /// Chunks to embed.
    pub chunks: Vec<ChunkInput>,
}

/// A single chunk to embed.
#[derive(Debug, Clone, Serialize)]
pub struct ChunkInput {
    /// Unique identifier for the chunk.
    pub chunk_id: String,
    /// Text content to embed.
    pub content: String,
}

/// Response from embedding a batch.
#[derive(Debug, Clone, Deserialize)]
pub struct EmbedResponse {
    /// Unique identifier for this batch.
    pub batch_id: String,
    /// Embedded chunks with vectors.
    pub vectors: Vec<ChunkVector>,
}

/// A chunk with its embedding vector.
#[derive(Debug, Clone, Deserialize)]
pub struct ChunkVector {
    /// Unique identifier for the chunk.
    pub chunk_id: String,
    /// 384-dimensional embedding vector.
    pub vector: Vec<f32>,
}

/// Client for calling the embedding worker.
#[derive(Clone)]
pub struct EmbeddingClient {
    base_url: String,
    client: reqwest::Client,
}

impl EmbeddingClient {
    /// Create a new embedding client.
    ///
    /// # Arguments
    /// * `base_url` - Base URL of the embedding worker (e.g., "http://localhost:8000")
    pub fn new(base_url: impl Into<String>) -> Self {
        // Create client with longer timeout for large batches
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120)) // 2 minutes
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            base_url: base_url.into(),
            client,
        }
    }

    /// Check if the embedding worker is healthy.
    pub async fn health_check(&self) -> Result<bool, DocError> {
        let url = format!("{}/health", self.base_url);
        let response = self.client.get(&url).send().await.map_err(|e| {
            DocError::Internal(format!("Failed to connect to embedding worker: {}", e))
        })?;

        Ok(response.status().is_success())
    }

    /// Embed a batch of chunks.
    ///
    /// # Arguments
    /// * `batch_id` - Unique identifier for this batch
    /// * `chunks` - List of (chunk_id, content) pairs
    ///
    /// # Returns
    /// Vector of (chunk_id, vector) pairs and the duration in milliseconds
    pub async fn embed_batch(
        &self,
        batch_id: impl Into<String>,
        chunks: Vec<(ChunkId, String)>,
    ) -> Result<(Vec<(ChunkId, Vec<f32>)>, u64), DocError> {
        let started = Instant::now();
        let batch_id = batch_id.into();

        // Convert to request format
        let request = EmbedRequest {
            batch_id: batch_id.clone(),
            chunks: chunks
                .into_iter()
                .map(|(id, content)| ChunkInput {
                    chunk_id: id.to_string(),
                    content,
                })
                .collect(),
        };

        // Call embedding worker
        let url = format!("{}/embed", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| DocError::Internal(format!("Failed to call embedding worker: {}", e)))?;

        if !response.status().is_success() {
            return Err(DocError::Internal(format!(
                "Embedding worker returned error: {}",
                response.status()
            )));
        }

        let embed_response: EmbedResponse = response.json().await.map_err(|e| {
            DocError::Internal(format!("Failed to parse embedding response: {}", e))
        })?;

        let duration_ms = started.elapsed().as_millis() as u64;

        // Convert back to our format
        let results = embed_response
            .vectors
            .into_iter()
            .filter_map(|cv| {
                ChunkId::new(cv.chunk_id)
                    .ok()
                    .map(|chunk_id| (chunk_id, cv.vector))
            })
            .collect();

        Ok((results, duration_ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Only run when embedding worker is running
    async fn test_health_check() {
        let client = EmbeddingClient::new("http://localhost:8000");
        let healthy = client.health_check().await.unwrap();
        assert!(healthy);
    }

    #[tokio::test]
    #[ignore] // Only run when embedding worker is running
    async fn test_embed_batch() {
        let client = EmbeddingClient::new("http://localhost:8000");

        let chunks = vec![
            (
                ChunkId::generate(),
                "function hello() { return 'world'; }".to_string(),
            ),
            (
                ChunkId::generate(),
                "class Parser { parse() {} }".to_string(),
            ),
        ];

        let (results, duration_ms) = client
            .embed_batch("test_batch", chunks.clone())
            .await
            .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].1.len(), 384); // 384-dimensional vectors
        assert!(duration_ms > 0);
    }
}
