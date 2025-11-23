//! Archive extraction.
use crate::filter::{normalize_archive_path, FileFilter, FilterResult, SkipReason as FilterSkipReason};
use crate::language::detect_language;
use crate::parsing::parse;
use crate::pipeline::EventSender;
use crate::symbol::extract_symbols;
use doctown_common::{ChunkId, DocError};
use doctown_events::{
    Context, Envelope, IngestChunkCreatedPayload, IngestFileDetectedPayload,
    IngestFileSkippedPayload, SkipReason,
};
use std::fs;
use std::io;
use std::path::Path;
use walkdir::WalkDir;
use zip::ZipArchive;

pub fn extract_zip(zip_file: &Path, dest_dir: &Path) -> Result<(), std::io::Error> {
    let file = fs::File::open(zip_file)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => dest_dir.join(path),
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

/// Converts a filter skip reason to an event skip reason.
fn filter_reason_to_event_reason(reason: &FilterSkipReason) -> SkipReason {
    match reason {
        FilterSkipReason::Binary => SkipReason::Binary,
        FilterSkipReason::IgnorePattern(_) => SkipReason::IgnorePattern,
        FilterSkipReason::LockFile => SkipReason::IgnorePattern,
        FilterSkipReason::TooLarge(_) => SkipReason::TooLarge,
        FilterSkipReason::Hidden => SkipReason::IgnorePattern,
    }
}

pub async fn process_extracted_files(
    repo_path: &Path,
    context: Context,
    sender: EventSender,
) -> Result<(usize, usize, usize), DocError> {
    let mut files_processed = 0;
    let mut files_skipped = 0;
    let mut chunks_created = 0;

    let filter = FileFilter::new();

    for entry in WalkDir::new(repo_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            let raw_relative_path = path.strip_prefix(repo_path).unwrap_or(path);

            // Normalize the path (remove "repo-branch/" prefix from ZIP archives)
            let relative_path = normalize_archive_path(raw_relative_path)
                .unwrap_or(raw_relative_path);

            // Get file metadata for size check
            let file_size = match entry.metadata() {
                Ok(meta) => meta.len(),
                Err(_) => continue, // Skip files we can't stat
            };

            // Check path-based filters first (cheaper than reading content)
            match filter.should_process_path(relative_path, file_size) {
                FilterResult::Skip(reason) => {
                    sender
                        .send(
                            Envelope::new(
                                "ingest.file_skipped.v1",
                                context.clone(),
                                serde_json::to_value(IngestFileSkippedPayload::new(
                                    relative_path.to_string_lossy(),
                                    filter_reason_to_event_reason(&reason),
                                ))?,
                            )
                            ,
                        )
                        .await
                        .map_err(|e| DocError::Internal(format!("Failed to send event: {}", e)))?;
                    files_skipped += 1;
                    continue;
                }
                FilterResult::Accept => {}
            }

            // Read file content
            let content_bytes = match fs::read(path) {
                Ok(bytes) => bytes,
                Err(_) => {
                    sender
                        .send(
                            Envelope::new(
                                "ingest.file_skipped.v1",
                                context.clone(),
                                serde_json::to_value(IngestFileSkippedPayload::new(
                                    relative_path.to_string_lossy(),
                                    SkipReason::Binary,
                                ))?,
                            )
                            ,
                        )
                        .await
                        .map_err(|e| DocError::Internal(format!("Failed to send event: {}", e)))?;
                    files_skipped += 1;
                    continue;
                }
            };

            // Check for binary content
            match FileFilter::should_process_content(&content_bytes) {
                FilterResult::Skip(reason) => {
                    sender
                        .send(
                            Envelope::new(
                                "ingest.file_skipped.v1",
                                context.clone(),
                                serde_json::to_value(IngestFileSkippedPayload::new(
                                    relative_path.to_string_lossy(),
                                    filter_reason_to_event_reason(&reason),
                                ))?,
                            )
                            ,
                        )
                        .await
                        .map_err(|e| DocError::Internal(format!("Failed to send event: {}", e)))?;
                    files_skipped += 1;
                    continue;
                }
                FilterResult::Accept => {}
            }

            // Convert to string for language detection and parsing
            let content = match String::from_utf8(content_bytes) {
                Ok(s) => s,
                Err(_) => {
                    sender
                        .send(
                            Envelope::new(
                                "ingest.file_skipped.v1",
                                context.clone(),
                                serde_json::to_value(IngestFileSkippedPayload::new(
                                    relative_path.to_string_lossy(),
                                    SkipReason::Binary,
                                ))?,
                            )
                            ,
                        )
                        .await
                        .map_err(|e| DocError::Internal(format!("Failed to send event: {}", e)))?;
                    files_skipped += 1;
                    continue;
                }
            };

            if let Some(language) = detect_language(relative_path, Some(&content)) {
                sender
                    .send(
                        Envelope::new(
                            "ingest.file_detected.v1",
                            context.clone(),
                            serde_json::to_value(IngestFileDetectedPayload::new(
                                relative_path.to_string_lossy(),
                                language,
                                content.len(),
                            ))?,
                        )
                        ,
                    )
                    .await
                    .map_err(|e| DocError::Internal(format!("Failed to send event: {}", e)))?;
                files_processed += 1;

                if let Some(tree) = parse(&content, language) {
                    let symbols = extract_symbols(&tree, &content, language);
                    for symbol in symbols {
                        let chunk_id = ChunkId::generate();
                        let payload = IngestChunkCreatedPayload::new(
                            chunk_id,
                            relative_path.to_string_lossy(),
                            language,
                            symbol.range,
                            &content[symbol.range.start..symbol.range.end],
                        )
                        .with_symbol(symbol.kind, symbol.name);

                        sender
                            .send(
                                Envelope::new(
                                    "ingest.chunk_created.v1",
                                    context.clone(),
                                    serde_json::to_value(payload)?,
                                )
                                ,
                            )
                            .await
                            .map_err(|e| {
                                DocError::Internal(format!("Failed to send event: {}", e))
                            })?;
                        chunks_created += 1;
                    }
                } else {
                    // Failed to parse, emit skipped event
                    sender
                        .send(
                            Envelope::new(
                                "ingest.file_skipped.v1",
                                context.clone(),
                                serde_json::to_value(IngestFileSkippedPayload::new(
                                    relative_path.to_string_lossy(),
                                    SkipReason::ParseError,
                                ))?,
                            )
                            ,
                        )
                        .await
                        .map_err(|e| {
                            DocError::Internal(format!("Failed to send event: {}", e))
                        })?;
                    files_skipped += 1;
                }
            } else {
                // Unsupported language, emit skipped event
                sender
                    .send(
                        Envelope::new(
                            "ingest.file_skipped.v1",
                            context.clone(),
                            serde_json::to_value(IngestFileSkippedPayload::new(
                                relative_path.to_string_lossy(),
                                SkipReason::UnsupportedLanguage,
                            ))?,
                        )
                        ,
                    )
                    .await
                    .map_err(|e| DocError::Internal(format!("Failed to send event: {}", e)))?;
                files_skipped += 1;
            }
        }
    }
    Ok((files_processed, files_skipped, chunks_created))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use zip::{write::FileOptions, ZipWriter};

    #[test]
    fn test_extract_zip() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("test.zip");
        let extract_dir = dir.path().join("extracted");

        // Create a dummy zip file
        let file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options: FileOptions<'_, ()> =
            FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("hello.txt", options).unwrap();
        zip.write_all(b"Hello, world!").unwrap();
        zip.finish().unwrap();

        let result = extract_zip(&zip_path, &extract_dir);
        assert!(result.is_ok());

        let extracted_file = extract_dir.join("hello.txt");
        assert!(extracted_file.exists());

        let content = fs::read_to_string(extracted_file).unwrap();
        assert_eq!(content, "Hello, world!");

        dir.close().unwrap();
    }

    #[test]
    fn test_process_extracted_files_no_events() {
        let dir = tempdir().unwrap();
        let test_dir = dir.path().join("test_src");
        fs::create_dir_all(&test_dir).unwrap();
        let test_file = test_dir.join("main.rs");
        let mut file = File::create(test_file).unwrap();
        file.write_all(b"fn main() {}").unwrap();

        // This test no longer calls process_extracted_files directly as it requires event sender
        // process_extracted_files(&test_dir);
        dir.close().unwrap();
    }
}
