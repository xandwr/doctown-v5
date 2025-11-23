//! AST parsing and symbol extraction.
use doctown_common::Language;
use tree_sitter::{Parser, Tree};

pub fn parse(source_code: &str, language: Language) -> Option<Tree> {
    let mut parser = Parser::new();
    let ts_lang = match language {
        Language::Rust => tree_sitter_rust::language(),
        // Add other languages here
        _ => return None,
    };
    parser
        .set_language(&ts_lang)
        .expect("Error loading grammar");
    parser.parse(source_code, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::symbol::extract_symbols;
    use doctown_common::Language;

    #[test]
    fn test_parse_rust() {
        let code = r#"
            fn main() {
                println!("Hello, world!");
            }
        "#;
        let tree = parse(code, Language::Rust);
        assert!(tree.is_some());
        let unwrapped_tree = tree.unwrap();
        let root_node = unwrapped_tree.root_node();
        assert_eq!(root_node.kind(), "source_file");
    }

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
}
