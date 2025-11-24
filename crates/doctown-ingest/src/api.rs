use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use async_stream::stream;
use serde::{Deserialize, Serialize};
use tokio::signal;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::github::GitHubUrl;
use crate::pipeline::run_pipeline;
use doctown_common::JobId;
use doctown_events::Envelope;

/// Configuration for the API server
#[derive(Clone)]
pub struct ServerConfig {
    /// Host to bind to (e.g., "127.0.0.1" or "0.0.0.0")
    pub host: String,
    /// Port to bind to
    pub port: u16,
    /// CORS allowed origins (e.g., "http://localhost:5173")
    pub cors_origins: Vec<String>,
    /// Allow any origin (for production environments)
    pub allow_any_origin: bool,
    /// Maximum request body size in bytes
    pub max_body_size: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            cors_origins: vec!["http://localhost:5173".to_string()],
            allow_any_origin: false,
            max_body_size: 10 * 1024 * 1024, // 10 MB
        }
    }
}

/// Request body for the /ingest endpoint
#[derive(Debug, Deserialize, Serialize)]
pub struct IngestRequest {
    /// GitHub repository URL
    pub repo_url: String,
    /// Git reference (branch, tag, or commit)
    #[serde(default = "default_git_ref")]
    pub git_ref: String,
    /// Job ID for tracking
    pub job_id: String,
}

/// Query parameters for the GET /ingest endpoint
#[derive(Debug, Deserialize)]
pub struct IngestQuery {
    /// GitHub repository URL
    pub repo_url: String,
    /// Git reference (branch, tag, or commit)
    #[serde(default = "default_git_ref")]
    pub git_ref: String,
    /// Job ID for tracking
    pub job_id: String,
}

fn default_git_ref() -> String {
    "main".to_string()
}

/// Response body for validation errors
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

impl IngestRequest {
    /// Validate the request fields
    pub fn validate(&self) -> Result<(), String> {
        // Validate repo_url
        if self.repo_url.is_empty() {
            return Err("repo_url cannot be empty".to_string());
        }

        // Try to parse as GitHub URL
        if let Err(e) = GitHubUrl::parse(&self.repo_url) {
            return Err(format!("Invalid GitHub URL: {}", e));
        }

        // Validate job_id
        if self.job_id.is_empty() {
            return Err("job_id cannot be empty".to_string());
        }

        // Validate job_id format
        if let Err(e) = JobId::new(&self.job_id) {
            return Err(format!("Invalid job_id: {}", e));
        }

        Ok(())
    }
}

/// Core handler logic for ingest requests
async fn handle_ingest_request(
    repo_url: String,
    git_ref: String,
    job_id_str: String,
) -> HttpResponse {
    // Parse job_id and github_url
    let job_id = match JobId::new(&job_id_str) {
        Ok(id) => id,
        Err(e) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Invalid job_id: {}", e),
            });
        }
    };

    let github_url = match GitHubUrl::parse(&repo_url) {
        Ok(mut url) => {
            // Set git_ref if provided and not default
            if git_ref != "main" && !git_ref.is_empty() {
                url.git_ref = Some(git_ref.clone());
            }
            url
        }
        Err(e) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: format!("Invalid repo_url: {}", e),
            });
        }
    };

    // Create event channel with large buffer to prevent deadlock during parallel embedding
    let (tx, mut rx) = mpsc::channel::<Envelope<serde_json::Value>>(1000);
    let cancel_token = CancellationToken::new();
    let cancel_token_clone = cancel_token.clone();

    // Spawn pipeline task
    tokio::spawn(async move {
        if let Err(e) = run_pipeline(job_id, &github_url, tx, cancel_token_clone).await {
            eprintln!("Pipeline error: {}", e);
        }
    });

    // Create SSE stream
    let stream = stream! {
        // Send keepalive comment every 15s
        let mut keepalive = tokio::time::interval(tokio::time::Duration::from_secs(15));
        keepalive.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                _ = keepalive.tick() => {
                    yield Ok::<_, actix_web::Error>(web::Bytes::from(": keepalive\n\n"));
                }
                event = rx.recv() => {
                    match event {
                        Some(envelope) => {
                            match serde_json::to_string(&envelope) {
                                Ok(json) => {
                                    yield Ok(web::Bytes::from(format!("data: {}\n\n", json)));

                                    // Check if this is a terminal event
                                    if envelope.event_type.ends_with(".completed.v1") {
                                        // Give the client time to process the completion event before closing
                                        // Send a final newline to ensure the message is flushed
                                        yield Ok(web::Bytes::from("\n"));
                                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                                        break;
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to serialize event: {}", e);
                                }
                            }
                        }
                        None => {
                            // Channel closed
                            break;
                        }
                    }
                }
            }
        }
    };

    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("X-Accel-Buffering", "no"))
        .streaming(stream)
}

