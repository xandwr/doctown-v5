//! Assembly Worker HTTP server binary.

use doctown_assembly::api::start_server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let host = std::env::var("ASSEMBLY_HOST")
        .or_else(|_| std::env::var("HOST"))
        .unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("ASSEMBLY_PORT")
        .or_else(|_| std::env::var("PORT"))
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8002);

    start_server(&host, port).await
}
