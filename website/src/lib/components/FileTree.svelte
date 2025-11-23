<script lang="ts">
	interface Props {
		events: any[];
	}

	let { events }: Props = $props();

	// Organize events into file tree structure
	const fileData = $derived.by(() => {
		const files: Map<
			string,
			{
				path: string;
				language: string;
				chunks: Array<{ name: string; kind: string; chunkId: string }>;
			}
		> = new Map();

		for (const event of events) {
			const eventType = event.event_type || '';
			const payload = event.payload || {};

			if (eventType.includes('file_detected')) {
				const path = payload.file_path || '';
				if (!files.has(path)) {
					files.set(path, {
						path,
						language: payload.language || 'unknown',
						chunks: []
					});
				}
			} else if (eventType.includes('chunk_created')) {
				const path = payload.file_path || '';
				const file = files.get(path);
				if (file) {
					file.chunks.push({
						name: payload.symbol_name || 'unknown',
						kind: payload.symbol_kind || 'unknown',
						chunkId: payload.chunk_id || ''
					});
				}
			}
		}

		return Array.from(files.values()).sort((a, b) => a.path.localeCompare(b.path));
	});

	let expandedFiles = $state(new Set<string>());

	function toggleFile(path: string) {
		if (expandedFiles.has(path)) {
			expandedFiles.delete(path);
		} else {
			expandedFiles.add(path);
		}
		expandedFiles = new Set(expandedFiles); // Trigger reactivity
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
				return 'text-purple-600';
			case 'struct':
			case 'class':
				return 'text-blue-600';
			case 'enum':
				return 'text-orange-600';
			case 'trait':
			case 'interface':
				return 'text-green-600';
			case 'const':
			case 'static':
				return 'text-gray-600';
			case 'module':
				return 'text-cyan-600';
			default:
				return 'text-gray-500';
		}
	}
</script>

{#if fileData.length > 0}
	<div class="overflow-hidden">
		<div class="bg-gray-100 px-4 py-3 border-b border-gray-200">
			<div class="flex items-center justify-between">
				<div>
					<h3 class="text-sm font-bold text-gray-900">Files with Symbols</h3>
					<p class="text-xs text-gray-600 mt-0.5">{fileData.length} files</p>
				</div>
			</div>
		</div>

		<div class="max-h-[600px] overflow-y-auto">
			{#each fileData as file}
				<div class="border-b border-gray-200 last:border-b-0">
					<!-- File header -->
					<button
						onclick={() => toggleFile(file.path)}
						class="w-full flex items-center justify-between px-4 py-3 hover:bg-gray-50 transition-colors"
					>
						<div class="flex items-center gap-3 flex-1 min-w-0">
							<span class="text-2xl flex-shrink-0">{getLanguageIcon(file.language)}</span>
							<div class="flex-1 min-w-0 text-left">
								<div class="font-mono text-sm font-medium text-gray-900 truncate">
									{file.path}
								</div>
								<div class="text-xs text-gray-500 mt-0.5">
									{file.language} · {file.chunks.length}
									{file.chunks.length === 1 ? 'symbol' : 'symbols'}
								</div>
							</div>
						</div>
						<span class="text-gray-400 flex-shrink-0 ml-2">
							{expandedFiles.has(file.path) ? '▼' : '▶'}
						</span>
					</button>

					<!-- Symbol list (collapsible) -->
					{#if expandedFiles.has(file.path)}
						<div class="bg-gray-50 px-4 py-2">
							{#if file.chunks.length === 0}
								<p class="text-sm text-gray-500 italic py-2">No symbols extracted</p>
							{:else}
								<div class="space-y-1">
									{#each file.chunks as chunk}
										<div
											class="flex items-center gap-3 py-2 px-3 bg-white rounded hover:bg-gray-100 transition-colors"
										>
											<span
												class={getSymbolColor(chunk.kind) +
													' font-bold text-sm w-6 text-center flex-shrink-0'}
											>
												{getSymbolIcon(chunk.kind)}
											</span>
											<div class="flex-1 min-w-0">
												<div class="font-mono text-sm font-medium text-gray-900 truncate">
													{chunk.name}
												</div>
												<div class="text-xs text-gray-500 capitalize">{chunk.kind}</div>
											</div>
											<span class="text-xs text-gray-400 font-mono flex-shrink-0">
												{chunk.chunkId.slice(0, 12)}...
											</span>
										</div>
									{/each}
								</div>
							{/if}
						</div>
					{/if}
				</div>
			{/each}
		</div>

		<!-- Legend at bottom -->
		<div class="bg-gray-50 px-4 py-3 border-t border-gray-200">
			<div class="text-xs text-gray-600 font-semibold mb-2">Symbol Types:</div>
			<div class="grid grid-cols-2 md:grid-cols-4 gap-2 text-xs">
				<div class="flex items-center gap-1">
					<span class="text-purple-600 font-bold">ƒ</span>
					<span class="text-gray-600">Function</span>
				</div>
				<div class="flex items-center gap-1">
					<span class="text-blue-600 font-bold">C</span>
					<span class="text-gray-600">Struct/Class</span>
				</div>
				<div class="flex items-center gap-1">
					<span class="text-orange-600 font-bold">E</span>
					<span class="text-gray-600">Enum</span>
				</div>
				<div class="flex items-center gap-1">
					<span class="text-green-600 font-bold">I</span>
					<span class="text-gray-600">Trait/Interface</span>
				</div>
				<div class="flex items-center gap-1">
					<span class="text-purple-600 font-bold">m</span>
					<span class="text-gray-600">Method</span>
				</div>
				<div class="flex items-center gap-1">
					<span class="text-gray-600 font-bold">K</span>
					<span class="text-gray-600">Const</span>
				</div>
				<div class="flex items-center gap-1">
					<span class="text-cyan-600 font-bold">M</span>
					<span class="text-gray-600">Module</span>
				</div>
			</div>
		</div>
	</div>
{/if}
