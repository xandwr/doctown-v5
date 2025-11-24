<script lang="ts">
	import type { SymbolContext } from '$lib/api-client';
	import SymbolDetail from './SymbolDetail.svelte';

	interface Props {
		symbolContexts: SymbolContext[];
	}

	let { symbolContexts }: Props = $props();

	// Organize symbols by file
	const symbolsByFile = $derived.by(() => {
		const grouped = new Map<string, SymbolContext[]>();
		
		for (const symbol of symbolContexts) {
			if (!grouped.has(symbol.file_path)) {
				grouped.set(symbol.file_path, []);
			}
			grouped.get(symbol.file_path)!.push(symbol);
		}
		
		return Array.from(grouped.entries())
			.map(([filePath, symbols]) => ({
				filePath,
				symbols: symbols.sort((a, b) => b.centrality - a.centrality) // Sort by centrality
			}))
			.sort((a, b) => a.filePath.localeCompare(b.filePath));
	});

	const totalSymbols = $derived(symbolContexts.length);

	// Filter state
	let searchQuery = $state('');
	let selectedKind = $state<string>('all');
	let selectedSymbol = $state<SymbolContext | null>(null);

	const filteredData = $derived.by(() => {
		const query = searchQuery.toLowerCase();
		return symbolsByFile
			.map((file) => ({
				...file,
				symbols: file.symbols.filter((symbol) => {
					const matchesSearch =
						query === '' ||
						symbol.name.toLowerCase().includes(query) ||
						symbol.signature.toLowerCase().includes(query) ||
						symbol.file_path.toLowerCase().includes(query) ||
						symbol.calls.some(c => c.toLowerCase().includes(query)) ||
						symbol.imports.some(i => i.toLowerCase().includes(query));
					const matchesKind =
						selectedKind === 'all' || symbol.kind.toLowerCase() === selectedKind.toLowerCase();
					return matchesSearch && matchesKind;
				})
			}))
			.filter((file) => file.symbols.length > 0);
	});

	const filteredSymbolCount = $derived(
		filteredData.reduce((sum, file) => sum + file.symbols.length, 0)
	);

	// Get unique symbol kinds for filter
	const symbolKinds = $derived.by(() => {
		const kinds = new Set<string>();
		symbolContexts.forEach((symbol) => {
			kinds.add(symbol.kind);
		});
		return Array.from(kinds).sort();
	});

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

	function getCentralityBadge(centrality: number): { color: string; label: string } {
		if (centrality >= 0.7) return { color: 'bg-red-100 text-red-700', label: 'Critical' };
		if (centrality >= 0.4) return { color: 'bg-orange-100 text-orange-700', label: 'High' };
		if (centrality >= 0.2) return { color: 'bg-yellow-100 text-yellow-700', label: 'Med' };
		return { color: 'bg-gray-100 text-gray-600', label: 'Low' };
	}
</script>

{#if selectedSymbol}
	<!-- Modal overlay -->
	<div
		role="button"
		tabindex="0"
		class="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center p-4"
		onclick={() => (selectedSymbol = null)}
		onkeydown={(e) => e.key === 'Escape' && (selectedSymbol = null)}
	>
		<div role="dialog" tabindex="-1" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.stopPropagation()}>
			<SymbolDetail symbol={selectedSymbol} onClose={() => (selectedSymbol = null)} />
		</div>
	</div>
{/if}

