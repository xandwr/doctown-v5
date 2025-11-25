//! Symbol context generation for LLM documentation.
//!
//! This module provides structured context about symbols that can be used
//! by LLMs to generate high-quality documentation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::graph::Graph;

/// Structured context about a symbol for LLM documentation generation.
///
/// This context includes both static information (name, kind, signature)
/// and relational information (what it calls, what calls it, related symbols).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SymbolContext {
    /// Symbol identifier.
    pub symbol_id: String,

    /// Symbol name (e.g., "calculate_total", "User").
    pub name: String,

    /// Kind of symbol (e.g., "function", "class", "method", "struct").
    pub kind: String,

    /// Programming language (e.g., "rust", "python").
    pub language: String,

    /// File path where the symbol is defined.
    pub file_path: String,

    /// Function/method signature or struct/class definition.
    pub signature: String,

    /// List of function/method names this symbol calls (max 10).
    pub calls: Vec<String>,

    /// List of function/method names that call this symbol (max 10).
    pub called_by: Vec<String>,

    /// List of imports used by this symbol (max 10).
    pub imports: Vec<String>,

    /// Top 3 related symbol names (based on semantic similarity).
    pub related_symbols: Vec<String>,

    /// Cluster label this symbol belongs to (if any).
    pub cluster_label: Option<String>,

    /// Centrality score (0.0-1.0) indicating importance in the codebase.
    pub centrality: f64,
}

impl SymbolContext {
    /// Create a new SymbolContext with all required fields.
    pub fn new(
        symbol_id: String,
        name: String,
        kind: String,
        language: String,
        file_path: String,
        signature: String,
    ) -> Self {
        Self {
            symbol_id,
            name,
            kind,
            language,
            file_path,
            signature,
            calls: Vec::new(),
            called_by: Vec::new(),
            imports: Vec::new(),
            related_symbols: Vec::new(),
            cluster_label: None,
            centrality: 0.0,
        }
    }

    /// Set the calls list (truncated to max 10 items).
    pub fn with_calls(mut self, calls: Vec<String>) -> Self {
        self.calls = Self::truncate_list(calls, 10);
        self
    }

    /// Set the called_by list (truncated to max 10 items).
    pub fn with_called_by(mut self, called_by: Vec<String>) -> Self {
        self.called_by = Self::truncate_list(called_by, 10);
        self
    }

    /// Set the imports list (truncated to max 10 items).
    pub fn with_imports(mut self, imports: Vec<String>) -> Self {
        self.imports = Self::truncate_list(imports, 10);
        self
    }

    /// Set the related symbols list (max 3 items).
    pub fn with_related_symbols(mut self, related: Vec<String>) -> Self {
        self.related_symbols = Self::truncate_list(related, 3);
        self
    }

    /// Set the cluster label.
    pub fn with_cluster_label(mut self, label: Option<String>) -> Self {
        self.cluster_label = label;
        self
    }

    /// Set the centrality score.
    pub fn with_centrality(mut self, centrality: f64) -> Self {
        self.centrality = centrality;
        self
    }

    /// Truncate a list to the specified maximum length.
    fn truncate_list(mut list: Vec<String>, max_len: usize) -> Vec<String> {
        list.truncate(max_len);
        list
    }
}

/// Generate symbol contexts from a graph and additional metadata.
pub struct ContextGenerator {
    /// Map from symbol_id to cluster label.
    cluster_labels: HashMap<String, String>,
    /// Map from symbol_id to language.
    languages: HashMap<String, String>,
    /// Map from symbol_id to imports.
    imports: HashMap<String, Vec<String>>,
}

impl ContextGenerator {
    /// Create a new context generator.
    pub fn new() -> Self {
        Self {
            cluster_labels: HashMap::new(),
            languages: HashMap::new(),
            imports: HashMap::new(),
        }
    }

    /// Set cluster labels for symbols.
    pub fn with_cluster_labels(mut self, labels: HashMap<String, String>) -> Self {
        self.cluster_labels = labels;
        self
    }

    /// Set languages for symbols.
    pub fn with_languages(mut self, languages: HashMap<String, String>) -> Self {
        self.languages = languages;
        self
    }

    /// Set imports for symbols.
    pub fn with_imports(mut self, imports: HashMap<String, Vec<String>>) -> Self {
        self.imports = imports;
        self
    }

