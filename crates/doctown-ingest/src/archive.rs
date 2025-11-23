//! Archive extraction.
use crate::{language::detect_language, parsing::parse, symbol::extract_symbols};
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

pub fn process_extracted_files(dest_dir: &Path) {
    for entry in WalkDir::new(dest_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let path = entry.path();
            if let Ok(content) = fs::read_to_string(path) {
                if let Some(language) = detect_language(path, Some(&content)) {
                    if let Some(tree) = parse(&content, language) {
                        let symbols = extract_symbols(&tree, &content, language);
                        println!("Found {} symbols in {:?}", symbols.len(), path);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use zip::{ZipWriter, write::FileOptions};

    #[test]
    fn test_extract_zip() {
        let dir = tempdir().unwrap();
        let zip_path = dir.path().join("test.zip");
        let extract_dir = dir.path().join("extracted");

        // Create a dummy zip file
        let file = File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options: FileOptions<'_, ()> = FileOptions::default().compression_method(zip::CompressionMethod::Stored);
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
    fn test_process_extracted_files() {
        let dir = tempdir().unwrap();
        let test_dir = dir.path().join("test_src");
        fs::create_dir_all(&test_dir).unwrap();
        let test_file = test_dir.join("main.rs");
        let mut file = File::create(test_file).unwrap();
        file.write_all(b"fn main() {}").unwrap();

        process_extracted_files(&test_dir);
        dir.close().unwrap();
    }
}