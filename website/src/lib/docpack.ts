/**
 * Docpack packaging utilities
 * Creates .docpack format according to specs/docpack.md
 */

export interface DocpackManifest {
	schema_version: string;
	docpack_id: string;
	created_at: string;
	generator: {
		version: string;
		pipeline_version: string;
	};
	source: {
		repo_url: string;
		git_ref: string;
		commit_hash?: string;
	};
	statistics: {
		file_count: number;
		symbol_count: number;
		cluster_count: number;
		embedding_dimensions: number;
	};
	checksum: {
		algorithm: string;
		value: string;
	};
	optional: {
		has_embeddings: boolean;
		has_symbol_contexts: boolean;
	};
}

export interface DocpackGraph {
	nodes: string[];
	edges: Array<{ from: string; to: string; kind: string }>;
	metrics: {
		density: number;
		avg_degree: number;
	};
}

export interface DocpackNode {
	id: string;
	name: string;
	kind: string;
	language: string;
	file_path: string;
	byte_range: [number, number];
	signature?: string;
	calls: string[];
	called_by: string[];
	imports: string[];
	cluster_id: string;
	centrality: number;
	documentation: {
		summary: string;
		details?: string;
	};
}

export interface DocpackCluster {
	cluster_id: string;
	label: string;
	member_count: number;
}

export interface SourceMapChunk {
	chunk_id: string;
	byte_range: [number, number];
	symbol_ids: string[];
}

export interface SourceMapFile {
	file_path: string;
	language: string;
	chunks: SourceMapChunk[];
}

export interface SourceMap {
	files: SourceMapFile[];
}

export interface Docpack {
	manifest: DocpackManifest;
	graph: DocpackGraph;
	nodes: { symbols: DocpackNode[] };
	clusters: { clusters: DocpackCluster[] };
	source_map: SourceMap;
}

/**
 * Creates a .docpack from assembly results
 */
export function createDocpack(
	repoUrl: string,
	gitRef: string,
	assemblyResult: any,
	symbols: any[]
): Docpack {
	// Generate manifest
	const manifest: DocpackManifest = {
		schema_version: 'docpack/1.0',
		docpack_id: `sha256:${generateChecksum(assemblyResult)}`,
		created_at: new Date().toISOString(),
		generator: {
			version: 'doctown-packer/1.0.0',
			pipeline_version: 'v5.0'
		},
		source: {
			repo_url: repoUrl,
			git_ref: gitRef
		},
		statistics: {
			file_count: getFileCount(symbols),
			symbol_count: symbols.length,
			cluster_count: assemblyResult.clusters?.length || 0,
			embedding_dimensions: 384
		},
		checksum: {
			algorithm: 'sha256',
			value: generateChecksum(assemblyResult)
		},
		optional: {
			has_embeddings: true,
			has_symbol_contexts: true
		}
	};

	// Build graph
	const graph: DocpackGraph = {
		nodes: assemblyResult.nodes?.map((n: any) => n.id) || [],
		edges: assemblyResult.edges || [],
		metrics: {
			density: 0.0, // TODO: Calculate from graph
			avg_degree: 0.0 // TODO: Calculate from graph
		}
	};

	// Build nodes
	const nodes = {
		symbols: (assemblyResult.nodes || []).map((node: any) => ({
			id: node.id,
			name: node.name,
			kind: node.kind,
			language: node.language || 'unknown',
			file_path: node.file_path || '',
			byte_range: node.byte_range || [0, 0],
			signature: node.signature,
			calls: node.calls || [],
			called_by: node.called_by || [],
			imports: node.imports || [],
			cluster_id: node.cluster_id || 'cluster_default',
			centrality: node.centrality || 0.0,
			documentation: {
				summary: node.documentation?.summary || 'No documentation available',
				details: node.documentation?.details
			}
		}))
	};

	// Build clusters
	const clusters = {
		clusters: (assemblyResult.clusters || []).map((cluster: any) => ({
			cluster_id: cluster.cluster_id,
			label: cluster.label,
			member_count: cluster.member_count
		}))
	};

	// Build source map from symbols
	const fileMap = new Map<string, SourceMapFile>();
	for (const symbol of symbols) {
		if (!symbol.file_path) continue;

		if (!fileMap.has(symbol.file_path)) {
			fileMap.set(symbol.file_path, {
				file_path: symbol.file_path,
				language: 'unknown', // TODO: Get from events
				chunks: []
			});
		}

		const file = fileMap.get(symbol.file_path)!;
		for (const chunkId of symbol.chunk_ids || []) {
			file.chunks.push({
				chunk_id: chunkId,
				byte_range: [0, 0], // TODO: Get from events
				symbol_ids: [symbol.symbol_id]
			});
		}
	}

	const source_map: SourceMap = {
		files: Array.from(fileMap.values())
	};

	return {
		manifest,
		graph,
		nodes,
		clusters,
		source_map
	};
}

/**
 * Simple checksum generator (simplified for now)
 */
function generateChecksum(data: any): string {
	const str = JSON.stringify(data);
	let hash = 0;
	for (let i = 0; i < str.length; i++) {
		const char = str.charCodeAt(i);
		hash = (hash << 5) - hash + char;
		hash = hash & hash;
	}
	return Math.abs(hash).toString(16).padStart(16, '0');
}

/**
 * Get unique file count from symbols
 */
function getFileCount(symbols: any[]): number {
	const files = new Set(symbols.map((s) => s.file_path).filter(Boolean));
	return files.size;
}

/**
 * Parse GitHub repo URL to extract owner and name
 */
export function parseRepoUrl(repoUrl: string): { owner: string; name: string } | null {
	const patterns = [
		/github\.com\/([^\/]+)\/([^\/\.]+)/,
		/github\.com[:/]([^\/]+)\/([^\/\.]+)\.git/
	];

	for (const pattern of patterns) {
		const match = repoUrl.match(pattern);
		if (match) {
			return { owner: match[1], name: match[2] };
		}
	}

	return null;
}
