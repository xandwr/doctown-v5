use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::io::Write;
use tokio::sync::broadcast;
use url::Url;
use futures_util::StreamExt;

struct AppState {
    sender: broadcast::Sender<String>,
}

#[derive(Deserialize)]
struct GenerateRequest {
    repo_url: String,
}

async fn events(data: web::Data<AppState>) -> impl Responder {
    let mut receiver = data.sender.subscribe();

    let stream = async_stream::stream! {
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let (sender, _) = broadcast::channel(100);
    let app_state = web::Data::new(AppState {
        sender: sender.clone(),
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173") // Adjust this to your Svelte app's URL
            .allowed_methods(vec!["GET", "POST"])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .route("/generate", web::post().to(generate))
            .route("/events", web::get().to(events))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
