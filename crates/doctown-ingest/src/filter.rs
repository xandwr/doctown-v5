//! File filtering for repository processing.
//!
//! Provides utilities for determining which files should be processed during ingestion,
//! including binary detection, ignore patterns, and size limits.

use std::path::Path;

/// Maximum file size in bytes (1MB).
pub const MAX_FILE_SIZE: u64 = 1024 * 1024;

/// Maximum total repository size in bytes (100MB).
pub const MAX_REPO_SIZE: u64 = 100 * 1024 * 1024;

/// Default ignore patterns for directories and files.
pub static DEFAULT_IGNORE_PATTERNS: &[&str] = &[
    // Version control
    ".git",
    ".svn",
    ".hg",
    // Dependencies
    "node_modules",
    "vendor",
    "bower_components",
    // Build outputs
    "target",       // Rust
    "dist",
    "build",
    "out",
    "_build",       // Elixir
    ".next",        // Next.js
    ".nuxt",        // Nuxt.js
    // Python
    "__pycache__",
    ".venv",
    "venv",
    ".tox",
    ".eggs",
    "*.egg-info",
    ".pytest_cache",
    ".mypy_cache",
    // IDE/Editor
    ".idea",
    ".vscode",
    ".vs",
    "*.swp",
    "*.swo",
    // OS
    ".DS_Store",
    "Thumbs.db",
    // Coverage/Testing
    "coverage",
    ".coverage",
    ".nyc_output",
    "htmlcov",
    // Cache
    ".cache",
    ".parcel-cache",
    // Logs
    "*.log",
    "logs",
];

/// Lock file patterns that should be skipped.
pub static LOCK_FILE_PATTERNS: &[&str] = &[
    "Cargo.lock",
    "package-lock.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "Gemfile.lock",
    "poetry.lock",
    "Pipfile.lock",
    "composer.lock",
    "go.sum",
    "flake.lock",
    "mix.lock",
];

/// Result of checking whether a file should be processed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterResult {
    /// File should be processed.
    Accept,
    /// File should be skipped with the given reason.
    Skip(SkipReason),
}

/// Reason why a file was skipped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// File is binary (contains null bytes).
    Binary,
    /// File matches an ignore pattern.
    IgnorePattern(String),
    /// File is a lock file.
    LockFile,
    /// File exceeds the size limit.
    TooLarge(u64),
    /// File is hidden (starts with dot).
    Hidden,
}

impl std::fmt::Display for SkipReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SkipReason::Binary => write!(f, "binary file"),
            SkipReason::IgnorePattern(pattern) => write!(f, "matches ignore pattern: {}", pattern),
            SkipReason::LockFile => write!(f, "lock file"),
            SkipReason::TooLarge(size) => write!(f, "file too large: {} bytes", size),
            SkipReason::Hidden => write!(f, "hidden file"),
        }
    }
}

/// A file filter that determines which files should be processed.
#[derive(Debug, Clone)]
pub struct FileFilter {
    /// Maximum file size in bytes.
    pub max_file_size: u64,
    /// Whether to skip hidden files (starting with dot).
    pub skip_hidden: bool,
    /// Whether to skip lock files.
    pub skip_lock_files: bool,
    /// Additional ignore patterns.
    pub ignore_patterns: Vec<String>,
}

