//! AST parsing with grammar selection and parser pooling.
//!
//! This module provides a `Parser` struct that:
//! - Selects the appropriate tree-sitter grammar based on language
//! - Implements parser pooling for efficient reuse
//! - Handles parsing errors gracefully (returning partial trees when possible)

use doctown_common::Language;
use std::cell::RefCell;
use std::collections::HashMap;
use tree_sitter::{Parser as TsParser, Tree};

/// A parser that selects the appropriate grammar based on language
/// and supports pooling for efficient reuse.
pub struct Parser {
    /// Pool of tree-sitter parsers, keyed by language.
    /// Using RefCell for interior mutability since parsing requires &mut self.
    parsers: RefCell<HashMap<Language, TsParser>>,
}

impl Parser {
    /// Creates a new Parser with an empty pool.
    pub fn new() -> Self {
        Self {
            parsers: RefCell::new(HashMap::new()),
        }
    }

    /// Returns the tree-sitter language for the given Language enum.
    /// Returns None for unsupported languages.
    fn get_ts_language(language: Language) -> Option<tree_sitter::Language> {
        match language {
            Language::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
            Language::Python => Some(tree_sitter_python::LANGUAGE.into()),
            Language::TypeScript => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            Language::JavaScript => Some(tree_sitter_javascript::LANGUAGE.into()),
            Language::Go => Some(tree_sitter_go::LANGUAGE.into()),
        }
    }

    /// Checks if the given language is supported for parsing.
    pub fn is_supported(language: Language) -> bool {
        Self::get_ts_language(language).is_some()
    }

