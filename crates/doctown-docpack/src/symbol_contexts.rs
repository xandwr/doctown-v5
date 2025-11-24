use serde::{Deserialize, Serialize};

/// Container for symbol contexts (optional, for reproducibility)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SymbolContexts {
    pub contexts: Vec<SymbolContext>,
}

/// Context for a single symbol (raw prompt text for reproducibility)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SymbolContext {
    pub symbol_id: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

impl SymbolContexts {
    /// Create a new SymbolContexts container
    pub fn new(contexts: Vec<SymbolContext>) -> Self {
        Self { contexts }
    }

    /// Create an empty SymbolContexts container
    pub fn empty() -> Self {
        Self {
            contexts: Vec::new(),
        }
    }

    /// Add a context
    pub fn add_context(&mut self, context: SymbolContext) {
        self.contexts.push(context);
    }

    /// Get the number of contexts
    pub fn len(&self) -> usize {
        self.contexts.len()
    }

    /// Check if there are no contexts
    pub fn is_empty(&self) -> bool {
        self.contexts.is_empty()
    }

    /// Find context by symbol_id
    pub fn get_context(&self, symbol_id: &str) -> Option<&SymbolContext> {
        self.contexts.iter().find(|c| c.symbol_id == symbol_id)
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

impl SymbolContext {
    /// Create a new symbol context
    pub fn new(symbol_id: String, prompt: String) -> Self {
        Self {
            symbol_id,
            prompt,
            model: None,
            temperature: None,
        }
    }

    /// Create a context with model information
    pub fn with_model(mut self, model: String) -> Self {
        self.model = Some(model);
        self
    }

    /// Create a context with temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_context_creation() {
        let context = SymbolContext::new(
            "sym_test".to_string(),
            "Explain this function".to_string(),
        );

        assert_eq!(context.symbol_id, "sym_test");
        assert_eq!(context.prompt, "Explain this function");
        assert_eq!(context.model, None);
        assert_eq!(context.temperature, None);
    }

    #[test]
    fn test_symbol_context_with_metadata() {
        let context = SymbolContext::new(
            "sym_test".to_string(),
            "Explain this function".to_string(),
        )
        .with_model("gpt-4".to_string())
        .with_temperature(0.7);

        assert_eq!(context.model, Some("gpt-4".to_string()));
        assert_eq!(context.temperature, Some(0.7));
    }

    #[test]
    fn test_symbol_contexts_creation() {
        let context1 = SymbolContext::new(
            "sym_a".to_string(),
            "Describe sym_a".to_string(),
        );
        let context2 = SymbolContext::new(
            "sym_b".to_string(),
            "Describe sym_b".to_string(),
        );

        let contexts = SymbolContexts::new(vec![context1, context2]);

        assert_eq!(contexts.len(), 2);
        assert!(!contexts.is_empty());
    }

    #[test]
    fn test_symbol_contexts_get() {
        let context1 = SymbolContext::new(
            "sym_a".to_string(),
            "Describe sym_a".to_string(),
        );
        let context2 = SymbolContext::new(
            "sym_b".to_string(),
            "Describe sym_b".to_string(),
        );

        let contexts = SymbolContexts::new(vec![context1, context2]);

        let found = contexts.get_context("sym_a");
        assert!(found.is_some());
        assert_eq!(found.unwrap().prompt, "Describe sym_a");

        let not_found = contexts.get_context("sym_c");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_symbol_contexts_json_roundtrip() {
        let context = SymbolContext::new(
            "sym_test".to_string(),
            "Test prompt".to_string(),
        )
        .with_model("gpt-4".to_string());

        let contexts = SymbolContexts::new(vec![context]);

        let json = contexts.to_json().unwrap();
        let parsed = SymbolContexts::from_json(&json).unwrap();

        assert_eq!(parsed, contexts);
    }

    #[test]
    fn test_empty_contexts() {
        let contexts = SymbolContexts::empty();
        assert_eq!(contexts.len(), 0);
        assert!(contexts.is_empty());
    }

    #[test]
    fn test_add_context() {
        let mut contexts = SymbolContexts::empty();
        
        contexts.add_context(SymbolContext::new(
            "sym_a".to_string(),
            "Test".to_string(),
        ));

        assert_eq!(contexts.len(), 1);
        assert!(!contexts.is_empty());
    }
}
