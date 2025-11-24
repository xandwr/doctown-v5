<script lang="ts">
	import RepoInput from '$lib/components/RepoInput.svelte';
	import EventLog from '$lib/components/EventLog.svelte';
	import StatsSummary from '$lib/components/StatsSummary.svelte';
	import FileTree from '$lib/components/FileTree.svelte';
	import SymbolList from '$lib/components/SymbolList.svelte';
	import { SSEClient } from '$lib/sse-client';
	import { EmbeddingClient, AssemblyClient, type Chunk, type SymbolMetadata, type ChunkWithEmbedding } from '$lib/api-client';
	import { createDocpack, parseRepoUrl } from '$lib/docpack';

	let isLoading = $state(false);
	let events = $state<any[]>([]);
	let errorMessage = $state<string | null>(null);
	let sseClient: SSEClient | null = null;
	let activeView = $state<'tree' | 'list'>('tree');
	let pipelineStage = $state<'ingest' | 'embedding' | 'assembly' | 'uploading' | 'complete'>('ingest');
	
	// Pipeline data storage
	let chunks = $state<Chunk[]>([]);
	let symbols = $state<SymbolMetadata[]>([]);
	let embeddings = $state<Map<string, number[]>>(new Map());
	let assemblyResult = $state<any>(null);
	let docpackUrl = $state<string | null>(null);
	
	// Stats for display without storing everything
	let stats = $state({
		filesProcessed: 0,
		filesSkipped: 0,
		chunksCreated: 0,
		chunksEmbedded: 0
	});

	async function handleSubmit(repoUrl: string) {
		console.log('Submitting repo:', repoUrl);
		isLoading = true;
		errorMessage = null;
		events = [];
		pipelineStage = 'ingest';
		chunks = [];
		symbols = [];
		embeddings = new Map();
		assemblyResult = null;
		docpackUrl = null;
		stats = { filesProcessed: 0, filesSkipped: 0, chunksCreated: 0, chunksEmbedded: 0 };

		const apiUrl = import.meta.env.VITE_INGEST_API_URL || 'http://localhost:3000';
		const embeddingUrl = import.meta.env.VITE_EMBEDDING_API_URL || 'http://localhost:8000';
		const assemblyUrl = import.meta.env.VITE_ASSEMBLY_API_URL || 'http://localhost:3001';
		const jobId = `job_${Date.now()}`;

		try {
			// Stage 1: Ingest - collect all chunks and symbols
			await runIngestStage(apiUrl, repoUrl, jobId);
			
			// Stage 2: Embedding - batch chunks and get embeddings
			pipelineStage = 'embedding';
			await runEmbeddingStage(embeddingUrl, jobId);
			
			// Stage 3: Assembly - create graph and clusters
			pipelineStage = 'assembly';
			await runAssemblyStage(assemblyUrl, repoUrl, jobId);
			
			// Stage 4: Upload .docpack to R2
			pipelineStage = 'uploading';
			await uploadDocpack(repoUrl, jobId);
			
			pipelineStage = 'complete';
			console.log('Pipeline complete! Docpack available at:', docpackUrl);
		} catch (error: any) {
			console.error('Pipeline error:', error);
			errorMessage = error.message;
		} finally {
			isLoading = false;
		}
	}

	async function runIngestStage(apiUrl: string, repoUrl: string, jobId: string): Promise<void> {
		return new Promise((resolve, reject) => {
			sseClient = new SSEClient(
				`${apiUrl}/ingest?repo_url=${encodeURIComponent(repoUrl)}&job_id=${jobId}`,
				{
				onMessage: (event: any) => {
					const eventType = event.event_type;
					
					// Only store important events to avoid memory issues
					if (eventType?.includes('completed') || eventType?.includes('started') || eventType?.includes('failed')) {
						console.log('SSE event received:', event);
						events = [...events, event];
						// Keep only last 50 events to prevent memory bloat
						if (events.length > 50) {
							events = events.slice(1);
						}
					}
					
					// Track chunks (store content for assembly stage)
					if (eventType === 'ingest.chunk_created.v1') {
						chunks = [...chunks, {
							chunk_id: event.payload.chunk_id,
							content: event.payload.content || '' // Store content for cluster labeling
						}];
						
						// Build symbol metadata if this is a symbol chunk
						if (event.payload.symbol_kind && event.payload.symbol_name) {
							// Create a unique symbol ID from chunk ID (since symbols don't have separate IDs in events)
							const symbolId = event.payload.chunk_id;
							const existingIndex = symbols.findIndex(s => s.symbol_id === symbolId);
							if (existingIndex === -1) {
								symbols = [...symbols, {
									symbol_id: symbolId,
									name: event.payload.symbol_name,
									kind: event.payload.symbol_kind,
									file_path: event.payload.file_path || '',
									signature: '',  // Not available in chunk_created events
									chunk_ids: [event.payload.chunk_id],
									calls: [],  // Not available in chunk_created events
									imports: [],  // Not available in chunk_created events
									language: event.payload.language || 'unknown'
								}];
							} else {
								// Update existing symbol's chunk_ids
								symbols = symbols.map((s, i) => 
									i === existingIndex 
										? { ...s, chunk_ids: [...s.chunk_ids, event.payload.chunk_id] }
										: s
								);
							}
						}
					}
						
						// Complete ingest stage on completion
						if (eventType === 'ingest.completed.v1') {
							console.log(`Ingest complete: ${chunks.length} chunks, ${symbols.length} symbols`);
							if (sseClient) {
								sseClient.close();
								sseClient = null;
							}
							// Small delay to ensure stream closes gracefully
							setTimeout(() => resolve(), 100);
						}
					},
					onError: (error) => {
						console.error('SSE error:', error);
						reject(error);
					},
					onOpen: () => {
						console.log('SSE connection opened');
					},
					onClose: () => {
						console.log('SSE connection closed');
					}
				}
			);

			sseClient.connect();
		});
	}

	async function runEmbeddingStage(embeddingUrl: string, jobId: string): Promise<void> {
		const embeddingClient = new EmbeddingClient(embeddingUrl);
		
		console.log(`Embedding stage: processing ${chunks.length} chunks`);
		
		// Add embedding started event
		events = [...events, {
			event_type: 'embedding.batch_started.v1',
			payload: { batch_id: jobId, chunk_count: chunks.length },
			timestamp: new Date().toISOString()
		}];
		
		// Batch chunks (max 256 per batch as configured in worker)
		const batchSize = 256;
		const batches: Chunk[][] = [];
		for (let i = 0; i < chunks.length; i += batchSize) {
			batches.push(chunks.slice(i, i + batchSize));
		}
		
		console.log(`Processing ${batches.length} batches of embeddings`);
		
		// Process batches sequentially to avoid overwhelming the worker
		for (let i = 0; i < batches.length; i++) {
			const batch = batches[i];
			const batchId = `${jobId}_batch_${i}`;
			
			const response = await embeddingClient.embed({
				batch_id: batchId,
				chunks: batch
			});
			
			// Store embeddings (use new Map to trigger reactivity)
			const newEmbeddings = new Map(embeddings);
			for (const vector of response.vectors) {
				newEmbeddings.set(vector.chunk_id, vector.vector);
			}
			embeddings = newEmbeddings;
			
			console.log(`Batch ${i + 1}/${batches.length} complete: ${response.vectors.length} vectors`);
		}
		
		// Add embedding completed event
		events = [...events, {
			event_type: 'embedding.batch_completed.v1',
			payload: { batch_id: jobId, chunk_count: chunks.length, duration_ms: 0 },
			timestamp: new Date().toISOString()
		}];
		
		console.log(`Embedding stage complete: ${embeddings.size} embeddings generated`);
	}

	async function runAssemblyStage(assemblyUrl: string, repoUrl: string, jobId: string): Promise<void> {
		const assemblyClient = new AssemblyClient(assemblyUrl);
		
		console.log(`Assembly stage: processing ${chunks.length} chunks with embeddings`);
		
		// Build chunks with embeddings
		const chunksWithEmbeddings: ChunkWithEmbedding[] = chunks
			.filter(chunk => embeddings.has(chunk.chunk_id))
			.map(chunk => ({
				chunk_id: chunk.chunk_id,
				vector: embeddings.get(chunk.chunk_id)!,
				content: chunk.content
			}));
		
		if (chunksWithEmbeddings.length === 0) {
			throw new Error('No chunks with embeddings available for assembly');
		}
		
		console.log(`Sending ${chunksWithEmbeddings.length} chunks and ${symbols.length} symbols to assembly`);
		
		// Call assembly API
		const response = await assemblyClient.assemble({
			job_id: jobId,
			repo_url: repoUrl,
			git_ref: 'main',
			chunks: chunksWithEmbeddings,
			symbols: symbols
		});
		
		// Store assembly result
		assemblyResult = response;
		
		// Add assembly events to event log
		events = [...events, ...response.events];
		
		console.log(`Assembly complete: ${response.clusters.length} clusters, ${response.nodes.length} nodes, ${response.edges.length} edges`);
	}

	async function uploadDocpack(repoUrl: string, jobId: string): Promise<void> {
		console.log('Packaging and uploading docpack to R2...');
		
		// Parse repo URL to get owner and name
		const repoInfo = parseRepoUrl(repoUrl);
		if (!repoInfo) {
			throw new Error('Invalid repository URL format');
		}
		
		// Create docpack from assembly results
		const docpack = createDocpack(
			repoUrl,
			'main',
			assemblyResult,
			symbols
		);
		
		console.log('Docpack created:', {
			files: docpack.source_map.files.length,
			symbols: docpack.nodes.symbols.length,
			clusters: docpack.clusters.clusters.length
		});
		
		// Upload to R2 via API endpoint
		const response = await fetch('/api/upload-docpack', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				repoOwner: repoInfo.owner,
				repoName: repoInfo.name,
				docpackData: docpack
			})
		});
		
		if (!response.ok) {
			const error = await response.text();
			throw new Error(`Failed to upload docpack: ${error}`);
		}
		
		const result = await response.json();
		docpackUrl = result.url;
		
		// Clear memory-intensive data now that it's uploaded
		embeddings.clear();
		chunks = [];
		
		console.log(`Docpack uploaded successfully: ${result.key} (${Math.round(result.size / 1024)}KB)`);
		
		// Add upload event
		events = [...events, {
			event_type: 'docpack.uploaded.v1',
			payload: { 
				key: result.key,
				size: result.size,
				url: result.url
			},
			timestamp: new Date().toISOString()
		}];
	}

	function handleDisconnect() {
		if (sseClient) {
			sseClient.close();
			sseClient = null;
		}
		isLoading = false;
	}
