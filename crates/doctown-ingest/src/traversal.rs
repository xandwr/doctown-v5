//! AST traversal utilities for tree-sitter trees.
//!
//! This module provides utilities for traversing and querying tree-sitter ASTs:
//! - [`TreeCursor`]: A wrapper around tree-sitter's cursor with convenient iteration
//! - Node type matching helpers for filtering nodes by kind
//! - Text extraction utilities for getting source text from nodes

use doctown_common::types::ByteRange;
use tree_sitter::Node;

/// A wrapper around tree-sitter's TreeCursor providing convenient traversal methods.
///
/// This cursor maintains position state and provides iteration capabilities
/// for depth-first traversal of the syntax tree.
pub struct TreeCursor<'tree> {
    cursor: tree_sitter::TreeCursor<'tree>,
}

impl<'tree> TreeCursor<'tree> {
    /// Creates a new TreeCursor starting at the given node.
    pub fn new(node: Node<'tree>) -> Self {
        Self {
            cursor: node.walk(),
        }
    }

    /// Returns the current node.
    pub fn node(&self) -> Node<'tree> {
        self.cursor.node()
    }

    /// Moves to the first child of the current node.
    /// Returns `true` if successful, `false` if there are no children.
    pub fn goto_first_child(&mut self) -> bool {
        self.cursor.goto_first_child()
    }

    /// Moves to the next sibling of the current node.
    /// Returns `true` if successful, `false` if there are no more siblings.
    pub fn goto_next_sibling(&mut self) -> bool {
        self.cursor.goto_next_sibling()
    }

    /// Moves to the parent of the current node.
    /// Returns `true` if successful, `false` if already at the root.
    pub fn goto_parent(&mut self) -> bool {
        self.cursor.goto_parent()
    }

    /// Returns the field name of the current node, if any.
    pub fn field_name(&self) -> Option<&'static str> {
        self.cursor.field_name()
    }

    /// Returns the depth of the current node in the tree.
    pub fn depth(&self) -> usize {
        self.cursor.depth() as usize
    }

    /// Resets the cursor to start at a new node.
    pub fn reset(&mut self, node: Node<'tree>) {
        self.cursor.reset(node);
    }

    /// Returns an iterator that performs depth-first traversal from the current position.
    ///
    /// The iterator yields each node exactly once in pre-order (parent before children).
    pub fn dfs_iter(self) -> DfsIterator<'tree> {
        DfsIterator::new(self)
    }

    /// Moves to the first child with the given field name.
    /// Returns `true` if found, `false` otherwise. Position is unchanged if not found.
    pub fn goto_first_child_for_field(&mut self, field_name: &str) -> bool {
        if !self.goto_first_child() {
            return false;
        }

        loop {
            if self.field_name() == Some(field_name) {
                return true;
            }
            if !self.goto_next_sibling() {
                self.goto_parent();
                return false;
            }
        }
    }
}

/// Iterator for depth-first traversal of a syntax tree.
pub struct DfsIterator<'tree> {
    cursor: TreeCursor<'tree>,
    started: bool,
    done: bool,
    start_depth: usize,
}

impl<'tree> DfsIterator<'tree> {
    fn new(cursor: TreeCursor<'tree>) -> Self {
        let start_depth = cursor.depth();
        Self {
            cursor,
            started: false,
            done: false,
            start_depth,
        }
    }
}

impl<'tree> Iterator for DfsIterator<'tree> {
    type Item = Node<'tree>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        if !self.started {
            self.started = true;
            return Some(self.cursor.node());
        }

        // Try to go to first child
        if self.cursor.goto_first_child() {
            return Some(self.cursor.node());
        }

        // Try to go to next sibling
        if self.cursor.goto_next_sibling() {
            return Some(self.cursor.node());
        }

        // Go up and try siblings
        loop {
            if !self.cursor.goto_parent() || self.cursor.depth() < self.start_depth {
                self.done = true;
                return None;
            }
            if self.cursor.goto_next_sibling() {
                return Some(self.cursor.node());
            }
        }
    }
}

// ============================================
// Node Type Matching Helpers
// ============================================

/// Checks if a node matches a specific kind.
pub fn matches_kind(node: Node<'_>, kind: &str) -> bool {
    node.kind() == kind
}

/// Checks if a node matches any of the given kinds.
pub fn matches_any_kind(node: Node<'_>, kinds: &[&str]) -> bool {
    kinds.contains(&node.kind())
}

/// Checks if a node is a named node (not anonymous syntax like punctuation).
pub fn is_named(node: Node<'_>) -> bool {
    node.is_named()
}

