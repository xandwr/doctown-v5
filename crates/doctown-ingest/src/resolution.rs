//! Symbol resolution and call graph construction.

use doctown_common::ids::SymbolId;
use doctown_common::types::{Call, Import};
use std::collections::HashMap;

use crate::symbol::Symbol;

/// A table mapping symbol names to their IDs for resolution.
#[derive(Debug, Clone)]
pub struct SymbolTable {
    /// Map from symbol name to symbol ID
    symbols: HashMap<String, SymbolId>,
    /// Map from symbol ID to symbol info
    symbol_info: HashMap<SymbolId, SymbolInfo>,
    /// Imports that are available in the current scope
    imports: Vec<Import>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct SymbolInfo {
    name: String,
    file_path: String,
}

impl SymbolTable {
    /// Create a new empty symbol table.
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            symbol_info: HashMap::new(),
            imports: Vec::new(),
        }
    }

    /// Add a symbol to the table.
    pub fn add_symbol(&mut self, name: String, symbol_id: SymbolId, file_path: String) {
        self.symbols.insert(name.clone(), symbol_id.clone());
        self.symbol_info.insert(symbol_id, SymbolInfo { name, file_path });
    }

    /// Add multiple symbols from a list.
    pub fn add_symbols(&mut self, symbols: &[Symbol], file_path: &str) {
        for symbol in symbols {
            // Generate a simple symbol ID based on name and file
            let id_string = format!("sym_{}::{}", file_path, symbol.name);
            if let Ok(symbol_id) = SymbolId::new(id_string) {
                self.add_symbol(symbol.name.clone(), symbol_id, file_path.to_string());
            }
        }
    }

    /// Add imports to the table.
    pub fn add_imports(&mut self, imports: Vec<Import>) {
        self.imports.extend(imports);
    }

    /// Look up a symbol by name, returning its ID if found.
    pub fn lookup(&self, name: &str) -> Option<SymbolId> {
        self.symbols.get(name).cloned()
    }

    /// Try to resolve a call to a symbol ID.
    /// Returns Some(symbol_id) if the call can be resolved locally,
    /// None if it's an external call.
    pub fn resolve_call(&self, call: &Call) -> Option<SymbolId> {
        // Simple resolution: try direct name lookup
        if let Some(id) = self.lookup(&call.name) {
            return Some(id);
        }

        // For method calls, try to extract just the method name
        if call.name.contains('.') {
            // Extract the last part after the dot
            if let Some(method_name) = call.name.split('.').last() {
                if let Some(id) = self.lookup(method_name) {
                    return Some(id);
                }
            }
        }

        // For Rust associated calls (Type::function), try to extract function name
        if call.name.contains("::") {
            if let Some(function_name) = call.name.split("::").last() {
                if let Some(id) = self.lookup(function_name) {
                    return Some(id);
                }
            }
        }

        // Check if the call matches any imported names
        for import in &self.imports {
            // Check if it's a direct import match
            if import.module_path == call.name {
                // This is an imported module/function - consider it external
                return None;
            }

            // Check if call matches an imported item
            if let Some(ref items) = import.imported_items {
                if items.iter().any(|item| item == &call.name) {
                    // This is an imported item - external
                    return None;
                }
            }

            // Check alias match
            if let Some(ref alias) = import.alias {
                if call.name.starts_with(alias) {
                    // This uses an aliased import - external
                    return None;
                }
            }
        }

        // Could not resolve - assume external
        None
    }

    /// Get the number of symbols in the table.
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if the table is empty.
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Get all symbol IDs in the table.
    pub fn symbol_ids(&self) -> Vec<SymbolId> {
        self.symbol_info.keys().cloned().collect()
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolve calls against a symbol table, marking which calls are resolved.
pub fn resolve_calls(calls: &mut [Call], symbol_table: &SymbolTable) {
    for call in calls {
        if let Some(_symbol_id) = symbol_table.resolve_call(call) {
            call.is_resolved = true;
        } else {
            call.is_resolved = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use doctown_common::types::{ByteRange, CallKind, SymbolKind, Visibility};

    fn create_test_symbol(name: &str) -> Symbol {
        Symbol {
            kind: SymbolKind::Function,
            name: name.to_string(),
            range: ByteRange::new(0, 10),
            name_range: ByteRange::new(0, 5),
            signature: None,
            visibility: Visibility::Public,
            is_async: false,
        }
    }

    #[test]
    fn test_symbol_table_add_and_lookup() {
        let mut table = SymbolTable::new();
        let symbol_id = SymbolId::new("sym_test::foo").unwrap();

        table.add_symbol("foo".to_string(), symbol_id.clone(), "test.rs".to_string());

        assert_eq!(table.lookup("foo"), Some(symbol_id));
        assert_eq!(table.lookup("bar"), None);
    }

    #[test]
    fn test_add_symbols_batch() {
        let mut table = SymbolTable::new();
        let symbols = vec![
            create_test_symbol("foo"),
            create_test_symbol("bar"),
            create_test_symbol("baz"),
        ];

        table.add_symbols(&symbols, "test.rs");

        assert_eq!(table.len(), 3);
        assert!(table.lookup("foo").is_some());
        assert!(table.lookup("bar").is_some());
        assert!(table.lookup("baz").is_some());
    }

    #[test]
    fn test_resolve_local_call() {
        let mut table = SymbolTable::new();
        let symbols = vec![create_test_symbol("calculate")];
        table.add_symbols(&symbols, "test.rs");

        let mut call = Call {
            name: "calculate".to_string(),
            range: ByteRange::new(20, 30),
            kind: CallKind::Function,
            is_resolved: false,
        };

        resolve_calls(std::slice::from_mut(&mut call), &table);

        assert!(call.is_resolved);
    }

    #[test]
    fn test_external_call_unresolved() {
        let table = SymbolTable::new();

        let mut call = Call {
            name: "println".to_string(),
            range: ByteRange::new(20, 30),
            kind: CallKind::Function,
            is_resolved: false,
        };

        resolve_calls(std::slice::from_mut(&mut call), &table);

        assert!(!call.is_resolved);
    }

    #[test]
    fn test_method_call_resolution() {
        let mut table = SymbolTable::new();
        let symbols = vec![create_test_symbol("process")];
        table.add_symbols(&symbols, "test.rs");

        let mut call = Call {
            name: "obj.process".to_string(),
            range: ByteRange::new(20, 30),
            kind: CallKind::Method,
            is_resolved: false,
        };

        resolve_calls(std::slice::from_mut(&mut call), &table);

        assert!(call.is_resolved);
    }

    #[test]
    fn test_associated_call_resolution() {
        let mut table = SymbolTable::new();
        let symbols = vec![create_test_symbol("new")];
        table.add_symbols(&symbols, "test.rs");

        let mut call = Call {
            name: "MyStruct::new".to_string(),
            range: ByteRange::new(20, 30),
            kind: CallKind::Associated,
            is_resolved: false,
        };

        resolve_calls(std::slice::from_mut(&mut call), &table);

        assert!(call.is_resolved);
    }

    #[test]
    fn test_imported_call_unresolved() {
        let mut table = SymbolTable::new();

        let import = Import {
            module_path: "std::collections".to_string(),
            imported_items: Some(vec!["HashMap".to_string()]),
            alias: None,
            range: ByteRange::new(0, 10),
            is_wildcard: false,
        };
        table.add_imports(vec![import]);

        let mut call = Call {
            name: "HashMap".to_string(),
            range: ByteRange::new(20, 30),
            kind: CallKind::Constructor,
            is_resolved: false,
        };

        resolve_calls(std::slice::from_mut(&mut call), &table);

        // Imported items are considered external
        assert!(!call.is_resolved);
    }
}
