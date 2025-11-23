use doctown_ingest::api::{start_server, ServerConfig};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Bind to 0.0.0.0 in production, 127.0.0.1 for local dev
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    // Allow CORS from Vercel and local dev
    let mut cors_origins = vec![
        "http://localhost:5173".to_string(),
        "http://127.0.0.1:5173".to_string(),
    ];

    // Add Vercel domain if set
    if let Ok(vercel_url) = env::var("VERCEL_URL") {
        cors_origins.push(format!("https://{}", vercel_url));
    }

    // In production (RunPod), allow all origins
    // Note: We'll handle this differently in the CORS middleware
    let allow_all = env::var("PRODUCTION").is_ok();

    let config = ServerConfig {
        host,
        port: 3000,
        cors_origins,
        allow_any_origin: allow_all,
        max_body_size: 10 * 1024 * 1024, // 10 MB
    };

    start_server(config).await
}
