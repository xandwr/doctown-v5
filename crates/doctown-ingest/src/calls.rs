//! Call detection from ASTs.

use doctown_common::types::{Call, CallKind};
use tree_sitter::{Node, Tree};

use crate::traversal::{find_nodes_by_kind, node_byte_range, node_text};

/// Extract all function/method calls from a parsed syntax tree.
pub fn extract_calls(
    tree: &Tree,
    source_code: &str,
    language: doctown_common::Language,
) -> Vec<Call> {
    match language {
        doctown_common::Language::Rust => extract_rust_calls(tree, source_code),
        doctown_common::Language::Python => extract_python_calls(tree, source_code),
        doctown_common::Language::TypeScript => extract_typescript_calls(tree, source_code),
        doctown_common::Language::JavaScript => extract_javascript_calls(tree, source_code),
        doctown_common::Language::Go => extract_go_calls(tree, source_code),
    }
}

/// Extract calls from Rust source code.
fn extract_rust_calls(tree: &Tree, source_code: &str) -> Vec<Call> {
    let mut calls = Vec::new();
    let root = tree.root_node();

    // Extract direct function calls: `foo()`
    for node in find_nodes_by_kind(root, "call_expression") {
        if let Some(call) = extract_rust_call_expression(node, source_code) {
            calls.push(call);
        }
    }

    // Extract method calls: `obj.method()`
    for node in find_nodes_by_kind(root, "field_expression") {
        // Check if this field expression is part of a call
        if let Some(parent) = node.parent() {
            if parent.kind() == "call_expression" {
                if let Some(call) = extract_rust_method_call(node, source_code) {
                    calls.push(call);
                }
            }
        }
    }

    calls
}

fn extract_rust_call_expression(node: Node<'_>, source_code: &str) -> Option<Call> {
    let function_node = node.child_by_field_name("function")?;
    let name = node_text(function_node, source_code);
    let range = node_byte_range(node);

    // Determine call kind based on the function expression
    let kind = if function_node.kind() == "field_expression" {
        CallKind::Method
    } else if name.contains("::") {
        CallKind::Associated
    } else {
        CallKind::Function
    };

    Some(Call {
        name: name.to_string(),
        range,
        kind,
        is_resolved: false, // Simple heuristic: mark all as unresolved for now
    })
}

fn extract_rust_method_call(node: Node<'_>, source_code: &str) -> Option<Call> {
    let field_node = node.child_by_field_name("field")?;
    let name = node_text(field_node, source_code);
    let range = node_byte_range(node.parent()?);

    Some(Call {
        name: name.to_string(),
        range,
        kind: CallKind::Method,
        is_resolved: false,
    })
}

/// Extract calls from Python source code.
fn extract_python_calls(tree: &Tree, source_code: &str) -> Vec<Call> {
    let mut calls = Vec::new();
    let root = tree.root_node();

    // Extract function calls
    for node in find_nodes_by_kind(root, "call") {
        if let Some(call) = extract_python_call(node, source_code) {
            calls.push(call);
        }
    }

    calls
}

fn extract_python_call(node: Node<'_>, source_code: &str) -> Option<Call> {
    let function_node = node.child_by_field_name("function")?;
    let name = node_text(function_node, source_code);
    let range = node_byte_range(node);

    // Determine call kind
    let kind = if function_node.kind() == "attribute" {
        CallKind::Method
    } else if name.chars().next().map_or(false, |c| c.is_uppercase()) {
        // Heuristic: capitalized names are likely classes
        CallKind::Constructor
    } else {
        CallKind::Function
    };

    Some(Call {
        name: name.to_string(),
        range,
        kind,
        is_resolved: false,
    })
}

/// Extract calls from TypeScript source code.
fn extract_typescript_calls(tree: &Tree, source_code: &str) -> Vec<Call> {
    let mut calls = Vec::new();
    let root = tree.root_node();

    // Extract function calls
    for node in find_nodes_by_kind(root, "call_expression") {
        if let Some(call) = extract_ts_call(node, source_code) {
            calls.push(call);
        }
    }

    // Extract new expressions
    for node in find_nodes_by_kind(root, "new_expression") {
        if let Some(call) = extract_ts_new_expression(node, source_code) {
            calls.push(call);
        }
    }

    calls
}