{#if symbolContexts.length > 0}
	<div class="overflow-hidden">
		<!-- Header -->
		<div class="bg-gray-100 px-4 py-3 border-b border-gray-200">
			<div class="flex items-center justify-between">
				<div>
					<h3 class="text-sm font-bold text-gray-900">Symbol Contexts</h3>
					<p class="text-xs text-gray-600 mt-0.5">
						{totalSymbols} symbols with full context and relationships
					</p>
				</div>
			</div>
		</div>

		<!-- Filters -->
		<div class="bg-gray-50 border-b border-gray-200 px-4 py-3">
			<div class="flex flex-col md:flex-row gap-3">
				<!-- Search input -->
				<div class="flex-1">
					<input
						type="text"
						bind:value={searchQuery}
						placeholder="Search symbols, calls, imports..."
						class="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
					/>
				</div>

				<!-- Kind filter -->
				<div class="flex items-center gap-2">
					<label for="kind-filter" class="text-sm text-gray-600 font-medium whitespace-nowrap">
						Filter:
					</label>
					<select
						id="kind-filter"
						bind:value={selectedKind}
						class="px-3 py-2 border border-gray-300 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent bg-white"
					>
						<option value="all">All Types</option>
						{#each symbolKinds as kind}
							<option value={kind}>{kind}</option>
						{/each}
					</select>
				</div>
			</div>

			{#if searchQuery || selectedKind !== 'all'}
				<div class="mt-2 text-xs text-gray-600">
					Showing {filteredSymbolCount} of {totalSymbols} symbols
				</div>
			{/if}
		</div>

		<!-- Symbol list -->
		<div class="max-h-[600px] overflow-y-auto">
			{#if filteredData.length === 0}
				<div class="p-8 text-center text-gray-500">
					<p class="text-lg mb-2">No symbols found</p>
					<p class="text-sm">Try adjusting your search or filters</p>
				</div>
			{:else}
				{#each filteredData as file}
					<div class="border-b border-gray-200 last:border-b-0">
						<!-- File header -->
						<div class="bg-gray-50 px-4 py-2 sticky top-0">
							<div class="flex items-center gap-2">
								<span class="text-lg">{getLanguageIcon(file.symbols[0]?.language)}</span>
								<span class="font-mono text-xs text-gray-700 font-medium">{file.filePath}</span>
								<span class="text-xs text-gray-500">
									({file.symbols.length} {file.symbols.length === 1 ? 'symbol' : 'symbols'})
								</span>
							</div>
						</div>

						<!-- Symbols in this file -->
						<div class="divide-y divide-gray-100">
							{#each file.symbols as symbol}
								<button
									onclick={() => (selectedSymbol = symbol)}
									class="w-full px-4 py-3 hover:bg-blue-50 transition-colors text-left cursor-pointer"
								>
									<div class="flex items-start gap-3">
										<!-- Symbol kind badge -->
										<div
											class={getSymbolColor(symbol.kind) +
												' px-2 py-1 rounded text-xs font-bold border shrink-0'}
										>
											<span class="inline-block w-4 text-center">
												{getSymbolIcon(symbol.kind)}
											</span>
										</div>

										<!-- Symbol details -->
										<div class="flex-1 min-w-0">
											<div class="flex items-center gap-2 mb-1">
												<span class="font-mono text-sm font-semibold text-gray-900">
													{symbol.name}
												</span>
												
												<!-- Centrality badge -->
												{#if true}
													{@const badge = getCentralityBadge(symbol.centrality)}
													<span class={badge.color + ' text-xs px-2 py-0.5 rounded font-medium'}>
														{badge.label} {(symbol.centrality * 100).toFixed(0)}%
													</span>
												{/if}
												
												<!-- Cluster label -->
												{#if symbol.cluster_label}
													<span class="text-xs bg-blue-100 text-blue-700 px-2 py-0.5 rounded">
														{symbol.cluster_label}
													</span>
												{/if}
											</div>

											{#if symbol.signature}
												<div
													class="font-mono text-xs text-gray-600 bg-gray-50 px-2 py-1 rounded overflow-x-auto whitespace-nowrap mb-2"
												>
													{symbol.signature}
												</div>
											{/if}

											<!-- Relationship counts -->
											<div class="flex items-center gap-4 text-xs text-gray-600">
												{#if symbol.calls.length > 0}
													<span class="flex items-center gap-1">
														<svg class="w-3 h-3 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
															<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
														</svg>
														{symbol.calls.length} calls
													</span>
												{/if}
												{#if symbol.called_by.length > 0}
													<span class="flex items-center gap-1">
														<svg class="w-3 h-3 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
															<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 17l-5-5m0 0l5-5m-5 5h12" />
														</svg>
														{symbol.called_by.length} callers
													</span>
												{/if}
												{#if symbol.imports.length > 0}
													<span class="flex items-center gap-1">
														<svg class="w-3 h-3 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
															<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M9 19l3 3m0 0l3-3m-3 3V10" />
														</svg>
														{symbol.imports.length} imports
													</span>
												{/if}
												{#if symbol.related_symbols.length > 0}
													<span class="flex items-center gap-1">
														<svg class="w-3 h-3 text-indigo-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
															<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
														</svg>
														{symbol.related_symbols.length} related
													</span>
												{/if}
											</div>
										</div>

										<!-- Click indicator -->
										<div class="shrink-0 text-gray-400">
											<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
												<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
											</svg>
										</div>
									</div>
								</button>
							{/each}
						</div>
					</div>
				{/each}
			{/if}
		</div>

		<!-- Footer stats -->
		<div class="bg-gray-50 px-4 py-3 border-t border-gray-200 text-xs text-gray-600">
			<div class="flex items-center justify-between">
				<span>
					{filteredData.length} files • {filteredSymbolCount} symbols
				</span>
				{#if searchQuery || selectedKind !== 'all'}
					<button
						onclick={() => {
							searchQuery = '';
							selectedKind = 'all';
						}}
						class="text-blue-600 hover:text-blue-700 font-medium"
					>
						Clear filters
					</button>
				{/if}
			</div>
		</div>
	</div>
{:else}
	<div class="p-8 text-center text-gray-500">
		<p class="text-sm">No symbol contexts available yet. Complete the assembly stage to see detailed symbol information.</p>
	</div>
{/if}
