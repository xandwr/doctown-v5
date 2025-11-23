//! Import statement extraction from ASTs.

use doctown_common::types::Import;
use tree_sitter::{Node, Tree};

use crate::traversal::{find_nodes_by_kind, node_byte_range, node_text};

/// Extract all import statements from a parsed syntax tree.
pub fn extract_imports(
    tree: &Tree,
    source_code: &str,
    language: doctown_common::Language,
) -> Vec<Import> {
    match language {
        doctown_common::Language::Rust => extract_rust_imports(tree, source_code),
        doctown_common::Language::Python => extract_python_imports(tree, source_code),
        doctown_common::Language::TypeScript => extract_typescript_imports(tree, source_code),
        doctown_common::Language::JavaScript => extract_javascript_imports(tree, source_code),
        doctown_common::Language::Go => extract_go_imports(tree, source_code),
    }
}

/// Extract imports from Rust source code (use statements).
fn extract_rust_imports(tree: &Tree, source_code: &str) -> Vec<Import> {
    let mut imports = Vec::new();
    let root = tree.root_node();

    // Extract use declarations
    for node in find_nodes_by_kind(root, "use_declaration") {
        if let Some(import) = extract_rust_use(node, source_code) {
            imports.push(import);
        }
    }

    imports
}

fn extract_rust_use(node: Node<'_>, source_code: &str) -> Option<Import> {
    let range = node_byte_range(node);

    // Find the use_clause which contains the path
    let use_clause = node.child_by_field_name("argument")?;

    // Check for wildcard: use foo::*;
    let full_text = node_text(use_clause, source_code);
    let is_wildcard = full_text.contains("::*");

    // Check for use list: use foo::{Bar, Baz};
    let has_use_list = find_nodes_by_kind(use_clause, "use_list").next().is_some();

    if has_use_list {
        // Extract items from use list
        let use_list = find_nodes_by_kind(use_clause, "use_list").next()?;
        let items: Vec<String> = use_list
            .named_children(&mut use_list.walk())
            .filter_map(|child| {
                if child.kind() == "use_wildcard" {
                    return None; // Skip wildcards in the item list
                }
                let text = node_text(child, source_code);
                // Handle "as" aliases - strip the alias part
                if let Some(as_pos) = text.find(" as ") {
                    Some(text[..as_pos].trim().to_string())
                } else {
                    Some(text.trim().to_string())
                }
            })
            .collect();

        // Get the base path by finding scoped_identifier nodes before the use_list
        let mut base_path = String::new();
        for child in use_clause.children(&mut use_clause.walk()) {
            if child.kind() == "use_list" {
                break;
            }
            if child.kind() == "scoped_identifier" || child.kind() == "identifier" {
                base_path = node_text(child, source_code).to_string();
            }
        }

        // If we couldn't extract it cleanly, parse from text
        if base_path.is_empty() {
            if let Some(pos) = full_text.find("::") {
                // Find the position of the opening brace
                if let Some(brace_pos) = full_text.find('{') {
                    base_path = full_text[..brace_pos]
                        .trim_end_matches("::")
                        .trim()
                        .to_string();
                } else {
                    base_path = full_text[..pos].trim().to_string();
                }
            }
        }

        return Some(Import {
            module_path: base_path,
            imported_items: if items.is_empty() { None } else { Some(items) },
            alias: None,
            range,
            is_wildcard: false,
        });
    }

    // Simple use statement: use foo::bar; or use foo::bar as baz;
    let full_text = full_text.trim().to_string();

    // Check for alias
    if let Some(as_pos) = full_text.find(" as ") {
        let path = full_text[..as_pos].trim().to_string();
        let alias = full_text[as_pos + 4..].trim().to_string();
        return Some(Import {
            module_path: path.clone(),
            imported_items: None,
            alias: Some(alias),
            range,
            is_wildcard,
        });
    }

    Some(Import {
        module_path: full_text,
        imported_items: None,
        alias: None,
        range,
        is_wildcard,
    })
}

