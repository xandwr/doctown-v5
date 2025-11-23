use doctown_ingest::api;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    api::start_api_server("127.0.0.1", 8080).await
}