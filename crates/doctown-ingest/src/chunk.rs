//! Chunk creation and management for code symbols.
//!
//! This module handles:
//! - Creating chunks from extracted symbols
//! - Splitting large symbols with overlap
//! - File-level fallback for files with no extractable symbols
//! - Deterministic chunk ID generation

use doctown_common::{ByteRange, ChunkId, Language, SymbolKind};
use sha2::{Digest, Sha256};

use crate::symbol::Symbol;

/// Default maximum chunk size in bytes (4KB).
pub const DEFAULT_MAX_CHUNK_SIZE: usize = 4096;

/// Default overlap size when splitting large chunks (256 bytes).
pub const DEFAULT_OVERLAP_SIZE: usize = 256;

/// A chunk of source code extracted from a file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunk {
    /// Unique identifier for this chunk.
    pub id: ChunkId,
    /// The content of the chunk.
    pub content: String,
    /// Path to the source file.
    pub file_path: String,
    /// The language of the source file.
    pub language: Language,
    /// Byte range in the source file.
    pub byte_range: ByteRange,
    /// Metadata about the chunk.
    pub metadata: ChunkMetadata,
}

/// Metadata about a chunk.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChunkMetadata {
    /// The kind of symbol this chunk represents.
    pub symbol_kind: Option<SymbolKind>,
    /// The name of the symbol (if applicable).
    pub symbol_name: Option<String>,
    /// The signature of the symbol (if applicable).
    pub symbol_signature: Option<String>,
    /// Whether this is a split chunk (part of a larger symbol).
    pub is_split: bool,
    /// The index of this split (0-based), if split.
    pub split_index: Option<usize>,
    /// Total number of splits, if split.
    pub split_total: Option<usize>,
}

impl Chunk {
    /// Creates a new chunk with a deterministic ID.
    pub fn new(
        file_path: impl Into<String>,
        language: Language,
        byte_range: ByteRange,
        content: impl Into<String>,
    ) -> Self {
        let file_path = file_path.into();
        let content = content.into();
        let id = generate_chunk_id(&file_path, &byte_range, &content);

        Self {
            id,
            content,
            file_path,
            language,
            byte_range,
            metadata: ChunkMetadata::default(),
        }
    }

    /// Adds symbol metadata to the chunk.
    pub fn with_symbol(mut self, symbol: &Symbol) -> Self {
        self.metadata.symbol_kind = Some(symbol.kind);
        self.metadata.symbol_name = Some(symbol.name.clone());
        self.metadata.symbol_signature = symbol.signature.clone();
        self
    }

    /// Marks this chunk as a split chunk.
    pub fn with_split_info(mut self, index: usize, total: usize) -> Self {
        self.metadata.is_split = true;
        self.metadata.split_index = Some(index);
        self.metadata.split_total = Some(total);
        self
    }
}

/// Configuration for the chunking process.
#[derive(Debug, Clone)]
pub struct ChunkingConfig {
    /// Maximum chunk size in bytes.
    pub max_chunk_size: usize,
    /// Overlap size when splitting large chunks.
    pub overlap_size: usize,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            max_chunk_size: DEFAULT_MAX_CHUNK_SIZE,
            overlap_size: DEFAULT_OVERLAP_SIZE,
        }
    }
}

/// Creates chunks from a source file and its extracted symbols.
///
/// This function:
/// 1. Creates one chunk per symbol
/// 2. Splits large symbols into multiple chunks with overlap
/// 3. Falls back to a file-level chunk if no symbols are extracted
pub fn create_chunks(
    file_path: &str,
    source_code: &str,
    language: Language,
    symbols: &[Symbol],
    config: &ChunkingConfig,
) -> Vec<Chunk> {
    // If no symbols, create a file-level chunk
    if symbols.is_empty() {
        return create_file_chunk(file_path, source_code, language, config);
    }

    let mut chunks = Vec::new();

    for symbol in symbols {
        let symbol_content = &source_code[symbol.range.start..symbol.range.end];
        let symbol_chunks =
            create_symbol_chunks(file_path, language, symbol, symbol_content, config);
        chunks.extend(symbol_chunks);
    }

    chunks
}

/// Creates chunks for a single symbol, splitting if necessary.
fn create_symbol_chunks(
    file_path: &str,
    language: Language,
    symbol: &Symbol,
    content: &str,
    config: &ChunkingConfig,
) -> Vec<Chunk> {
    let content_size = content.len();

    // If small enough, create a single chunk
    if content_size <= config.max_chunk_size {
        let chunk =
            Chunk::new(file_path, language, symbol.range.clone(), content).with_symbol(symbol);
        return vec![chunk];
    }

    // Split the symbol into multiple chunks with overlap
    split_symbol(file_path, language, symbol, content, config)
}