/// Checks if a node has any error in its subtree.
pub fn has_error(node: Node<'_>) -> bool {
    node.has_error()
}

/// Checks if a node is an ERROR node.
pub fn is_error(node: Node<'_>) -> bool {
    node.is_error()
}

/// Checks if a node is a MISSING node (inserted by error recovery).
pub fn is_missing(node: Node<'_>) -> bool {
    node.is_missing()
}

/// Returns an iterator over nodes matching the given kind using DFS traversal.
pub fn find_nodes_by_kind<'tree>(
    root: Node<'tree>,
    kind: &'static str,
) -> impl Iterator<Item = Node<'tree>> {
    TreeCursor::new(root)
        .dfs_iter()
        .filter(move |node| node.kind() == kind)
}

/// Returns an iterator over nodes matching any of the given kinds using DFS traversal.
pub fn find_nodes_by_kinds<'tree>(
    root: Node<'tree>,
    kinds: &'static [&'static str],
) -> impl Iterator<Item = Node<'tree>> {
    TreeCursor::new(root)
        .dfs_iter()
        .filter(move |node| kinds.contains(&node.kind()))
}

/// Returns the first ancestor of the given node matching the specified kind.
pub fn find_ancestor_by_kind<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    let mut current = node.parent();
    while let Some(n) = current {
        if n.kind() == kind {
            return Some(n);
        }
        current = n.parent();
    }
    None
}

/// Returns all ancestors of the given node, from immediate parent to root.
pub fn ancestors<'tree>(node: Node<'tree>) -> impl Iterator<Item = Node<'tree>> {
    std::iter::successors(node.parent(), |n| n.parent())
}

/// Returns the first child of the given node matching the specified kind.
pub fn find_child_by_kind<'tree>(node: Node<'tree>, kind: &str) -> Option<Node<'tree>> {
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            if cursor.node().kind() == kind {
                return Some(cursor.node());
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    None
}

/// Returns all children of the given node matching the specified kind.
pub fn find_children_by_kind<'tree>(node: Node<'tree>, kind: &str) -> Vec<Node<'tree>> {
    let mut result = Vec::new();
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            if cursor.node().kind() == kind {
                result.push(cursor.node());
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    result
}

/// Returns the child at the given field name, if it exists.
pub fn child_by_field<'tree>(node: Node<'tree>, field_name: &str) -> Option<Node<'tree>> {
    node.child_by_field_name(field_name)
}

// ============================================
// Text Extraction Helpers
// ============================================

/// Extracts the text content of a node from the source code.
pub fn node_text<'a>(node: Node<'_>, source: &'a str) -> &'a str {
    &source[node.start_byte()..node.end_byte()]
}

/// Extracts the text content of a node, returning an owned String.
pub fn node_text_owned(node: Node<'_>, source: &str) -> String {
    source[node.start_byte()..node.end_byte()].to_string()
}

/// Returns the byte range of a node.
pub fn node_byte_range(node: Node<'_>) -> ByteRange {
    ByteRange {
        start: node.start_byte(),
        end: node.end_byte(),
    }
}

/// Extracts text from a byte range in the source code.
pub fn text_from_range<'a>(source: &'a str, range: &ByteRange) -> &'a str {
    &source[range.start..range.end]
}

/// Returns the start position (row, column) of a node.
pub fn node_start_position(node: Node<'_>) -> (usize, usize) {
    let point = node.start_position();
    (point.row, point.column)
}

/// Returns the end position (row, column) of a node.
pub fn node_end_position(node: Node<'_>) -> (usize, usize) {
    let point = node.end_position();
    (point.row, point.column)
}

/// Returns the number of lines spanned by a node.
pub fn node_line_count(node: Node<'_>) -> usize {
    node.end_position().row - node.start_position().row + 1
}

/// Extracts the text of a child at the given field name.
pub fn child_text<'a>(node: Node<'_>, field_name: &str, source: &'a str) -> Option<&'a str> {
    node.child_by_field_name(field_name)
        .map(|child| node_text(child, source))
}

