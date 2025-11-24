<script lang="ts">
	import type { SymbolContext } from '$lib/api-client';

	interface Props {
		symbol: SymbolContext;
		onClose?: () => void;
	}

	let { symbol, onClose }: Props = $props();

	function getSymbolIcon(kind: string): string {
		switch (kind.toLowerCase()) {
			case 'function':
				return 'ƒ';
			case 'struct':
			case 'class':
				return 'C';
			case 'enum':
				return 'E';
			case 'trait':
			case 'interface':
				return 'I';
			case 'method':
				return 'm';
			case 'const':
			case 'static':
				return 'K';
			case 'module':
				return 'M';
			default:
				return '◦';
		}
	}

	function getSymbolColor(kind: string): string {
		switch (kind.toLowerCase()) {
			case 'function':
			case 'method':
				return 'bg-purple-100 text-purple-700 border-purple-300';
			case 'struct':
			case 'class':
				return 'bg-blue-100 text-blue-700 border-blue-300';
			case 'enum':
				return 'bg-orange-100 text-orange-700 border-orange-300';
			case 'trait':
			case 'interface':
				return 'bg-green-100 text-green-700 border-green-300';
			case 'const':
			case 'static':
				return 'bg-gray-100 text-gray-700 border-gray-300';
			case 'module':
				return 'bg-cyan-100 text-cyan-700 border-cyan-300';
			default:
				return 'bg-gray-100 text-gray-500 border-gray-300';
		}
	}

	function getLanguageIcon(language: string): string {
		switch (language.toLowerCase()) {
			case 'rust':
				return '🦀';
			case 'python':
				return '🐍';
			case 'javascript':
				return '📜';
			case 'typescript':
				return '📘';
			case 'go':
				return '🐹';
			default:
				return '📄';
		}
	}

	function getCentralityColor(centrality: number): string {
		if (centrality >= 0.7) return 'text-red-600 bg-red-50';
		if (centrality >= 0.4) return 'text-orange-600 bg-orange-50';
		if (centrality >= 0.2) return 'text-yellow-600 bg-yellow-50';
		return 'text-gray-600 bg-gray-50';
	}

	function getCentralityLabel(centrality: number): string {
		if (centrality >= 0.7) return 'Very High';
		if (centrality >= 0.4) return 'High';
		if (centrality >= 0.2) return 'Medium';
		return 'Low';
	}
</script>

