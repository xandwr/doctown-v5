//! HTTP API for the Assembly Worker.

use actix_cors::Cors;
use actix_web::{
    get, middleware, post,
    web::{Data, Json, JsonConfig, PayloadConfig},
    App, HttpResponse, HttpServer, Responder,
};
use doctown_events::{
    AssemblyClusterCreatedPayload, AssemblyCompletedPayload, AssemblyGraphCompletedPayload,
    AssemblyStartedPayload, Context, EdgeTypeBreakdown, Envelope, EventType, Status,
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crate::{cluster::Clusterer, context::ContextGenerator, graph::GraphBuilder, label::ClusterLabeler, EdgeKind, SymbolContext};
use crate::packer::{Packer, PackRequest};

/// Request schema for the /assemble endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssembleRequest {
    /// Job ID for tracing.
    pub job_id: String,
    /// Repository URL for context.
    pub repo_url: String,
    /// Git reference (branch/tag/commit).
    pub git_ref: String,
    /// Chunks with their embeddings.
    pub chunks: Vec<ChunkWithEmbedding>,
    /// Symbol metadata for graph construction.
    pub symbols: Vec<SymbolMetadata>,
}

/// A chunk with its embedding vector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkWithEmbedding {
    /// Chunk ID.
    pub chunk_id: String,
    /// Embedding vector (384 dimensions for all-MiniLM-L6-v2).
    pub vector: Vec<f32>,
    /// Text content of the chunk.
    pub content: String,
}

/// Symbol metadata for graph construction.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SymbolMetadata {
    /// Symbol ID.
    pub symbol_id: String,
    /// Symbol name.
    pub name: String,
    /// Symbol kind (e.g., "function", "class").
    pub kind: String,
    /// Programming language (e.g., "rust", "python").
    #[serde(default)]
    pub language: String,
    /// File path.
    pub file_path: String,
    /// Symbol signature.
    pub signature: String,
    /// Chunk IDs associated with this symbol.
    pub chunk_ids: Vec<String>,
    /// Calls made by this symbol.
    #[serde(default)]
    pub calls: Vec<String>,
    /// Imports used by this symbol.
    #[serde(default)]
    pub imports: Vec<String>,
}

/// Response schema for the /assemble endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct AssembleResponse {
    /// Job ID.
    pub job_id: String,
    /// Clusters created.
    pub clusters: Vec<ClusterInfo>,
    /// Graph nodes.
    pub nodes: Vec<NodeInfo>,
    /// Graph edges.
    pub edges: Vec<EdgeInfo>,
    /// Symbol contexts for LLM documentation.
    pub symbol_contexts: Vec<SymbolContext>,
    /// Statistics.
    pub stats: AssemblyStats,
    /// Events emitted during assembly.
    pub events: Vec<Envelope<serde_json::Value>>,
}

/// Information about a cluster.
#[derive(Debug, Clone, Serialize)]
pub struct ClusterInfo {
    /// Cluster ID.
    pub cluster_id: String,
    /// Human-readable label.
    pub label: String,
    /// Symbol IDs in this cluster.
    pub members: Vec<String>,
}

/// Information about a graph node.
#[derive(Debug, Clone, Serialize)]
pub struct NodeInfo {
    /// Node ID (symbol_id).
    pub id: String,
    /// Node metadata.
    pub metadata: HashMap<String, String>,
    /// Cluster assignment.
    pub cluster_id: String,
    /// Centrality score (0-1).
    pub centrality: f64,
}

/// Information about a graph edge.
#[derive(Debug, Clone, Serialize)]
pub struct EdgeInfo {
    /// Source node ID.
    pub source: String,
    /// Target node ID.
    pub target: String,
    /// Edge kind ("calls", "imports", "related").
    pub kind: String,
    /// Optional weight.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f32>,
}

