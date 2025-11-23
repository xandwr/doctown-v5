//! Domain types for Doctown.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::Path;

/// A byte range in source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ByteRange {
    pub start: usize,
    pub end: usize,
}

impl ByteRange {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns true if this range contains the other range.
    pub fn contains(&self, other: &ByteRange) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    /// Returns true if this range overlaps with the other range.
    pub fn overlaps(&self, other: &ByteRange) -> bool {
        self.start < other.end && other.start < self.end
    }
}

impl fmt::Display for ByteRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// Supported programming languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    Python,
    TypeScript,
    JavaScript,
    Go,
}

impl Language {
    /// Detect language from file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "rs" => Some(Language::Rust),
            "py" => Some(Language::Python),
            "ts" | "tsx" => Some(Language::TypeScript),
            "js" | "jsx" | "mjs" | "cjs" => Some(Language::JavaScript),
            "go" => Some(Language::Go),
            _ => None,
        }
    }

    /// Detect language from file path.
    pub fn from_path(path: &Path) -> Option<Self> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(Self::from_extension)
    }

    /// Returns the canonical file extension for this language.
    pub fn extension(&self) -> &'static str {
        match self {
            Language::Rust => "rs",
            Language::Python => "py",
            Language::TypeScript => "ts",
            Language::JavaScript => "js",
            Language::Go => "go",
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Language::Rust => write!(f, "Rust"),
            Language::Python => write!(f, "Python"),
            Language::TypeScript => write!(f, "TypeScript"),
            Language::JavaScript => write!(f, "JavaScript"),
            Language::Go => write!(f, "Go"),
        }
    }
}

/// The kind of symbol extracted from source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Function,
    Class,
    Module,
    Struct,
    Trait,
    Enum,
    Method,
    Const,
    Interface,
    TypeAlias,
}

impl SymbolKind {
    /// Returns a human-readable short form.
    pub fn short_name(&self) -> &'static str {
        match self {
            SymbolKind::Function => "fn",
            SymbolKind::Class => "class",
            SymbolKind::Module => "mod",
            SymbolKind::Struct => "struct",
            SymbolKind::Trait => "trait",
            SymbolKind::Enum => "enum",
            SymbolKind::Method => "method",
            SymbolKind::Const => "const",
            SymbolKind::Interface => "interface",
            SymbolKind::TypeAlias => "type",
        }
    }
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.short_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_byte_range_len() {
        let range = ByteRange::new(10, 20);
        assert_eq!(range.len(), 10);
    }

    #[test]
    fn test_byte_range_contains() {
        let outer = ByteRange::new(0, 100);
        let inner = ByteRange::new(10, 50);
        assert!(outer.contains(&inner));
        assert!(!inner.contains(&outer));
    }

    #[test]
    fn test_byte_range_overlaps() {
        let a = ByteRange::new(0, 50);
        let b = ByteRange::new(25, 75);
        let c = ByteRange::new(60, 100);
        assert!(a.overlaps(&b));
        assert!(b.overlaps(&a));
        assert!(!a.overlaps(&c));
    }

    #[test]
    fn test_language_from_extension() {
        assert_eq!(Language::from_extension("rs"), Some(Language::Rust));
        assert_eq!(Language::from_extension("py"), Some(Language::Python));
        assert_eq!(Language::from_extension("ts"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("tsx"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("js"), Some(Language::JavaScript));
        assert_eq!(Language::from_extension("jsx"), Some(Language::JavaScript));
        assert_eq!(Language::from_extension("go"), Some(Language::Go));
        assert_eq!(Language::from_extension("unknown"), None);
    }

    #[test]
    fn test_language_from_path() {
        assert_eq!(
            Language::from_path(&PathBuf::from("src/main.rs")),
            Some(Language::Rust)
        );
        assert_eq!(
            Language::from_path(&PathBuf::from("script.py")),
            Some(Language::Python)
        );
        assert_eq!(Language::from_path(&PathBuf::from("Makefile")), None);
    }

    #[test]
    fn test_symbol_kind_display() {
        assert_eq!(SymbolKind::Function.to_string(), "fn");
        assert_eq!(SymbolKind::Class.to_string(), "class");
        assert_eq!(SymbolKind::Struct.to_string(), "struct");
    }

    #[test]
    fn test_language_display() {
        assert_eq!(Language::Rust.to_string(), "Rust");
        assert_eq!(Language::Python.to_string(), "Python");
    }
}
