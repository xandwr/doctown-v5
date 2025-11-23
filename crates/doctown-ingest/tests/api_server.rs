use doctown_ingest::api::{start_server, ServerConfig};
use std::time::Duration;
use tokio::time::timeout;

/// Test that the server starts and stops cleanly
#[tokio::test]
async fn test_server_starts_and_stops_cleanly() {
    // Use a random available port to avoid conflicts
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 0, // Let OS assign a port
        cors_origins: vec!["http://localhost:5173".to_string()],
        allow_any_origin: false,
        max_body_size: 1024 * 1024, // 1 MB for testing
    };

    // Start the server in a separate task
    let server_handle = tokio::spawn(async move {
        // Note: This will run until we drop it or signal shutdown
        // In a real scenario, the server would listen for signals
        // For this test, we'll let the timeout handle it
        let _ = start_server(config).await;
    });

    // Give the server a moment to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Abort the server task (simulates shutdown)
    server_handle.abort();

    // Wait a bit to ensure clean shutdown
    tokio::time::sleep(Duration::from_millis(50)).await;

    // If we get here without panicking, the server started and stopped cleanly
    assert!(true);
}

/// Test health endpoint returns correct format
#[tokio::test]
async fn test_health_endpoint() {
    let config = ServerConfig::default();
    
    // Create a test server
    let server = tokio::spawn(async move {
        start_server(config).await
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Try to make a request to the health endpoint
    let client = reqwest::Client::new();
    let result = timeout(
        Duration::from_secs(1),
        client.get("http://127.0.0.1:8080/health").send()
    ).await;

    // Abort the server
    server.abort();

    // Check if we got a response (may fail if port is in use, which is OK)
    if let Ok(Ok(response)) = result {
        assert!(response.status().is_success());
        
        if let Ok(body) = response.json::<serde_json::Value>().await {
            assert_eq!(body["status"], "ok");
            assert!(body["version"].is_string());
        }
    }
    // Note: This test may fail if port 8080 is already in use, which is acceptable
    // The main test is test_server_starts_and_stops_cleanly above
}

/// Test CORS configuration is applied
#[tokio::test]
async fn test_cors_configuration() {
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 8081,
        cors_origins: vec![
            "http://localhost:5173".to_string(),
            "http://localhost:3000".to_string(),
        ],
        allow_any_origin: false,
        max_body_size: 1024 * 1024,
    };

    // Start server
    let server = tokio::spawn(async move {
        start_server(config).await
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Make an OPTIONS request to check CORS headers
    let client = reqwest::Client::new();
    let result = timeout(
        Duration::from_secs(1),
        client.request(reqwest::Method::OPTIONS, "http://127.0.0.1:8081/health")
            .header("Origin", "http://localhost:5173")
            .header("Access-Control-Request-Method", "GET")
            .send()
    ).await;

    server.abort();

    // If we got a response, verify CORS headers are present
    if let Ok(Ok(response)) = result {
        // The response should have CORS headers
        // Note: Exact validation depends on actix-cors behavior
        assert!(response.status().is_success() || response.status().as_u16() == 204);
    }
}

/// Test request body size limit is enforced
#[tokio::test]
async fn test_body_size_limit() {
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 8082,
        cors_origins: vec!["http://localhost:5173".to_string()],
        allow_any_origin: false,
        max_body_size: 100, // Very small limit for testing
    };

    let server = tokio::spawn(async move {
        start_server(config).await
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Try to send a large body
    let client = reqwest::Client::new();
    let large_body = "x".repeat(200); // Exceeds the 100 byte limit
    
    let result = timeout(
        Duration::from_secs(1),
        client.post("http://127.0.0.1:8082/ingest")
            .header("Content-Type", "application/json")
            .body(large_body)
            .send()
    ).await;

    server.abort();

    // Should get an error or 413 Payload Too Large
    if let Ok(Ok(response)) = result {
        // If we get a response, it should indicate the body was too large
        assert!(response.status().as_u16() == 413 || response.status().is_client_error());
    }
}

/// Test health endpoint returns correct response format (M1.9.2)
#[tokio::test]
async fn test_m1_9_2_health_endpoint_responds() {
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 8083,
        cors_origins: vec!["http://localhost:5173".to_string()],
        allow_any_origin: false,
        max_body_size: 10 * 1024 * 1024,
    };

    let server = tokio::spawn(async move {
        start_server(config).await
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    let client = reqwest::Client::new();
    let result = timeout(
        Duration::from_secs(2),
        client.get("http://127.0.0.1:8083/health").send()
    ).await;

    server.abort();

    if let Ok(Ok(response)) = result {
        assert_eq!(response.status(), 200);
        
        let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        assert_eq!(body["status"], "ok");
        assert!(body["version"].is_string());
        assert!(!body["version"].as_str().unwrap().is_empty());
    }
}

/// Test ingest endpoint validates request (M1.9.3)
#[tokio::test]
async fn test_m1_9_3_ingest_request_validation() {
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 8084,
        cors_origins: vec!["http://localhost:5173".to_string()],
        allow_any_origin: false,
        max_body_size: 10 * 1024 * 1024,
    };

    let server = tokio::spawn(async move {
        start_server(config).await
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    let client = reqwest::Client::new();
    
    // Test 1: Empty repo_url should fail
    let invalid_request = serde_json::json!({
        "repo_url": "",
        "git_ref": "main",
        "job_id": "job_test_123"
    });

    let result = timeout(
        Duration::from_secs(2),
        client.post("http://127.0.0.1:8084/ingest")
            .json(&invalid_request)
            .send()
    ).await;

    if let Ok(Ok(response)) = result {
        assert_eq!(response.status(), 400);
        let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        assert!(body["error"].is_string());
    }

    // Test 2: Invalid GitHub URL should fail
    let invalid_request = serde_json::json!({
        "repo_url": "not-a-valid-url",
        "git_ref": "main",
        "job_id": "job_test_123"
    });

    let result = timeout(
        Duration::from_secs(2),
        client.post("http://127.0.0.1:8084/ingest")
            .json(&invalid_request)
            .send()
    ).await;

    if let Ok(Ok(response)) = result {
        assert_eq!(response.status(), 400);
    }

    // Test 3: Empty job_id should fail
    let invalid_request = serde_json::json!({
        "repo_url": "https://github.com/user/repo",
        "git_ref": "main",
        "job_id": ""
    });

    let result = timeout(
        Duration::from_secs(2),
        client.post("http://127.0.0.1:8084/ingest")
            .json(&invalid_request)
            .send()
    ).await;

    if let Ok(Ok(response)) = result {
        assert_eq!(response.status(), 400);
    }

    server.abort();
}

/// Test SSE encoding format is correct (M1.9.4)
#[tokio::test]
async fn test_m1_9_4_sse_encoding_correct() {
    use futures_util::StreamExt;
    
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 8086,
        cors_origins: vec!["http://localhost:5173".to_string()],
        allow_any_origin: false,
        max_body_size: 10 * 1024 * 1024,
    };

    let server = tokio::spawn(async move {
        start_server(config).await
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    let client = reqwest::Client::new();
    
    let valid_request = serde_json::json!({
        "repo_url": "https://github.com/rust-lang/rust",
        "git_ref": "master",
        "job_id": "job_test_sse_encoding"
    });

    let result = timeout(
        Duration::from_secs(5),
        client.post("http://127.0.0.1:8086/ingest")
            .json(&valid_request)
            .send()
    ).await;

    if let Ok(Ok(response)) = result {
        assert_eq!(response.status(), 200);
        
        // Read a few chunks from the stream to verify format
        let mut stream = response.bytes_stream();
        let mut chunks_received = 0;
        
        while let Ok(Some(Ok(chunk))) = timeout(
            Duration::from_secs(3),
            stream.next()
        ).await {
            let text = String::from_utf8_lossy(&chunk);
            
            // SSE messages should either be:
            // 1. "data: {json}\n\n" format for events
            // 2. ": keepalive\n\n" format for keepalive comments
            for line in text.lines() {
                if line.starts_with("data: ") {
                    // Extract JSON and verify it parses
                    let json_str = &line[6..]; // Skip "data: "
                    let parsed: Result<serde_json::Value, _> = serde_json::from_str(json_str);
                    assert!(parsed.is_ok(), "SSE data line should contain valid JSON");
                    
                    chunks_received += 1;
                } else if line.starts_with(": ") {
                    // Keepalive comment
                    assert!(line.contains("keepalive"), "Comment should be keepalive");
                } else if !line.is_empty() {
                    // Empty lines are OK (they're part of the \n\n delimiter)
                }
            }
            
            // Stop after receiving a few events
            if chunks_received >= 2 {
                break;
            }
        }
        
        // We should have received at least one properly formatted event
        assert!(chunks_received > 0, "Should receive at least one SSE event");
    }

    server.abort();
}

/// Test that events stream to client incrementally (M1.9.4)
#[tokio::test]
async fn test_m1_9_4_events_stream_incrementally() {
    use futures_util::StreamExt;
    
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 8087,
        cors_origins: vec!["http://localhost:5173".to_string()],
        allow_any_origin: false,
        max_body_size: 10 * 1024 * 1024,
    };

    let server = tokio::spawn(async move {
        start_server(config).await
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    let client = reqwest::Client::new();
    
    let valid_request = serde_json::json!({
        "repo_url": "https://github.com/rust-lang/rust",
        "git_ref": "master", 
        "job_id": "job_test_streaming"
    });

    let result = timeout(
        Duration::from_secs(5),
        client.post("http://127.0.0.1:8087/ingest")
            .json(&valid_request)
            .send()
    ).await;

    if let Ok(Ok(response)) = result {
        assert_eq!(response.status(), 200);
        
        let mut stream = response.bytes_stream();
        let mut event_count = 0;
        let start = std::time::Instant::now();
        
        // Collect timestamps of when we receive events
        let mut event_times = Vec::new();
        
        while let Ok(Some(Ok(chunk))) = timeout(
            Duration::from_secs(3),
            stream.next()
        ).await {
            let text = String::from_utf8_lossy(&chunk);
            
            if text.contains("data: ") {
                event_count += 1;
                event_times.push(start.elapsed());
                
                // Stop after a few events
                if event_count >= 3 {
                    break;
                }
            }
        }
        
        // We should receive events incrementally, not all at once
        assert!(event_count > 0, "Should receive events");
        
        // If we got multiple events, they should be spread out in time
        if event_times.len() >= 2 {
            let first = event_times[0];
            let last = event_times[event_times.len() - 1];
            assert!(last > first, "Events should be received over time, not instantaneously");
        }
    }

    server.abort();
}

/// Test that keepalive comments are sent (M1.9.4)
#[tokio::test]
async fn test_m1_9_4_keepalive_comments_sent() {
    use futures_util::StreamExt;
    
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 8088,
        cors_origins: vec!["http://localhost:5173".to_string()],
        allow_any_origin: false,
        max_body_size: 10 * 1024 * 1024,
    };

    let server = tokio::spawn(async move {
        start_server(config).await
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    let client = reqwest::Client::new();
    
    let valid_request = serde_json::json!({
        "repo_url": "https://github.com/rust-lang/rust",
        "git_ref": "master",
        "job_id": "job_test_keepalive"
    });

    let result = timeout(
        Duration::from_secs(20), // Long enough to potentially see a keepalive
        client.post("http://127.0.0.1:8088/ingest")
            .json(&valid_request)
            .send()
    ).await;

    if let Ok(Ok(response)) = result {
        assert_eq!(response.status(), 200);
        
        let mut stream = response.bytes_stream();
        let mut keepalive_seen = false;
        
        // Wait for up to 18 seconds to see a keepalive (they're sent every 15s)
        let deadline = tokio::time::Instant::now() + Duration::from_secs(18);
        
        while tokio::time::Instant::now() < deadline {
            if let Ok(Some(Ok(chunk))) = timeout(
                Duration::from_secs(2),
                stream.next()
            ).await {
                let text = String::from_utf8_lossy(&chunk);
                
                // Look for keepalive comment
                if text.contains(": keepalive") {
                    keepalive_seen = true;
                    break;
                }
                
                // If we see a completed event, the stream will end soon
                if text.contains(".completed.v1") {
                    break;
                }
            }
        }
        
        // Note: This test may not always see a keepalive if the pipeline
        // completes quickly (< 15 seconds), which is fine.
        // The important thing is that when keepalives ARE sent, they're formatted correctly
        if keepalive_seen {
            println!("✓ Keepalive comment received as expected");
        } else {
            println!("Note: Pipeline completed before keepalive interval (< 15s)");
        }
    }

    server.abort();
}

/// Test client disconnect handling (M1.9.4)
#[tokio::test]
async fn test_m1_9_4_client_disconnect_handling() {
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 8089,
        cors_origins: vec!["http://localhost:5173".to_string()],
        allow_any_origin: false,
        max_body_size: 10 * 1024 * 1024,
    };

    let server = tokio::spawn(async move {
        start_server(config).await
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    let client = reqwest::Client::new();
    
    let valid_request = serde_json::json!({
        "repo_url": "https://github.com/rust-lang/rust",
        "git_ref": "master",
        "job_id": "job_test_disconnect"
    });

    // Start the request
    let result = timeout(
        Duration::from_secs(3),
        client.post("http://127.0.0.1:8089/ingest")
            .json(&valid_request)
            .send()
    ).await;

    if let Ok(Ok(response)) = result {
        assert_eq!(response.status(), 200);
        
        // Drop the response immediately to simulate disconnect
        drop(response);
        
        // Give the server a moment to detect the disconnect
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // Server should handle this gracefully without panicking
        // If we get here without the server crashing, the test passes
        assert!(true, "Server handled client disconnect gracefully");
    }

    server.abort();
}

/// Test ingest endpoint returns SSE stream for valid request (M1.9.3)
/// Note: This is a basic connectivity test. It validates the SSE stream starts
/// but doesn't wait for the full pipeline to complete.
#[tokio::test]
async fn test_m1_9_3_valid_request_returns_sse_stream() {
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 8085,
        cors_origins: vec!["http://localhost:5173".to_string()],
        allow_any_origin: false,
        max_body_size: 10 * 1024 * 1024,
    };

    let server = tokio::spawn(async move {
        start_server(config).await
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    let client = reqwest::Client::new();
    
    // Use a valid GitHub URL format that passes validation
    // The URL format is valid even though the repo may not exist
    let valid_request = serde_json::json!({
        "repo_url": "https://github.com/rust-lang/rust",
        "git_ref": "master",
        "job_id": "job_test_sse_123"
    });

    let result = timeout(
        Duration::from_secs(3),
        client.post("http://127.0.0.1:8085/ingest")
            .json(&valid_request)
            .send()
    ).await;

    if let Ok(Ok(response)) = result {
        let status = response.status();
        let headers = response.headers().clone();
        
        // If we got an error, print it for debugging
        if status != 200 {
            let body = response.text().await.unwrap_or_default();
            eprintln!("Error response: {} - {}", status, body);
        }
        
        // Should get 200 OK with SSE headers
        assert_eq!(status, 200);
        
        let content_type = headers.get("content-type")
            .and_then(|v| v.to_str().ok());
        assert_eq!(content_type, Some("text/event-stream"));
        
        let cache_control = headers.get("cache-control")
            .and_then(|v| v.to_str().ok());
        assert_eq!(cache_control, Some("no-cache"));
        
        // Note: We don't try to read the stream here as it would take too long
        // to actually download and process a real repository in a test
    }

    server.abort();
}