/// Extract imports from Python source code.
fn extract_python_imports(tree: &Tree, source_code: &str) -> Vec<Import> {
    let mut imports = Vec::new();
    let root = tree.root_node();

    // Extract "import" statements
    for node in find_nodes_by_kind(root, "import_statement") {
        if let Some(import) = extract_python_import(node, source_code) {
            imports.push(import);
        }
    }

    // Extract "from...import" statements
    for node in find_nodes_by_kind(root, "import_from_statement") {
        if let Some(import) = extract_python_from_import(node, source_code) {
            imports.push(import);
        }
    }

    imports
}

fn extract_python_import(node: Node<'_>, source_code: &str) -> Option<Import> {
    let range = node_byte_range(node);

    // Get the dotted_name or aliased_import nodes
    let mut imports_to_create = Vec::new();

    for child in node.named_children(&mut node.walk()) {
        if child.kind() == "dotted_name" {
            let module = node_text(child, source_code).to_string();
            imports_to_create.push(Import {
                module_path: module,
                imported_items: None,
                alias: None,
                range,
                is_wildcard: false,
            });
        } else if child.kind() == "aliased_import" {
            // Handle "import foo as bar"
            let name_node = child.child_by_field_name("name")?;
            let module = node_text(name_node, source_code).to_string();
            let alias = child
                .child_by_field_name("alias")
                .map(|n| node_text(n, source_code).to_string());

            imports_to_create.push(Import {
                module_path: module,
                imported_items: None,
                alias,
                range,
                is_wildcard: false,
            });
        }
    }

    imports_to_create.into_iter().next()
}

fn extract_python_from_import(node: Node<'_>, source_code: &str) -> Option<Import> {
    let range = node_byte_range(node);

    // Get module name
    let module_name = node
        .child_by_field_name("module_name")
        .map(|n| node_text(n, source_code).to_string())
        .unwrap_or_else(|| ".".to_string()); // relative imports

    // Check for wildcard: from foo import *
    let is_wildcard = node_text(node, source_code).contains("import *");

    if is_wildcard {
        return Some(Import {
            module_path: module_name,
            imported_items: None,
            alias: None,
            range,
            is_wildcard: true,
        });
    }

    // Extract imported items - they come after "import" keyword
    let mut items = Vec::new();
    let mut after_import = false;

    for child in node.named_children(&mut node.walk()) {
        // Skip the module_name itself
        if Some(child.id()) == node.child_by_field_name("module_name").map(|n| n.id()) {
            after_import = true;
            continue;
        }

        if after_import {
            if child.kind() == "dotted_name" || child.kind() == "identifier" {
                items.push(node_text(child, source_code).to_string());
            } else if child.kind() == "aliased_import" {
                let name_node = child.child_by_field_name("name")?;
                items.push(node_text(name_node, source_code).to_string());
            }
        }
    }

    Some(Import {
        module_path: module_name,
        imported_items: if items.is_empty() { None } else { Some(items) },
        alias: None,
        range,
        is_wildcard: false,
    })
}

/// Extract imports from TypeScript source code.
fn extract_typescript_imports(tree: &Tree, source_code: &str) -> Vec<Import> {
    let mut imports = Vec::new();
    let root = tree.root_node();

    // Extract import statements
    for node in find_nodes_by_kind(root, "import_statement") {
        if let Some(import) = extract_ts_import(node, source_code) {
            imports.push(import);
        }
    }

    imports
}

