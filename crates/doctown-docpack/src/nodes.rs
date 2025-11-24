use serde::{Deserialize, Serialize};

/// Container for all symbols in the docpack
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Nodes {
    pub symbols: Vec<Symbol>,
}

/// A symbol with its metadata and documentation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Symbol {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub language: String,
    pub file_path: String,
    pub byte_range: (usize, usize),
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    
    pub calls: Vec<String>,
    pub called_by: Vec<String>,
    pub imports: Vec<String>,
    
    pub cluster_id: String,
    pub centrality: f64,
    
    pub documentation: Documentation,
}

/// Documentation for a symbol
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Documentation {
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl Nodes {
    /// Create a new Nodes container
    pub fn new(symbols: Vec<Symbol>) -> Self {
        Self { symbols }
    }

    /// Create an empty Nodes container
    pub fn empty() -> Self {
        Self {
            symbols: Vec::new(),
        }
    }

    /// Add a symbol to the collection
    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.symbols.push(symbol);
    }

    /// Get the number of symbols
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if there are no symbols
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Serialize to JSON bytes
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec_pretty(self)
    }

    /// Deserialize from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Deserialize from JSON bytes
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

impl Symbol {
    /// Create a new symbol with required fields
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        name: String,
        kind: String,
        language: String,
        file_path: String,
        byte_range: (usize, usize),
        cluster_id: String,
        documentation_summary: String,
    ) -> Self {
        Self {
            id,
            name,
            kind,
            language,
            file_path,
            byte_range,
            signature: None,
            calls: Vec::new(),
            called_by: Vec::new(),
            imports: Vec::new(),
            cluster_id,
            centrality: 0.0,
            documentation: Documentation {
                summary: documentation_summary,
                details: None,
            },
        }
    }

    /// Set the signature
    pub fn with_signature(mut self, signature: String) -> Self {
        self.signature = Some(signature);
        self
    }

    /// Set the calls
    pub fn with_calls(mut self, calls: Vec<String>) -> Self {
        self.calls = calls;
        self
    }

    /// Set the called_by
    pub fn with_called_by(mut self, called_by: Vec<String>) -> Self {
        self.called_by = called_by;
        self
    }

    /// Set the imports
    pub fn with_imports(mut self, imports: Vec<String>) -> Self {
        self.imports = imports;
        self
    }

    /// Set the centrality
    pub fn with_centrality(mut self, centrality: f64) -> Self {
        self.centrality = centrality;
        self
    }

    /// Set the documentation details
    pub fn with_documentation_details(mut self, details: String) -> Self {
        self.documentation.details = Some(details);
        self
    }
}

impl Documentation {
    /// Create new documentation with just a summary
    pub fn new(summary: String) -> Self {
        Self {
            summary,
            details: None,
        }
    }

    /// Create documentation with summary and details
    pub fn with_details(summary: String, details: String) -> Self {
        Self {
            summary,
            details: Some(details),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_creation() {
        let symbol = Symbol::new(
            "sym_main".to_string(),
            "main".to_string(),
            "function".to_string(),
            "rust".to_string(),
            "src/main.rs".to_string(),
            (0, 200),
            "cluster_init".to_string(),
            "Main entry point".to_string(),
        );

        assert_eq!(symbol.id, "sym_main");
        assert_eq!(symbol.name, "main");
        assert_eq!(symbol.kind, "function");
        assert_eq!(symbol.language, "rust");
        assert_eq!(symbol.file_path, "src/main.rs");
        assert_eq!(symbol.byte_range, (0, 200));
        assert_eq!(symbol.cluster_id, "cluster_init");
        assert_eq!(symbol.documentation.summary, "Main entry point");
        assert_eq!(symbol.centrality, 0.0);
    }

    #[test]
    fn test_symbol_builder() {
        let symbol = Symbol::new(
            "sym_main".to_string(),
            "main".to_string(),
            "function".to_string(),
            "rust".to_string(),
            "src/main.rs".to_string(),
            (0, 200),
            "cluster_init".to_string(),
            "Main entry point".to_string(),
        )
        .with_signature("fn main()".to_string())
        .with_calls(vec!["sym_parse_args".to_string()])
        .with_centrality(0.84);

        assert_eq!(symbol.signature, Some("fn main()".to_string()));
        assert_eq!(symbol.calls, vec!["sym_parse_args".to_string()]);
        assert_eq!(symbol.centrality, 0.84);
    }

    #[test]
    fn test_nodes_creation() {
        let symbol1 = Symbol::new(
            "sym_1".to_string(),
            "func1".to_string(),
            "function".to_string(),
            "rust".to_string(),
            "src/lib.rs".to_string(),
            (0, 100),
            "cluster_1".to_string(),
            "Test function".to_string(),
        );

        let symbol2 = Symbol::new(
            "sym_2".to_string(),
            "func2".to_string(),
            "function".to_string(),
            "rust".to_string(),
            "src/lib.rs".to_string(),
            (100, 200),
            "cluster_1".to_string(),
            "Another test function".to_string(),
        );

        let nodes = Nodes::new(vec![symbol1, symbol2]);

        assert_eq!(nodes.len(), 2);
        assert!(!nodes.is_empty());
        assert_eq!(nodes.symbols[0].id, "sym_1");
        assert_eq!(nodes.symbols[1].id, "sym_2");
    }

    #[test]
    fn test_nodes_json_roundtrip() {
        let symbol = Symbol::new(
            "sym_main".to_string(),
            "main".to_string(),
            "function".to_string(),
            "rust".to_string(),
            "src/main.rs".to_string(),
            (0, 200),
            "cluster_init".to_string(),
            "Main entry point".to_string(),
        )
        .with_signature("fn main()".to_string());

        let nodes = Nodes::new(vec![symbol]);
        let json = nodes.to_json().unwrap();
        let parsed = Nodes::from_json(&json).unwrap();

        assert_eq!(nodes.len(), parsed.len());
        assert_eq!(nodes.symbols[0].id, parsed.symbols[0].id);
        assert_eq!(nodes.symbols[0].signature, parsed.symbols[0].signature);
    }

    #[test]
    fn test_nodes_json_format() {
        let symbol = Symbol::new(
            "sym_main_fn".to_string(),
            "main".to_string(),
            "function".to_string(),
            "rust".to_string(),
            "src/main.rs".to_string(),
            (0, 200),
            "cluster_auth".to_string(),
            "This function initializes the application...".to_string(),
        )
        .with_signature("fn main()".to_string())
        .with_calls(vec!["sym_parse_args".to_string()])
        .with_imports(vec!["std::env".to_string()])
        .with_centrality(0.84);

        let nodes = Nodes::new(vec![symbol]);
        let json = nodes.to_json().unwrap();

        // Verify key fields are present
        assert!(json.contains("\"id\": \"sym_main_fn\""));
        assert!(json.contains("\"name\": \"main\""));
        assert!(json.contains("\"kind\": \"function\""));
        assert!(json.contains("\"language\": \"rust\""));
        assert!(json.contains("\"file_path\": \"src/main.rs\""));
        assert!(json.contains("\"signature\": \"fn main()\""));
        assert!(json.contains("\"cluster_id\": \"cluster_auth\""));
        assert!(json.contains("\"centrality\": 0.84"));
        assert!(json.contains("\"summary\": \"This function initializes the application...\""));
    }
}