/// Assembly statistics.
#[derive(Debug, Clone, Serialize)]
pub struct AssemblyStats {
    /// Number of clusters created.
    pub cluster_count: usize,
    /// Number of nodes in the graph.
    pub node_count: usize,
    /// Number of edges in the graph.
    pub edge_count: usize,
    /// Processing duration in milliseconds.
    pub duration_ms: u64,
}

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub version: String,
}

/// Health check endpoint.
#[get("/health")]
async fn health(state: Data<Arc<AppState>>) -> impl Responder {
    Json(serde_json::json!({
        "status": "ok",
        "version": state.version,
        "service": "assembly-worker"
    }))
}

/// Assembly endpoint.
#[post("/assemble")]
async fn assemble(req: Json<AssembleRequest>) -> impl Responder {
    let start = Instant::now();
    let mut events = Vec::new();

    // Create context for events
    use doctown_common::JobId;
    let job_id = JobId::new(&req.job_id).unwrap_or_else(|_| JobId::generate());
    let context = Context::new(job_id, req.repo_url.clone())
        .with_git_ref(req.git_ref.clone());

    info!(
        "Starting assembly for job {} with {} chunks, {} symbols",
        req.job_id,
        req.chunks.len(),
        req.symbols.len()
    );

    // Emit assembly.started event
    let started_payload = AssemblyStartedPayload {
        chunk_count: req.chunks.len(),
        symbol_count: req.symbols.len(),
    };
    events.push(Envelope::typed(
        EventType::AssemblyStarted,
        context.clone(),
        serde_json::to_value(&started_payload).unwrap(),
    ));

    // Step 1: Cluster the embeddings
    let vectors = ndarray::Array2::from_shape_vec(
        (req.chunks.len(), req.chunks[0].vector.len()),
        req.chunks.iter().flat_map(|c| c.vector.clone()).collect(),
    )
    .map_err(|e| {
        error!("Failed to create vector array: {}", e);
        HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Vector array creation failed: {}", e)
        }))
    })
    .unwrap();

    // Determine optimal cluster count (sqrt(n/2) heuristic)
    let k = ((req.chunks.len() as f64 / 2.0).sqrt().ceil() as usize).max(2).min(20);
    info!("Using k={} clusters for {} chunks", k, req.chunks.len());

    let clusterer = Clusterer::new(k);
    let cluster_result = match clusterer.cluster(&vectors) {
        Ok(result) => result,
        Err(e) => {
            error!("Clustering failed: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Clustering failed: {}", e)
            }));
        }
    };

    // Step 2: Label the clusters
    let mut clusters = Vec::new();

    for cluster_id in 0..k {
        let chunk_indices: Vec<usize> = cluster_result
            .assignments
            .iter()
            .enumerate()
            .filter(|(_, &c)| c == cluster_id)
            .map(|(i, _)| i)
            .collect();

        let chunk_contents: Vec<String> = chunk_indices
            .iter()
            .map(|&i| req.chunks[i].content.clone())
            .collect();

        let label = ClusterLabeler::label_cluster(&chunk_contents);

        // Find symbols in this cluster
        let cluster_chunk_ids: Vec<String> = chunk_indices
            .iter()
            .map(|&i| req.chunks[i].chunk_id.clone())
            .collect();

        let member_symbol_ids: Vec<String> = req
            .symbols
            .iter()
            .filter(|s| s.chunk_ids.iter().any(|cid| cluster_chunk_ids.contains(cid)))
            .map(|s| s.symbol_id.clone())
            .collect();

        clusters.push(ClusterInfo {
            cluster_id: format!("cluster_{}", cluster_id),
            label: label.clone(),
            members: member_symbol_ids.clone(),
        });

        // Emit cluster_created event
        let cluster_payload = AssemblyClusterCreatedPayload {
            cluster_id: format!("cluster_{}", cluster_id),
            label: label.clone(),
            member_count: member_symbol_ids.len(),
        };
        events.push(Envelope::typed(
            EventType::AssemblyClusterCreated,
            context.clone(),
            serde_json::to_value(&cluster_payload).unwrap(),
        ));
    }

    info!("Created {} clusters", clusters.len());

    // Step 3: Build the graph
    let mut builder = GraphBuilder::new();

    // Convert symbols to SymbolData
    use crate::graph::SymbolData;
    use doctown_common::types::{ByteRange, Call, CallKind, Import};

    let symbol_data: Vec<SymbolData> = req
        .symbols
        .iter()
        .map(|s| SymbolData {
            symbol_id: s.symbol_id.clone(),
            name: s.name.clone(),
            kind: s.kind.clone(),
            file_path: s.file_path.clone(),
            signature: Some(s.signature.clone()),
        })
        .collect();

    builder.build_nodes(&symbol_data);

    // Build call edges
    let call_data: Vec<(String, Call)> = req
        .symbols
        .iter()
        .flat_map(|s| {
            s.calls.iter().map(move |target| {
                (
                    s.symbol_id.clone(),
                    Call {
                        name: target.clone(),
                        range: ByteRange::new(0, 0), // Placeholder range
                        kind: CallKind::Function,
                        is_resolved: true,
                    },
                )
            })
        })
        .collect();

    builder.build_calls_edges(&call_data);

    // Build import edges
    let import_data: Vec<(String, Import)> = req
        .symbols
        .iter()
        .flat_map(|s| {
            s.imports.iter().map(move |import| {
                (
                    s.symbol_id.clone(),
                    Import {
                        module_path: import.clone(),
                        imported_items: None,
                        alias: None,
                        range: ByteRange::new(0, 0), // Placeholder range
                        is_wildcard: false,
                    },
                )
            })
        })
        .collect();

    builder.build_imports_edges(&import_data);

    // Build similarity edges
    let mut embeddings_map: HashMap<String, Vec<f32>> = HashMap::new();
    for chunk in &req.chunks {
        // Find symbols associated with this chunk
        for symbol in &req.symbols {
            if symbol.chunk_ids.contains(&chunk.chunk_id) {
                embeddings_map.insert(symbol.symbol_id.clone(), chunk.vector.clone());
            }
        }
    }

    let similarity_threshold = 0.7;
    let top_k = 5;
    builder.build_similarity_edges(&embeddings_map, similarity_threshold, top_k);

    let graph = builder.build();

    info!(
        "Built graph with {} nodes and {} edges",
        graph.nodes.len(),
        graph.edges.len()
    );

    // Count edge types
    let mut calls_count = 0;
    let mut imports_count = 0;
    let mut related_count = 0;
    for edge in &graph.edges {
        match edge.kind {
            EdgeKind::Calls => calls_count += 1,
            EdgeKind::Imports => imports_count += 1,
            EdgeKind::Related => related_count += 1,
        }
    }

    // Emit graph_completed event
    let graph_payload = AssemblyGraphCompletedPayload {
        node_count: graph.nodes.len(),
        edge_count: graph.edges.len(),
        edge_types: EdgeTypeBreakdown {
            calls: calls_count,
            imports: imports_count,
            related: related_count,
        },
    };
    events.push(Envelope::typed(
        EventType::AssemblyGraphCompleted,
        context.clone(),
        serde_json::to_value(&graph_payload).unwrap(),
    ));

    // Step 3.5: Generate symbol contexts for LLM documentation
    // Build cluster label map
    let mut cluster_labels: HashMap<String, String> = HashMap::new();
    for cluster in &clusters {
        for symbol_id in &cluster.members {
            cluster_labels.insert(symbol_id.clone(), cluster.label.clone());
        }
    }

    // Build language map
    let mut languages: HashMap<String, String> = HashMap::new();
    for symbol in &req.symbols {
        languages.insert(symbol.symbol_id.clone(), symbol.language.clone());
    }

    // Build imports map
    let mut imports_map: HashMap<String, Vec<String>> = HashMap::new();
    for symbol in &req.symbols {
        imports_map.insert(symbol.symbol_id.clone(), symbol.imports.clone());
    }

    // Generate contexts
    let context_generator = ContextGenerator::new()
        .with_cluster_labels(cluster_labels)
        .with_languages(languages)
        .with_imports(imports_map);
    
    let symbol_contexts = context_generator.generate(&graph);
    info!("Generated {} symbol contexts", symbol_contexts.len());

    // Step 4: Compute centrality
    let mut nodes = Vec::new();
    for node in &graph.nodes {
        let centrality = graph.degree_centrality(&node.id);

        // Find cluster for this node
        let cluster_id = clusters
            .iter()
            .find(|c| c.members.contains(&node.id))
            .map(|c| c.cluster_id.clone())
            .unwrap_or_else(|| "unclustered".to_string());

        nodes.push(NodeInfo {
            id: node.id.clone(),
            metadata: node.metadata.clone(),
            cluster_id,
            centrality,
        });
    }

    // Build edge info
    let edges: Vec<EdgeInfo> = graph
        .edges
        .iter()
        .map(|e| EdgeInfo {
            source: e.source.clone(),
            target: e.target.clone(),
            kind: match e.kind {
                EdgeKind::Calls => "calls".to_string(),
                EdgeKind::Imports => "imports".to_string(),
                EdgeKind::Related => "related".to_string(),
            },
            weight: e.weight,
        })
        .collect();

    let duration = start.elapsed();
    let duration_ms = duration.as_millis() as u64;

    // Emit assembly.completed event
    let completed_payload = AssemblyCompletedPayload {
        cluster_count: clusters.len(),
        node_count: nodes.len(),
        edge_count: edges.len(),
        duration_ms,
    };
    events.push(
        Envelope::typed(
            EventType::AssemblyCompleted,
            context.clone(),
            serde_json::to_value(&completed_payload).unwrap(),
        )
        .with_status(Status::Success),
    );

    info!("Assembly completed in {}ms", duration_ms);

    let response = AssembleResponse {
        job_id: req.job_id.clone(),
        clusters,
        nodes,
        edges,
        symbol_contexts,
        stats: AssemblyStats {
            cluster_count: k,
            node_count: graph.nodes.len(),
            edge_count: graph.edges.len(),
            duration_ms,
        },
        events,
    };

    HttpResponse::Ok().json(response)
}