    /// Returns a list of all supported languages.
    pub fn supported_languages() -> &'static [Language] {
        &[
            Language::Rust,
            Language::Python,
            Language::TypeScript,
            Language::JavaScript,
            Language::Go,
        ]
    }

    /// Parses source code using the appropriate grammar for the given language.
    ///
    /// Returns `Some(Tree)` on success (including partial trees for invalid syntax),
    /// or `None` if the language is not supported or parsing completely fails.
    ///
    /// # Arguments
    /// * `source_code` - The source code to parse
    /// * `language` - The programming language of the source code
    ///
    /// # Examples
    /// ```
    /// use doctown_ingest::Parser;
    /// use doctown_common::Language;
    ///
    /// let parser = Parser::new();
    /// let tree = parser.parse("fn main() {}", Language::Rust);
    /// assert!(tree.is_some());
    /// ```
    pub fn parse(&self, source_code: &str, language: Language) -> Option<Tree> {
        let ts_lang = Self::get_ts_language(language)?;

        let mut parsers = self.parsers.borrow_mut();
        let parser = parsers.entry(language).or_insert_with(|| {
            let mut p = TsParser::new();
            // This should never fail for languages we support
            p.set_language(&ts_lang)
                .expect("Error loading grammar - this is a bug");
            p
        });

        parser.parse(source_code, None)
    }

    /// Parses source code with an optional previous tree for incremental parsing.
    ///
    /// When editing a file, passing the previous tree allows tree-sitter to
    /// reuse unchanged parts of the syntax tree for better performance.
    ///
    /// # Arguments
    /// * `source_code` - The source code to parse
    /// * `language` - The programming language of the source code
    /// * `old_tree` - Optional previous tree for incremental parsing
    pub fn parse_with_old_tree(
        &self,
        source_code: &str,
        language: Language,
        old_tree: Option<&Tree>,
    ) -> Option<Tree> {
        let ts_lang = Self::get_ts_language(language)?;

        let mut parsers = self.parsers.borrow_mut();
        let parser = parsers.entry(language).or_insert_with(|| {
            let mut p = TsParser::new();
            p.set_language(&ts_lang)
                .expect("Error loading grammar - this is a bug");
            p
        });

        parser.parse(source_code, old_tree)
    }

    /// Clears the parser pool, releasing all cached parsers.
    pub fn clear_pool(&self) {
        self.parsers.borrow_mut().clear();
    }

    /// Returns the number of parsers currently in the pool.
    pub fn pool_size(&self) -> usize {
        self.parsers.borrow().len()
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

// Thread-local parser pool for efficient reuse across calls
thread_local! {
    static PARSER_POOL: Parser = Parser::new();
}

/// Convenience function that uses a thread-local parser pool.
///
/// This is the simplest way to parse source code when you don't need
/// to manage parser lifecycle yourself.
///
/// # Arguments
/// * `source_code` - The source code to parse
/// * `language` - The programming language of the source code
///
/// # Returns
/// `Some(Tree)` on success, `None` if the language is unsupported.
pub fn parse(source_code: &str, language: Language) -> Option<Tree> {
    PARSER_POOL.with(|parser| parser.parse(source_code, language))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol::extract_symbols;

    // ============================================
    // Basic Rust Parsing Tests
    // ============================================

    #[test]
    fn test_parse_rust_simple_function() {
        let code = r#"
            fn main() {
                println!("Hello, world!");
            }
        "#;
        let tree = parse(code, Language::Rust);
        assert!(tree.is_some());
        let tree = tree.unwrap();
        let root = tree.root_node();
        assert_eq!(root.kind(), "source_file");
        assert!(!root.has_error());
    }

    #[test]
    fn test_parse_rust_multiple_functions() {
        let code = r#"
            fn add(a: i32, b: i32) -> i32 {
                a + b
            }

            fn subtract(a: i32, b: i32) -> i32 {
                a - b
            }

            pub fn main() {
                let result = add(5, 3);
                println!("{}", result);
            }
        "#;
        let tree = parse(code, Language::Rust).unwrap();
        let root = tree.root_node();
        assert_eq!(root.kind(), "source_file");
        assert!(!root.has_error());
        // Should have multiple children (function items)
        assert!(root.child_count() >= 3);
    }

    #[test]
    fn test_parse_rust_struct_and_impl() {
        let code = r#"
            struct Point {
                x: f64,
                y: f64,
            }

            impl Point {
                fn new(x: f64, y: f64) -> Self {
                    Self { x, y }
                }

                fn distance(&self, other: &Point) -> f64 {
                    ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
                }
            }
        "#;
        let tree = parse(code, Language::Rust).unwrap();
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_rust_with_generics_and_lifetimes() {
        let code = r#"
            struct Container<'a, T: Clone> {
                data: &'a T,
            }

            impl<'a, T: Clone> Container<'a, T> {
                fn get(&self) -> T {
                    self.data.clone()
                }
            }

            fn process<T: std::fmt::Display>(item: T) {
                println!("{}", item);
            }
        "#;
        let tree = parse(code, Language::Rust).unwrap();
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_rust_async_code() {
        let code = r#"
            async fn fetch_data() -> Result<String, Error> {
                let response = client.get(url).await?;
                Ok(response.text().await?)
            }

            #[tokio::main]
            async fn main() {
                let data = fetch_data().await.unwrap();
            }
        "#;
        let tree = parse(code, Language::Rust).unwrap();
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_rust_macros() {
        let code = r#"
            macro_rules! say_hello {
                () => {
                    println!("Hello!");
                };
                ($name:expr) => {
                    println!("Hello, {}!", $name);
                };
            }

            fn main() {
                say_hello!();
                say_hello!("world");
            }
        "#;
        let tree = parse(code, Language::Rust).unwrap();
        assert!(!tree.root_node().has_error());
    }

    // ============================================
    // Basic Python Parsing Tests
    // ============================================

    #[test]
    fn test_parse_python_simple_function() {
        let code = r#"
def hello():
    print("Hello, world!")
"#;
        let tree = parse(code, Language::Python);
        assert!(tree.is_some());
        let tree = tree.unwrap();
        let root = tree.root_node();
        assert_eq!(root.kind(), "module");
        assert!(!root.has_error());
    }

    #[test]
    fn test_parse_python_class() {
        let code = r#"
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y

    def distance(self, other):
        return ((self.x - other.x) ** 2 + (self.y - other.y) ** 2) ** 0.5

    def __repr__(self):
        return f"Point({self.x}, {self.y})"
"#;
        let tree = parse(code, Language::Python).unwrap();
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_python_decorators() {
        let code = r#"
from functools import lru_cache

@lru_cache(maxsize=128)
def fibonacci(n):
    if n < 2:
        return n
    return fibonacci(n - 1) + fibonacci(n - 2)

class MyClass:
    @property
    def value(self):
        return self._value

    @value.setter
    def value(self, val):
        self._value = val

    @staticmethod
    def helper():
        pass

    @classmethod
    def create(cls):
        return cls()
"#;
        let tree = parse(code, Language::Python).unwrap();
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_python_async() {
        let code = r#"
import asyncio

async def fetch_data(url):
    async with aiohttp.ClientSession() as session:
        async for chunk in response.content.iter_chunked(1024):
            yield chunk

async def main():
    await fetch_data("http://example.com")

asyncio.run(main())
"#;
        let tree = parse(code, Language::Python).unwrap();
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_python_comprehensions() {
        let code = r#"
# List comprehension
squares = [x ** 2 for x in range(10)]

# Dict comprehension
word_lengths = {word: len(word) for word in words}

# Set comprehension
unique_chars = {char.lower() for char in text if char.isalpha()}

# Generator expression
sum_of_squares = sum(x ** 2 for x in range(1000000))

# Nested comprehension
matrix = [[i * j for j in range(5)] for i in range(5)]
"#;
        let tree = parse(code, Language::Python).unwrap();
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_parse_python_type_hints() {
        let code = r#"
from typing import List, Dict, Optional, Union, Callable

def process(
    items: List[int],
    mapping: Dict[str, int],
    optional_value: Optional[str] = None,
) -> Union[int, str]:
    return sum(items)

class Container[T]:
    def __init__(self, value: T) -> None:
        self.value = value

    def get(self) -> T:
        return self.value
"#;
        let tree = parse(code, Language::Python).unwrap();
        assert!(!tree.root_node().has_error());
    }

    // ============================================
    // Invalid Syntax Tests (Partial Tree)
    // ============================================

    #[test]
    fn test_parse_rust_invalid_syntax_returns_partial_tree() {
        let code = r#"
            fn main() {
                let x = 5
                // Missing semicolon above
                let y = 10;
            }
        "#;
        let tree = parse(code, Language::Rust);
        // Tree-sitter should still return a tree, but with error nodes
        assert!(tree.is_some());
        let tree = tree.unwrap();
        // The tree should have error markers
        assert!(tree.root_node().has_error());
    }

    #[test]
    fn test_parse_rust_unclosed_brace() {
        let code = r#"
            fn main() {
                let x = 5;
            // Missing closing brace
        "#;
        let tree = parse(code, Language::Rust);
        assert!(tree.is_some());
        assert!(tree.unwrap().root_node().has_error());
    }

    #[test]
    fn test_parse_rust_invalid_token() {
        let code = r#"
            fn main() {
                let @ = 5;  // '@' is not valid here
            }
        "#;
        let tree = parse(code, Language::Rust);
        assert!(tree.is_some());
        assert!(tree.unwrap().root_node().has_error());
    }

    #[test]
    fn test_parse_python_invalid_indentation() {
        let code = r#"
def foo():
    if True:
  x = 1  # Wrong indentation
    y = 2
"#;
        let tree = parse(code, Language::Python);
        // Python parser should still return a tree
        assert!(tree.is_some());
        // Note: tree-sitter-python may not catch all indentation errors
        // but it should at least parse something
    }

    #[test]
    fn test_parse_python_unclosed_parenthesis() {
        let code = r#"
def foo():
    print("hello"
    x = 1
"#;
        let tree = parse(code, Language::Python);
        assert!(tree.is_some());
        assert!(tree.unwrap().root_node().has_error());
    }

    #[test]
    fn test_parse_python_invalid_syntax() {
        let code = r#"
def foo()  # Missing colon
    pass
"#;
        let tree = parse(code, Language::Python);
        assert!(tree.is_some());
        assert!(tree.unwrap().root_node().has_error());
    }

    // ============================================
    // Edge Cases
    // ============================================

    #[test]
    fn test_parse_empty_string() {
        let tree = parse("", Language::Rust);
        assert!(tree.is_some());
        let tree = tree.unwrap();
        assert_eq!(tree.root_node().kind(), "source_file");
        assert_eq!(tree.root_node().child_count(), 0);
    }

    #[test]
    fn test_parse_whitespace_only() {
        let tree = parse("   \n\n\t\t  \n", Language::Rust);
        assert!(tree.is_some());
    }

    #[test]
    fn test_parse_comments_only_rust() {
        let code = r#"
            // This is a comment
            /* This is a
               multiline comment */
            /// Doc comment
            //! Module doc
        "#;
        let tree = parse(code, Language::Rust);
        assert!(tree.is_some());
        assert!(!tree.unwrap().root_node().has_error());
    }

    #[test]
    fn test_parse_comments_only_python() {
        let code = r#"
# This is a comment
# Another comment

"""
This is a docstring
"""
"#;
        let tree = parse(code, Language::Python);
        assert!(tree.is_some());
    }

    #[test]
    fn test_parse_unicode_rust() {
        let code = r#"
            fn main() {
                let greeting = "Hello, 世界! 🌍";
                let émoji = "🦀";
                println!("{} {}", greeting, émoji);
            }
        "#;
        let tree = parse(code, Language::Rust);
        assert!(tree.is_some());
        assert!(!tree.unwrap().root_node().has_error());
    }

    #[test]
    fn test_parse_unicode_python() {
        let code = r#"
def greet():
    message = "Hello, 世界! 🌍"
    émoji = "🐍"
    print(f"{message} {émoji}")

# Identifier with unicode
π = 3.14159
"#;
        let tree = parse(code, Language::Python);
        assert!(tree.is_some());
        assert!(!tree.unwrap().root_node().has_error());
    }

    #[test]
    fn test_parse_very_long_line() {
        let long_string = "x".repeat(10000);
        let code = format!("fn main() {{ let s = \"{}\"; }}", long_string);
        let tree = parse(&code, Language::Rust);
        assert!(tree.is_some());
    }

    #[test]
    fn test_parse_deeply_nested_rust() {
        let code = r#"
            fn main() {
                if true {
                    if true {
                        if true {
                            if true {
                                if true {
                                    if true {
                                        if true {
                                            println!("deep");
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        "#;
        let tree = parse(code, Language::Rust);
        assert!(tree.is_some());
        assert!(!tree.unwrap().root_node().has_error());
    }

    #[test]
    fn test_parse_deeply_nested_python() {
        let code = r#"
def main():
    if True:
        if True:
            if True:
                if True:
                    if True:
                        if True:
                            if True:
                                print("deep")
"#;
        let tree = parse(code, Language::Python);
        assert!(tree.is_some());
        assert!(!tree.unwrap().root_node().has_error());
    }

    // ============================================
    // Unsupported Language Tests
    // ============================================

    #[test]
    fn test_parse_typescript() {
        let code = "const x: number = 5;";
        let tree = parse(code, Language::TypeScript);
        assert!(tree.is_some());
    }

    #[test]
    fn test_parse_javascript() {
        let code = "const x = 5;";
        let tree = parse(code, Language::JavaScript);
        assert!(tree.is_some());
    }

    #[test]
    fn test_parse_go() {
        let code = "package main\nfunc main() {}";
        let tree = parse(code, Language::Go);
        assert!(tree.is_some());
    }

    // ============================================
    // Parser Struct Tests
    // ============================================

    #[test]
    fn test_parser_new() {
        let parser = Parser::new();
        assert_eq!(parser.pool_size(), 0);
    }

    #[test]
    fn test_parser_pooling() {
        let parser = Parser::new();

        // First parse creates a parser for Rust
        let _ = parser.parse("fn main() {}", Language::Rust);
        assert_eq!(parser.pool_size(), 1);

        // Second parse reuses the same parser
        let _ = parser.parse("fn foo() {}", Language::Rust);
        assert_eq!(parser.pool_size(), 1);

        // Parsing a different language adds another parser
        let _ = parser.parse("def foo(): pass", Language::Python);
        assert_eq!(parser.pool_size(), 2);
    }

    #[test]
    fn test_parser_clear_pool() {
        let parser = Parser::new();
        let _ = parser.parse("fn main() {}", Language::Rust);
        let _ = parser.parse("def foo(): pass", Language::Python);
        assert_eq!(parser.pool_size(), 2);

        parser.clear_pool();
        assert_eq!(parser.pool_size(), 0);
    }

    #[test]
    fn test_parser_is_supported() {
        assert!(Parser::is_supported(Language::Rust));
        assert!(Parser::is_supported(Language::Python));
        assert!(Parser::is_supported(Language::Go));
        assert!(Parser::is_supported(Language::TypeScript));
        assert!(Parser::is_supported(Language::JavaScript));
    }

    #[test]
    fn test_parser_supported_languages() {
        let supported = Parser::supported_languages();
        assert!(supported.contains(&Language::Rust));
        assert!(supported.contains(&Language::Python));
        assert!(supported.contains(&Language::Go));
        assert!(supported.contains(&Language::TypeScript));
        assert!(supported.contains(&Language::JavaScript));
    }

    #[test]
    fn test_parser_default() {
        let parser = Parser::default();
        assert_eq!(parser.pool_size(), 0);
    }

    // ============================================
    // Incremental Parsing Tests
    // ============================================

    #[test]
    fn test_parse_with_old_tree() {
        let parser = Parser::new();

        // Initial parse
        let code_v1 = "fn main() { let x = 1; }";
        let tree_v1 = parser.parse(code_v1, Language::Rust).unwrap();

        // Incremental parse with old tree
        let code_v2 = "fn main() { let x = 2; }";
        let tree_v2 = parser
            .parse_with_old_tree(code_v2, Language::Rust, Some(&tree_v1))
            .unwrap();

        assert!(!tree_v2.root_node().has_error());
    }

    // ============================================
    // Integration with Symbol Extraction
    // ============================================

    #[test]
    fn test_extract_rust_symbols() {
        let code = r#"
            fn main() {
                println!("Hello, world!");
            }
        "#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);
        assert!(!symbols.is_empty());
        assert_eq!(symbols[0].name, "main");
    }

    // ============================================
    // Stress Tests
    // ============================================

    #[test]
    fn test_parse_many_functions_rust() {
        let mut code = String::new();
        for i in 0..100 {
            code.push_str(&format!("fn func_{}() {{}}\n", i));
        }
        let tree = parse(&code, Language::Rust);
        assert!(tree.is_some());
        assert!(!tree.unwrap().root_node().has_error());
    }

    #[test]
    fn test_parse_many_functions_python() {
        let mut code = String::new();
        for i in 0..100 {
            code.push_str(&format!("def func_{}():\n    pass\n\n", i));
        }
        let tree = parse(&code, Language::Python);
        assert!(tree.is_some());
        assert!(!tree.unwrap().root_node().has_error());
    }

    #[test]
    fn test_parser_reuse_many_times() {
        let parser = Parser::new();
        for i in 0..100 {
            let code = format!("fn func_{}() {{}}", i);
            let tree = parser.parse(&code, Language::Rust);
            assert!(tree.is_some());
        }
        // Should still only have one parser in the pool
        assert_eq!(parser.pool_size(), 1);
    }
}
