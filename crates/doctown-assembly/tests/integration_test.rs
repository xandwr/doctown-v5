//! Integration tests for the assembly worker.

use doctown_assembly::api::{AssembleRequest, ChunkWithEmbedding, SymbolMetadata};
use doctown_events::EventType;

/// Test the full assembly pipeline with sample data.
#[test]
fn test_assembly_pipeline() {
    // Create sample data
    let chunks = vec![
        ChunkWithEmbedding {
            chunk_id: "chunk_1".to_string(),
            vector: vec![0.1; 384], // 384-dim vector
            content: "function calculate_total() { return sum; }".to_string(),
        },
        ChunkWithEmbedding {
            chunk_id: "chunk_2".to_string(),
            vector: vec![0.2; 384],
            content: "function calculate_average() { return mean; }".to_string(),
        },
        ChunkWithEmbedding {
            chunk_id: "chunk_3".to_string(),
            vector: vec![0.9; 384],
            content: "class DataLoader { load() { } }".to_string(),
        },
    ];

    let symbols = vec![
        SymbolMetadata {
            symbol_id: "sym_1".to_string(),
            name: "calculate_total".to_string(),
            kind: "function".to_string(),
            language: "rust".to_string(),
            file_path: "src/math.rs".to_string(),
            signature: "fn calculate_total() -> i32".to_string(),
            chunk_ids: vec!["chunk_1".to_string()],
            calls: vec!["sum".to_string()],
            imports: vec!["std::collections".to_string()],
        },
        SymbolMetadata {
            symbol_id: "sym_2".to_string(),
            name: "calculate_average".to_string(),
            kind: "function".to_string(),
            language: "rust".to_string(),
            file_path: "src/math.rs".to_string(),
            signature: "fn calculate_average() -> f64".to_string(),
            chunk_ids: vec!["chunk_2".to_string()],
            calls: vec!["mean".to_string()],
            imports: vec![],
        },
        SymbolMetadata {
            symbol_id: "sym_3".to_string(),
            name: "DataLoader".to_string(),
            kind: "class".to_string(),
            language: "rust".to_string(),
            file_path: "src/loader.rs".to_string(),
            signature: "class DataLoader".to_string(),
            chunk_ids: vec!["chunk_3".to_string()],
            calls: vec![],
            imports: vec![],
        },
    ];

    let request = AssembleRequest {
        job_id: "job_test_123".to_string(),
        repo_url: "https://github.com/test/repo".to_string(),
        git_ref: "main".to_string(),
        chunks,
        symbols,
    };

    // For now, just verify the request can be serialized/deserialized
    let json = serde_json::to_string(&request).unwrap();
    let _deserialized: AssembleRequest = serde_json::from_str(&json).unwrap();

    // TODO: Once the server is running, we can make actual HTTP requests
    // For now, this test validates the types and structure
}

/// Test event payload serialization.
#[test]
fn test_event_serialization() {
    use doctown_events::{
        AssemblyClusterCreatedPayload, AssemblyCompletedPayload, AssemblyGraphCompletedPayload,
        AssemblyStartedPayload, EdgeTypeBreakdown,
    };

    // Test started event
    let started = AssemblyStartedPayload {
        chunk_count: 100,
        symbol_count: 50,
    };
    let json = serde_json::to_string(&started).unwrap();
    assert!(json.contains("chunk_count"));
    assert!(json.contains("symbol_count"));

    // Test cluster created event
    let cluster = AssemblyClusterCreatedPayload {
        cluster_id: "cluster_0".to_string(),
        label: "Math utilities".to_string(),
        member_count: 10,
    };
    let json = serde_json::to_string(&cluster).unwrap();
    assert!(json.contains("cluster_id"));
    assert!(json.contains("label"));

    // Test graph completed event
    let graph = AssemblyGraphCompletedPayload {
        node_count: 50,
        edge_count: 120,
        edge_types: EdgeTypeBreakdown {
            calls: 80,
            imports: 30,
            related: 10,
        },
    };
    let json = serde_json::to_string(&graph).unwrap();
    assert!(json.contains("node_count"));
    assert!(json.contains("edge_types"));

    // Test completed event
    let completed = AssemblyCompletedPayload {
        cluster_count: 5,
        node_count: 50,
        edge_count: 120,
        duration_ms: 1500,
    };
    let json = serde_json::to_string(&completed).unwrap();
    assert!(json.contains("duration_ms"));
}

