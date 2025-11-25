<script lang="ts">
	import { onMount } from 'svelte';
	import DocpackCard from '$lib/components/DocpackCard.svelte';

	interface DocpackInfo {
		key: string;
		owner: string;
		repo: string;
		size: number;
		lastModified: string;
		url: string;
	}

	let docpacks = $state<DocpackInfo[]>([]);
	let isLoading = $state(true);
	let errorMessage = $state<string | null>(null);
	let searchQuery = $state('');

	onMount(async () => {
		await loadDocpacks();
	});

	async function loadDocpacks() {
		isLoading = true;
		errorMessage = null;

		try {
			const response = await fetch('/api/list-docpacks');
			const data = await response.json();

			if (!response.ok) {
				throw new Error(data.error || 'Failed to load docpacks');
			}

			docpacks = data.docpacks;
		} catch (err: any) {
			console.error('Error loading docpacks:', err);
			errorMessage = err.message;
		} finally {
			isLoading = false;
		}
	}

	const filteredDocpacks = $derived(
		searchQuery.trim()
			? docpacks.filter(
					(d) =>
						d.owner.toLowerCase().includes(searchQuery.toLowerCase()) ||
						d.repo.toLowerCase().includes(searchQuery.toLowerCase())
				)
			: docpacks
	);

	function formatBytes(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
	}

	function formatDate(dateString: string): string {
		const date = new Date(dateString);
		return date.toLocaleDateString('en-US', {
			year: 'numeric',
			month: 'short',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}
</script>

<svelte:head>
	<title>Commons - Doctown</title>
	<meta name="description" content="Browse publicly available docpacks for popular repositories" />
</svelte:head>

<div class="bg-gray-50 py-12 px-4">
	<div class="max-w-6xl mx-auto">
		<!-- Header -->
		<header class="text-center mb-12">
			<h1 class="text-4xl font-bold text-gray-900 mb-2">📚 Docpack Commons</h1>
			<p class="text-lg text-gray-600">
				Browse and download pre-generated documentation packs for popular repositories
			</p>
		</header>

		<!-- Search & Actions -->
		<div class="flex flex-col sm:flex-row gap-4 mb-8">
			<div class="flex-1 relative">
				<svg
					class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400"
					fill="none"
					stroke="currentColor"
					viewBox="0 0 24 24"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
					/>
				</svg>
				<input
					type="text"
					placeholder="Search by owner or repository name..."
					bind:value={searchQuery}
					class="w-full pl-10 pr-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 bg-white"
				/>
			</div>
			<button
				onclick={loadDocpacks}
				disabled={isLoading}
				class="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
			>
				{#if isLoading}
					<svg class="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
						<circle
							class="opacity-25"
							cx="12"
							cy="12"
							r="10"
							stroke="currentColor"
							stroke-width="4"
						></circle>
						<path
							class="opacity-75"
							fill="currentColor"
							d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
						></path>
					</svg>
					Loading...
				{:else}
					<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
						/>
					</svg>
					Refresh
				{/if}
			</button>
		</div>

		<!-- Error Message -->
		{#if errorMessage}
			<div class="bg-red-50 border border-red-200 rounded-lg p-4 mb-8">
				<p class="text-red-800 font-medium">Error: {errorMessage}</p>
			</div>
		{/if}

		<!-- Loading State -->
		{#if isLoading && docpacks.length === 0}
			<div class="flex items-center justify-center py-16">
				<div class="text-center">
					<svg class="w-12 h-12 animate-spin mx-auto mb-4 text-blue-600" fill="none" viewBox="0 0 24 24">
						<circle
							class="opacity-25"
							cx="12"
							cy="12"
							r="10"
							stroke="currentColor"
							stroke-width="4"
						></circle>
						<path
							class="opacity-75"
							fill="currentColor"
							d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
						></path>
					</svg>
					<p class="text-gray-600">Loading docpacks from R2 storage...</p>
				</div>
			</div>
		{:else if filteredDocpacks.length === 0}
			<!-- Empty State -->
			<div class="text-center py-16">
				<svg
					class="w-16 h-16 mx-auto mb-4 text-gray-400"
					fill="none"
					stroke="currentColor"
					viewBox="0 0 24 24"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="1.5"
						d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
					/>
				</svg>
				{#if searchQuery}
					<h3 class="text-lg font-medium text-gray-900 mb-2">No matching docpacks</h3>
					<p class="text-gray-600">
						No docpacks found matching "{searchQuery}". Try a different search term.
					</p>
				{:else}
					<h3 class="text-lg font-medium text-gray-900 mb-2">No docpacks yet</h3>
					<p class="text-gray-600 mb-4">
						Be the first to generate a docpack! Head to the home page and analyze a repository.
					</p>
					<a
						href="/"
						class="inline-flex items-center px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium"
					>
						Generate a Docpack
					</a>
				{/if}
			</div>
		{:else}
			<!-- Stats Bar -->
			<div class="bg-white rounded-lg shadow-sm p-4 mb-6 flex items-center justify-between">
				<div class="text-gray-600">
					Showing <span class="font-medium text-gray-900">{filteredDocpacks.length}</span>
					{filteredDocpacks.length === 1 ? 'docpack' : 'docpacks'}
					{#if searchQuery}
						matching "<span class="font-medium">{searchQuery}</span>"
					{/if}
				</div>
				<div class="text-sm text-gray-500">
					Total size: {formatBytes(filteredDocpacks.reduce((sum, d) => sum + d.size, 0))}
				</div>
			</div>

			<!-- Docpack Grid -->
			<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
				{#each filteredDocpacks as docpack (docpack.key)}
					<DocpackCard {docpack} {formatBytes} {formatDate} />
				{/each}
			</div>
		{/if}
	</div>
</div>