fn extract_ts_call(node: Node<'_>, source_code: &str) -> Option<Call> {
    let function_node = node.child_by_field_name("function")?;
    let name = node_text(function_node, source_code);
    let range = node_byte_range(node);

    // Determine call kind
    let kind = if function_node.kind() == "member_expression" {
        CallKind::Method
    } else {
        CallKind::Function
    };

    Some(Call {
        name: name.to_string(),
        range,
        kind,
        is_resolved: false,
    })
}

fn extract_ts_new_expression(node: Node<'_>, source_code: &str) -> Option<Call> {
    let constructor_node = node.child_by_field_name("constructor")?;
    let name = node_text(constructor_node, source_code);
    let range = node_byte_range(node);

    Some(Call {
        name: name.to_string(),
        range,
        kind: CallKind::Constructor,
        is_resolved: false,
    })
}

/// Extract calls from JavaScript source code.
fn extract_javascript_calls(tree: &Tree, source_code: &str) -> Vec<Call> {
    // JavaScript uses the same AST structure as TypeScript
    extract_typescript_calls(tree, source_code)
}

/// Extract calls from Go source code.
fn extract_go_calls(tree: &Tree, source_code: &str) -> Vec<Call> {
    let mut calls = Vec::new();
    let root = tree.root_node();

    // Extract function calls
    for node in find_nodes_by_kind(root, "call_expression") {
        if let Some(call) = extract_go_call(node, source_code) {
            calls.push(call);
        }
    }

    calls
}

