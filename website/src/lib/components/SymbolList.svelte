<script lang="ts">
	interface Props {
		events: any[];
	}

	let { events }: Props = $props();

	// Organize symbols from events
	const symbolData = $derived.by(() => {
		console.log('[SymbolList] Processing events:', events.length);
		const symbols: Array<{
			name: string;
			kind: string;
			signature: string;
			filePath: string;
			language: string;
			chunkId: string;
		}> = [];

		// Track file metadata
		const fileMetadata = new Map<string, { language: string }>();

		for (const event of events) {
			const eventType = event.event_type || '';
			const payload = event.payload || {};

			if (eventType.includes('file_detected')) {
				fileMetadata.set(payload.file_path, {
					language: payload.language || 'unknown'
				});
			} else if (eventType.includes('chunk_created')) {
				const filePath = payload.file_path || 'unknown';
				const fileInfo = fileMetadata.get(filePath);

				symbols.push({
					name: payload.symbol_name || 'unnamed',
					kind: payload.symbol_kind || 'unknown',
					signature: payload.symbol_signature || '',
					filePath,
					language: fileInfo?.language || 'unknown',
					chunkId: payload.chunk_id || ''
				});
			}
		}

		// Group by file and sort
		const grouped = new Map<string, typeof symbols>();
		for (const symbol of symbols) {
			if (!grouped.has(symbol.filePath)) {
				grouped.set(symbol.filePath, []);
			}
			grouped.get(symbol.filePath)!.push(symbol);
		}

		return Array.from(grouped.entries())
			.map(([filePath, symbols]) => ({
				filePath,
				symbols: symbols.sort((a, b) => a.name.localeCompare(b.name))
			}))
			.sort((a, b) => a.filePath.localeCompare(b.filePath));
	});

	const totalSymbols = $derived(
		symbolData.reduce((sum, file) => sum + file.symbols.length, 0)
	);

	// Filter state
	let searchQuery = $state('');
	let selectedKind = $state<string>('all');

	const filteredData = $derived.by(() => {
		const query = searchQuery.toLowerCase();
		return symbolData
			.map((file) => ({
				...file,
				symbols: file.symbols.filter((symbol) => {
					const matchesSearch =
						query === '' ||
						symbol.name.toLowerCase().includes(query) ||
						symbol.signature.toLowerCase().includes(query) ||
						symbol.filePath.toLowerCase().includes(query);
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
		symbolData.forEach((file) => {
			file.symbols.forEach((symbol) => {
				kinds.add(symbol.kind);
			});
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
</script>

{#if symbolData.length > 0}
	<div class="overflow-hidden">
		<!-- Header -->
		<div class="bg-gray-100 px-4 py-3 border-b border-gray-200">
			<div class="flex items-center justify-between">
				<div>
					<h3 class="text-sm font-bold text-gray-900">All Symbols</h3>
					<p class="text-xs text-gray-600 mt-0.5">
						{totalSymbols} symbols across {symbolData.length} files
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
						placeholder="Search symbols..."
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
								<div class="px-4 py-3 hover:bg-gray-50 transition-colors">
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
											<div class="font-mono text-sm font-semibold text-gray-900 mb-1">
												{symbol.name}
											</div>

											{#if symbol.signature}
												<div
													class="font-mono text-xs text-gray-600 bg-gray-50 px-2 py-1 rounded overflow-x-auto whitespace-nowrap"
												>
													{symbol.signature}
												</div>
											{/if}

											<div class="flex items-center gap-3 mt-2 text-xs text-gray-500">
												<span class="capitalize">{symbol.kind}</span>
												<span>•</span>
												<span class="font-mono truncate" title={symbol.chunkId}>
													{symbol.chunkId.slice(0, 16)}...
												</span>
											</div>
										</div>
									</div>
								</div>
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
	<div class="px-4 py-12 text-center text-gray-500">
		<p class="text-sm">No symbols detected yet.</p>
		<p class="text-xs mt-2">Events received: {events.length}</p>
	</div>
{/if}
