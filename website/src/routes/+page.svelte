<script lang="ts">
	import RepoInput from '$lib/components/RepoInput.svelte';
	import EventLog from '$lib/components/EventLog.svelte';
	import { SSEClient } from '$lib/sse-client';

	let isLoading = $state(false);
	let events = $state<any[]>([]);
	let errorMessage = $state<string | null>(null);
	let sseClient: SSEClient | null = null;

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
			<EventLog {events} {isLoading} />
		{/if}
	</div>
</div>