fn extract_ts_import(node: Node<'_>, source_code: &str) -> Option<Import> {
    let range = node_byte_range(node);

    // Get the source module (string literal)
    let source_node = node.child_by_field_name("source")?;
    let module_path = node_text(source_node, source_code)
        .trim_matches(|c| c == '"' || c == '\'')
        .to_string();

    // Check for different import types
    let import_clause = node
        .children(&mut node.walk())
        .find(|n| n.kind() == "import_clause");

    if let Some(clause) = import_clause {
        let clause_text = node_text(clause, source_code);

        // Check for namespace import: import * as foo from 'bar'
        if clause_text.contains("* as") {
            let alias = clause_text
                .split("* as")
                .nth(1)
                .map(|s| s.trim().to_string());

            return Some(Import {
                module_path,
                imported_items: None,
                alias,
                range,
                is_wildcard: true,
            });
        }

        // Check for named imports: import { foo, bar } from 'baz'
        let named_imports = find_nodes_by_kind(clause, "named_imports").next();
        if let Some(named) = named_imports {
            let items: Vec<String> = find_nodes_by_kind(named, "import_specifier")
                .filter_map(|spec| {
                    let name_node = spec.child_by_field_name("name")?;
                    Some(node_text(name_node, source_code).to_string())
                })
                .collect();

            return Some(Import {
                module_path,
                imported_items: Some(items),
                alias: None,
                range,
                is_wildcard: false,
            });
        }

        // Default import: import foo from 'bar'
        if let Some(default) = find_nodes_by_kind(clause, "identifier").next() {
            let alias = node_text(default, source_code).to_string();
            return Some(Import {
                module_path,
                imported_items: None,
                alias: Some(alias),
                range,
                is_wildcard: false,
            });
        }
    }

    // Side-effect import: import 'foo'
    Some(Import {
        module_path,
        imported_items: None,
        alias: None,
        range,
        is_wildcard: false,
    })
}

/// Extract imports from JavaScript source code.
fn extract_javascript_imports(tree: &Tree, source_code: &str) -> Vec<Import> {
    // JavaScript uses the same AST structure as TypeScript
    extract_typescript_imports(tree, source_code)
}

/// Extract imports from Go source code.
fn extract_go_imports(tree: &Tree, source_code: &str) -> Vec<Import> {
    let mut imports = Vec::new();
    let root = tree.root_node();

    // Extract import declarations
    for node in find_nodes_by_kind(root, "import_declaration") {
        extract_go_import_declaration(node, source_code, &mut imports);
    }

    imports
}

fn extract_go_import_declaration(node: Node<'_>, source_code: &str, imports: &mut Vec<Import>) {
    let range = node_byte_range(node);

    // Check for single import: import "fmt"
    if let Some(spec) = node
        .children(&mut node.walk())
        .find(|n| n.kind() == "import_spec")
    {
        if let Some(import) = extract_go_import_spec(spec, source_code, range) {
            imports.push(import);
        }
        return;
    }

    // Check for multiple imports: import ( ... )
    for spec in find_nodes_by_kind(node, "import_spec") {
        if let Some(import) = extract_go_import_spec(spec, source_code, range) {
            imports.push(import);
        }
    }
}

