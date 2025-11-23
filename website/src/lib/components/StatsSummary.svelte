<script lang="ts">
	interface Props {
		events: any[];
		isLoading: boolean;
	}

	let { events, isLoading }: Props = $props();

	// Compute statistics from events
	const stats = $derived.by(() => {
		let filesDetected = 0;
		let filesSkipped = 0;
		let chunksCreated = 0;
		let chunksEmbedded = 0;
		let repoUrl = '';
		let status: 'idle' | 'processing' | 'completed' | 'failed' = 'idle';
		let duration = 0;
		const languageCounts: Record<string, number> = {};
		const skipReasons: Record<string, number> = {};

		for (const event of events) {
			const eventType = event.event_type || '';
			const payload = event.payload || {};

			if (eventType.includes('started')) {
				status = 'processing';
				repoUrl = payload.repo_url || '';
			} else if (eventType.includes('file_detected')) {
				filesDetected++;
				const lang = payload.language || 'unknown';
				languageCounts[lang] = (languageCounts[lang] || 0) + 1;
			} else if (eventType.includes('file_skipped')) {
				filesSkipped++;
				const reason = payload.reason || 'unknown';
				skipReasons[reason] = (skipReasons[reason] || 0) + 1;
			} else if (eventType.includes('chunk_created')) {
				chunksCreated++;
			} else if (eventType.includes('completed')) {
				status = payload.status === 'failed' ? 'failed' : 'completed';
				duration = payload.duration_ms || 0;
				// Use final counts from payload if available
				if (payload.files_processed !== undefined) filesDetected = payload.files_processed;
				if (payload.files_skipped !== undefined) filesSkipped = payload.files_skipped;
				if (payload.chunks_created !== undefined) chunksCreated = payload.chunks_created;
				if (payload.chunks_embedded !== undefined) chunksEmbedded = payload.chunks_embedded;
			}
		}

		const languageArray = Object.entries(languageCounts)
			.map(([lang, count]) => ({ lang, count }))
			.sort((a, b) => b.count - a.count);

		const skipReasonArray = Object.entries(skipReasons)
			.map(([reason, count]) => ({ reason, count }))
			.sort((a, b) => b.count - a.count);

		return {
			filesDetected,
			filesSkipped,
			chunksCreated,
			chunksEmbedded,
			repoUrl,
			status,
			duration,
			languages: languageArray,
			skipReasons: skipReasonArray,
			totalFiles: filesDetected + filesSkipped
		};
	});

	function formatDuration(ms: number): string {
		if (ms < 1000) return `${ms}ms`;
		return `${(ms / 1000).toFixed(2)}s`;
	}

	function formatReasonText(reason: string): string {
		return reason
			.split('_')
			.map((word) => word.charAt(0).toUpperCase() + word.slice(1))
			.join(' ');
	}

	function getStatusColor(status: string): string {
		switch (status) {
			case 'processing':
				return 'text-blue-600';
			case 'completed':
				return 'text-green-600';
			case 'failed':
				return 'text-red-600';
			default:
				return 'text-gray-600';
		}
	}

	function getStatusIcon(status: string): string {
		switch (status) {
			case 'processing':
				return '⏳';
			case 'completed':
				return '✅';
			case 'failed':
				return '❌';
			default:
				return '⚪';
		}
	}
</script>