    /// Generate contexts for all symbols in the graph.
    pub fn generate(&self, graph: &Graph) -> Vec<SymbolContext> {
        let centralities = graph.all_degree_centralities();
        let mut contexts = Vec::new();

        for node in &graph.nodes {
            let symbol_id = &node.id;

            // Extract metadata from the node
            let name = node.metadata.get("name").cloned().unwrap_or_default();
            let kind = node.metadata.get("kind").cloned().unwrap_or_default();
            let file_path = node.metadata.get("file_path").cloned().unwrap_or_default();
            let signature = node.metadata.get("signature").cloned().unwrap_or_default();

            // Get language from metadata
            let language = self.languages.get(symbol_id).cloned().unwrap_or_default();

            // Build lists of calls and called_by
            let calls = self.get_calls(graph, symbol_id);
            let called_by = self.get_called_by(graph, symbol_id);

            // Get related symbols (top 3 by similarity)
            let related_symbols = self.get_related_symbols(graph, symbol_id, 3);

            // Get imports for this symbol
            let imports = self.imports.get(symbol_id).cloned().unwrap_or_default();

            // Get cluster label
            let cluster_label = self.cluster_labels.get(symbol_id).cloned();

            // Get centrality score
            let centrality = centralities.get(symbol_id).copied().unwrap_or(0.0);

            // Build context
            let context = SymbolContext::new(
                symbol_id.clone(),
                name,
                kind,
                language,
                file_path,
                signature,
            )
            .with_calls(calls)
            .with_called_by(called_by)
            .with_imports(imports)
            .with_related_symbols(related_symbols)
            .with_cluster_label(cluster_label)
            .with_centrality(centrality);

            contexts.push(context);
        }

        contexts
    }

    /// Get the list of symbols this symbol calls.
    fn get_calls(&self, graph: &Graph, symbol_id: &str) -> Vec<String> {
        graph
            .edges
            .iter()
            .filter(|e| e.source == symbol_id && e.kind == crate::EdgeKind::Calls)
            .filter_map(|e| {
                // Get the target node's name
                graph
                    .get_node(&e.target)
                    .and_then(|n| n.metadata.get("name"))
                    .cloned()
            })
            .collect()
    }

    /// Get the list of symbols that call this symbol.
    fn get_called_by(&self, graph: &Graph, symbol_id: &str) -> Vec<String> {
        graph
            .edges
            .iter()
            .filter(|e| e.target == symbol_id && e.kind == crate::EdgeKind::Calls)
            .filter_map(|e| {
                // Get the source node's name
                graph
                    .get_node(&e.source)
                    .and_then(|n| n.metadata.get("name"))
                    .cloned()
            })
            .collect()
    }

    /// Get the top N related symbols based on semantic similarity.
    fn get_related_symbols(&self, graph: &Graph, symbol_id: &str, top_n: usize) -> Vec<String> {
        let mut related: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| {
                (e.source == symbol_id || e.target == symbol_id)
                    && e.kind == crate::EdgeKind::Related
            })
            .filter_map(|e| {
                // Get the other end of the edge
                let other_id = if e.source == symbol_id {
                    &e.target
                } else {
                    &e.source
                };

                // Get the node's name and weight
                graph
                    .get_node(other_id)
                    .and_then(|n| n.metadata.get("name"))
                    .map(|name| (name.clone(), e.weight.unwrap_or(0.0)))
            })
            .collect();

        // Sort by weight descending
        related.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top N and return just the names
        related
            .into_iter()
            .take(top_n)
            .map(|(name, _)| name)
            .collect()
    }
}

impl Default for ContextGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_context_creation() {
        let context = SymbolContext::new(
            "test::foo".to_string(),
            "foo".to_string(),
            "function".to_string(),
            "rust".to_string(),
            "src/lib.rs".to_string(),
            "pub fn foo() -> i32".to_string(),
        );

