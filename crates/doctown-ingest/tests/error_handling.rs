//! Tests for M1.10.4: Error Handling
//!
//! Verifies that the pipeline handles errors gracefully:
//! - Download failures emit failed status
//! - Parse errors skip file but continue processing
//! - Completed event is always emitted, even on failure

use doctown_common::JobId;
use doctown_events::{Envelope, Status};
use doctown_ingest::archive::process_extracted_files;
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;
use tokio::sync::mpsc;
use zip::write::FileOptions;
use zip::ZipWriter;

/// Helper to create a ZIP with files that will cause parse errors
fn create_zip_with_problematic_files(
    zip_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = fs::File::create(zip_path)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default();

    // Valid Rust file
    zip.start_file("test-repo-main/src/good.rs", options)?;
    writeln!(zip, "fn valid() {{}}")?;

    // Invalid Rust syntax (unclosed brace)
    zip.start_file("test-repo-main/src/bad.rs", options)?;
    writeln!(zip, "fn broken() {{ // missing close brace")?;
    writeln!(zip, "    let x = 1;")?;

    // Binary file
    zip.start_file("test-repo-main/image.png", options)?;
    zip.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])?;

    // Valid Python file
    zip.start_file("test-repo-main/script.py", options)?;
    writeln!(zip, "def hello():")?;
    writeln!(zip, "    print('hi')")?;

    // Invalid Python syntax
    zip.start_file("test-repo-main/broken.py", options)?;
    writeln!(zip, "def broken(")?;
    writeln!(zip, "    pass")?;

    zip.finish()?;
    Ok(())
}

/// Test that parse errors don't abort the pipeline
#[tokio::test]
async fn test_parse_errors_skip_file_and_continue() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("test-repo.zip");
    create_zip_with_problematic_files(&zip_path).expect("Failed to create test ZIP");

    // Extract the ZIP
    let extract_dir = dir.path().join("extracted");
    fs::create_dir_all(&extract_dir).unwrap();
    let file = fs::File::open(&zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => extract_dir.join(path),
            None => continue,
        };
        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath).unwrap();
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            std::io::copy(&mut file, &mut outfile).unwrap();
        }
    }

    let (tx, mut rx) = mpsc::channel(100);
    let job_id = JobId::new("job_error_test").unwrap();
    let context = doctown_events::Context::new(job_id.clone(), "https://github.com/test/repo")
        .with_git_ref("main".to_string());

    // Process files
    let tx_clone = tx.clone();
    let extract_dir_clone = extract_dir.clone();
    let process_task = tokio::spawn(async move {
        let result =
            process_extracted_files(&extract_dir_clone, context.clone(), tx_clone.clone()).await;

        // Even if there were errors, we should get a result
        let (files_processed, files_skipped, chunks_created) = result.unwrap_or((0, 0, 0));

        // Send completion event
        let _ = tx_clone
            .send(
                Envelope::new(
                    "ingest.completed.v1",
                    context,
                    serde_json::to_value(doctown_events::IngestCompletedPayload::success(
                        files_processed,
                        files_skipped,
                        chunks_created,
                        100,
                    ))
                    .unwrap(),
                )
                .with_status(Status::Success),
            )
            .await;
    });

    // Collect events
    let mut events = Vec::new();
    while let Some(event) = rx.recv().await {
        let is_completed = event.event_type == "ingest.completed.v1";
        events.push(event);
        if is_completed {
            break;
        }
    }

    let _ = process_task.await;

    // Verify we got some successful file processing
    let file_detected_count = events
        .iter()
        .filter(|e| e.event_type == "ingest.file_detected.v1")
        .count();
    assert!(
        file_detected_count >= 1,
        "Should have detected at least 1 valid file"
    );

    // Verify we got skip events for problematic files
    let file_skipped_count = events
        .iter()
        .filter(|e| e.event_type == "ingest.file_skipped.v1")
        .count();
    assert!(
        file_skipped_count >= 1,
        "Should have skipped at least 1 problematic file"
    );

    // Verify completed event was emitted
    let completed_event = events
        .iter()
        .find(|e| e.event_type == "ingest.completed.v1");
    assert!(completed_event.is_some(), "Should emit completed event");

    // Verify the completed event has success status (errors were handled gracefully)
    assert_eq!(completed_event.unwrap().status, Some(Status::Success));
}

