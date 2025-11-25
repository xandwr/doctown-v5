//! Demo of graph construction from symbols, calls, and imports.

use doctown_assembly::{GraphBuilder, SymbolData};
use doctown_common::types::{ByteRange, Call, CallKind, Import};

fn main() {
    println!("=== Graph Construction Demo ===\n");

    let mut builder = GraphBuilder::new();

    // Step 1: Build nodes from symbols
    println!("Step 1: Building nodes from symbols...");
    let symbols = vec![
        SymbolData {
            symbol_id: "fn_main".to_string(),
            name: "main".to_string(),
            kind: "function".to_string(),
            file_path: "src/main.rs".to_string(),
            signature: Some("fn main()".to_string()),
        },
        SymbolData {
            symbol_id: "fn_process_data".to_string(),
            name: "process_data".to_string(),
            kind: "function".to_string(),
            file_path: "src/lib.rs".to_string(),
            signature: Some("fn process_data(input: &str) -> Result<()>".to_string()),
        },
        SymbolData {
            symbol_id: "struct_Config".to_string(),
            name: "Config".to_string(),
            kind: "struct".to_string(),
            file_path: "src/config.rs".to_string(),
            signature: None,
        },
    ];

    builder.build_nodes(&symbols);
    println!("  Created {} nodes", builder.graph().nodes.len());

    // Step 2: Build call edges
    println!("\nStep 2: Building call edges...");
    let calls = vec![(
        "fn_main".to_string(),
        Call {
            name: "fn_process_data".to_string(),
            range: ByteRange::new(100, 120),
            kind: CallKind::Function,
            is_resolved: true,
        },
    )];

    builder.build_calls_edges(&calls);
    let calls_count = builder
        .graph()
        .edges
        .iter()
        .filter(|e| e.kind == doctown_assembly::EdgeKind::Calls)
        .count();
    println!("  Created {} call edges", calls_count);

    // Step 3: Build import edges
    println!("\nStep 3: Building import edges...");
    let imports = vec![(
        "fn_main".to_string(),
        Import {
            module_path: "crate::config".to_string(),
            imported_items: Some(vec!["struct_Config".to_string()]),
            alias: None,
            range: ByteRange::new(0, 25),
            is_wildcard: false,
        },
    )];

    builder.build_imports_edges(&imports);
    let imports_count = builder
        .graph()
        .edges
        .iter()
        .filter(|e| e.kind == doctown_assembly::EdgeKind::Imports)
        .count();
    println!("  Created {} import edges", imports_count);

    // Step 4: Get final graph
    let graph = builder.build();

    println!("\n=== Final Graph Statistics ===");
    println!("Total nodes: {}", graph.nodes.len());
    println!("Total edges: {}", graph.edges.len());
    println!("Graph density: {:.4}", graph.density());

    println!("\n=== Node Details ===");
    for node in &graph.nodes {
        println!("\nNode: {}", node.id);
        println!("  Name: {}", node.metadata.get("name").unwrap());
        println!("  Kind: {}", node.metadata.get("kind").unwrap());
        println!("  File: {}", node.metadata.get("file_path").unwrap());
        println!("  In-degree: {}", graph.in_degree(&node.id));
        println!("  Out-degree: {}", graph.out_degree(&node.id));
    }

    println!("\n=== Edge Details ===");
    for edge in &graph.edges {
        println!("\n{} --[{:?}]--> {}", edge.source, edge.kind, edge.target);
    }
}