</script>

<div class="min-h-screen bg-gray-50 py-12 px-4">
	<div class="max-w-4xl mx-auto">
		<header class="text-center mb-12">
			<h1 class="text-4xl font-bold text-gray-900 mb-2">Doctown v5</h1>
			<p class="text-lg text-gray-600">
				Analyze and document your GitHub repositories with AI-powered insights
			</p>
		</header>

		<div class="bg-white rounded-lg shadow-md p-8 mb-8">
			<RepoInput onSubmit={handleSubmit} {isLoading} />
		</div>

		{#if isLoading}
			<div class="bg-white rounded-lg shadow-md p-8 mb-8">
				<div class="flex items-center justify-between mb-4">
					<h2 class="text-xl font-semibold text-gray-900">Processing Repository</h2>
					<button
						onclick={handleDisconnect}
						class="text-sm text-red-600 hover:text-red-700 font-medium"
					>
						Cancel
					</button>
				</div>
				
				<!-- Pipeline stage indicator -->
				<div class="mb-6">
					<div class="flex items-center justify-between mb-2">
						<span class="text-sm font-medium text-gray-700">Pipeline Progress</span>
						<span class="text-sm text-gray-500">
							{#if pipelineStage === 'ingest'}
								Stage 1/4: Ingesting files...
							{:else if pipelineStage === 'embedding'}
								Stage 2/4: Generating embeddings...
							{:else if pipelineStage === 'assembly'}
								Stage 3/4: Building graph and clusters...
							{:else if pipelineStage === 'uploading'}
								Stage 4/4: Uploading .docpack to R2...
							{:else}
								Complete!
							{/if}
						</span>
					</div>
					<div class="flex space-x-2">
						<div class="flex-1 h-2 bg-gray-200 rounded-full overflow-hidden">
							<div 
								class="h-full bg-blue-600 transition-all duration-500"
								style="width: {pipelineStage === 'ingest' ? '25%' : pipelineStage === 'embedding' ? '50%' : pipelineStage === 'assembly' ? '75%' : pipelineStage === 'uploading' ? '95%' : '100%'}"
							></div>
						</div>
					</div>
					<div class="flex justify-between mt-2 text-xs text-gray-500">
						<span class="{pipelineStage !== 'ingest' ? 'text-blue-600 font-medium' : ''}">
							{pipelineStage !== 'ingest' ? '✓' : '○'} Ingest
						</span>
						<span class="{pipelineStage === 'embedding' || pipelineStage === 'assembly' || pipelineStage === 'uploading' || pipelineStage === 'complete' ? 'text-blue-600 font-medium' : ''}">
							{pipelineStage === 'embedding' || pipelineStage === 'assembly' || pipelineStage === 'uploading' || pipelineStage === 'complete' ? '✓' : '○'} Embedding
						</span>
						<span class="{pipelineStage === 'assembly' || pipelineStage === 'uploading' || pipelineStage === 'complete' ? 'text-blue-600 font-medium' : ''}">
							{pipelineStage === 'assembly' || pipelineStage === 'uploading' || pipelineStage === 'complete' ? '✓' : '○'} Assembly
						</span>
						<span class="{pipelineStage === 'uploading' || pipelineStage === 'complete' ? 'text-blue-600 font-medium' : ''}">
							{pipelineStage === 'uploading' || pipelineStage === 'complete' ? '✓' : '○'} Upload
						</span>
						<span class="{pipelineStage === 'complete' ? 'text-green-600 font-medium' : ''}">
							{pipelineStage === 'complete' ? '✓' : '○'} Complete
						</span>
					</div>
				</div>
				
				<p class="text-gray-600 mb-4">
					{#if pipelineStage === 'ingest'}
						Extracting symbols and chunks from repository...
					{:else if pipelineStage === 'embedding'}
						Collected {chunks.length} chunks, generating embeddings...
					{:else if pipelineStage === 'assembly'}
						Creating semantic clusters and call graph...
					{:else if pipelineStage === 'uploading'}
						Packaging and uploading .docpack to R2 storage...
					{/if}
				</p>
			</div>
		{/if}

		{#if errorMessage}
			<div class="bg-red-50 border border-red-200 rounded-lg p-4 mb-8">
				<p class="text-red-800 font-medium">Error: {errorMessage}</p>
			</div>
		{/if}

		{#if events.length > 0}
			<StatsSummary {events} {isLoading} />
			
			<!-- Docpack Download -->
			{#if docpackUrl && pipelineStage === 'complete'}
				<div class="bg-green-50 border border-green-200 rounded-lg shadow-md p-6 mb-8">
					<div class="flex items-start gap-4">
						<div class="shrink-0">
							<svg class="w-12 h-12 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
								<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
							</svg>
						</div>
						<div class="flex-1">
							<h3 class="text-lg font-semibold text-green-900 mb-2">✨ Docpack Ready!</h3>
							<p class="text-green-700 mb-4">
								Your repository has been analyzed and packaged as a .docpack file.
								The file has been uploaded to R2 storage and is ready to use.
							</p>
							<div class="flex flex-col sm:flex-row gap-3">
								<a 
									href={docpackUrl} 
									download
									class="inline-flex items-center justify-center px-4 py-2 bg-green-600 text-white rounded-md hover:bg-green-700 transition-colors font-medium"
								>
									<svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
									</svg>
									Download .docpack
								</a>
								<button 
									onclick={() => docpackUrl && navigator.clipboard.writeText(docpackUrl)}
									class="inline-flex items-center justify-center px-4 py-2 bg-white text-green-700 border border-green-300 rounded-md hover:bg-green-50 transition-colors font-medium"
								>
									<svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
										<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
									</svg>
									Copy URL
								</button>
							</div>
							<div class="mt-3 text-sm text-green-600">
								<code class="bg-green-100 px-2 py-1 rounded">{docpackUrl}</code>
							</div>
						</div>
					</div>
				</div>
			{/if}
			
			<!-- Assembly Results -->
			{#if assemblyResult}
				<div class="bg-white rounded-lg shadow-md p-8 mb-8">
					<h2 class="text-xl font-semibold text-gray-900 mb-4">📊 Assembly Results</h2>
					
					<div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
						<div class="bg-blue-50 rounded-lg p-4">
							<div class="text-sm text-blue-600 font-medium mb-1">Clusters</div>
							<div class="text-2xl font-bold text-blue-900">{assemblyResult.clusters.length}</div>
						</div>
						<div class="bg-green-50 rounded-lg p-4">
							<div class="text-sm text-green-600 font-medium mb-1">Nodes</div>
							<div class="text-2xl font-bold text-green-900">{assemblyResult.nodes.length}</div>
						</div>
						<div class="bg-purple-50 rounded-lg p-4">
							<div class="text-sm text-purple-600 font-medium mb-1">Edges</div>
							<div class="text-2xl font-bold text-purple-900">{assemblyResult.edges.length}</div>
						</div>
					</div>
					
					<!-- Clusters -->
					<div class="mb-6">
						<h3 class="text-lg font-semibold text-gray-900 mb-3">🏷️ Semantic Clusters</h3>
						<div class="space-y-2">
							{#each assemblyResult.clusters as cluster}
								<div class="border border-gray-200 rounded-lg p-4 hover:border-blue-300 transition-colors">
									<div class="flex items-center justify-between mb-2">
										<span class="font-medium text-gray-900">{cluster.label}</span>
										<span class="text-sm text-gray-500">{cluster.members.length} symbols</span>
									</div>
									<div class="text-sm text-gray-600">
										Cluster ID: <code class="text-xs bg-gray-100 px-1 py-0.5 rounded">{cluster.cluster_id}</code>
									</div>
								</div>
							{/each}
						</div>
					</div>
					
					<!-- Top nodes by centrality -->
					<div>
						<h3 class="text-lg font-semibold text-gray-900 mb-3">⭐ Most Important Symbols</h3>
						<div class="space-y-2">
							{#each [...assemblyResult.nodes].sort((a: any, b: any) => b.centrality - a.centrality).slice(0, 10) as node}
								<div class="border border-gray-200 rounded-lg p-3 hover:border-blue-300 transition-colors">
									<div class="flex items-center justify-between">
										<div class="flex-1">
											<span class="font-medium text-gray-900">{node.metadata.name || node.id}</span>
											<span class="text-sm text-gray-500 ml-2">({node.metadata.kind || 'unknown'})</span>
										</div>
										<div class="flex items-center space-x-3">
											<span class="text-xs text-gray-500">
												Centrality: {node.centrality.toFixed(3)}
											</span>
											<span class="text-xs bg-blue-100 text-blue-700 px-2 py-1 rounded">
												{assemblyResult.clusters.find((c: any) => c.cluster_id === node.cluster_id)?.label || 'Unknown cluster'}
											</span>
										</div>
									</div>
								</div>
							{/each}
						</div>
					</div>
				</div>
			{/if}
			
			<!-- View switcher -->
			<div class="mb-8">
				<div class="bg-white rounded-lg shadow-md overflow-hidden">
					<!-- Tab buttons -->
					<div class="flex border-b border-gray-200 bg-gray-50">
						<button
							onclick={() => (activeView = 'tree')}
							class="flex-1 px-4 py-3 text-sm font-medium transition-colors {activeView === 'tree'
								? 'bg-white text-blue-600 border-b-2 border-blue-600'
								: 'text-gray-600 hover:text-gray-900 hover:bg-gray-100'}"
						>
							📁 File Tree
						</button>
						<button
							onclick={() => (activeView = 'list')}
							class="flex-1 px-4 py-3 text-sm font-medium transition-colors {activeView === 'list'
								? 'bg-white text-blue-600 border-b-2 border-blue-600'
								: 'text-gray-600 hover:text-gray-900 hover:bg-gray-100'}"
						>
							📋 Symbol List
						</button>
					</div>

					<!-- View content -->
					<div>
						{#if activeView === 'tree'}
							<FileTree {events} />
						{:else}
							<SymbolList {events} />
						{/if}
					</div>
				</div>
			</div>
			
			<EventLog {events} {isLoading} />
		{/if}
	</div>
</div>