/// Start the Assembly Worker HTTP server.
pub async fn start_server(host: &str, port: u16) -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let state = Arc::new(AppState {
        version: env!("CARGO_PKG_VERSION").to_string(),
    });

    info!("Starting Assembly Worker on {}:{}", host, port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(Data::new(state.clone()))
            .app_data(
                // Increase JSON payload limit to 100MB for large embedding batches
                PayloadConfig::new(100 * 1024 * 1024)
            )
            .app_data(
                // Also set JSON parser limit to 100MB (separate from raw payload limit)
                JsonConfig::default().limit(100 * 1024 * 1024)
            )
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .service(health)
            .service(assemble)
            .service(pack)
    })
    .bind((host, port))?
    .run()
    .await
}

/// Pack endpoint: Assembles a complete .docpack file from assembly results and source data
#[post("/pack")]
async fn pack(req: Json<PackRequest>) -> impl Responder {
    info!("Packing docpack for repo {}", req.repo_url);
    
    let packer = Packer::new();
    
    match packer.pack(req.into_inner()) {
        Ok(response) => {
            info!(
                "Successfully packed docpack: {} ({} bytes, {} files, {} symbols, {} clusters)",
                response.docpack_id,
                response.docpack_bytes.len(),
                response.statistics.file_count,
                response.statistics.symbol_count,
                response.statistics.cluster_count
            );
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            error!("Failed to pack docpack: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Packing failed: {}", e)
            }))
        }
    }
}
