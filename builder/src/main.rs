use doctown_ingest::api::{start_server, ServerConfig};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Configuration for local development
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 3000,
        cors_origins: vec![
            "http://localhost:5173".to_string(),
            "http://127.0.0.1:5173".to_string(),
        ],
        max_body_size: 10 * 1024 * 1024, // 10 MB
    };

    start_server(config).await
}