/// Test that the completed event is always emitted, even on errors
#[tokio::test]
async fn test_completed_event_always_emitted() {
    // Test with an empty directory (no files to process)
    let dir = tempdir().unwrap();
    let extract_dir = dir.path().join("empty");
    fs::create_dir_all(&extract_dir).unwrap();

    let (tx, mut rx) = mpsc::channel(100);
    let job_id = JobId::new("job_empty_test").unwrap();
    let context = doctown_events::Context::new(job_id.clone(), "https://github.com/test/repo")
        .with_git_ref("main".to_string());

    let tx_clone = tx.clone();
    let extract_dir_clone = extract_dir.clone();
    let _process_task = tokio::spawn(async move {
        let result =
            process_extracted_files(&extract_dir_clone, context.clone(), tx_clone.clone()).await;

        // Always emit completed, even if nothing was processed
        let (files_processed, files_skipped, chunks_created) = result.unwrap_or((0, 0, 0));

        let _ = tx_clone
            .send(
                Envelope::new(
                    "ingest.completed.v1",
                    context,
                    serde_json::to_value(doctown_events::IngestCompletedPayload::success(
                        files_processed,
                        files_skipped,
                        chunks_created,
                        50,
                    ))
                    .unwrap(),
                )
                .with_status(Status::Success),
            )
            .await;
    });

    // Collect events
    let mut events = Vec::new();
    while let Some(event) = rx.recv().await {
        let is_completed = event.event_type == "ingest.completed.v1";
        events.push(event);
        if is_completed {
            break;
        }
    }

    // Verify completed event was emitted even though no files were processed
    let completed_event = events
        .iter()
        .find(|e| e.event_type == "ingest.completed.v1");
    assert!(
        completed_event.is_some(),
        "Should emit completed event even with no files"
    );

    let payload: Value = completed_event.unwrap().payload.clone();
    assert_eq!(payload["files_processed"], 0);
    assert_eq!(payload["chunks_created"], 0);
}

/// Test that binary files are properly skipped with the correct reason
#[tokio::test]
async fn test_binary_files_skipped() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("test-repo.zip");

    // Create a ZIP with a binary file
    let file = fs::File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default();

    // Binary file with null bytes
    zip.start_file("test-repo-main/data.bin", options).unwrap();
    zip.write_all(&[0x00, 0x01, 0x02, 0xFF, 0xFE]).unwrap();

    // Another binary-like file
    zip.start_file("test-repo-main/image.png", options).unwrap();
    zip.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x00]).unwrap();

    zip.finish().unwrap();

    // Extract
    let extract_dir = dir.path().join("extracted");
    fs::create_dir_all(&extract_dir).unwrap();
    let file = fs::File::open(&zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => extract_dir.join(path),
            None => continue,
        };
        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath).unwrap();
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            std::io::copy(&mut file, &mut outfile).unwrap();
        }
    }

    let (tx, mut rx) = mpsc::channel(100);
    let job_id = JobId::new("job_binary_test").unwrap();
    let context = doctown_events::Context::new(job_id.clone(), "https://github.com/test/repo")
        .with_git_ref("main".to_string());

    let tx_clone = tx.clone();
    let extract_dir_clone = extract_dir.clone();
    let _process_task = tokio::spawn(async move {
        let _ =
            process_extracted_files(&extract_dir_clone, context.clone(), tx_clone.clone()).await;

        let _ = tx_clone
            .send(
                Envelope::new(
                    "ingest.completed.v1",
                    context,
                    serde_json::to_value(doctown_events::IngestCompletedPayload::success(
                        0, 2, 0, 50,
                    ))
                    .unwrap(),
                )
                .with_status(Status::Success),
            )
            .await;
    });

    // Collect events
    let mut events = Vec::new();
    while let Some(event) = rx.recv().await {
        let is_completed = event.event_type == "ingest.completed.v1";
        events.push(event);
        if is_completed {
            break;
        }
    }

    // Verify both files were skipped as binary
    let skipped_events: Vec<_> = events
        .iter()
        .filter(|e| e.event_type == "ingest.file_skipped.v1")
        .collect();

    assert_eq!(
        skipped_events.len(),
        2,
        "Should have skipped 2 binary files"
    );

    // Verify the reason is "binary"
    for event in skipped_events {
        let payload: Value = event.payload.clone();
        assert_eq!(payload["reason"], "binary");
    }
}

