//! Symbol extraction from ASTs.
use doctown_common::types::{ByteRange, SymbolKind};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Query, QueryCursor, Tree};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub kind: SymbolKind,
    pub name: String,
    pub range: ByteRange,
    pub name_range: ByteRange,
}

pub fn extract_symbols(
    tree: &Tree,
    source_code: &str,
    language: doctown_common::Language,
) -> Vec<Symbol> {
    let mut symbols = Vec::new();
    let query_str = match language {
        doctown_common::Language::Rust => "(function_item name: (identifier) @name)",
        _ => return symbols,
    };

    let ts_lang: tree_sitter::Language = tree_sitter_rust::LANGUAGE.into();
    let query = Query::new(&ts_lang, query_str).expect("Failed to create query");

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source_code.as_bytes());

    while let Some(mat) = matches.next() {
        for cap in mat.captures {
            let node = cap.node;
            let name_range = ByteRange {
                start: node.start_byte(),
                end: node.end_byte(),
            };
            let name = source_code[name_range.start..name_range.end].to_string();

            let range = if let Some(parent) = node.parent() {
                ByteRange {
                    start: parent.start_byte(),
                    end: parent.end_byte(),
                }
            } else {
                name_range
            };

            symbols.push(Symbol {
                kind: SymbolKind::Function,
                name,
                range,
                name_range,
            });
        }
    }

    symbols
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::parse;
    use doctown_common::Language;

    #[test]
    fn test_extract_rust_functions() {
        let code = r#"
            fn main() {
                println!("Hello, world!");
            }

            fn another_function() {
                // ...
            }
        "#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);
        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0].name, "main");
        assert_eq!(symbols[0].kind, SymbolKind::Function);
        assert_eq!(symbols[1].name, "another_function");
        assert_eq!(symbols[1].kind, SymbolKind::Function);
    }
}