fn extract_go_import_spec(
    node: Node<'_>,
    source_code: &str,
    range: doctown_common::types::ByteRange,
) -> Option<Import> {
    // Get the import path (string literal)
    let path_node = node
        .children(&mut node.walk())
        .find(|n| n.kind() == "interpreted_string_literal")?;

    let module_path = node_text(path_node, source_code)
        .trim_matches('"')
        .to_string();

    // Check for aliased import: import foo "github.com/bar/baz"
    let alias = node
        .child_by_field_name("name")
        .or_else(|| {
            // Sometimes the alias is just the first identifier child
            node.children(&mut node.walk())
                .find(|n| n.kind() == "package_identifier" || n.kind() == "identifier")
        })
        .map(|n| node_text(n, source_code).to_string());

    // Check for dot import: import . "foo"
    let is_wildcard = alias.as_ref().map(|a| a == ".").unwrap_or(false);

    Some(Import {
        module_path,
        imported_items: None,
        alias: if is_wildcard { None } else { alias },
        range,
        is_wildcard,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::Parser;

    #[test]
    fn test_rust_simple_import() {
        let code = "use std::collections::HashMap;";
        let parser = Parser::new();
        let tree = parser.parse(code, doctown_common::Language::Rust).unwrap();
        let imports = extract_rust_imports(&tree, code);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].module_path, "std::collections::HashMap");
        assert_eq!(imports[0].imported_items, None);
        assert!(!imports[0].is_wildcard);
    }

    #[test]
    fn test_rust_use_list() {
        let code = "use std::collections::{HashMap, HashSet};";
        let parser = Parser::new();
        let tree = parser.parse(code, doctown_common::Language::Rust).unwrap();
        let imports = extract_rust_imports(&tree, code);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].module_path, "std::collections");
        assert_eq!(
            imports[0].imported_items,
            Some(vec!["HashMap".to_string(), "HashSet".to_string()])
        );
    }

    #[test]
    fn test_rust_wildcard() {
        let code = "use std::collections::*;";
        let parser = Parser::new();
        let tree = parser.parse(code, doctown_common::Language::Rust).unwrap();
        let imports = extract_rust_imports(&tree, code);

        assert_eq!(imports.len(), 1);
        assert!(imports[0].is_wildcard);
    }

    #[test]
    fn test_python_simple_import() {
        let code = "import os";
        let parser = Parser::new();
        let tree = parser
            .parse(code, doctown_common::Language::Python)
            .unwrap();
        let imports = extract_python_imports(&tree, code);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].module_path, "os");
        assert_eq!(imports[0].imported_items, None);
    }

    #[test]
    fn test_python_from_import() {
        let code = "from os.path import join, exists";
        let parser = Parser::new();
        let tree = parser
            .parse(code, doctown_common::Language::Python)
            .unwrap();
        let imports = extract_python_imports(&tree, code);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].module_path, "os.path");
        assert_eq!(
            imports[0].imported_items,
            Some(vec!["join".to_string(), "exists".to_string()])
        );
    }

    #[test]
    fn test_python_wildcard_import() {
        let code = "from os import *";
        let parser = Parser::new();
        let tree = parser
            .parse(code, doctown_common::Language::Python)
            .unwrap();
        let imports = extract_python_imports(&tree, code);

        assert_eq!(imports.len(), 1);
        assert!(imports[0].is_wildcard);
    }

    #[test]
    fn test_typescript_named_imports() {
        let code = "import { foo, bar } from './utils';";
        let parser = Parser::new();
        let tree = parser
            .parse(code, doctown_common::Language::TypeScript)
            .unwrap();
        let imports = extract_typescript_imports(&tree, code);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].module_path, "./utils");
        assert_eq!(
            imports[0].imported_items,
            Some(vec!["foo".to_string(), "bar".to_string()])
        );
    }

    #[test]
    fn test_typescript_namespace_import() {
        let code = "import * as utils from './utils';";
        let parser = Parser::new();
        let tree = parser
            .parse(code, doctown_common::Language::TypeScript)
            .unwrap();
        let imports = extract_typescript_imports(&tree, code);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].module_path, "./utils");
        assert!(imports[0].is_wildcard);
        assert_eq!(imports[0].alias, Some("utils".to_string()));
    }

    #[test]
    fn test_go_simple_import() {
        let code = r#"import "fmt""#;
        let parser = Parser::new();
        let tree = parser.parse(code, doctown_common::Language::Go).unwrap();
        let imports = extract_go_imports(&tree, code);

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].module_path, "fmt");
    }

    #[test]
    fn test_go_multiple_imports() {
        let code = r#"
import (
    "fmt"
    "os"
)
"#;
        let parser = Parser::new();
        let tree = parser.parse(code, doctown_common::Language::Go).unwrap();
        let imports = extract_go_imports(&tree, code);

        assert_eq!(imports.len(), 2);
        assert_eq!(imports[0].module_path, "fmt");
        assert_eq!(imports[1].module_path, "os");
    }
}
