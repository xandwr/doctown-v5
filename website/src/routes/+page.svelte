<script lang="ts">
	import RepoInput from '$lib/components/RepoInput.svelte';
	import EventLog from '$lib/components/EventLog.svelte';
	import StatsSummary from '$lib/components/StatsSummary.svelte';
	import FileTree from '$lib/components/FileTree.svelte';
	import SymbolList from '$lib/components/SymbolList.svelte';
	import { SSEClient } from '$lib/sse-client';

	let isLoading = $state(false);
	let events = $state<any[]>([]);
	let errorMessage = $state<string | null>(null);
	let sseClient: SSEClient | null = null;
	let activeView = $state<'tree' | 'list'>('tree');

	function handleSubmit(repoUrl: string) {
		console.log('Submitting repo:', repoUrl);
		isLoading = true;
		errorMessage = null;
		events = [];

		// For now, create a demo SSE connection (will be replaced with actual API)
		// In M1.10, this will connect to the ingest worker
		const apiUrl = import.meta.env.VITE_INGEST_API_URL || 'http://localhost:3000';
		const jobId = `job_${Date.now()}`;

		sseClient = new SSEClient(
			`${apiUrl}/ingest?repo_url=${encodeURIComponent(repoUrl)}&job_id=${jobId}`,
			{
				onMessage: (event) => {
					// Only log completed events to console, not every chunk
					const eventType = (event as any).event_type;
					if (eventType?.includes('completed') || eventType?.includes('started')) {
						console.log('SSE event received:', event);
					}
					events = [...events, event];
					
					// Auto-stop loading and disconnect on completed event
					if (eventType?.includes('completed')) {
						isLoading = false;
						if (sseClient) {
							sseClient.close();
							sseClient = null;
						}
					}
				},
				onError: (error) => {
					console.error('SSE error:', error);
					errorMessage = error.message;
				},
				onOpen: () => {
					console.log('SSE connection opened');
				},
				onClose: () => {
					console.log('SSE connection closed');
					isLoading = false;
				}
			}
		);

		// Connect to SSE endpoint
		sseClient.connect();
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
				<p class="text-gray-600 mb-4">Streaming events from ingest worker...</p>
			</div>
		{/if}

		{#if errorMessage}
			<div class="bg-red-50 border border-red-200 rounded-lg p-4 mb-8">
				<p class="text-red-800 font-medium">Error: {errorMessage}</p>
			</div>
		{/if}

		{#if events.length > 0}
			<StatsSummary {events} {isLoading} />
			
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