/// POST /ingest endpoint handler
///
/// Validates the request, runs the ingest pipeline, and streams events via SSE.
async fn ingest_post(req: web::Json<IngestRequest>) -> impl Responder {
    // Validate request
    if let Err(e) = req.validate() {
        return HttpResponse::BadRequest().json(ErrorResponse { error: e });
    }

    handle_ingest_request(
        req.repo_url.clone(),
        req.git_ref.clone(),
        req.job_id.clone(),
    )
    .await
}

/// GET /ingest endpoint handler
///
/// Accepts query parameters, runs the ingest pipeline, and streams events via SSE.
async fn ingest_get(query: web::Query<IngestQuery>) -> impl Responder {
    handle_ingest_request(
        query.repo_url.clone(),
        query.git_ref.clone(),
        query.job_id.clone(),
    )
    .await
}

/// Health check endpoint handler
///
/// GET /health returns {"status": "ok", "version": "..."}
async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Start the API server with graceful shutdown support
///
/// The server will listen for SIGINT (Ctrl-C) and SIGTERM signals
/// and shutdown gracefully when received.
pub async fn start_server(config: ServerConfig) -> std::io::Result<()> {
    let bind_addr = format!("{}:{}", config.host, config.port);

    println!("Starting Doctown Ingest API server on {}", bind_addr);
    println!("Configured CORS origins: {:?}", config.cors_origins);
    println!("Max request body size: {} bytes", config.max_body_size);

    let server = HttpServer::new(move || {
        // Build CORS middleware
        let cors = if config.allow_any_origin {
            // Allow any origin for production environments
            Cors::permissive()
        } else {
            // Specific origins for development
            let mut cors = Cors::default()
                .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                .allowed_headers(vec![
                    actix_web::http::header::CONTENT_TYPE,
                    actix_web::http::header::ACCEPT,
                ])
                .max_age(3600);

            // Add all configured origins
            for origin in &config.cors_origins {
                cors = cors.allowed_origin(origin);
            }

            cors
        };

        App::new()
            .wrap(cors)
            .app_data(web::PayloadConfig::new(config.max_body_size))
            .app_data(web::JsonConfig::default().limit(config.max_body_size))
            .route("/health", web::get().to(health))
            .route("/ingest", web::get().to(ingest_get))
            .route("/ingest", web::post().to(ingest_post))
    })
    .bind(&bind_addr)?
    .run();

    let server_handle = server.handle();
    let server_task = tokio::spawn(server);

    // Wait for shutdown signal
    tokio::select! {
        _ = signal::ctrl_c() => {
            println!("Received Ctrl-C signal, initiating graceful shutdown...");
        }
        _ = shutdown_signal() => {
            println!("Received termination signal, initiating graceful shutdown...");
        }
    }

    // Stop the server gracefully
    server_handle.stop(true).await;

    // Wait for the server to finish
    match server_task.await {
        Ok(Ok(())) => {
            println!("Server shutdown complete");
            Ok(())
        }
        Ok(Err(e)) => {
            eprintln!("Server error during shutdown: {}", e);
            Err(e)
        }
        Err(e) => {
            eprintln!("Server task panicked: {}", e);
            Err(std::io::Error::new(std::io::ErrorKind::Other, e))
        }
    }
}

/// Helper function to handle SIGTERM on Unix systems
#[cfg(unix)]
async fn shutdown_signal() {
    use tokio::signal::unix::{signal, SignalKind};

    let mut sigterm = signal(SignalKind::terminate()).expect("Failed to install SIGTERM handler");

    sigterm.recv().await;
}

/// Helper function for shutdown signal on non-Unix systems
#[cfg(not(unix))]
async fn shutdown_signal() {
    // On non-Unix systems, only Ctrl-C is supported
    std::future::pending::<()>().await;
}