fn extract_go_call(node: Node<'_>, source_code: &str) -> Option<Call> {
    let function_node = node.child_by_field_name("function")?;
    let name = node_text(function_node, source_code);
    let range = node_byte_range(node);

    // Determine call kind
    let kind = if function_node.kind() == "selector_expression" {
        // Could be method call or package function
        CallKind::Method
    } else {
        CallKind::Function
    };

    Some(Call {
        name: name.to_string(),
        range,
        kind,
        is_resolved: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::parse;

    #[test]
    fn test_rust_function_call() {
        let code = r#"
fn main() {
    println!("hello");
    foo();
    bar(1, 2);
}
"#;
        let tree = parse(code, doctown_common::Language::Rust).unwrap();
        let calls = extract_rust_calls(&tree, code);

        // Should find println!, foo, and bar
        assert!(calls.len() >= 2); // At least foo and bar
        assert!(calls.iter().any(|c| c.name.contains("foo")));
        assert!(calls.iter().any(|c| c.name.contains("bar")));
    }

    #[test]
    fn test_rust_method_call() {
        let code = r#"
fn main() {
    let s = String::new();
    s.push_str("hello");
    s.len();
}
"#;
        let tree = parse(code, doctown_common::Language::Rust).unwrap();
        let calls = extract_rust_calls(&tree, code);

        // Should find String::new, push_str, and len
        assert!(calls.iter().any(|c| c.name.contains("new")));
        assert!(calls.iter().any(|c| c.name.contains("push_str")));
        assert!(calls.iter().any(|c| c.name.contains("len")));
    }

    #[test]
    fn test_rust_chained_calls() {
        let code = r#"
fn main() {
    let result = vec![1, 2, 3]
        .iter()
        .map(|x| x * 2)
        .collect();
}
"#;
        let tree = parse(code, doctown_common::Language::Rust).unwrap();
        let calls = extract_rust_calls(&tree, code);

        // Should find iter, map, and collect
        assert!(calls.iter().any(|c| c.name.contains("iter")));
        assert!(calls.iter().any(|c| c.name.contains("map")));
        assert!(calls.iter().any(|c| c.name.contains("collect")));
    }

    #[test]
    fn test_python_function_call() {
        let code = r#"
def main():
    print("hello")
    foo()
    bar(1, 2)
"#;
        let tree = parse(code, doctown_common::Language::Python).unwrap();
        let calls = extract_python_calls(&tree, code);

        assert!(calls.iter().any(|c| c.name == "print"));
        assert!(calls.iter().any(|c| c.name == "foo"));
        assert!(calls.iter().any(|c| c.name == "bar"));
    }

    #[test]
    fn test_python_method_call() {
        let code = r#"
def main():
    s = "hello"
    s.upper()
    s.replace("h", "H")
"#;
        let tree = parse(code, doctown_common::Language::Python).unwrap();
        let calls = extract_python_calls(&tree, code);

        assert!(calls.iter().any(|c| c.name.contains("upper")));
        assert!(calls.iter().any(|c| c.name.contains("replace")));
        assert!(calls
            .iter()
            .any(|c| c.kind == CallKind::Method && c.name.contains("upper")));
    }

    #[test]
    fn test_python_class_instantiation() {
        let code = r#"
def main():
    obj = MyClass()
    obj2 = MyClass(arg1, arg2)
"#;
        let tree = parse(code, doctown_common::Language::Python).unwrap();
        let calls = extract_python_calls(&tree, code);

        let class_calls: Vec<_> = calls
            .iter()
            .filter(|c| c.name == "MyClass")
            .collect();
        assert_eq!(class_calls.len(), 2);
        assert!(class_calls
            .iter()
            .all(|c| c.kind == CallKind::Constructor));
    }

    #[test]
    fn test_typescript_function_call() {
        let code = r#"
function main() {
    console.log("hello");
    foo();
    bar(1, 2);
}
"#;
        let tree = parse(code, doctown_common::Language::TypeScript).unwrap();
        let calls = extract_typescript_calls(&tree, code);

        assert!(calls.iter().any(|c| c.name.contains("log")));
        assert!(calls.iter().any(|c| c.name == "foo"));
        assert!(calls.iter().any(|c| c.name == "bar"));
    }

    #[test]
    fn test_typescript_method_call() {
        let code = r#"
function main() {
    const arr = [1, 2, 3];
    arr.map(x => x * 2);
    arr.filter(x => x > 1);
}
"#;
        let tree = parse(code, doctown_common::Language::TypeScript).unwrap();
        let calls = extract_typescript_calls(&tree, code);

        assert!(calls.iter().any(|c| c.name.contains("map")));
        assert!(calls.iter().any(|c| c.name.contains("filter")));
    }

    #[test]
    fn test_typescript_constructor_call() {
        let code = r#"
function main() {
    const obj = new MyClass();
    const obj2 = new MyClass(arg1, arg2);
}
"#;
        let tree = parse(code, doctown_common::Language::TypeScript).unwrap();
        let calls = extract_typescript_calls(&tree, code);

        let constructor_calls: Vec<_> = calls
            .iter()
            .filter(|c| c.kind == CallKind::Constructor)
            .collect();
        assert_eq!(constructor_calls.len(), 2);
        assert!(constructor_calls.iter().all(|c| c.name == "MyClass"));
    }

    #[test]
    fn test_javascript_function_call() {
        let code = r#"
function main() {
    console.log("hello");
    foo();
}
"#;
        let tree = parse(code, doctown_common::Language::JavaScript).unwrap();
        let calls = extract_javascript_calls(&tree, code);

        assert!(calls.iter().any(|c| c.name.contains("log")));
        assert!(calls.iter().any(|c| c.name == "foo"));
    }

    #[test]
    fn test_go_function_call() {
        let code = r#"
func main() {
    fmt.Println("hello")
    foo()
    bar(1, 2)
}
"#;
        let tree = parse(code, doctown_common::Language::Go).unwrap();
        let calls = extract_go_calls(&tree, code);

        assert!(calls.iter().any(|c| c.name.contains("Println")));
        assert!(calls.iter().any(|c| c.name == "foo"));
        assert!(calls.iter().any(|c| c.name == "bar"));
    }

    #[test]
    fn test_go_method_call() {
        let code = r#"
func main() {
    s := "hello"
    len(s)
    strings.ToUpper(s)
}
"#;
        let tree = parse(code, doctown_common::Language::Go).unwrap();
        let calls = extract_go_calls(&tree, code);

        assert!(calls.iter().any(|c| c.name == "len"));
        assert!(calls.iter().any(|c| c.name.contains("ToUpper")));
    }
}