{#if events.length > 0}
	<div class="bg-white rounded-lg shadow-md p-6 mb-8">
		<div class="flex items-center justify-between mb-6">
			<h2 class="text-2xl font-bold text-gray-900">Statistics Summary</h2>
			<div class="flex items-center gap-2">
				<span class={getStatusColor(stats.status) + ' font-semibold text-lg'}>
					{getStatusIcon(stats.status)}
					{stats.status.charAt(0).toUpperCase() + stats.status.slice(1)}
				</span>
			</div>
		</div>

		{#if stats.repoUrl}
			<div class="mb-6 p-4 bg-gray-50 rounded-lg">
				<span class="text-sm text-gray-600 font-medium">Repository:</span>
				<a
					href={stats.repoUrl}
					target="_blank"
					rel="noopener noreferrer"
					class="ml-2 text-blue-600 hover:text-blue-700 font-mono text-sm"
				>
					{stats.repoUrl}
				</a>
			</div>
		{/if}

		<!-- Main Stats Grid -->
		<div class="grid grid-cols-2 md:grid-cols-5 gap-4 mb-6">
			<div class="bg-blue-50 rounded-lg p-4">
				<div class="text-3xl font-bold text-blue-600">{stats.filesDetected}</div>
				<div class="text-sm text-gray-600 mt-1">Files Processed</div>
			</div>

			<div class="bg-green-50 rounded-lg p-4">
				<div class="text-3xl font-bold text-green-600">{stats.chunksCreated}</div>
				<div class="text-sm text-gray-600 mt-1">Chunks Created</div>
			</div>

			{#if stats.chunksEmbedded > 0}
			<div class="bg-indigo-50 rounded-lg p-4">
				<div class="text-3xl font-bold text-indigo-600">{stats.chunksEmbedded}</div>
				<div class="text-sm text-gray-600 mt-1">🧠 Embedded</div>
			</div>
			{/if}

			<div class="bg-yellow-50 rounded-lg p-4">
				<div class="text-3xl font-bold text-yellow-600">{stats.filesSkipped}</div>
				<div class="text-sm text-gray-600 mt-1">Files Skipped</div>
			</div>

			<div class="bg-purple-50 rounded-lg p-4">
				<div class="text-3xl font-bold text-purple-600">{stats.totalFiles}</div>
				<div class="text-sm text-gray-600 mt-1">Total Files</div>
			</div>
		</div>

		{#if stats.duration > 0}
			<div class="mb-6 text-center">
				<span class="text-gray-600">Processing Time:</span>
				<span class="ml-2 font-bold text-gray-900">{formatDuration(stats.duration)}</span>
				{#if stats.chunksCreated > 0 && stats.duration > 0}
					<span class="ml-4 text-sm text-gray-500">
						(~{Math.round((stats.duration / stats.chunksCreated) * 100) / 100}ms per chunk)
					</span>
				{/if}
			</div>
		{/if}

		<!-- Language Breakdown -->
		{#if stats.languages.length > 0}
			<div class="mb-6">
				<h3 class="text-lg font-semibold text-gray-900 mb-3">Languages Detected</h3>
				<div class="space-y-2">
					{#each stats.languages as { lang, count }}
						<div class="flex items-center justify-between">
							<div class="flex items-center gap-2">
								<span
									class="inline-block w-3 h-3 rounded-full"
									class:bg-orange-500={lang === 'rust'}
									class:bg-blue-500={lang === 'python'}
									class:bg-yellow-500={lang === 'javascript'}
									class:bg-blue-600={lang === 'typescript'}
									class:bg-cyan-500={lang === 'go'}
									class:bg-gray-400={!['rust', 'python', 'javascript', 'typescript', 'go'].includes(
										lang
									)}
								></span>
								<span class="font-mono text-sm font-medium text-gray-700 capitalize">{lang}</span>
							</div>
							<div class="flex items-center gap-3">
								<div class="flex-1 bg-gray-200 rounded-full h-2 w-32">
									<div
										class="bg-blue-600 h-2 rounded-full transition-all"
										style="width: {(count / stats.filesDetected) * 100}%"
									></div>
								</div>
								<span class="text-sm font-semibold text-gray-600 w-8 text-right">{count}</span>
							</div>
						</div>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Skip Reasons -->
		{#if stats.skipReasons.length > 0}
			<div>
				<h3 class="text-lg font-semibold text-gray-900 mb-3">Skip Reasons</h3>
				<div class="grid grid-cols-1 md:grid-cols-2 gap-3">
					{#each stats.skipReasons as { reason, count }}
						<div class="flex items-center justify-between p-3 bg-gray-50 rounded-lg">
							<span class="text-sm text-gray-700">{formatReasonText(reason)}</span>
							<span class="text-sm font-semibold text-gray-900 bg-white px-2 py-1 rounded">
								{count}
							</span>
						</div>
					{/each}
				</div>
			</div>
		{/if}

		{#if isLoading}
			<div class="mt-4 flex items-center justify-center gap-2 text-blue-600">
				<div class="inline-block w-4 h-4 border-2 border-blue-600 border-t-transparent rounded-full animate-spin"></div>
				<span class="text-sm font-medium">Streaming events...</span>
			</div>
		{/if}
	</div>
{/if}
