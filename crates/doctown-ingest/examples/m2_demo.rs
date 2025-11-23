//! Demonstration of M2.1 Call Graph Extraction capabilities
//!
//! This example shows how to:
//! 1. Extract symbols from source code
//! 2. Extract import statements
//! 3. Extract function/method calls
//! 4. Build a symbol table
//! 5. Resolve calls to determine if they're local or external

use doctown_common::Language;
use doctown_ingest::{
    extract_calls, extract_imports, extract_symbols, resolve_calls, Parser, SymbolTable,
};

fn main() {
    println!("=== M2.1 Call Graph Extraction Demo ===\n");

    // Example Rust code with imports and calls
    let rust_code = r#"
use std::collections::HashMap;
use std::fs::File;

pub fn process_data(input: &str) -> HashMap<String, i32> {
    let mut map = HashMap::new();
    let result = parse_input(input);
    map.insert("count".to_string(), result);
    map
}

fn parse_input(s: &str) -> i32 {
    s.len() as i32
}

fn read_file(path: &str) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}
"#;

    // Parse the code
    let parser = Parser::new();
    let tree = parser.parse(rust_code, Language::Rust).unwrap();

    // 1. Extract symbols
    let symbols = extract_symbols(&tree, rust_code, Language::Rust);
    println!("📦 Extracted {} symbols:", symbols.len());
    for symbol in &symbols {
        println!(
            "  - {} {:?} at {:?}",
            symbol.kind, symbol.name, symbol.range
        );
    }

    // 2. Extract imports
    let imports = extract_imports(&tree, rust_code, Language::Rust);
    println!("\n📥 Extracted {} imports:", imports.len());
    for import in &imports {
        println!(
            "  - {} (wildcard: {})",
            import.module_path, import.is_wildcard
        );
        if let Some(ref items) = import.imported_items {
            println!("    Items: {:?}", items);
        }
    }

    // 3. Extract calls
    let mut calls = extract_calls(&tree, rust_code, Language::Rust);
    println!("\n📞 Extracted {} calls:", calls.len());
    for call in &calls {
        println!("  - {} ({:?}) at {:?}", call.name, call.kind, call.range);
    }

    // 4. Build symbol table
    let mut symbol_table = SymbolTable::new();
    symbol_table.add_symbols(&symbols, "example.rs");
    symbol_table.add_imports(imports);
    println!("\n🗂️  Symbol table contains {} symbols", symbol_table.len());

    // 5. Resolve calls
    resolve_calls(&mut calls, &symbol_table);
    println!("\n✅ Call resolution results:");
    for call in &calls {
        let status = if call.is_resolved {
            "LOCAL"
        } else {
            "EXTERNAL"
        };
        println!("  - {} → {} ({:?})", call.name, status, call.kind);
    }

    // Demo with other languages
    demo_python();
    demo_typescript();
}

fn demo_python() {
    println!("\n\n=== Python Import & Call Detection ===\n");

    let python_code = r#"
import os
from typing import List, Dict

def process_files(paths: List[str]) -> Dict[str, int]:
    result = {}
    for path in paths:
        size = os.path.getsize(path)
        result[path] = size
    return result

def helper_function(x):
    return x * 2
"#;

    let parser = Parser::new();
    let tree = parser.parse(python_code, Language::Python).unwrap();

    let imports = extract_imports(&tree, python_code, Language::Python);
    println!("📥 Python imports:");
    for import in &imports {
        println!("  - {}", import.module_path);
        if let Some(ref items) = import.imported_items {
            println!("    Imported: {:?}", items);
        }
    }

    let calls = extract_calls(&tree, python_code, Language::Python);
    println!("\n📞 Python calls:");
    for call in &calls {
        println!("  - {} ({:?})", call.name, call.kind);
    }
}

fn demo_typescript() {
    println!("\n\n=== TypeScript Import & Call Detection ===\n");

    let ts_code = r#"
import { useState, useEffect } from 'react';
import * as utils from './utils';

export function Component() {
    const [count, setCount] = useState(0);
    
    useEffect(() => {
        utils.initializeApp();
        const value = calculateValue(count);
        setCount(value);
    }, [count]);
    
    return <div>{count}</div>;
}

function calculateValue(n: number): number {
    return n * 2;
}
"#;

    let parser = Parser::new();
    let tree = parser.parse(ts_code, Language::TypeScript).unwrap();

    let imports = extract_imports(&tree, ts_code, Language::TypeScript);
    println!("📥 TypeScript imports:");
    for import in &imports {
        println!("  - {}", import.module_path);
        if let Some(ref items) = import.imported_items {
            println!("    Imported: {:?}", items);
        }
        if import.is_wildcard {
            if let Some(ref alias) = import.alias {
                println!("    Namespace alias: {}", alias);
            }
        }
    }

    let calls = extract_calls(&tree, ts_code, Language::TypeScript);
    println!("\n📞 TypeScript calls:");
    for call in &calls {
        println!("  - {} ({:?})", call.name, call.kind);
    }
}