<div class="bg-white rounded-lg shadow-lg border border-gray-200 overflow-hidden max-w-4xl">
	<!-- Header -->
	<div class="bg-linear-to-r from-blue-50 to-indigo-50 px-6 py-4 border-b border-gray-200">
		<div class="flex items-start justify-between">
			<div class="flex items-start gap-3 flex-1">
				<div class={getSymbolColor(symbol.kind) + ' px-3 py-2 rounded text-sm font-bold border'}>
					<span class="inline-block w-5 text-center">{getSymbolIcon(symbol.kind)}</span>
				</div>
				<div class="flex-1 min-w-0">
					<h2 class="text-2xl font-bold text-gray-900 mb-1">{symbol.name}</h2>
					<div class="flex items-center gap-3 text-sm text-gray-600">
						<span class="flex items-center gap-1">
							<span class="text-lg">{getLanguageIcon(symbol.language)}</span>
							<span class="capitalize">{symbol.language}</span>
						</span>
						<span>•</span>
						<span class="capitalize">{symbol.kind}</span>
						<span>•</span>
						<span class="font-mono text-xs">{symbol.file_path}</span>
					</div>
				</div>
			</div>
			{#if onClose}
				<button
					onclick={onClose}
					class="text-gray-400 hover:text-gray-600 transition-colors ml-4"
					aria-label="Close"
				>
					<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M6 18L18 6M6 6l12 12"
						/>
					</svg>
				</button>
			{/if}
		</div>
	</div>

	<div class="p-6 space-y-6">
		<!-- Signature -->
		{#if symbol.signature}
			<div>
				<h3 class="text-sm font-semibold text-gray-700 mb-2">Signature</h3>
				<div
					class="font-mono text-sm text-gray-800 bg-gray-50 px-4 py-3 rounded-lg border border-gray-200 overflow-x-auto"
				>
					{symbol.signature}
				</div>
			</div>
		{/if}

		<!-- Metrics Row -->
		<div class="grid grid-cols-2 md:grid-cols-4 gap-4">
			<!-- Centrality -->
			<div class="bg-gray-50 rounded-lg p-4 border border-gray-200">
				<div class="text-xs font-medium text-gray-600 mb-1">Centrality</div>
				<div class="flex items-baseline gap-2">
					<div class="text-2xl font-bold text-gray-900">{(symbol.centrality * 100).toFixed(1)}%</div>
					<div class={getCentralityColor(symbol.centrality) + ' text-xs font-medium px-2 py-1 rounded'}>
						{getCentralityLabel(symbol.centrality)}
					</div>
				</div>
			</div>

			<!-- Calls Made -->
			<div class="bg-purple-50 rounded-lg p-4 border border-purple-200">
				<div class="text-xs font-medium text-purple-600 mb-1">Calls Made</div>
				<div class="text-2xl font-bold text-purple-900">{symbol.calls.length}</div>
			</div>

			<!-- Called By -->
			<div class="bg-blue-50 rounded-lg p-4 border border-blue-200">
				<div class="text-xs font-medium text-blue-600 mb-1">Called By</div>
				<div class="text-2xl font-bold text-blue-900">{symbol.called_by.length}</div>
			</div>

			<!-- Imports -->
			<div class="bg-green-50 rounded-lg p-4 border border-green-200">
				<div class="text-xs font-medium text-green-600 mb-1">Imports</div>
				<div class="text-2xl font-bold text-green-900">{symbol.imports.length}</div>
			</div>
		</div>

		<!-- Cluster Membership -->
		{#if symbol.cluster_label}
			<div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
				<div class="flex items-center gap-2">
					<svg class="w-5 h-5 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M7 20l4-16m2 16l4-16M6 9h14M4 15h14"
						/>
					</svg>
					<span class="text-sm font-semibold text-blue-900">Cluster:</span>
					<span class="text-sm text-blue-800 font-medium">{symbol.cluster_label}</span>
				</div>
			</div>
		{/if}

		<!-- Relationships Grid -->
		<div class="grid md:grid-cols-2 gap-6">
			<!-- Calls -->
			{#if symbol.calls.length > 0}
				<div>
					<h3 class="text-sm font-semibold text-gray-700 mb-3 flex items-center gap-2">
						<svg class="w-4 h-4 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M13 7l5 5m0 0l-5 5m5-5H6"
							/>
						</svg>
						Calls ({symbol.calls.length})
					</h3>
					<div class="space-y-2">
						{#each symbol.calls as call}
							<div
								class="text-sm font-mono bg-purple-50 text-purple-900 px-3 py-2 rounded border border-purple-200"
							>
								{call}
							</div>
						{/each}
					</div>
				</div>
			{/if}

			<!-- Called By -->
			{#if symbol.called_by.length > 0}
				<div>
					<h3 class="text-sm font-semibold text-gray-700 mb-3 flex items-center gap-2">
						<svg class="w-4 h-4 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M11 17l-5-5m0 0l5-5m-5 5h12"
							/>
						</svg>
						Called By ({symbol.called_by.length})
					</h3>
					<div class="space-y-2">
						{#each symbol.called_by as caller}
							<div
								class="text-sm font-mono bg-blue-50 text-blue-900 px-3 py-2 rounded border border-blue-200"
							>
								{caller}
							</div>
						{/each}
					</div>
				</div>
			{/if}

			<!-- Imports -->
			{#if symbol.imports.length > 0}
				<div>
					<h3 class="text-sm font-semibold text-gray-700 mb-3 flex items-center gap-2">
						<svg class="w-4 h-4 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10"
							/>
						</svg>
						Imports ({symbol.imports.length})
					</h3>
					<div class="space-y-2">
						{#each symbol.imports as importPath}
							<div
								class="text-sm font-mono bg-green-50 text-green-900 px-3 py-2 rounded border border-green-200"
							>
								{importPath}
							</div>
						{/each}
					</div>
				</div>
			{/if}

			<!-- Related Symbols -->
			{#if symbol.related_symbols.length > 0}
				<div>
					<h3 class="text-sm font-semibold text-gray-700 mb-3 flex items-center gap-2">
						<svg class="w-4 h-4 text-indigo-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"
							/>
						</svg>
						Related Symbols ({symbol.related_symbols.length})
					</h3>
					<div class="text-xs text-gray-600 mb-2">
						Semantically similar symbols based on embeddings
					</div>
					<div class="space-y-2">
						{#each symbol.related_symbols as related}
							<div
								class="text-sm font-mono bg-indigo-50 text-indigo-900 px-3 py-2 rounded border border-indigo-200"
							>
								{related}
							</div>
						{/each}
					</div>
				</div>
			{/if}
		</div>

		<!-- Empty State -->
		{#if symbol.calls.length === 0 && symbol.called_by.length === 0 && symbol.imports.length === 0 && symbol.related_symbols.length === 0}
			<div class="text-center py-8 text-gray-500">
				<svg class="w-12 h-12 mx-auto mb-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
					/>
				</svg>
				<p class="text-sm">No relationships or imports detected for this symbol</p>
			</div>
		{/if}
	</div>
</div>
