use serde::{Deserialize, Serialize};

/// Maps internal docpack structure to source files
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceMap {
    pub files: Vec<SourceMapFile>,
}

/// A source file with its chunks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceMapFile {
    pub file_path: String,
    pub language: String,
    pub chunks: Vec<SourceMapChunk>,
}

/// A chunk within a source file
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SourceMapChunk {
    pub chunk_id: String,
    pub byte_range: (usize, usize),
    pub symbol_ids: Vec<String>,
}

impl SourceMap {
    /// Create a new SourceMap
    pub fn new(files: Vec<SourceMapFile>) -> Self {
        Self { files }
    }

    /// Create an empty SourceMap
    pub fn empty() -> Self {
        Self { files: Vec::new() }
    }

    /// Add a file to the source map
    pub fn add_file(&mut self, file: SourceMapFile) {
        self.files.push(file);
    }

    /// Get the number of files
    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    /// Check if there are no files
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Serialize to JSON bytes
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec_pretty(self)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Deserialize from JSON bytes
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

impl SourceMapFile {
    /// Create a new source map file
    pub fn new(file_path: String, language: String, chunks: Vec<SourceMapChunk>) -> Self {
        Self {
            file_path,
            language,
            chunks,
        }
    }

    /// Add a chunk to this file
    pub fn add_chunk(&mut self, chunk: SourceMapChunk) {
        self.chunks.push(chunk);
    }

    /// Get the number of chunks
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }
}

impl SourceMapChunk {
    /// Create a new chunk
    pub fn new(chunk_id: String, byte_range: (usize, usize), symbol_ids: Vec<String>) -> Self {
        Self {
            chunk_id,
            byte_range,
            symbol_ids,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_creation() {
        let chunk = SourceMapChunk::new(
            "chunk_abc".to_string(),
            (0, 200),
            vec!["sym_main_fn".to_string()],
        );

        assert_eq!(chunk.chunk_id, "chunk_abc");
        assert_eq!(chunk.byte_range, (0, 200));
        assert_eq!(chunk.symbol_ids, vec!["sym_main_fn".to_string()]);
    }

    #[test]
    fn test_file_creation() {
        let chunk = SourceMapChunk::new(
            "chunk_abc".to_string(),
            (0, 200),
            vec!["sym_main_fn".to_string()],
        );

        let file = SourceMapFile::new(
            "src/main.rs".to_string(),
            "rust".to_string(),
            vec![chunk],
        );

        assert_eq!(file.file_path, "src/main.rs");
        assert_eq!(file.language, "rust");
        assert_eq!(file.chunk_count(), 1);
    }

    #[test]
    fn test_source_map_creation() {
        let chunk = SourceMapChunk::new(
            "chunk_abc".to_string(),
            (0, 200),
            vec!["sym_main_fn".to_string()],
        );

        let file = SourceMapFile::new(
            "src/main.rs".to_string(),
            "rust".to_string(),
            vec![chunk],
        );

        let source_map = SourceMap::new(vec![file]);

        assert_eq!(source_map.file_count(), 1);
        assert!(!source_map.is_empty());
        assert_eq!(source_map.files[0].file_path, "src/main.rs");
    }

    #[test]
    fn test_empty_source_map() {
        let source_map = SourceMap::empty();

        assert_eq!(source_map.file_count(), 0);
        assert!(source_map.is_empty());
    }

    #[test]
    fn test_source_map_json_roundtrip() {
        let chunk = SourceMapChunk::new(
            "chunk_abc".to_string(),
            (0, 200),
            vec!["sym_main_fn".to_string()],
        );

        let file = SourceMapFile::new(
            "src/main.rs".to_string(),
            "rust".to_string(),
            vec![chunk],
        );

        let source_map = SourceMap::new(vec![file]);
        let json = source_map.to_json().unwrap();
        let parsed = SourceMap::from_json(&json).unwrap();

        assert_eq!(source_map.file_count(), parsed.file_count());
        assert_eq!(source_map.files[0].file_path, parsed.files[0].file_path);
        assert_eq!(source_map.files[0].chunks[0].chunk_id, parsed.files[0].chunks[0].chunk_id);
    }

    #[test]
    fn test_source_map_json_format() {
        let chunk = SourceMapChunk::new(
            "chunk_abc".to_string(),
            (0, 200),
            vec!["sym_main_fn".to_string()],
        );

        let file = SourceMapFile::new(
            "src/main.rs".to_string(),
            "rust".to_string(),
            vec![chunk],
        );

        let source_map = SourceMap::new(vec![file]);
        let json = source_map.to_json().unwrap();

        assert!(json.contains("\"file_path\": \"src/main.rs\""));
        assert!(json.contains("\"language\": \"rust\""));
        assert!(json.contains("\"chunk_id\": \"chunk_abc\""));
        assert!(json.contains("\"symbol_ids\""));
    }
}