impl Default for FileFilter {
    fn default() -> Self {
        Self {
            max_file_size: MAX_FILE_SIZE,
            skip_hidden: false, // Don't skip all hidden files, just specific patterns
            skip_lock_files: true,
            ignore_patterns: DEFAULT_IGNORE_PATTERNS
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

impl FileFilter {
    /// Creates a new file filter with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the maximum file size.
    pub fn with_max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = size;
        self
    }

    /// Sets whether to skip hidden files.
    pub fn with_skip_hidden(mut self, skip: bool) -> Self {
        self.skip_hidden = skip;
        self
    }

    /// Adds additional ignore patterns.
    pub fn with_ignore_patterns(mut self, patterns: Vec<String>) -> Self {
        self.ignore_patterns.extend(patterns);
        self
    }

    /// Checks if a path matches any ignore pattern.
    pub fn matches_ignore_pattern(&self, path: &Path) -> Option<&str> {
        let path_str = path.to_string_lossy();

        for pattern in &self.ignore_patterns {
            // Check if any path component matches the pattern
            for component in path.components() {
                let component_str = component.as_os_str().to_string_lossy();

                // Exact match
                if component_str == *pattern {
                    return Some(pattern);
                }

                // Glob-like pattern matching (simple implementation)
                if let Some(suffix) = pattern.strip_prefix('*') {
                    if component_str.ends_with(suffix) {
                        return Some(pattern);
                    }
                }
            }

            // Also check full path for patterns with paths
            if path_str.contains(pattern.as_str()) {
                return Some(pattern);
            }
        }

        None
    }

    /// Checks if a file is a lock file.
    pub fn is_lock_file(&self, path: &Path) -> bool {
        if !self.skip_lock_files {
            return false;
        }

        if let Some(file_name) = path.file_name() {
            let name = file_name.to_string_lossy();
            LOCK_FILE_PATTERNS.iter().any(|&pattern| name == pattern)
        } else {
            false
        }
    }

    /// Checks if content appears to be binary (contains null bytes).
    pub fn is_binary(content: &[u8]) -> bool {
        // Check first 8KB for null bytes (common heuristic)
        let check_len = content.len().min(8192);
        content[..check_len].contains(&0)
    }

    /// Checks if a file should be processed based on its path and metadata.
    ///
    /// This does not check content; use `should_process_content` for that.
    pub fn should_process_path(&self, path: &Path, file_size: u64) -> FilterResult {
        // Check size limit
        if file_size > self.max_file_size {
            return FilterResult::Skip(SkipReason::TooLarge(file_size));
        }

        // Check hidden files
        if self.skip_hidden {
            if let Some(file_name) = path.file_name() {
                if file_name.to_string_lossy().starts_with('.') {
                    return FilterResult::Skip(SkipReason::Hidden);
                }
            }
        }

        // Check lock files
        if self.is_lock_file(path) {
            return FilterResult::Skip(SkipReason::LockFile);
        }

        // Check ignore patterns
        if let Some(pattern) = self.matches_ignore_pattern(path) {
            return FilterResult::Skip(SkipReason::IgnorePattern(pattern.to_string()));
        }

        FilterResult::Accept
    }

    /// Checks if content should be processed (not binary).
    pub fn should_process_content(content: &[u8]) -> FilterResult {
        if Self::is_binary(content) {
            FilterResult::Skip(SkipReason::Binary)
        } else {
            FilterResult::Accept
        }
    }

    /// Full check combining path and content checks.
    pub fn should_process(&self, path: &Path, content: &[u8]) -> FilterResult {
        // Check path first (cheaper)
        let path_result = self.should_process_path(path, content.len() as u64);
        if let FilterResult::Skip(_) = path_result {
            return path_result;
        }

        // Then check content
        Self::should_process_content(content)
    }
}

/// Normalizes a path extracted from a ZIP archive.
///
/// GitHub ZIP archives have a top-level directory like "repo-branch/".
/// This function removes that prefix to get the actual repository paths.
pub fn normalize_archive_path(path: &Path) -> Option<&Path> {
    let mut components = path.components();

    // Skip the first component (the "repo-branch" directory)
    components.next()?;

    // Return the rest of the path
    let remaining = components.as_path();
    if remaining.as_os_str().is_empty() {
        None
    } else {
        Some(remaining)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ==========================================================================
    // Binary Detection Tests
    // ==========================================================================

    #[test]
    fn test_is_binary_with_null_bytes() {
        let content = b"Hello\x00World";
        assert!(FileFilter::is_binary(content));
    }

    #[test]
    fn test_is_binary_text_file() {
        let content = b"Hello, World!\nThis is a text file.";
        assert!(!FileFilter::is_binary(content));
    }

    #[test]
    fn test_is_binary_empty_file() {
        let content = b"";
        assert!(!FileFilter::is_binary(content));
    }

    #[test]
    fn test_is_binary_utf8_with_unicode() {
        let content = "Hello, 世界! 🌍".as_bytes();
        assert!(!FileFilter::is_binary(content));
    }

    // ==========================================================================
    // Ignore Pattern Tests
    // ==========================================================================

    #[test]
    fn test_ignore_pattern_node_modules() {
        let filter = FileFilter::new();
        let path = PathBuf::from("src/node_modules/package/index.js");
        assert!(filter.matches_ignore_pattern(&path).is_some());
    }

    #[test]
    fn test_ignore_pattern_target_dir() {
        let filter = FileFilter::new();
        let path = PathBuf::from("target/debug/myapp");
        assert!(filter.matches_ignore_pattern(&path).is_some());
    }

    #[test]
    fn test_ignore_pattern_pycache() {
        let filter = FileFilter::new();
        let path = PathBuf::from("src/__pycache__/module.pyc");
        assert!(filter.matches_ignore_pattern(&path).is_some());
    }

    #[test]
    fn test_ignore_pattern_venv() {
        let filter = FileFilter::new();
        let path = PathBuf::from(".venv/lib/python3.9/site-packages");
        assert!(filter.matches_ignore_pattern(&path).is_some());
    }

    #[test]
    fn test_ignore_pattern_git() {
        let filter = FileFilter::new();
        let path = PathBuf::from(".git/objects/pack");
        assert!(filter.matches_ignore_pattern(&path).is_some());
    }

    #[test]
    fn test_no_ignore_pattern_src() {
        let filter = FileFilter::new();
        let path = PathBuf::from("src/main.rs");
        assert!(filter.matches_ignore_pattern(&path).is_none());
    }

    #[test]
    fn test_glob_pattern_swp() {
        let filter = FileFilter::new();
        let path = PathBuf::from("src/main.rs.swp");
        assert!(filter.matches_ignore_pattern(&path).is_some());
    }

    // ==========================================================================
    // Lock File Tests
    // ==========================================================================

    #[test]
    fn test_lock_file_cargo() {
        let filter = FileFilter::new();
        let path = PathBuf::from("Cargo.lock");
        assert!(filter.is_lock_file(&path));
    }

    #[test]
    fn test_lock_file_package() {
        let filter = FileFilter::new();
        let path = PathBuf::from("package-lock.json");
        assert!(filter.is_lock_file(&path));
    }

    #[test]
    fn test_lock_file_yarn() {
        let filter = FileFilter::new();
        let path = PathBuf::from("yarn.lock");
        assert!(filter.is_lock_file(&path));
    }

    #[test]
    fn test_not_lock_file_cargo_toml() {
        let filter = FileFilter::new();
        let path = PathBuf::from("Cargo.toml");
        assert!(!filter.is_lock_file(&path));
    }

    // ==========================================================================
    // Size Limit Tests
    // ==========================================================================

    #[test]
    fn test_size_limit_under() {
        let filter = FileFilter::new();
        let path = PathBuf::from("src/main.rs");
        assert_eq!(filter.should_process_path(&path, 1000), FilterResult::Accept);
    }

    #[test]
    fn test_size_limit_over() {
        let filter = FileFilter::new();
        let path = PathBuf::from("src/main.rs");
        let result = filter.should_process_path(&path, 2 * 1024 * 1024);
        assert!(matches!(result, FilterResult::Skip(SkipReason::TooLarge(_))));
    }

    #[test]
    fn test_custom_size_limit() {
        let filter = FileFilter::new().with_max_file_size(500);
        let path = PathBuf::from("src/main.rs");
        let result = filter.should_process_path(&path, 600);
        assert!(matches!(result, FilterResult::Skip(SkipReason::TooLarge(_))));
    }

    // ==========================================================================
    // Full Filter Tests
    // ==========================================================================

    #[test]
    fn test_should_process_valid_rust_file() {
        let filter = FileFilter::new();
        let path = PathBuf::from("src/main.rs");
        let content = b"fn main() {}";
        assert_eq!(filter.should_process(&path, content), FilterResult::Accept);
    }

    #[test]
    fn test_should_process_binary_file() {
        let filter = FileFilter::new();
        let path = PathBuf::from("image.png");
        let content = b"\x89PNG\r\n\x1a\n\x00\x00\x00";
        assert!(matches!(
            filter.should_process(&path, content),
            FilterResult::Skip(SkipReason::Binary)
        ));
    }

    #[test]
    fn test_should_process_node_modules() {
        let filter = FileFilter::new();
        let path = PathBuf::from("node_modules/lodash/index.js");
        let content = b"module.exports = {};";
        assert!(matches!(
            filter.should_process(&path, content),
            FilterResult::Skip(SkipReason::IgnorePattern(_))
        ));
    }

    // ==========================================================================
    // Path Normalization Tests
    // ==========================================================================

    #[test]
    fn test_normalize_archive_path() {
        let path = PathBuf::from("repo-main/src/main.rs");
        let normalized = normalize_archive_path(&path).unwrap();
        assert_eq!(normalized, Path::new("src/main.rs"));
    }

    #[test]
    fn test_normalize_archive_path_single_file() {
        let path = PathBuf::from("repo-main/README.md");
        let normalized = normalize_archive_path(&path).unwrap();
        assert_eq!(normalized, Path::new("README.md"));
    }

    #[test]
    fn test_normalize_archive_path_root_only() {
        let path = PathBuf::from("repo-main");
        let normalized = normalize_archive_path(&path);
        assert!(normalized.is_none());
    }

    #[test]
    fn test_normalize_archive_path_nested() {
        let path = PathBuf::from("my-project-feature-branch/src/lib/utils/helpers.rs");
        let normalized = normalize_archive_path(&path).unwrap();
        assert_eq!(normalized, Path::new("src/lib/utils/helpers.rs"));
    }
}
