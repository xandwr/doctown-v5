use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::io::Write;
use tokio::sync::broadcast;
use url::Url;
use futures_util::StreamExt;
use async_stream::stream;
use tokio::signal;

/// Configuration for the API server
#[derive(Clone)]
pub struct ServerConfig {
    /// Host to bind to (e.g., "127.0.0.1" or "0.0.0.0")
    pub host: String,
    /// Port to bind to
    pub port: u16,
    /// CORS allowed origins (e.g., "http://localhost:5173")
    pub cors_origins: Vec<String>,
    /// Maximum request body size in bytes
    pub max_body_size: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            cors_origins: vec!["http://localhost:5173".to_string()],
            max_body_size: 10 * 1024 * 1024, // 10 MB
        }
    }
}

struct AppState {
    sender: broadcast::Sender<String>,
}

#[derive(Deserialize)]
struct GenerateRequest {
    repo_url: String,
}

async fn events(data: web::Data<AppState>) -> impl Responder {
    let mut receiver = data.sender.subscribe();

    let stream = stream! {
        while let Ok(msg) = receiver.recv().await {
            yield Ok(web::Bytes::from(format!("data: {}\n\n", msg))) as Result<web::Bytes, actix_web::Error>;
        }
    };

    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .streaming(stream)
}

async fn generate(
    req: web::Json<GenerateRequest>,
    data: web::Data<AppState>,
) -> impl Responder {
    let repo_url = req.repo_url.clone();
    let sender = data.sender.clone();

    tokio::spawn(async move {
        let _ = sender.send("Starting download...".to_string());

        let url = match Url::parse(&repo_url) {
            Ok(url) => url,
            Err(_) => {
                let _ = sender.send("Error: Invalid URL format".to_string());
                return;
            }
        };

        let path_segments: Vec<&str> = url.path_segments().map_or(vec![], |s| s.collect());
        if path_segments.len() < 2 {
            let _ = sender.send("Error: Invalid GitHub repository URL".to_string());
            return;
        }

        let owner = &path_segments[0];
        let repo = &path_segments[1];
        let download_url =
            format!("https://github.com/{}/{}/archive/refs/heads/main.zip", owner, repo);
        let zip_file_name = format!("{}.zip", repo);

        let _ = sender.send(format!("Downloading from: {}", download_url));

        let response = match reqwest::get(&download_url).await {
            Ok(res) => res,
            Err(_) => {
                let _ = sender.send("Error: Failed to start download".to_string());
                return;
            }
        };

        if !response.status().is_success() {
            let _ = sender.send(format!(
                "Error: GitHub returned non-success status: {}",
                response.status()
            ));
            return;
        }

        let mut file = match std::fs::File::create(&zip_file_name) {
            Ok(f) => f,
            Err(_) => {
                let _ = sender.send("Error: Failed to create zip file".to_string());
                return;
            }
        };

        let mut stream = response.bytes_stream();
        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    if let Err(_) = file.write_all(&chunk) {
                        let _ = sender.send("Error: Failed to write to zip file".to_string());
                        return;
                    }
                }
                Err(_) => {
                    let _ = sender.send("Error: Failed to download chunk".to_string());
                    return;
                }
            }
        }
        
        let _ = sender.send(format!("Successfully downloaded and saved {}", zip_file_name));

        let _ = sender.send(format!("success: {}", zip_file_name));
    });

    HttpResponse::Ok().body("Request received, processing started.")
}

/// Health check endpoint handler
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
        let (sender, _) = broadcast::channel(100);
        let app_state = web::Data::new(AppState {
            sender: sender.clone(),
        });

        // Build CORS middleware
        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .supports_credentials()
            .max_age(3600);

        // Add all configured origins
        for origin in &config.cors_origins {
            cors = cors.allowed_origin(origin);
        }

        App::new()
            .wrap(cors)
            .app_data(web::PayloadConfig::new(config.max_body_size))
            .app_data(app_state.clone())
            .route("/health", web::get().to(health))
            .route("/generate", web::post().to(generate))
            .route("/events", web::get().to(events))
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
    
    let mut sigterm = signal(SignalKind::terminate())
        .expect("Failed to install SIGTERM handler");
    
    sigterm.recv().await;
}

/// Helper function for shutdown signal on non-Unix systems
#[cfg(not(unix))]
async fn shutdown_signal() {
    // On non-Unix systems, only Ctrl-C is supported
    std::future::pending::<()>().await;
}
