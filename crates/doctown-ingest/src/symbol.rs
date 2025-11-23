//! Symbol extraction from ASTs.
use doctown_common::types::{ByteRange, SymbolKind, Visibility};
use tree_sitter::{Node, Tree};

use crate::traversal::{
    ancestors, child_by_field, find_child_by_kind, find_nodes_by_kind, node_byte_range, node_text,
};

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
        doctown_common::Language::Python => extract_python_symbols(tree, source_code),
        _ => Vec::new(),
    }
}

/// Check if a node is inside an impl or trait block.
fn is_inside_impl_or_trait(node: Node<'_>) -> bool {
    ancestors(node).any(|n| n.kind() == "impl_item" || n.kind() == "trait_item")
}

/// Extract symbols from Rust source code.
fn extract_rust_symbols(tree: &Tree, source_code: &str) -> Vec<Symbol> {
    let mut symbols = Vec::new();
    let root = tree.root_node();

    // Extract function definitions (only top-level, not inside impl/trait blocks)
    for node in find_nodes_by_kind(root, "function_item") {
        // Skip functions that are inside impl or trait blocks
        if is_inside_impl_or_trait(node) {
            continue;
        }
        if let Some(symbol) = extract_rust_function(node, source_code) {
            symbols.push(symbol);
        }
    }

    // Extract struct definitions
    for node in find_nodes_by_kind(root, "struct_item") {
        if let Some(symbol) = extract_rust_struct(node, source_code) {
            symbols.push(symbol);
        }
    }

    // Extract enum definitions
    for node in find_nodes_by_kind(root, "enum_item") {
        if let Some(symbol) = extract_rust_enum(node, source_code) {
            symbols.push(symbol);
        }
    }

    // Extract trait definitions
    for node in find_nodes_by_kind(root, "trait_item") {
        if let Some(symbol) = extract_rust_trait(node, source_code) {
            symbols.push(symbol);
        }
    }

    // Extract impl blocks
    for node in find_nodes_by_kind(root, "impl_item") {
        if let Some(symbol) = extract_rust_impl(node, source_code) {
            symbols.push(symbol);
        }
    }

    // Extract module declarations
    for node in find_nodes_by_kind(root, "mod_item") {
        if let Some(symbol) = extract_rust_module(node, source_code) {
            symbols.push(symbol);
        }
    }

    // Extract const items
    for node in find_nodes_by_kind(root, "const_item") {
        if let Some(symbol) = extract_rust_const(node, source_code) {
            symbols.push(symbol);
        }
    }

    // Extract static items
    for node in find_nodes_by_kind(root, "static_item") {
        if let Some(symbol) = extract_rust_static(node, source_code) {
            symbols.push(symbol);
        }
    }

    // Extract type aliases
    for node in find_nodes_by_kind(root, "type_item") {
        if let Some(symbol) = extract_rust_type_alias(node, source_code) {
            symbols.push(symbol);
        }
    }

    // Extract macro definitions
    for node in find_nodes_by_kind(root, "macro_definition") {
        if let Some(symbol) = extract_rust_macro(node, source_code) {
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

/// Extract a Rust struct definition.
fn extract_rust_struct(node: Node<'_>, source: &str) -> Option<Symbol> {
    // Try to find the struct name
    let name_node =
        child_by_field(node, "name").or_else(|| find_child_by_kind(node, "type_identifier"));
    let name = if let Some(n) = name_node {
        node_text(n, source).to_string()
    } else {
        "".to_string()
    };
    let name_range = name_node
        .map(node_byte_range)
        .unwrap_or(ByteRange::new(node.start_byte(), node.start_byte()));

    let range = node_byte_range(node);

    let visibility = extract_visibility(node, source);

    // Build a compact signature for the struct: from name to end of node
    let sig_start = name_node
        .map(|n| n.start_byte())
        .unwrap_or(node.start_byte());
    let signature = Some(source[sig_start..node.end_byte()].trim().to_string());

    Some(Symbol {
        kind: SymbolKind::Struct,
        name,
        range,
        name_range,
        signature,
        visibility,
        is_async: false,
    })
}

/// Extract a Rust enum definition.
fn extract_rust_enum(node: Node<'_>, source: &str) -> Option<Symbol> {
    let name_node =
        child_by_field(node, "name").or_else(|| find_child_by_kind(node, "type_identifier"))?;
    let name = node_text(name_node, source).to_string();
    let name_range = node_byte_range(name_node);
    let range = node_byte_range(node);
    let visibility = extract_visibility(node, source);

    // Signature includes the whole enum definition
    let sig_start = name_node.start_byte();
    let signature = Some(source[sig_start..node.end_byte()].trim().to_string());

    Some(Symbol {
        kind: SymbolKind::Enum,
        name,
        range,
        name_range,
        signature,
        visibility,
        is_async: false,
    })
}

/// Extract a Rust trait definition.
fn extract_rust_trait(node: Node<'_>, source: &str) -> Option<Symbol> {
    let name_node =
        child_by_field(node, "name").or_else(|| find_child_by_kind(node, "type_identifier"))?;
    let name = node_text(name_node, source).to_string();
    let name_range = node_byte_range(name_node);
    let range = node_byte_range(node);
    let visibility = extract_visibility(node, source);

    // Signature includes the whole trait definition
    let sig_start = name_node.start_byte();
    let signature = Some(source[sig_start..node.end_byte()].trim().to_string());

    Some(Symbol {
        kind: SymbolKind::Trait,
        name,
        range,
        name_range,
        signature,
        visibility,
        is_async: false,
    })
}

/// Extract a Rust impl block.
fn extract_rust_impl(node: Node<'_>, source: &str) -> Option<Symbol> {
    let range = node_byte_range(node);

    // Get the type being implemented (the "type" field)
    let type_node = child_by_field(node, "type")?;
    let type_text = node_text(type_node, source);

    // Check if this is a trait impl by looking for "trait" field
    let trait_node = child_by_field(node, "trait");
    let name = if let Some(trait_n) = trait_node {
        let trait_text = node_text(trait_n, source);
        format!("{} for {}", trait_text, type_text)
    } else {
        type_text.to_string()
    };

    // Name range is the type being implemented
    let name_range = node_byte_range(type_node);

    // Build signature from "impl" keyword to just before the body
    let body_node = child_by_field(node, "body");
    let sig_end = body_node.map(|b| b.start_byte()).unwrap_or(node.end_byte());
    let signature = Some(source[node.start_byte()..sig_end].trim().to_string());

    Some(Symbol {
        kind: SymbolKind::Impl,
        name,
        range,
        name_range,
        signature,
        visibility: Visibility::Private, // impl blocks don't have visibility
        is_async: false,
    })
}

/// Extract a Rust module declaration.
fn extract_rust_module(node: Node<'_>, source: &str) -> Option<Symbol> {
    let name_node =
        child_by_field(node, "name").or_else(|| find_child_by_kind(node, "identifier"))?;
    let name = node_text(name_node, source).to_string();
    let name_range = node_byte_range(name_node);
    let range = node_byte_range(node);
    let visibility = extract_visibility(node, source);

    // Check if this is an inline module (has a body) or file module (just declaration)
    let body_node = child_by_field(node, "body");
    let is_inline = body_node.is_some();

    // Signature: just "mod name" for file modules, full definition for inline
    let signature = if is_inline {
        Some(
            source[name_node.start_byte()..node.end_byte()]
                .trim()
                .to_string(),
        )
    } else {
        Some(format!("mod {}", name))
    };

    Some(Symbol {
        kind: SymbolKind::Module,
        name,
        range,
        name_range,
        signature,
        visibility,
        is_async: false,
    })
}

/// Extract a Rust const item.
fn extract_rust_const(node: Node<'_>, source: &str) -> Option<Symbol> {
    let name_node =
        child_by_field(node, "name").or_else(|| find_child_by_kind(node, "identifier"))?;
    let name = node_text(name_node, source).to_string();
    let name_range = node_byte_range(name_node);
    let range = node_byte_range(node);
    let visibility = extract_visibility(node, source);

    // Signature is the full const declaration
    let signature = Some(node_text(node, source).trim().to_string());

    Some(Symbol {
        kind: SymbolKind::Const,
        name,
        range,
        name_range,
        signature,
        visibility,
        is_async: false,
    })
}

/// Extract a Rust static item.
fn extract_rust_static(node: Node<'_>, source: &str) -> Option<Symbol> {
    let name_node =
        child_by_field(node, "name").or_else(|| find_child_by_kind(node, "identifier"))?;
    let name = node_text(name_node, source).to_string();
    let name_range = node_byte_range(name_node);
    let range = node_byte_range(node);
    let visibility = extract_visibility(node, source);

    // Signature is the full static declaration
    let signature = Some(node_text(node, source).trim().to_string());

    Some(Symbol {
        kind: SymbolKind::Static,
        name,
        range,
        name_range,
        signature,
        visibility,
        is_async: false,
    })
}

/// Extract a Rust type alias.
fn extract_rust_type_alias(node: Node<'_>, source: &str) -> Option<Symbol> {
    let name_node =
        child_by_field(node, "name").or_else(|| find_child_by_kind(node, "type_identifier"))?;
    let name = node_text(name_node, source).to_string();
    let name_range = node_byte_range(name_node);
    let range = node_byte_range(node);
    let visibility = extract_visibility(node, source);

    // Signature is the full type alias
    let signature = Some(node_text(node, source).trim().to_string());

    Some(Symbol {
        kind: SymbolKind::TypeAlias,
        name,
        range,
        name_range,
        signature,
        visibility,
        is_async: false,
    })
}

/// Extract a Rust macro_rules! definition.
fn extract_rust_macro(node: Node<'_>, source: &str) -> Option<Symbol> {
    let name_node =
        child_by_field(node, "name").or_else(|| find_child_by_kind(node, "identifier"))?;
    let name = node_text(name_node, source).to_string();
    let name_range = node_byte_range(name_node);
    let range = node_byte_range(node);

    // Macros use #[macro_export] for visibility, but we'll mark as private by default
    let visibility = Visibility::Private;

    // Signature is just the macro name for brevity
    let signature = Some(format!("macro_rules! {}", name));

    Some(Symbol {
        kind: SymbolKind::Macro,
        name,
        range,
        name_range,
        signature,
        visibility,
        is_async: false,
    })
}

// ============================================
// Python Symbol Extraction
// ============================================

/// Check if a Python node is inside a class definition.
fn is_inside_class(node: Node<'_>) -> bool {
    ancestors(node).any(|n| n.kind() == "class_definition")
}

/// Extract symbols from Python source code.
fn extract_python_symbols(tree: &Tree, source_code: &str) -> Vec<Symbol> {
    let mut symbols = Vec::new();
    let root = tree.root_node();

    // Extract function definitions (only top-level, not inside classes - those are methods)
    for node in find_nodes_by_kind(root, "function_definition") {
        if is_inside_class(node) {
            continue;
        }
        if let Some(symbol) = extract_python_function(node, source_code) {
            symbols.push(symbol);
        }
    }

    // Extract class definitions
    for node in find_nodes_by_kind(root, "class_definition") {
        if let Some(symbol) = extract_python_class(node, source_code) {
            symbols.push(symbol);
        }
    }

    // Extract module-level assignments (constants)
    for node in find_nodes_by_kind(root, "expression_statement") {
        // Only process top-level assignments
        if node.parent().map(|p| p.kind()) != Some("module") {
            continue;
        }
        if let Some(assignment) = find_child_by_kind(node, "assignment") {
            if let Some(symbol) = extract_python_assignment(assignment, source_code) {
                symbols.push(symbol);
            }
        }
    }

    symbols
}

/// Extract a Python function definition.
fn extract_python_function(node: Node<'_>, source: &str) -> Option<Symbol> {
    // Get the function name
    let name_node = child_by_field(node, "name")?;
    let name = node_text(name_node, source).to_string();
    let name_range = node_byte_range(name_node);

    // Get the full range of the function (including decorators)
    let range = node_byte_range(node);

    // Check if async - the node kind for async functions is still "function_definition"
    // but they have an "async" keyword as a child
    let is_async = find_child_by_kind(node, "async").is_some();

    // Extract signature (function name + parameters + return type annotation)
    let signature = extract_python_function_signature(node, source);

    // Get decorators
    let _decorators = extract_python_decorators(node, source);

    Some(Symbol {
        kind: SymbolKind::Function,
        name,
        range,
        name_range,
        signature,
        visibility: Visibility::Public, // Python doesn't have visibility modifiers
        is_async,
    })
}

/// Extract the signature of a Python function.
fn extract_python_function_signature(node: Node<'_>, source: &str) -> Option<String> {
    let name_node = child_by_field(node, "name")?;
    let params_node = child_by_field(node, "parameters")?;

    let name = node_text(name_node, source);
    let params = node_text(params_node, source);

    // Check for return type annotation
    let return_type =
        child_by_field(node, "return_type").map(|n| format!(" -> {}", node_text(n, source)));

    Some(format!(
        "{}{}{}",
        name,
        params,
        return_type.unwrap_or_default()
    ))
}

/// Extract decorators from a Python function or class.
fn extract_python_decorators(node: Node<'_>, source: &str) -> Vec<String> {
    let mut decorators = Vec::new();

    // Look for decorator siblings that appear before this node
    // In tree-sitter-python, decorators are children of a decorated_definition node
    if let Some(parent) = node.parent() {
        if parent.kind() == "decorated_definition" {
            for i in 0..parent.child_count() {
                if let Some(child) = parent.child(i) {
                    if child.kind() == "decorator" {
                        decorators.push(node_text(child, source).to_string());
                    }
                }
            }
        }
    }

    decorators
}

/// Extract a Python class definition.
fn extract_python_class(node: Node<'_>, source: &str) -> Option<Symbol> {
    let name_node = child_by_field(node, "name")?;
    let name = node_text(name_node, source).to_string();
    let name_range = node_byte_range(name_node);
    let range = node_byte_range(node);

    // Build signature: class name + base classes
    let signature = extract_python_class_signature(node, source);

    Some(Symbol {
        kind: SymbolKind::Class,
        name,
        range,
        name_range,
        signature,
        visibility: Visibility::Public,
        is_async: false,
    })
}

/// Extract the signature of a Python class.
fn extract_python_class_signature(node: Node<'_>, source: &str) -> Option<String> {
    let name_node = child_by_field(node, "name")?;
    let name = node_text(name_node, source);

    // Check for superclasses (argument_list contains base classes)
    let superclasses =
        child_by_field(node, "superclasses").map(|n| node_text(n, source).to_string());

    match superclasses {
        Some(supers) => Some(format!("{}{}", name, supers)),
        None => Some(name.to_string()),
    }
}

/// Extract a Python module-level assignment (constant).
fn extract_python_assignment(node: Node<'_>, source: &str) -> Option<Symbol> {
    // Get the left side of the assignment (the name)
    let left_node = child_by_field(node, "left")?;

    // Only extract simple identifier assignments, not tuple unpacking etc.
    if left_node.kind() != "identifier" {
        return None;
    }

    let name = node_text(left_node, source).to_string();

    // Skip dunder attributes (they're typically not user-defined constants)
    // except for __all__ which we want to capture
    if name.starts_with("__") && name.ends_with("__") && name != "__all__" {
        return None;
    }

    let name_range = node_byte_range(left_node);
    let range = node_byte_range(node);

    // Get the type annotation if present
    let type_annotation =
        child_by_field(node, "type").map(|n| format!(": {}", node_text(n, source)));

    // Build signature
    let signature = match type_annotation {
        Some(ann) => Some(format!("{}{}", name, ann)),
        None => Some(name.clone()),
    };

    Some(Symbol {
        kind: SymbolKind::Const, // Using Const for module-level assignments
        name,
        range,
        name_range,
        signature,
        visibility: Visibility::Public,
        is_async: false,
    })
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
        assert!(symbols[2]
            .signature
            .as_ref()
            .unwrap()
            .contains("with_bounds<T: Clone + Debug>"));

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
        assert!(symbols[0]
            .signature
            .as_ref()
            .unwrap()
            .contains("name: String"));
        assert!(symbols[0]
            .signature
            .as_ref()
            .unwrap()
            .contains("-> Option<Vec<String>>"));
    }

    #[test]
    fn test_extract_function_byte_ranges() {
        let code = "fn foo() {}\nfn bar() {}";
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        assert_eq!(symbols.len(), 2);

        // First function: "fn foo() {}"
        assert_eq!(
            &code[symbols[0].range.start..symbols[0].range.end],
            "fn foo() {}"
        );
        assert_eq!(
            &code[symbols[0].name_range.start..symbols[0].name_range.end],
            "foo"
        );

        // Second function: "fn bar() {}"
        assert_eq!(
            &code[symbols[1].range.start..symbols[1].range.end],
            "fn bar() {}"
        );
        assert_eq!(
            &code[symbols[1].name_range.start..symbols[1].name_range.end],
            "bar"
        );
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
        assert!(sym
            .signature
            .as_ref()
            .unwrap()
            .contains("-> Result<T, Error>"));
    }

    #[test]
    fn test_unsupported_language_returns_empty() {
        // Go is in the Language enum but has no extraction implemented yet
        // We test the fallback path by parsing with Rust grammar
        // but extracting with Go language (which has no extractor)
        let rust_code = "fn main() {}";
        let tree = parse(rust_code, Language::Rust).unwrap();
        // Force Go language to test the fallback path
        let symbols = extract_symbols(&tree, rust_code, Language::Go);

        assert!(symbols.is_empty());
    }

    #[test]
    fn test_extract_simple_struct() {
        let code = r#"
struct Point {
    x: f64,
    y: f64,
}
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        assert_eq!(symbols.len(), 1);
        let s = &symbols[0];
        assert_eq!(s.name, "Point");
        assert_eq!(s.kind, SymbolKind::Struct);
        assert_eq!(s.visibility, doctown_common::types::Visibility::Private);
        // Signature should contain field names
        let sig = s.signature.as_ref().unwrap();
        assert!(sig.contains("x"));
        assert!(sig.contains("y"));
    }

    #[test]
    fn test_extract_tuple_struct() {
        let code = r#"
struct Pair(i32, i32);
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        assert_eq!(symbols.len(), 1);
        let s = &symbols[0];
        assert_eq!(s.name, "Pair");
        assert_eq!(s.kind, SymbolKind::Struct);
        let sig = s.signature.as_ref().unwrap();
        assert!(sig.contains("i32"));
    }

    #[test]
    fn test_extract_generic_struct() {
        let code = r#"
struct Container<'a, T: Clone> {
    data: &'a T,
}
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        assert_eq!(symbols.len(), 1);
        let s = &symbols[0];
        assert_eq!(s.name, "Container");
        assert_eq!(s.kind, SymbolKind::Struct);
        let sig = s.signature.as_ref().unwrap();
        assert!(sig.contains("<'a"));
        assert!(sig.contains("T: Clone"));
    }

    // ============================================
    // Enum Extraction Tests
    // ============================================

    #[test]
    fn test_extract_simple_enum() {
        let code = r#"
enum Color {
    Red,
    Green,
    Blue,
}
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        assert_eq!(symbols.len(), 1);
        let s = &symbols[0];
        assert_eq!(s.name, "Color");
        assert_eq!(s.kind, SymbolKind::Enum);
        assert_eq!(s.visibility, Visibility::Private);
        let sig = s.signature.as_ref().unwrap();
        assert!(sig.contains("Red"));
        assert!(sig.contains("Green"));
        assert!(sig.contains("Blue"));
    }

    #[test]
    fn test_extract_enum_with_data_variants() {
        let code = r#"
pub enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        assert_eq!(symbols.len(), 1);
        let s = &symbols[0];
        assert_eq!(s.name, "Message");
        assert_eq!(s.kind, SymbolKind::Enum);
        assert_eq!(s.visibility, Visibility::Public);
        let sig = s.signature.as_ref().unwrap();
        assert!(sig.contains("Quit"));
        assert!(sig.contains("Move"));
        assert!(sig.contains("Write"));
        assert!(sig.contains("ChangeColor"));
    }

    // ============================================
    // Trait Extraction Tests
    // ============================================

    #[test]
    fn test_extract_trait_with_methods() {
        let code = r#"
pub trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        assert_eq!(symbols.len(), 1);
        let s = &symbols[0];
        assert_eq!(s.name, "Iterator");
        assert_eq!(s.kind, SymbolKind::Trait);
        assert_eq!(s.visibility, Visibility::Public);
        let sig = s.signature.as_ref().unwrap();
        assert!(sig.contains("fn next"));
        assert!(sig.contains("fn size_hint"));
    }

    // ============================================
    // Impl Block Extraction Tests
    // ============================================

    #[test]
    fn test_extract_inherent_impl() {
        let code = r#"
struct Point { x: i32, y: i32 }

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn distance(&self) -> f64 {
        ((self.x.pow(2) + self.y.pow(2)) as f64).sqrt()
    }
}
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        // Should have struct + impl
        let impls: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Impl)
            .collect();
        assert_eq!(impls.len(), 1);

        let impl_sym = impls[0];
        assert_eq!(impl_sym.name, "Point");
        assert_eq!(impl_sym.kind, SymbolKind::Impl);
        assert!(impl_sym
            .signature
            .as_ref()
            .unwrap()
            .starts_with("impl Point"));
    }

    #[test]
    fn test_extract_trait_impl() {
        let code = r#"
struct MyStruct;

impl Default for MyStruct {
    fn default() -> Self {
        MyStruct
    }
}

impl Clone for MyStruct {
    fn clone(&self) -> Self {
        MyStruct
    }
}
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        let impls: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Impl)
            .collect();
        assert_eq!(impls.len(), 2);

        // Check Default impl
        let default_impl = impls.iter().find(|s| s.name.contains("Default")).unwrap();
        assert_eq!(default_impl.name, "Default for MyStruct");
        assert!(default_impl
            .signature
            .as_ref()
            .unwrap()
            .contains("impl Default for MyStruct"));

        // Check Clone impl
        let clone_impl = impls.iter().find(|s| s.name.contains("Clone")).unwrap();
        assert_eq!(clone_impl.name, "Clone for MyStruct");
    }

    // ============================================
    // Module Extraction Tests
    // ============================================

    #[test]
    fn test_extract_inline_module() {
        let code = r#"
mod utils {
    pub fn helper() {}
}
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        let mods: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Module)
            .collect();
        assert_eq!(mods.len(), 1);

        let m = mods[0];
        assert_eq!(m.name, "utils");
        assert_eq!(m.kind, SymbolKind::Module);
        assert_eq!(m.visibility, Visibility::Private);
        // Inline module signature includes body
        assert!(m.signature.as_ref().unwrap().contains("utils"));
    }

    #[test]
    fn test_extract_file_module_declaration() {
        let code = r#"
pub mod parser;
mod lexer;
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        let mods: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Module)
            .collect();
        assert_eq!(mods.len(), 2);

        let parser_mod = mods.iter().find(|s| s.name == "parser").unwrap();
        assert_eq!(parser_mod.visibility, Visibility::Public);
        assert_eq!(parser_mod.signature.as_deref(), Some("mod parser"));

        let lexer_mod = mods.iter().find(|s| s.name == "lexer").unwrap();
        assert_eq!(lexer_mod.visibility, Visibility::Private);
        assert_eq!(lexer_mod.signature.as_deref(), Some("mod lexer"));
    }

    // ============================================
    // Const/Static/TypeAlias/Macro Tests
    // ============================================

    #[test]
    fn test_extract_const_items() {
        let code = r#"
const PI: f64 = 3.14159;
pub const MAX_SIZE: usize = 1024;
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        let consts: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Const)
            .collect();
        assert_eq!(consts.len(), 2);

        let pi = consts.iter().find(|s| s.name == "PI").unwrap();
        assert_eq!(pi.visibility, Visibility::Private);
        assert!(pi.signature.as_ref().unwrap().contains("f64"));

        let max_size = consts.iter().find(|s| s.name == "MAX_SIZE").unwrap();
        assert_eq!(max_size.visibility, Visibility::Public);
        assert!(max_size.signature.as_ref().unwrap().contains("usize"));
    }

    #[test]
    fn test_extract_static_items() {
        let code = r#"
static COUNTER: AtomicUsize = AtomicUsize::new(0);
pub static mut BUFFER: [u8; 1024] = [0; 1024];
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        let statics: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Static)
            .collect();
        assert_eq!(statics.len(), 2);

        let counter = statics.iter().find(|s| s.name == "COUNTER").unwrap();
        assert_eq!(counter.visibility, Visibility::Private);
        assert!(counter.signature.as_ref().unwrap().contains("AtomicUsize"));

        let buffer = statics.iter().find(|s| s.name == "BUFFER").unwrap();
        assert_eq!(buffer.visibility, Visibility::Public);
        assert!(buffer.signature.as_ref().unwrap().contains("mut"));
    }

    #[test]
    fn test_extract_type_alias() {
        let code = r#"
type Result<T> = std::result::Result<T, Error>;
pub type Callback = Box<dyn Fn(i32) -> bool>;
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        let types: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::TypeAlias)
            .collect();
        assert_eq!(types.len(), 2);

        let result = types.iter().find(|s| s.name == "Result").unwrap();
        assert_eq!(result.visibility, Visibility::Private);
        assert!(result.signature.as_ref().unwrap().contains("type Result"));

        let callback = types.iter().find(|s| s.name == "Callback").unwrap();
        assert_eq!(callback.visibility, Visibility::Public);
        assert!(callback.signature.as_ref().unwrap().contains("Box<dyn Fn"));
    }

    #[test]
    fn test_extract_macro_definition() {
        let code = r#"
macro_rules! vec {
    () => { Vec::new() };
    ($($x:expr),+) => { { let mut v = Vec::new(); $(v.push($x);)+ v } };
}

macro_rules! println {
    () => { print!("\n") };
    ($($arg:tt)*) => { print!("{}\n", format_args!($($arg)*)) };
}
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        let macros: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Macro)
            .collect();
        assert_eq!(macros.len(), 2);

        let vec_macro = macros.iter().find(|s| s.name == "vec").unwrap();
        assert_eq!(vec_macro.signature.as_deref(), Some("macro_rules! vec"));

        let println_macro = macros.iter().find(|s| s.name == "println").unwrap();
        assert_eq!(
            println_macro.signature.as_deref(),
            Some("macro_rules! println")
        );
    }

    // ============================================
    // Integration Test
    // ============================================

    #[test]
    fn test_extract_all_rust_items() {
        let code = r#"
// A comprehensive Rust file with all item types

pub mod utils;

const VERSION: &str = "1.0.0";
pub static INSTANCE_COUNT: AtomicUsize = AtomicUsize::new(0);

type BoxedError = Box<dyn std::error::Error>;

pub struct Config {
    pub name: String,
    pub value: i32,
}

pub enum Status {
    Active,
    Inactive,
    Pending(String),
}

pub trait Processor {
    fn process(&self) -> Result<(), BoxedError>;
}

impl Config {
    pub fn new(name: String, value: i32) -> Self {
        Self { name, value }
    }
}

impl Processor for Config {
    fn process(&self) -> Result<(), BoxedError> {
        Ok(())
    }
}

pub fn main() {
    println!("Hello!");
}

pub async fn async_main() {
    todo!()
}

macro_rules! my_macro {
    () => {};
}
"#;
        let tree = parse(code, Language::Rust).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Rust);

        // Count each type
        let count_kind = |kind: SymbolKind| symbols.iter().filter(|s| s.kind == kind).count();

        assert_eq!(count_kind(SymbolKind::Module), 1, "Should have 1 module");
        assert_eq!(count_kind(SymbolKind::Const), 1, "Should have 1 const");
        assert_eq!(count_kind(SymbolKind::Static), 1, "Should have 1 static");
        assert_eq!(
            count_kind(SymbolKind::TypeAlias),
            1,
            "Should have 1 type alias"
        );
        assert_eq!(count_kind(SymbolKind::Struct), 1, "Should have 1 struct");
        assert_eq!(count_kind(SymbolKind::Enum), 1, "Should have 1 enum");
        assert_eq!(count_kind(SymbolKind::Trait), 1, "Should have 1 trait");
        assert_eq!(count_kind(SymbolKind::Impl), 2, "Should have 2 impl blocks");
        assert_eq!(
            count_kind(SymbolKind::Function),
            2,
            "Should have 2 functions"
        );
        assert_eq!(count_kind(SymbolKind::Macro), 1, "Should have 1 macro");

        // Verify async function is marked as async
        let async_fn = symbols.iter().find(|s| s.name == "async_main").unwrap();
        assert!(async_fn.is_async);

        // Verify visibility is correct
        let main_fn = symbols.iter().find(|s| s.name == "main").unwrap();
        assert_eq!(main_fn.visibility, Visibility::Public);

        let version_const = symbols.iter().find(|s| s.name == "VERSION").unwrap();
        assert_eq!(version_const.visibility, Visibility::Private);
    }

    // ============================================
    // Python Function Extraction Tests
    // ============================================

    #[test]
    fn test_extract_python_simple_function() {
        let code = r#"
def hello():
    print("Hello, world!")

def greet(name):
    return f"Hello, {name}!"
"#;
        let tree = parse(code, Language::Python).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Python);

        assert_eq!(symbols.len(), 2);

        // First function
        assert_eq!(symbols[0].name, "hello");
        assert_eq!(symbols[0].kind, SymbolKind::Function);
        assert!(!symbols[0].is_async);
        assert_eq!(symbols[0].signature.as_deref(), Some("hello()"));

        // Second function
        assert_eq!(symbols[1].name, "greet");
        assert_eq!(symbols[1].kind, SymbolKind::Function);
        assert_eq!(symbols[1].signature.as_deref(), Some("greet(name)"));
    }

    #[test]
    fn test_extract_python_async_function() {
        let code = r#"
async def fetch_data(url):
    return await get(url)

async def process():
    data = await fetch_data("http://example.com")
    return data
"#;
        let tree = parse(code, Language::Python).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Python);

        assert_eq!(symbols.len(), 2);

        // First async function
        assert_eq!(symbols[0].name, "fetch_data");
        assert!(symbols[0].is_async);
        assert_eq!(symbols[0].signature.as_deref(), Some("fetch_data(url)"));

        // Second async function
        assert_eq!(symbols[1].name, "process");
        assert!(symbols[1].is_async);
    }

    #[test]
    fn test_extract_python_decorated_function() {
        let code = r#"
@decorator
def simple():
    pass

@lru_cache(maxsize=128)
def cached(n):
    return n * 2

@app.route("/")
@requires_auth
def handler():
    pass
"#;
        let tree = parse(code, Language::Python).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Python);

        assert_eq!(symbols.len(), 3);

        assert_eq!(symbols[0].name, "simple");
        assert_eq!(symbols[1].name, "cached");
        assert_eq!(symbols[2].name, "handler");
    }

    #[test]
    fn test_extract_python_function_with_type_hints() {
        let code = r#"
def greet(name: str) -> str:
    return f"Hello, {name}"

def process(items: List[int], mapping: Dict[str, int]) -> Optional[int]:
    return sum(items)

def complex(
    a: int,
    b: str = "default",
    *args: Any,
    **kwargs: Any
) -> None:
    pass
"#;
        let tree = parse(code, Language::Python).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Python);

        assert_eq!(symbols.len(), 3);

        // Function with simple type hints
        assert_eq!(symbols[0].name, "greet");
        let sig = symbols[0].signature.as_ref().unwrap();
        assert!(sig.contains("name: str"));
        assert!(sig.contains("-> str"));

        // Function with complex type hints
        assert_eq!(symbols[1].name, "process");
        let sig = symbols[1].signature.as_ref().unwrap();
        assert!(sig.contains("items: List[int]"));
        assert!(sig.contains("-> Optional[int]"));

        // Function with many parameters
        assert_eq!(symbols[2].name, "complex");
    }

    #[test]
    fn test_extract_python_function_byte_ranges() {
        let code = "def foo():\n    pass\ndef bar():\n    pass";
        let tree = parse(code, Language::Python).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Python);

        assert_eq!(symbols.len(), 2);

        // First function byte range
        let foo_text = &code[symbols[0].range.start..symbols[0].range.end];
        assert!(foo_text.starts_with("def foo()"));
        assert_eq!(
            &code[symbols[0].name_range.start..symbols[0].name_range.end],
            "foo"
        );

        // Second function byte range
        let bar_text = &code[symbols[1].range.start..symbols[1].range.end];
        assert!(bar_text.starts_with("def bar()"));
        assert_eq!(
            &code[symbols[1].name_range.start..symbols[1].name_range.end],
            "bar"
        );
    }

    // ============================================
    // Python Class Extraction Tests
    // ============================================

    #[test]
    fn test_extract_python_simple_class() {
        let code = r#"
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y

    def distance(self):
        return (self.x ** 2 + self.y ** 2) ** 0.5
"#;
        let tree = parse(code, Language::Python).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Python);

        // Should only extract the class, not the methods (methods are inside class)
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Point");
        assert_eq!(symbols[0].kind, SymbolKind::Class);
        assert_eq!(symbols[0].signature.as_deref(), Some("Point"));
    }

    #[test]
    fn test_extract_python_class_with_inheritance() {
        let code = r#"
class Animal:
    pass

class Dog(Animal):
    pass

class Labrador(Dog, Friendly):
    pass
"#;
        let tree = parse(code, Language::Python).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Python);

        assert_eq!(symbols.len(), 3);

        // Base class
        assert_eq!(symbols[0].name, "Animal");
        assert_eq!(symbols[0].signature.as_deref(), Some("Animal"));

        // Single inheritance
        assert_eq!(symbols[1].name, "Dog");
        assert_eq!(symbols[1].signature.as_deref(), Some("Dog(Animal)"));

        // Multiple inheritance
        assert_eq!(symbols[2].name, "Labrador");
        assert_eq!(
            symbols[2].signature.as_deref(),
            Some("Labrador(Dog, Friendly)")
        );
    }

    #[test]
    fn test_extract_python_dataclass() {
        let code = r#"
from dataclasses import dataclass

@dataclass
class Point:
    x: float
    y: float

@dataclass(frozen=True)
class ImmutablePoint:
    x: float
    y: float
"#;
        let tree = parse(code, Language::Python).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Python);

        // Should extract both dataclasses
        let classes: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Class)
            .collect();
        assert_eq!(classes.len(), 2);
        assert_eq!(classes[0].name, "Point");
        assert_eq!(classes[1].name, "ImmutablePoint");
    }

    // ============================================
    // Python Module-level Items Tests
    // ============================================

    #[test]
    fn test_extract_python_module_constants() {
        let code = r#"
VERSION = "1.0.0"
MAX_SIZE = 1024
PI = 3.14159

# Type-annotated constants
NAME: str = "MyApp"
COUNT: int = 42
"#;
        let tree = parse(code, Language::Python).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Python);

        let consts: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Const)
            .collect();
        assert_eq!(consts.len(), 5);

        let version = consts.iter().find(|s| s.name == "VERSION").unwrap();
        assert_eq!(version.signature.as_deref(), Some("VERSION"));

        let name = consts.iter().find(|s| s.name == "NAME").unwrap();
        assert_eq!(name.signature.as_deref(), Some("NAME: str"));
    }

    #[test]
    fn test_extract_python_all_definition() {
        let code = r#"
__all__ = ["foo", "bar", "baz"]

def foo():
    pass

def bar():
    pass
"#;
        let tree = parse(code, Language::Python).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Python);

        // Should have __all__ and two functions
        let all_sym = symbols.iter().find(|s| s.name == "__all__");
        assert!(all_sym.is_some());
        assert_eq!(all_sym.unwrap().kind, SymbolKind::Const);

        let funcs: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Function)
            .collect();
        assert_eq!(funcs.len(), 2);
    }

    // ============================================
    // Python Integration Test
    // ============================================

    #[test]
    fn test_extract_all_python_items() {
        let code = r#"
# A comprehensive Python file with various item types

__all__ = ["Config", "main", "process"]

VERSION = "1.0.0"
MAX_RETRIES: int = 3

class Config:
    def __init__(self, name: str):
        self.name = name

    def validate(self) -> bool:
        return True

class AdvancedConfig(Config):
    pass

def main() -> None:
    config = Config("test")
    print(config.name)

async def process(data: List[int]) -> int:
    return sum(data)

@decorator
def helper():
    pass
"#;
        let tree = parse(code, Language::Python).unwrap();
        let symbols = extract_symbols(&tree, code, Language::Python);

        // Count each type
        let count_kind = |kind: SymbolKind| symbols.iter().filter(|s| s.kind == kind).count();

        assert_eq!(
            count_kind(SymbolKind::Const),
            3,
            "Should have 3 constants (__all__, VERSION, MAX_RETRIES)"
        );
        assert_eq!(count_kind(SymbolKind::Class), 2, "Should have 2 classes");
        assert_eq!(
            count_kind(SymbolKind::Function),
            3,
            "Should have 3 functions"
        );

        // Verify async function is marked as async
        let process_fn = symbols.iter().find(|s| s.name == "process").unwrap();
        assert!(process_fn.is_async);
        assert!(process_fn.signature.as_ref().unwrap().contains("-> int"));

        // Verify class inheritance
        let advanced = symbols.iter().find(|s| s.name == "AdvancedConfig").unwrap();
        assert!(advanced.signature.as_ref().unwrap().contains("(Config)"));

        // Verify constant with type annotation
        let max_retries = symbols.iter().find(|s| s.name == "MAX_RETRIES").unwrap();
        assert!(max_retries.signature.as_ref().unwrap().contains(": int"));
    }
}
