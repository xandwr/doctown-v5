<script lang="ts">
	import { onMount } from 'svelte';

	interface Props {
		events: any[];
		isLoading: boolean;
	}

	let { events, isLoading }: Props = $props();
	let logContainer: HTMLDivElement;
	let autoScroll = $state(true);

	// Auto-scroll to bottom when new events arrive
	$effect(() => {
		if (autoScroll && logContainer && events.length > 0) {
			logContainer.scrollTop = logContainer.scrollHeight;
		}
	});

	function handleScroll() {
		// Disable auto-scroll if user scrolls up
		if (logContainer) {
			const isAtBottom =
				logContainer.scrollHeight - logContainer.scrollTop <= logContainer.clientHeight + 50;
			autoScroll = isAtBottom;
		}
	}

	function formatEventType(eventType: string): string {
		return eventType.replace('ingest.', '').replace('.v1', '');
	}

	function getEventColor(eventType: string): string {
		if (eventType.includes('started')) return 'text-blue-600';
		if (eventType.includes('completed')) return 'text-green-600';
		if (eventType.includes('failed')) return 'text-red-600';
		if (eventType.includes('skipped')) return 'text-yellow-600';
		if (eventType.includes('detected')) return 'text-purple-600';
		if (eventType.includes('chunk_created')) return 'text-gray-500';
		return 'text-gray-700';
	}

	function formatTimestamp(timestamp: string): string {
		const date = new Date(timestamp);
		const time = date.toLocaleTimeString('en-US', { hour12: false });
		const ms = date.getMilliseconds().toString().padStart(3, '0');
		return `${time}.${ms}`;
	}

	function getEventSummary(event: any): string {
		const payload = event.payload || {};
		const eventType = event.event_type || '';

		if (eventType.includes('started')) {
			return `Started ingesting ${payload.repo_url || 'repository'}`;
		}
		if (eventType.includes('completed')) {
			const filesDetected = payload.files_processed || payload.files_detected || 0;
			return `Completed (${filesDetected} files, ${payload.chunks_created || 0} chunks, ${payload.files_skipped || 0} skipped)`;
		}
		if (eventType.includes('file_detected')) {
			return `Detected: ${payload.file_path || 'unknown'} (${payload.language || 'unknown'})`;
		}
		if (eventType.includes('file_skipped')) {
			return `Skipped: ${payload.file_path || 'unknown'} (${payload.reason || 'unknown reason'})`;
		}
		if (eventType.includes('chunk_created')) {
			return `Chunk: ${payload.symbol_name || 'unnamed'} in ${payload.file_path || 'unknown'}`;
		}
		return JSON.stringify(payload);
	}
</script>

<div class="bg-white rounded-lg shadow-md overflow-hidden">
	<div class="bg-gray-800 text-white px-4 py-2 flex items-center justify-between">
		<div class="flex items-center gap-3">
			<span class="font-mono text-sm font-semibold">Event Log</span>
			<span class="text-xs text-gray-400">({events.length} events)</span>
			{#if isLoading}
				<span class="flex items-center gap-1 text-xs text-green-400">
					<span class="inline-block w-2 h-2 bg-green-400 rounded-full animate-pulse"></span>
					Live
				</span>
			{/if}
		</div>
		<button
			onclick={() => (autoScroll = !autoScroll)}
			class="text-xs px-2 py-1 rounded {autoScroll
				? 'bg-green-600 hover:bg-green-700'
				: 'bg-gray-600 hover:bg-gray-700'}"
		>
			{autoScroll ? '📌 Auto-scroll ON' : '📌 Auto-scroll OFF'}
		</button>
	</div>

	<div
		bind:this={logContainer}
		onscroll={handleScroll}
		class="bg-gray-900 text-gray-100 font-mono text-xs p-4 h-[500px] overflow-y-auto scrollbar-thin scrollbar-thumb-gray-700 scrollbar-track-gray-800"
	>
		{#if events.length === 0}
			<div class="text-gray-500 text-center py-8">No events yet. Submit a repository to start.</div>
		{:else}
			{#each events as event, i}
				<div class="py-1 hover:bg-gray-800 px-2 -mx-2 rounded">
					<span class="text-gray-500">[{formatTimestamp(event.timestamp)}]</span>
					<span class={getEventColor(event.event_type) + ' font-semibold'}>
						{formatEventType(event.event_type)}
					</span>
					<span class="text-gray-300">→</span>
					<span class="text-gray-200">{getEventSummary(event)}</span>
				</div>
			{/each}
		{/if}
	</div>

	<div class="bg-gray-800 px-4 py-2 text-xs text-gray-400 flex items-center justify-between">
		<span>
			{#if autoScroll}
				Scrolling automatically with new events
			{:else}
				Auto-scroll disabled (scroll to bottom to re-enable)
			{/if}
		</span>
		<button
			onclick={() => {
				if (logContainer) {
					logContainer.scrollTop = logContainer.scrollHeight;
					autoScroll = true;
				}
			}}
			class="text-blue-400 hover:text-blue-300"
		>
			↓ Jump to bottom
		</button>
	</div>
</div>