        assert_eq!(context.symbol_id, "test::foo");
        assert_eq!(context.name, "foo");
        assert_eq!(context.kind, "function");
        assert_eq!(context.language, "rust");
        assert_eq!(context.file_path, "src/lib.rs");
        assert_eq!(context.signature, "pub fn foo() -> i32");
        assert!(context.calls.is_empty());
        assert!(context.called_by.is_empty());
        assert!(context.imports.is_empty());
        assert!(context.related_symbols.is_empty());
        assert_eq!(context.cluster_label, None);
        assert_eq!(context.centrality, 0.0);
    }

    #[test]
    fn test_symbol_context_with_calls() {
        let calls = vec!["bar".to_string(), "baz".to_string()];
        let context = SymbolContext::new(
            "test::foo".to_string(),
            "foo".to_string(),
            "function".to_string(),
            "rust".to_string(),
            "src/lib.rs".to_string(),
            "pub fn foo()".to_string(),
        )
        .with_calls(calls.clone());

        assert_eq!(context.calls, calls);
    }

    #[test]
    fn test_truncate_calls_list() {
        let calls: Vec<String> = (0..20).map(|i| format!("fn_{}", i)).collect();
        let context = SymbolContext::new(
            "test::foo".to_string(),
            "foo".to_string(),
            "function".to_string(),
            "rust".to_string(),
            "src/lib.rs".to_string(),
            "pub fn foo()".to_string(),
        )
        .with_calls(calls);

        assert_eq!(context.calls.len(), 10);
    }

    #[test]
    fn test_truncate_related_symbols() {
        let related: Vec<String> = (0..10).map(|i| format!("symbol_{}", i)).collect();
        let context = SymbolContext::new(
            "test::foo".to_string(),
            "foo".to_string(),
            "function".to_string(),
            "rust".to_string(),
            "src/lib.rs".to_string(),
            "pub fn foo()".to_string(),
        )
        .with_related_symbols(related);

        assert_eq!(context.related_symbols.len(), 3);
    }

    #[test]
    fn test_context_with_all_fields() {
        let context = SymbolContext::new(
            "test::foo".to_string(),
            "foo".to_string(),
            "function".to_string(),
            "rust".to_string(),
            "src/lib.rs".to_string(),
            "pub fn foo() -> i32".to_string(),
        )
        .with_calls(vec!["bar".to_string()])
        .with_called_by(vec!["main".to_string()])
        .with_imports(vec!["std::collections::HashMap".to_string()])
        .with_related_symbols(vec!["baz".to_string()])
        .with_cluster_label(Some("data_processing".to_string()))
        .with_centrality(0.75);

        assert_eq!(context.calls, vec!["bar"]);
        assert_eq!(context.called_by, vec!["main"]);
        assert_eq!(context.imports, vec!["std::collections::HashMap"]);
        assert_eq!(context.related_symbols, vec!["baz"]);
        assert_eq!(context.cluster_label, Some("data_processing".to_string()));
        assert_eq!(context.centrality, 0.75);
    }

    #[test]
    fn test_context_serialization() {
        let context = SymbolContext::new(
            "test::foo".to_string(),
            "foo".to_string(),
            "function".to_string(),
            "rust".to_string(),
            "src/lib.rs".to_string(),
            "pub fn foo()".to_string(),
        )
        .with_centrality(0.5);

        let json = serde_json::to_string(&context).unwrap();
        let deserialized: SymbolContext = serde_json::from_str(&json).unwrap();

        assert_eq!(context, deserialized);
    }

    #[test]
    fn test_context_generator_basic() {
        use crate::graph::{Edge, EdgeKind, Graph, Node};
        use std::collections::HashMap;

        // Build a simple graph
        let mut graph = Graph::new();

        // Add nodes
        let mut metadata1 = HashMap::new();
        metadata1.insert("name".to_string(), "foo".to_string());
        metadata1.insert("kind".to_string(), "function".to_string());
        metadata1.insert("file_path".to_string(), "src/lib.rs".to_string());
        metadata1.insert("signature".to_string(), "fn foo()".to_string());
        graph.add_node(Node::new("sym1".to_string(), metadata1));

        let mut metadata2 = HashMap::new();
        metadata2.insert("name".to_string(), "bar".to_string());
        metadata2.insert("kind".to_string(), "function".to_string());
        metadata2.insert("file_path".to_string(), "src/lib.rs".to_string());
        metadata2.insert("signature".to_string(), "fn bar()".to_string());
        graph.add_node(Node::new("sym2".to_string(), metadata2));

        // Add call edge: foo calls bar
        graph.add_edge(Edge {
            source: "sym1".to_string(),
            target: "sym2".to_string(),
            kind: EdgeKind::Calls,
            weight: None,
        });

        // Build language map
        let mut languages = HashMap::new();
        languages.insert("sym1".to_string(), "rust".to_string());
        languages.insert("sym2".to_string(), "rust".to_string());

        // Build cluster labels
        let mut cluster_labels = HashMap::new();
        cluster_labels.insert("sym1".to_string(), "utility_functions".to_string());
        cluster_labels.insert("sym2".to_string(), "utility_functions".to_string());

        // Build imports
        let mut imports = HashMap::new();
        imports.insert("sym1".to_string(), vec!["std::io".to_string()]);

        // Generate contexts
        let generator = ContextGenerator::new()
            .with_languages(languages)
            .with_cluster_labels(cluster_labels)
            .with_imports(imports);

        let contexts = generator.generate(&graph);

        assert_eq!(contexts.len(), 2);

        // Check foo context
        let foo_ctx = contexts.iter().find(|c| c.symbol_id == "sym1").unwrap();
        assert_eq!(foo_ctx.name, "foo");
        assert_eq!(foo_ctx.language, "rust");
        assert_eq!(foo_ctx.calls, vec!["bar"]);
        assert!(foo_ctx.called_by.is_empty());
        assert_eq!(foo_ctx.imports, vec!["std::io"]);
        assert_eq!(foo_ctx.cluster_label, Some("utility_functions".to_string()));
        assert!(foo_ctx.centrality > 0.0);

        // Check bar context
        let bar_ctx = contexts.iter().find(|c| c.symbol_id == "sym2").unwrap();
        assert_eq!(bar_ctx.name, "bar");
        assert_eq!(bar_ctx.called_by, vec!["foo"]);
        assert!(bar_ctx.calls.is_empty());
    }

    #[test]
    fn test_context_generator_related_symbols() {
        use crate::graph::{Edge, EdgeKind, Graph, Node};
        use std::collections::HashMap;

        let mut graph = Graph::new();

        // Add nodes
        let mut metadata1 = HashMap::new();
        metadata1.insert("name".to_string(), "foo".to_string());
        metadata1.insert("kind".to_string(), "function".to_string());
        metadata1.insert("file_path".to_string(), "src/lib.rs".to_string());
        metadata1.insert("signature".to_string(), "fn foo()".to_string());
        graph.add_node(Node::new("sym1".to_string(), metadata1));

        // Add related symbols
        for i in 2..=5 {
            let mut metadata = HashMap::new();
            metadata.insert("name".to_string(), format!("related_{}", i));
            metadata.insert("kind".to_string(), "function".to_string());
            metadata.insert("file_path".to_string(), "src/lib.rs".to_string());
            graph.add_node(Node::new(format!("sym{}", i), metadata));

            // Add related edge with weight
            graph.add_edge(Edge {
                source: "sym1".to_string(),
                target: format!("sym{}", i),
                kind: EdgeKind::Related,
                weight: Some(1.0 - (i as f32 * 0.1)),
            });
        }

        let mut languages = HashMap::new();
        for i in 1..=5 {
            languages.insert(format!("sym{}", i), "rust".to_string());
        }

        let generator = ContextGenerator::new().with_languages(languages);
        let contexts = generator.generate(&graph);

        // Find foo context
        let foo_ctx = contexts.iter().find(|c| c.symbol_id == "sym1").unwrap();

        // Should have top 3 related symbols
        assert_eq!(foo_ctx.related_symbols.len(), 3);
        // Should be sorted by similarity (highest first)
        assert_eq!(foo_ctx.related_symbols[0], "related_2");
        assert_eq!(foo_ctx.related_symbols[1], "related_3");
        assert_eq!(foo_ctx.related_symbols[2], "related_4");
    }

    #[test]
    fn test_context_generator_truncates_lists() {
        use crate::graph::{Edge, EdgeKind, Graph, Node};
        use std::collections::HashMap;

        let mut graph = Graph::new();

        // Add main node
        let mut metadata = HashMap::new();
        metadata.insert("name".to_string(), "main".to_string());
        metadata.insert("kind".to_string(), "function".to_string());
        metadata.insert("file_path".to_string(), "src/main.rs".to_string());
        graph.add_node(Node::new("main".to_string(), metadata));

        // Add 15 nodes that main calls
        for i in 1..=15 {
            let mut metadata = HashMap::new();
            metadata.insert("name".to_string(), format!("fn_{}", i));
            metadata.insert("kind".to_string(), "function".to_string());
            graph.add_node(Node::new(format!("fn_{}", i), metadata));

            graph.add_edge(Edge {
                source: "main".to_string(),
                target: format!("fn_{}", i),
                kind: EdgeKind::Calls,
                weight: None,
            });
        }

        let mut languages = HashMap::new();
        languages.insert("main".to_string(), "rust".to_string());

        // Add 15 imports
        let mut imports = HashMap::new();
        imports.insert(
            "main".to_string(),
            (1..=15).map(|i| format!("module_{}", i)).collect(),
        );

        let generator = ContextGenerator::new()
            .with_languages(languages)
            .with_imports(imports);

        let contexts = generator.generate(&graph);
        let main_ctx = contexts.iter().find(|c| c.symbol_id == "main").unwrap();

        // Should truncate calls to 10
        assert_eq!(main_ctx.calls.len(), 10);
        // Should truncate imports to 10
        assert_eq!(main_ctx.imports.len(), 10);
    }
}