/// Test that EventType enum includes assembly events.
#[test]
fn test_assembly_event_types() {
    assert_eq!(EventType::AssemblyStarted.as_str(), "assembly.started.v1");
    assert_eq!(
        EventType::AssemblyClusterCreated.as_str(),
        "assembly.cluster_created.v1"
    );
    assert_eq!(
        EventType::AssemblyGraphCompleted.as_str(),
        "assembly.graph_completed.v1"
    );
    assert_eq!(
        EventType::AssemblyCompleted.as_str(),
        "assembly.completed.v1"
    );

    // Test terminal event detection
    assert!(EventType::AssemblyCompleted.is_terminal());
    assert!(!EventType::AssemblyStarted.is_terminal());
}

/// Test that assembly events can be parsed from strings.
#[test]
fn test_assembly_event_parsing() {
    assert_eq!(
        EventType::try_from_str("assembly.started.v1"),
        Some(EventType::AssemblyStarted)
    );
    assert_eq!(
        EventType::try_from_str("assembly.cluster_created.v1"),
        Some(EventType::AssemblyClusterCreated)
    );
    assert_eq!(
        EventType::try_from_str("assembly.graph_completed.v1"),
        Some(EventType::AssemblyGraphCompleted)
    );
    assert_eq!(
        EventType::try_from_str("assembly.completed.v1"),
        Some(EventType::AssemblyCompleted)
    );
}

/// Test that symbol contexts are included in assembly response.
#[test]
fn test_symbol_contexts_in_response() {
    use doctown_assembly::api::AssembleResponse;
    use doctown_assembly::SymbolContext;

    // Create a mock response with symbol contexts
    let contexts = vec![
        SymbolContext::new(
            "sym_1".to_string(),
            "foo".to_string(),
            "function".to_string(),
            "rust".to_string(),
            "src/lib.rs".to_string(),
            "fn foo() -> i32".to_string(),
        )
        .with_calls(vec!["bar".to_string()])
        .with_centrality(0.5),
        SymbolContext::new(
            "sym_2".to_string(),
            "bar".to_string(),
            "function".to_string(),
            "rust".to_string(),
            "src/lib.rs".to_string(),
            "fn bar() -> i32".to_string(),
        )
        .with_called_by(vec!["foo".to_string()])
        .with_centrality(0.3),
    ];

    let response = AssembleResponse {
        job_id: "test_job".to_string(),
        clusters: vec![],
        nodes: vec![],
        edges: vec![],
        symbol_contexts: contexts.clone(),
        stats: doctown_assembly::api::AssemblyStats {
            cluster_count: 1,
            node_count: 2,
            edge_count: 1,
            duration_ms: 100,
        },
        events: vec![],
    };

    // Verify symbol_contexts are present
    assert_eq!(response.symbol_contexts.len(), 2);
    assert_eq!(response.symbol_contexts[0].symbol_id, "sym_1");
    assert_eq!(response.symbol_contexts[0].name, "foo");
    assert_eq!(response.symbol_contexts[0].calls, vec!["bar"]);
    assert_eq!(response.symbol_contexts[1].symbol_id, "sym_2");
    assert_eq!(response.symbol_contexts[1].called_by, vec!["foo"]);

    // Verify serialization includes symbol_contexts
    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("symbol_contexts"));
    assert!(json.contains("\"symbol_id\":\"sym_1\""));
    assert!(json.contains("\"calls\":[\"bar\"]"));
    assert!(json.contains("\"centrality\":0.5"));
}