/// Splits a large symbol into multiple chunks with overlap.
fn split_symbol(
    file_path: &str,
    language: Language,
    symbol: &Symbol,
    content: &str,
    config: &ChunkingConfig,
) -> Vec<Chunk> {
    let mut chunks = Vec::new();
    let content_bytes = content.as_bytes();
    let content_len = content_bytes.len();

    // Calculate effective chunk size (accounting for overlap)
    let effective_size = config.max_chunk_size - config.overlap_size;

    // Calculate number of splits needed
    let num_splits = (content_len + effective_size - 1) / effective_size;

    for i in 0..num_splits {
        let start_offset = i * effective_size;
        let end_offset = std::cmp::min(start_offset + config.max_chunk_size, content_len);

        // Adjust to avoid splitting mid-character (UTF-8 safety)
        let (adjusted_start, adjusted_end) =
            find_safe_boundaries(content, start_offset, end_offset);

        let chunk_content = &content[adjusted_start..adjusted_end];
        let byte_range = ByteRange::new(
            symbol.range.start + adjusted_start,
            symbol.range.start + adjusted_end,
        );

        let chunk = Chunk::new(file_path, language, byte_range, chunk_content)
            .with_symbol(symbol)
            .with_split_info(i, num_splits);

        chunks.push(chunk);
    }

    chunks
}

/// Finds safe UTF-8 boundaries for splitting.
fn find_safe_boundaries(content: &str, start: usize, end: usize) -> (usize, usize) {
    let bytes = content.as_bytes();

    // Adjust start to not be in the middle of a UTF-8 character
    let mut adjusted_start = start;
    while adjusted_start > 0 && !is_char_boundary(bytes, adjusted_start) {
        adjusted_start -= 1;
    }

    // Adjust end to not be in the middle of a UTF-8 character
    let mut adjusted_end = end;
    while adjusted_end < bytes.len() && !is_char_boundary(bytes, adjusted_end) {
        adjusted_end += 1;
    }

    (adjusted_start, adjusted_end)
}

/// Checks if a byte position is a valid UTF-8 character boundary.
fn is_char_boundary(bytes: &[u8], index: usize) -> bool {
    if index >= bytes.len() {
        return true;
    }
    // A byte is a char boundary if it's not a continuation byte (10xxxxxx)
    (bytes[index] & 0xC0) != 0x80
}

/// Creates a file-level chunk when no symbols are extracted.
fn create_file_chunk(
    file_path: &str,
    source_code: &str,
    language: Language,
    config: &ChunkingConfig,
) -> Vec<Chunk> {
    let content_len = source_code.len();

    // If the file is small enough, create a single chunk
    if content_len <= config.max_chunk_size {
        let byte_range = ByteRange::new(0, content_len);
        let chunk = Chunk::new(file_path, language, byte_range, source_code);
        return vec![chunk];
    }

    // Split the file into multiple chunks
    let mut chunks = Vec::new();
    let effective_size = config.max_chunk_size - config.overlap_size;
    let num_splits = (content_len + effective_size - 1) / effective_size;

    for i in 0..num_splits {
        let start_offset = i * effective_size;
        let end_offset = std::cmp::min(start_offset + config.max_chunk_size, content_len);

        let (adjusted_start, adjusted_end) =
            find_safe_boundaries(source_code, start_offset, end_offset);

        let chunk_content = &source_code[adjusted_start..adjusted_end];
        let byte_range = ByteRange::new(adjusted_start, adjusted_end);

        let mut chunk = Chunk::new(file_path, language, byte_range, chunk_content);
        if num_splits > 1 {
            chunk = chunk.with_split_info(i, num_splits);
        }

        chunks.push(chunk);
    }

    chunks
}

