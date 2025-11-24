/**
 * API clients for embedding and assembly workers
 */

export interface Chunk {
	chunk_id: string;
	content: string;
}

export interface ChunkVector {
	chunk_id: string;
	vector: number[];
}

export interface EmbedRequest {
	batch_id: string;
	chunks: Chunk[];
}

export interface EmbedResponse {
	batch_id: string;
	vectors: ChunkVector[];
}

export interface SymbolMetadata {
	symbol_id: string;
	name: string;
	kind: string;
	file_path: string;
	signature: string;
	chunk_ids: string[];
	calls?: string[];
	imports?: string[];
	language?: string;
}

export interface SymbolContext {
	symbol_id: string;
	name: string;
	kind: string;
	language: string;
	file_path: string;
	signature: string;
	calls: string[];
	called_by: string[];
	imports: string[];
	related_symbols: string[];
	cluster_label: string | null;
	centrality: number;
}

export interface ChunkWithEmbedding {
	chunk_id: string;
	vector: number[];
	content: string;
}

export interface AssembleRequest {
	job_id: string;
	repo_url: string;
	git_ref: string;
	chunks: ChunkWithEmbedding[];
	symbols: SymbolMetadata[];
}

export interface ClusterInfo {
	cluster_id: string;
	label: string;
	members: string[];
}

export interface NodeInfo {
	id: string;
	metadata: Record<string, string>;
	cluster_id: string;
	centrality: number;
}

export interface EdgeInfo {
	source: string;
	target: string;
	kind: string;
	weight?: number;
}

export interface AssemblyStats {
	cluster_count: number;
	node_count: number;
	edge_count: number;
	duration_ms: number;
}

export interface AssembleResponse {
	job_id: string;
	clusters: ClusterInfo[];
	nodes: NodeInfo[];
	edges: EdgeInfo[];
	symbol_contexts: SymbolContext[];
	stats: AssemblyStats;
	events: any[];
}

/**
 * Client for the embedding worker API
 */
export class EmbeddingClient {
	private baseUrl: string;

	constructor(baseUrl: string) {
		this.baseUrl = baseUrl;
	}

	async health(): Promise<{ status: string; model_loaded: boolean; embedding_dim: number }> {
		const response = await fetch(`${this.baseUrl}/health`);
		if (!response.ok) {
			throw new Error(`Embedding worker health check failed: ${response.statusText}`);
		}
		return response.json();
	}

	async embed(request: EmbedRequest): Promise<EmbedResponse> {
		const response = await fetch(`${this.baseUrl}/embed`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(request)
		});

		if (!response.ok) {
			const error = await response.text();
			throw new Error(`Embedding request failed: ${error}`);
		}

		return response.json();
	}
}

/**
 * Client for the assembly worker API
 */
export class AssemblyClient {
	private baseUrl: string;

	constructor(baseUrl: string) {
		this.baseUrl = baseUrl;
	}

	async health(): Promise<{ status: string; version: string; service: string }> {
		const response = await fetch(`${this.baseUrl}/health`);
		if (!response.ok) {
			throw new Error(`Assembly worker health check failed: ${response.statusText}`);
		}
		return response.json();
	}

	async assemble(request: AssembleRequest): Promise<AssembleResponse> {
		const response = await fetch(`${this.baseUrl}/assemble`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(request)
		});

		if (!response.ok) {
			const error = await response.text();
			throw new Error(`Assembly request failed: ${error}`);
		}

		return response.json();
	}
}

export interface SymbolInput {
	symbol_id: string;
	context: SymbolContext;
}

export interface GenerateRequest {
	job_id: string;
	symbols: SymbolInput[];
}

export interface DocumentedSymbol {
	symbol_id: string;
	summary: string;
	tokens_used: number;
}

export interface GenerateResponse {
	documented_symbols: DocumentedSymbol[];
	total_tokens: number;
	total_cost: number;
}

/**
 * Client for the generation worker API
 */
export class GenerationClient {
	private baseUrl: string;

	constructor(baseUrl: string) {
		this.baseUrl = baseUrl;
	}

	async health(): Promise<{ status: string; model: string; ready: boolean }> {
		const response = await fetch(`${this.baseUrl}/health`);
		if (!response.ok) {
			throw new Error(`Generation worker health check failed: ${response.statusText}`);
		}
		return response.json();
	}

	async generate(request: GenerateRequest): Promise<GenerateResponse> {
		const response = await fetch(`${this.baseUrl}/generate`, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(request)
		});

		if (!response.ok) {
			const error = await response.text();
			throw new Error(`Generation request failed: ${error}`);
		}

		return response.json();
	}
}
