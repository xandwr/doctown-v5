//! Symbol extraction from ASTs.
use doctown_common::types::{ByteRange, SymbolKind, Visibility};
use tree_sitter::{Node, Tree};

use crate::traversal::{child_by_field, find_child_by_kind, find_nodes_by_kind, node_byte_range, node_text};

/// A symbol extracted from source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    /// The kind of symbol (function, struct, etc.)
    pub kind: SymbolKind,
    /// The name of the symbol
    pub name: String,
    /// Byte range of the entire symbol definition
    pub range: ByteRange,
    /// Byte range of just the symbol name
    pub name_range: ByteRange,
    /// The full signature (for functions: params + return type)
    pub signature: Option<String>,
    /// Visibility modifier (pub, pub(crate), private, etc.)
    pub visibility: Visibility,
    /// Whether this is an async function
    pub is_async: bool,
}

/// Extract all symbols from a parsed syntax tree.
pub fn extract_symbols(
    tree: &Tree,
    source_code: &str,
    language: doctown_common::Language,
) -> Vec<Symbol> {
    match language {
        doctown_common::Language::Rust => extract_rust_symbols(tree, source_code),
        _ => Vec::new(),
    }
}

/// Extract symbols from Rust source code.
fn extract_rust_symbols(tree: &Tree, source_code: &str) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    // Extract function definitions
    for func_node in find_nodes_by_kind(tree.root_node(), "function_item") {
        if let Some(symbol) = extract_rust_function(func_node, source_code) {
            symbols.push(symbol);
        }
    }

    symbols
}

/// Extract a single Rust function from a function_item node.
fn extract_rust_function(node: Node<'_>, source: &str) -> Option<Symbol> {
    // Get the function name
    let name_node = child_by_field(node, "name")?;
    let name = node_text(name_node, source).to_string();
    let name_range = node_byte_range(name_node);

    // Get the full range of the function
    let range = node_byte_range(node);

    // Check if async - look for function_modifiers child containing "async"
    let is_async = if let Some(mods_node) = find_child_by_kind(node, "function_modifiers") {
        node_text(mods_node, source).contains("async")
    } else {
        false
    };

    // Extract visibility
    let visibility = extract_visibility(node, source);

    // Extract signature (everything from name to body, excluding body)
    let signature = extract_function_signature(node, source);

    Some(Symbol {
        kind: SymbolKind::Function,
        name,
        range,
        name_range,
        signature,
        visibility,
        is_async,
    })
}

/// Extract visibility from a node that may have a visibility_modifier child.
fn extract_visibility(node: Node<'_>, source: &str) -> Visibility {
    let vis_node = match find_child_by_kind(node, "visibility_modifier") {
        Some(n) => n,
        None => return Visibility::Private,
    };

    let vis_text = node_text(vis_node, source);

    // Parse the visibility modifier
    match vis_text {
        "pub" => Visibility::Public,
        s if s.starts_with("pub(crate)") => Visibility::PublicCrate,
        s if s.starts_with("pub(super)") => Visibility::PublicSuper,
        s if s.starts_with("pub(self)") => Visibility::PublicSelf,
        s if s.starts_with("pub(in") => Visibility::PublicIn,
        _ => Visibility::Private,
    }
}

/// Extract the function signature (name + generics + params + return type).
fn extract_function_signature(node: Node<'_>, source: &str) -> Option<String> {
    let name_node = child_by_field(node, "name")?;
    let body_node = child_by_field(node, "body");

    // Signature spans from name to just before body (or end of node if no body)
    let sig_start = name_node.start_byte();
    let sig_end = match body_node {
        Some(body) => body.start_byte(),
        None => node.end_byte(),
    };

    let signature = source[sig_start..sig_end].trim().to_string();
    Some(signature)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::parse;
    use doctown_common::Language;

    #[test]
    fn test_extract_simple_function() {
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

        // First function
        assert_eq!(symbols[0].name, "main");
        assert_eq!(symbols[0].kind, SymbolKind::Function);
        assert!(!symbols[0].is_async);
        assert_eq!(symbols[0].visibility, Visibility::Private);
        assert_eq!(symbols[0].signature.as_deref(), Some("main()"));

        // Second function
        assert_eq!(symbols[1].name, "another_function");
        assert_eq!(symbols[1].kind, SymbolKind::Function);
        assert!(!symbols[1].is_async);
        assert_eq!(symbols[1].visibility, Visibility::Private);
        assert_eq!(symbols[1].signature.as_deref(), Some("another_function()"));
    }

    #[test]
    fn test_extract_async_function() {
        let code = r#"
async fn fetch_data() -> Result<String, Error> {
    Ok("data".to_string())
}

async fn process() {
    let _ = fetch_data().await;
}
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        assert_eq!(symbols.len(), 2);

        // First async function
        assert_eq!(symbols[0].name, "fetch_data");
        assert!(symbols[0].is_async);
        assert_eq!(symbols[0].visibility, Visibility::Private);
        assert_eq!(
            symbols[0].signature.as_deref(),
            Some("fetch_data() -> Result<String, Error>")
        );

        // Second async function
        assert_eq!(symbols[1].name, "process");
        assert!(symbols[1].is_async);
    }

    #[test]
    fn test_extract_generic_function() {
        let code = r#"
fn identity<T>(value: T) -> T {
    value
}

fn swap<T, U>(a: T, b: U) -> (U, T) {
    (b, a)
}

fn with_bounds<T: Clone + Debug>(value: T) -> T {
    value.clone()
}

fn with_where<T>(value: T) -> T
where
    T: Clone + Send,
{
    value
}
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        assert_eq!(symbols.len(), 4);

        // identity<T>
        assert_eq!(symbols[0].name, "identity");
        assert_eq!(
            symbols[0].signature.as_deref(),
            Some("identity<T>(value: T) -> T")
        );

        // swap<T, U>
        assert_eq!(symbols[1].name, "swap");
        assert_eq!(
            symbols[1].signature.as_deref(),
            Some("swap<T, U>(a: T, b: U) -> (U, T)")
        );

        // with_bounds<T: Clone + Debug>
        assert_eq!(symbols[2].name, "with_bounds");
        assert!(symbols[2].signature.as_ref().unwrap().contains("with_bounds<T: Clone + Debug>"));

        // with_where clause
        assert_eq!(symbols[3].name, "with_where");
        assert!(symbols[3].signature.as_ref().unwrap().contains("where"));
    }

    #[test]
    fn test_extract_function_with_lifetime_params() {
        let code = r#"
fn first<'a>(items: &'a [i32]) -> &'a i32 {
    &items[0]
}