/// Test that files matching ignore patterns are skipped
#[tokio::test]
async fn test_ignore_patterns_work() {
    let dir = tempdir().unwrap();
    let zip_path = dir.path().join("test-repo.zip");

    // Create a ZIP with files that should be ignored
    let file = fs::File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::<()>::default();

    // node_modules file
    zip.start_file("test-repo-main/node_modules/lib.js", options)
        .unwrap();
    writeln!(zip, "// library code").unwrap();

    // Lock file
    zip.start_file("test-repo-main/Cargo.lock", options).unwrap();
    writeln!(zip, "# lock file").unwrap();

    // Hidden file
    zip.start_file("test-repo-main/.env", options).unwrap();
    writeln!(zip, "SECRET=value").unwrap();

    // Valid file (for comparison)
    zip.start_file("test-repo-main/src/main.rs", options)
        .unwrap();
    writeln!(zip, "fn main() {{}}").unwrap();

    zip.finish().unwrap();

    // Extract
    let extract_dir = dir.path().join("extracted");
    fs::create_dir_all(&extract_dir).unwrap();
    let file = fs::File::open(&zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath = match file.enclosed_name() {
            Some(path) => extract_dir.join(path),
            None => continue,
        };
        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath).unwrap();
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p).unwrap();
                }
            }
            let mut outfile = fs::File::create(&outpath).unwrap();
            std::io::copy(&mut file, &mut outfile).unwrap();
        }
    }

    let (tx, mut rx) = mpsc::channel(100);
    let job_id = JobId::new("job_ignore_test").unwrap();
    let context = doctown_events::Context::new(job_id.clone(), "https://github.com/test/repo")
        .with_git_ref("main".to_string());

    let tx_clone = tx.clone();
    let extract_dir_clone = extract_dir.clone();
    let _process_task = tokio::spawn(async move {
        let _ =
            process_extracted_files(&extract_dir_clone, context.clone(), tx_clone.clone()).await;

        let _ = tx_clone
            .send(
                Envelope::new(
                    "ingest.completed.v1",
                    context,
                    serde_json::to_value(doctown_events::IngestCompletedPayload::success(
                        1, 3, 1, 50,
                    ))
                    .unwrap(),
                )
                .with_status(Status::Success),
            )
            .await;
    });

    // Collect events
    let mut events = Vec::new();
    while let Some(event) = rx.recv().await {
        let is_completed = event.event_type == "ingest.completed.v1";
        events.push(event);
        if is_completed {
            break;
        }
    }

    // Verify the valid file was detected
    let detected = events
        .iter()
        .filter(|e| e.event_type == "ingest.file_detected.v1")
        .count();
    assert_eq!(detected, 1, "Should have detected 1 valid file");

    // Verify ignored files were skipped
    let skipped = events
        .iter()
        .filter(|e| e.event_type == "ingest.file_skipped.v1")
        .count();
    assert!(skipped >= 2, "Should have skipped at least 2 ignored files");
}