/// Generates a deterministic chunk ID based on content and location.
///
/// The ID is derived from a hash of:
/// - File path
/// - Byte range (start, end)
/// - Content hash
fn generate_chunk_id(file_path: &str, byte_range: &ByteRange, content: &str) -> ChunkId {
    let mut hasher = Sha256::new();
    hasher.update(file_path.as_bytes());
    hasher.update(byte_range.start.to_le_bytes());
    hasher.update(byte_range.end.to_le_bytes());
    hasher.update(content.as_bytes());

    let hash = hasher.finalize();
    let hex = hex::encode(&hash[..8]); // Use first 8 bytes (16 hex chars)

    ChunkId::new(format!("chunk_{}", hex)).expect("Generated chunk ID should be valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::parse;
    use crate::symbol::extract_symbols;
    use doctown_common::types::Visibility;

    // ============================================
    // Chunk Creation Tests
    // ============================================

    #[test]
    fn test_chunk_new() {
        let chunk = Chunk::new(
            "src/main.rs",
            Language::Rust,
            ByteRange::new(0, 100),
            "fn main() {}",
        );

        assert!(chunk.id.as_str().starts_with("chunk_"));
        assert_eq!(chunk.file_path, "src/main.rs");
        assert_eq!(chunk.language, Language::Rust);
        assert_eq!(chunk.byte_range.start, 0);
        assert_eq!(chunk.byte_range.end, 100);
        assert_eq!(chunk.content, "fn main() {}");
    }

    #[test]
    fn test_chunk_with_symbol() {
        let symbol = Symbol {
            kind: SymbolKind::Function,
            name: "main".to_string(),
            range: ByteRange::new(0, 50),
            name_range: ByteRange::new(3, 7),
            signature: Some("main()".to_string()),
            visibility: Visibility::Public,
            is_async: false,
        };

        let chunk = Chunk::new(
            "src/main.rs",
            Language::Rust,
            ByteRange::new(0, 50),
            "fn main() {}",
        )
        .with_symbol(&symbol);

        assert_eq!(chunk.metadata.symbol_kind, Some(SymbolKind::Function));
        assert_eq!(chunk.metadata.symbol_name, Some("main".to_string()));
        assert_eq!(chunk.metadata.symbol_signature, Some("main()".to_string()));
    }

    #[test]
    fn test_chunk_with_split_info() {
        let chunk = Chunk::new(
            "src/main.rs",
            Language::Rust,
            ByteRange::new(0, 100),
            "fn main() {}",
        )
        .with_split_info(0, 3);

        assert!(chunk.metadata.is_split);
        assert_eq!(chunk.metadata.split_index, Some(0));
        assert_eq!(chunk.metadata.split_total, Some(3));
    }

    // ============================================
    // Chunking Strategy Tests
    // ============================================

    #[test]
    fn test_create_chunks_from_symbols() {
        let code = r#"
fn foo() {
    println!("foo");
}

fn bar() {
    println!("bar");
}
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);
        let config = ChunkingConfig::default();

        let chunks = create_chunks("src/lib.rs", code, Language::Rust, &symbols, &config);

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].metadata.symbol_name, Some("foo".to_string()));
        assert_eq!(chunks[1].metadata.symbol_name, Some("bar".to_string()));
    }

    #[test]
    fn test_nested_symbols_produce_separate_chunks() {
        // In Rust, impl blocks and their methods are both extracted
        let code = r#"
struct Point { x: i32, y: i32 }

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);
        let config = ChunkingConfig::default();

        let chunks = create_chunks("src/point.rs", code, Language::Rust, &symbols, &config);

        // Should have struct + impl
        assert!(chunks.len() >= 2);

        let struct_chunk = chunks
            .iter()
            .find(|c| c.metadata.symbol_name == Some("Point".to_string()));
        let impl_chunk = chunks
            .iter()
            .find(|c| c.metadata.symbol_kind == Some(SymbolKind::Impl));

        assert!(struct_chunk.is_some());
        assert!(impl_chunk.is_some());
    }

    // ============================================
    // Large Symbol Handling Tests
    // ============================================

    #[test]
    fn test_large_function_split_correctly() {
        // Create a function larger than the max chunk size
        let large_body = "    let x = 1;\n".repeat(500); // ~8000 bytes
        let code = format!("fn large_function() {{\n{}}}", large_body);

        let tree = parse(&code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, &code, Language::Rust);
        let config = ChunkingConfig {
            max_chunk_size: 1024,
            overlap_size: 128,
        };

        let chunks = create_chunks("src/large.rs", &code, Language::Rust, &symbols, &config);

        // Should be split into multiple chunks
        assert!(chunks.len() > 1);

        // All chunks should reference the same symbol
        for chunk in &chunks {
            assert_eq!(
                chunk.metadata.symbol_name,
                Some("large_function".to_string())
            );
            assert!(chunk.metadata.is_split);
        }

        // Check split indices
        assert_eq!(chunks[0].metadata.split_index, Some(0));
        assert_eq!(
            chunks[0].metadata.split_total,
            chunks.last().unwrap().metadata.split_total
        );
    }

    #[test]
    fn test_overlap_preserved() {
        let large_body = "x".repeat(2000);
        let code = format!("fn test() {{ {} }}", large_body);

        let tree = parse(&code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, &code, Language::Rust);
        let config = ChunkingConfig {
            max_chunk_size: 512,
            overlap_size: 64,
        };

        let chunks = create_chunks("src/overlap.rs", &code, Language::Rust, &symbols, &config);

        // Verify overlap between consecutive chunks
        for i in 1..chunks.len() {
            let prev_end = chunks[i - 1].byte_range.end;
            let curr_start = chunks[i].byte_range.start;

            // There should be overlap (prev_end > curr_start)
            assert!(
                prev_end > curr_start,
                "Expected overlap between chunk {} and {}: prev_end={}, curr_start={}",
                i - 1,
                i,
                prev_end,
                curr_start
            );
        }
    }

    // ============================================
    // File-level Fallback Tests
    // ============================================

    #[test]
    fn test_file_without_symbols_gets_file_chunk() {
        // A file with only comments, no extractable symbols
        let code = r#"
# This is a comment
# Another comment
# No actual code here
"#;
        let tree = parse(code, Language::Python).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Python);
        let config = ChunkingConfig::default();

        assert!(symbols.is_empty());

        let chunks = create_chunks("src/comments.py", code, Language::Python, &symbols, &config);

        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].metadata.symbol_kind.is_none());
        assert!(chunks[0].metadata.symbol_name.is_none());
        assert!(!chunks[0].metadata.is_split);
    }

    #[test]
    fn test_large_file_without_symbols_split() {
        let large_content = "# comment\n".repeat(500); // ~5000 bytes
        let config = ChunkingConfig {
            max_chunk_size: 1024,
            overlap_size: 128,
        };

        let chunks = create_chunks(
            "src/comments.py",
            &large_content,
            Language::Python,
            &[], // No symbols
            &config,
        );

        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(chunk.metadata.is_split);
            assert!(chunk.metadata.symbol_kind.is_none());
        }
    }

    // ============================================
    // Chunk ID Generation Tests
    // ============================================

    #[test]
    fn test_chunk_id_stability() {
        let chunk1 = Chunk::new(
            "src/main.rs",
            Language::Rust,
            ByteRange::new(0, 100),
            "fn main() {}",
        );
        let chunk2 = Chunk::new(
            "src/main.rs",
            Language::Rust,
            ByteRange::new(0, 100),
            "fn main() {}",
        );

        // Same input should produce same ID
        assert_eq!(chunk1.id, chunk2.id);
    }

    #[test]
    fn test_different_content_different_id() {
        let chunk1 = Chunk::new(
            "src/main.rs",
            Language::Rust,
            ByteRange::new(0, 100),
            "fn foo() {}",
        );
        let chunk2 = Chunk::new(
            "src/main.rs",
            Language::Rust,
            ByteRange::new(0, 100),
            "fn bar() {}",
        );

        // Different content should produce different ID
        assert_ne!(chunk1.id, chunk2.id);
    }

    #[test]
    fn test_different_path_different_id() {
        let chunk1 = Chunk::new(
            "src/foo.rs",
            Language::Rust,
            ByteRange::new(0, 100),
            "fn main() {}",
        );
        let chunk2 = Chunk::new(
            "src/bar.rs",
            Language::Rust,
            ByteRange::new(0, 100),
            "fn main() {}",
        );

        // Different path should produce different ID
        assert_ne!(chunk1.id, chunk2.id);
    }

    #[test]
    fn test_different_range_different_id() {
        let chunk1 = Chunk::new(
            "src/main.rs",
            Language::Rust,
            ByteRange::new(0, 100),
            "fn main() {}",
        );
        let chunk2 = Chunk::new(
            "src/main.rs",
            Language::Rust,
            ByteRange::new(50, 150),
            "fn main() {}",
        );

        // Different range should produce different ID
        assert_ne!(chunk1.id, chunk2.id);
    }

    #[test]
    fn test_chunk_id_format() {
        let chunk = Chunk::new(
            "src/main.rs",
            Language::Rust,
            ByteRange::new(0, 100),
            "fn main() {}",
        );

        // ID should start with chunk_ and be a valid format
        assert!(chunk.id.as_str().starts_with("chunk_"));
        assert!(chunk.id.as_str().len() >= 10);
    }

    // ============================================
    // UTF-8 Safety Tests
    // ============================================

    #[test]
    fn test_utf8_boundary_handling() {
        // Content with multi-byte UTF-8 characters
        let content = "fn test() { let emoji = \"😀🎉🚀\"; }";
        let config = ChunkingConfig {
            max_chunk_size: 20,
            overlap_size: 5,
        };

        // Should not panic when splitting
        let chunks = create_chunks("src/emoji.rs", content, Language::Rust, &[], &config);

        // All chunks should be valid UTF-8
        for chunk in &chunks {
            assert!(chunk.content.is_ascii() || !chunk.content.is_empty());
        }
    }
}