fn longest<'a, 'b: 'a>(x: &'a str, y: &'b str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}

fn mixed<'a, T>(data: &'a T) -> &'a T
where
    T: Debug,
{
    data
}
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        assert_eq!(symbols.len(), 3);

        // first<'a>
        assert_eq!(symbols[0].name, "first");
        assert!(symbols[0].signature.as_ref().unwrap().contains("'a"));
        assert_eq!(
            symbols[0].signature.as_deref(),
            Some("first<'a>(items: &'a [i32]) -> &'a i32")
        );

        // longest<'a, 'b: 'a>
        assert_eq!(symbols[1].name, "longest");
        assert!(symbols[1].signature.as_ref().unwrap().contains("'a"));
        assert!(symbols[1].signature.as_ref().unwrap().contains("'b"));

        // mixed<'a, T> with where clause
        assert_eq!(symbols[2].name, "mixed");
        assert!(symbols[2].signature.as_ref().unwrap().contains("'a"));
        assert!(symbols[2].signature.as_ref().unwrap().contains("where"));
    }

    #[test]
    fn test_extract_function_visibility() {
        let code = r#"
fn private_fn() {}

pub fn public_fn() {}

pub(crate) fn crate_fn() {}

pub(super) fn super_fn() {}

pub(self) fn self_fn() {}
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        assert_eq!(symbols.len(), 5);

        assert_eq!(symbols[0].name, "private_fn");
        assert_eq!(symbols[0].visibility, Visibility::Private);

        assert_eq!(symbols[1].name, "public_fn");
        assert_eq!(symbols[1].visibility, Visibility::Public);

        assert_eq!(symbols[2].name, "crate_fn");
        assert_eq!(symbols[2].visibility, Visibility::PublicCrate);

        assert_eq!(symbols[3].name, "super_fn");
        assert_eq!(symbols[3].visibility, Visibility::PublicSuper);

        assert_eq!(symbols[4].name, "self_fn");
        assert_eq!(symbols[4].visibility, Visibility::PublicSelf);
    }

    #[test]
    fn test_extract_function_with_complex_params() {
        let code = r#"
fn complex(
    name: String,
    count: usize,
    callback: impl Fn(i32) -> bool,
) -> Option<Vec<String>> {
    None
}
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "complex");
        assert!(symbols[0].signature.as_ref().unwrap().contains("name: String"));
        assert!(symbols[0].signature.as_ref().unwrap().contains("-> Option<Vec<String>>"));
    }

    #[test]
    fn test_extract_function_byte_ranges() {
        let code = "fn foo() {}\nfn bar() {}";
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        assert_eq!(symbols.len(), 2);

        // First function: "fn foo() {}"
        assert_eq!(&code[symbols[0].range.start..symbols[0].range.end], "fn foo() {}");
        assert_eq!(&code[symbols[0].name_range.start..symbols[0].name_range.end], "foo");

        // Second function: "fn bar() {}"
        assert_eq!(&code[symbols[1].range.start..symbols[1].range.end], "fn bar() {}");
        assert_eq!(&code[symbols[1].name_range.start..symbols[1].name_range.end], "bar");
    }

    #[test]
    fn test_extract_combined_async_pub_generic_lifetime() {
        let code = r#"
pub async fn fetch<'a, T: Deserialize<'a>>(url: &'a str) -> Result<T, Error> {
    todo!()
}
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        assert_eq!(symbols.len(), 1);

        let sym = &symbols[0];
        assert_eq!(sym.name, "fetch");
        assert!(sym.is_async);
        assert_eq!(sym.visibility, Visibility::Public);
        assert!(sym.signature.as_ref().unwrap().contains("'a"));
        assert!(sym.signature.as_ref().unwrap().contains("T: Deserialize"));
        assert!(sym.signature.as_ref().unwrap().contains("-> Result<T, Error>"));
    }

    #[test]
    fn test_unsupported_language_returns_empty() {
        let code = "def foo(): pass";
        let tree = parse(code, Language::Python).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Python);

        assert!(symbols.is_empty());
    }
}