/// Collects all text from named children, concatenated with a separator.
pub fn collect_named_children_text(node: Node<'_>, source: &str, separator: &str) -> String {
    let mut cursor = node.walk();
    let mut texts = Vec::new();

    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if child.is_named() {
                texts.push(node_text(child, source));
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    texts.join(separator)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsing::parse;
    use doctown_common::Language;

    // ============================================
    // TreeCursor Tests
    // ============================================

    #[test]
    fn test_tree_cursor_new() {
        let code = "fn main() {}";
        let tree = parse(code, Language::Rust).unwrap();
        let cursor = TreeCursor::new(tree.root_node());
        assert_eq!(cursor.node().kind(), "source_file");
    }

    #[test]
    fn test_tree_cursor_navigation() {
        let code = "fn main() { let x = 1; }";
        let tree = parse(code, Language::Rust).unwrap();
        let mut cursor = TreeCursor::new(tree.root_node());

        // Start at root
        assert_eq!(cursor.node().kind(), "source_file");
        assert_eq!(cursor.depth(), 0);

        // Go to first child (function_item)
        assert!(cursor.goto_first_child());
        assert_eq!(cursor.node().kind(), "function_item");
        assert_eq!(cursor.depth(), 1);

        // Go back to parent
        assert!(cursor.goto_parent());
        assert_eq!(cursor.node().kind(), "source_file");
    }

    #[test]
    fn test_tree_cursor_siblings() {
        let code = "fn foo() {} fn bar() {}";
        let tree = parse(code, Language::Rust).unwrap();
        let mut cursor = TreeCursor::new(tree.root_node());

        assert!(cursor.goto_first_child());
        assert_eq!(cursor.node().kind(), "function_item");

        assert!(cursor.goto_next_sibling());
        assert_eq!(cursor.node().kind(), "function_item");

        // No more siblings
        assert!(!cursor.goto_next_sibling());
    }

    #[test]
    fn test_tree_cursor_field_name() {
        let code = "fn main() {}";
        let tree = parse(code, Language::Rust).unwrap();
        let mut cursor = TreeCursor::new(tree.root_node());

        cursor.goto_first_child(); // function_item
        cursor.goto_first_child(); // 'fn' keyword

        // Navigate to find 'name' field
        loop {
            if cursor.field_name() == Some("name") {
                break;
            }
            if !cursor.goto_next_sibling() {
                panic!("Could not find 'name' field");
            }
        }

        assert_eq!(cursor.field_name(), Some("name"));
        assert_eq!(cursor.node().kind(), "identifier");
    }

    #[test]
    fn test_tree_cursor_goto_first_child_for_field() {
        let code = "fn main() {}";
        let tree = parse(code, Language::Rust).unwrap();
        let mut cursor = TreeCursor::new(tree.root_node());

        cursor.goto_first_child(); // function_item
        assert!(cursor.goto_first_child_for_field("name"));
        assert_eq!(cursor.node().kind(), "identifier");
        assert_eq!(node_text(cursor.node(), code), "main");
    }

    #[test]
    fn test_tree_cursor_reset() {
        let code = "fn foo() {} fn bar() {}";
        let tree = parse(code, Language::Rust).unwrap();
        let mut cursor = TreeCursor::new(tree.root_node());

        cursor.goto_first_child();
        cursor.goto_first_child();
        assert_ne!(cursor.node().kind(), "source_file");

        cursor.reset(tree.root_node());
        assert_eq!(cursor.node().kind(), "source_file");
    }

    // ============================================
    // DFS Iterator Tests
    // ============================================

    #[test]
    fn test_dfs_iterator_basic() {
        let code = "fn main() {}";
        let tree = parse(code, Language::Rust).unwrap();
        let cursor = TreeCursor::new(tree.root_node());

        let nodes: Vec<_> = cursor.dfs_iter().collect();
        assert!(!nodes.is_empty());
        assert_eq!(nodes[0].kind(), "source_file");
    }

    #[test]
    fn test_dfs_iterator_visits_all_nodes() {
        let code = "fn main() { let x = 1; }";
        let tree = parse(code, Language::Rust).unwrap();
        let cursor = TreeCursor::new(tree.root_node());

        let kinds: Vec<_> = cursor.dfs_iter().map(|n| n.kind()).collect();
        assert!(kinds.contains(&"source_file"));
        assert!(kinds.contains(&"function_item"));
        assert!(kinds.contains(&"identifier"));
        assert!(kinds.contains(&"block"));
    }

    #[test]
    fn test_dfs_iterator_from_subtree() {
        let code = "fn main() { let x = 1; }";
        let tree = parse(code, Language::Rust).unwrap();
        let mut cursor = TreeCursor::new(tree.root_node());
        cursor.goto_first_child(); // function_item

        let func_node = cursor.node();
        let subtree_cursor = TreeCursor::new(func_node);
        let kinds: Vec<_> = subtree_cursor.dfs_iter().map(|n| n.kind()).collect();

        // Should start from function_item, not source_file
        assert_eq!(kinds[0], "function_item");
        assert!(!kinds.contains(&"source_file"));
    }

    #[test]
    fn test_dfs_iterator_preorder() {
        let code = "fn foo() {}";
        let tree = parse(code, Language::Rust).unwrap();
        let cursor = TreeCursor::new(tree.root_node());

        let nodes: Vec<_> = cursor.dfs_iter().collect();

        // In pre-order, parent comes before children
        let source_idx = nodes.iter().position(|n| n.kind() == "source_file");
        let func_idx = nodes.iter().position(|n| n.kind() == "function_item");
        assert!(source_idx < func_idx);
    }

    // ============================================
    // Node Type Matching Tests
    // ============================================

    #[test]
    fn test_matches_kind() {
        let code = "fn main() {}";
        let tree = parse(code, Language::Rust).unwrap();
        let root = tree.root_node();

        assert!(matches_kind(root, "source_file"));
        assert!(!matches_kind(root, "function_item"));
    }

    #[test]
    fn test_matches_any_kind() {
        let code = "fn main() {}";
        let tree = parse(code, Language::Rust).unwrap();
        let root = tree.root_node();

        assert!(matches_any_kind(root, &["source_file", "module"]));
        assert!(!matches_any_kind(root, &["function_item", "struct_item"]));
    }

    #[test]
    fn test_is_named() {
        let code = "fn main() {}";
        let tree = parse(code, Language::Rust).unwrap();
        let cursor = TreeCursor::new(tree.root_node());

        for node in cursor.dfs_iter() {
            // 'fn', '(', ')', '{', '}' are not named
            // 'source_file', 'function_item', 'identifier', etc. are named
            if node.kind() == "source_file" || node.kind() == "function_item" {
                assert!(is_named(node));
            }
        }
    }

    #[test]
    fn test_has_error() {
        let valid_code = "fn main() {}";
        let tree = parse(valid_code, Language::Rust).unwrap();
        assert!(!has_error(tree.root_node()));

        let invalid_code = "fn main( {}";
        let tree = parse(invalid_code, Language::Rust).unwrap();
        assert!(has_error(tree.root_node()));
    }

    #[test]
    fn test_is_error() {
        let invalid_code = "fn @@ main() {}";
        let tree = parse(invalid_code, Language::Rust).unwrap();

        let has_error_node = TreeCursor::new(tree.root_node())
            .dfs_iter()
            .any(|n| is_error(n));
        assert!(has_error_node);
    }

    #[test]
    fn test_find_nodes_by_kind() {
        let code = "fn foo() {} fn bar() {} fn baz() {}";
        let tree = parse(code, Language::Rust).unwrap();

        let functions: Vec<_> = find_nodes_by_kind(tree.root_node(), "function_item").collect();
        assert_eq!(functions.len(), 3);
    }

    #[test]
    fn test_find_nodes_by_kinds() {
        let code = r#"
            fn foo() {}
            struct Bar {}
            fn baz() {}
        "#;
        let tree = parse(code, Language::Rust).unwrap();

        let items: Vec<_> =
            find_nodes_by_kinds(tree.root_node(), &["function_item", "struct_item"]).collect();
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_find_ancestor_by_kind() {
        let code = "fn main() { let x = 1; }";
        let tree = parse(code, Language::Rust).unwrap();

        // Find the identifier 'x'
        let x_node = TreeCursor::new(tree.root_node())
            .dfs_iter()
            .find(|n| n.kind() == "identifier" && node_text(*n, code) == "x")
            .unwrap();

        let func = find_ancestor_by_kind(x_node, "function_item");
        assert!(func.is_some());
        assert_eq!(func.unwrap().kind(), "function_item");
    }

    #[test]
    fn test_ancestors() {
        let code = "fn main() { let x = 1; }";
        let tree = parse(code, Language::Rust).unwrap();

        let x_node = TreeCursor::new(tree.root_node())
            .dfs_iter()
            .find(|n| n.kind() == "identifier" && node_text(*n, code) == "x")
            .unwrap();

        let ancestor_kinds: Vec<_> = ancestors(x_node).map(|n| n.kind()).collect();
        assert!(ancestor_kinds.contains(&"let_declaration"));
        assert!(ancestor_kinds.contains(&"block"));
        assert!(ancestor_kinds.contains(&"function_item"));
        assert!(ancestor_kinds.contains(&"source_file"));
    }

    #[test]
    fn test_find_child_by_kind() {
        let code = "fn main() {}";
        let tree = parse(code, Language::Rust).unwrap();

        let func = tree.root_node().child(0).unwrap();
        assert_eq!(func.kind(), "function_item");

        let name = find_child_by_kind(func, "identifier");
        assert!(name.is_some());
        assert_eq!(node_text(name.unwrap(), code), "main");
    }

    #[test]
    fn test_find_children_by_kind() {
        let code = "fn foo(a: i32, b: i32) {}";
        let tree = parse(code, Language::Rust).unwrap();

        let func = tree.root_node().child(0).unwrap();
        let params = find_child_by_kind(func, "parameters").unwrap();
        let param_nodes = find_children_by_kind(params, "parameter");

        assert_eq!(param_nodes.len(), 2);
    }

    #[test]
    fn test_child_by_field() {
        let code = "fn main() {}";
        let tree = parse(code, Language::Rust).unwrap();

        let func = tree.root_node().child(0).unwrap();
        let name = child_by_field(func, "name");
        assert!(name.is_some());
        assert_eq!(node_text(name.unwrap(), code), "main");

        let body = child_by_field(func, "body");
        assert!(body.is_some());
        assert_eq!(body.unwrap().kind(), "block");
    }

    // ============================================
    // Text Extraction Tests
    // ============================================

    #[test]
    fn test_node_text() {
        let code = "fn main() {}";
        let tree = parse(code, Language::Rust).unwrap();

        let func = tree.root_node().child(0).unwrap();
        assert_eq!(node_text(func, code), "fn main() {}");
    }

    #[test]
    fn test_node_text_owned() {
        let code = "fn main() {}";
        let tree = parse(code, Language::Rust).unwrap();

        let func = tree.root_node().child(0).unwrap();
        let text: String = node_text_owned(func, code);
        assert_eq!(text, "fn main() {}");
    }

    #[test]
    fn test_node_byte_range() {
        let code = "fn main() {}";
        let tree = parse(code, Language::Rust).unwrap();

        let func = tree.root_node().child(0).unwrap();
        let range = node_byte_range(func);
        assert_eq!(range.start, 0);
        assert_eq!(range.end, code.len());
    }

    #[test]
    fn test_text_from_range() {
        let code = "fn main() {}";
        let tree = parse(code, Language::Rust).unwrap();

        let func = tree.root_node().child(0).unwrap();
        let range = node_byte_range(func);
        assert_eq!(text_from_range(code, &range), code);
    }

    #[test]
    fn test_node_positions() {
        let code = "fn main() {\n    let x = 1;\n}";
        let tree = parse(code, Language::Rust).unwrap();

        let root = tree.root_node();
        let (start_row, start_col) = node_start_position(root);
        let (end_row, end_col) = node_end_position(root);

        assert_eq!(start_row, 0);
        assert_eq!(start_col, 0);
        assert_eq!(end_row, 2);
        assert_eq!(end_col, 1);
    }

    #[test]
    fn test_node_line_count() {
        let code = "fn main() {\n    let x = 1;\n}";
        let tree = parse(code, Language::Rust).unwrap();

        assert_eq!(node_line_count(tree.root_node()), 3);
    }

    #[test]
    fn test_child_text() {
        let code = "fn main() {}";
        let tree = parse(code, Language::Rust).unwrap();

        let func = tree.root_node().child(0).unwrap();
        let name_text = child_text(func, "name", code);
        assert_eq!(name_text, Some("main"));
    }

    #[test]
    fn test_collect_named_children_text() {
        let code = "use std::io::Write;";
        let tree = parse(code, Language::Rust).unwrap();

        // Get the use_declaration node
        let use_decl = tree.root_node().child(0).unwrap();
        let text = collect_named_children_text(use_decl, code, " ");
        // Should collect the named children text
        assert!(!text.is_empty());
    }

    // ============================================
    // Python Tests (ensure language agnostic)
    // ============================================

    #[test]
    fn test_python_traversal() {
        let code = "def foo():\n    pass\n\ndef bar():\n    pass";
        let tree = parse(code, Language::Python).unwrap();

        let functions: Vec<_> =
            find_nodes_by_kind(tree.root_node(), "function_definition").collect();
        assert_eq!(functions.len(), 2);
    }

    #[test]
    fn test_python_text_extraction() {
        let code = "def hello():\n    print('world')";
        let tree = parse(code, Language::Python).unwrap();

        let func = tree.root_node().child(0).unwrap();
        let name = child_by_field(func, "name");
        assert!(name.is_some());
        assert_eq!(node_text(name.unwrap(), code), "hello");
    }

    #[test]
    fn test_python_nested_traversal() {
        let code = r#"
class MyClass:
    def method1(self):
        pass

    def method2(self):
        pass
"#;
        let tree = parse(code, Language::Python).unwrap();

        let methods: Vec<_> =
            find_nodes_by_kind(tree.root_node(), "function_definition").collect();
        assert_eq!(methods.len(), 2);
    }
}
