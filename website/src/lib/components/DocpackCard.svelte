<script lang="ts">
	interface DocpackInfo {
		key: string;
		owner: string;
		repo: string;
		size: number;
		lastModified: string;
		url: string;
	}

	interface Props {
		docpack: DocpackInfo;
		formatBytes: (bytes: number) => string;
		formatDate: (dateString: string) => string;
	}

	let { docpack, formatBytes, formatDate }: Props = $props();
	let copied = $state(false);

	async function copyUrl() {
		await navigator.clipboard.writeText(docpack.url);
		copied = true;
		setTimeout(() => (copied = false), 2000);
	}
</script>

<div class="bg-white rounded-lg shadow-md hover:shadow-lg transition-shadow overflow-hidden">
	<!-- Header with GitHub-style owner/repo -->
	<div class="p-4 border-b border-gray-100">
		<div class="flex items-center gap-3">
			<div class="w-10 h-10 bg-linear-to-br from-blue-500 to-purple-600 rounded-lg flex items-center justify-center text-white font-bold text-lg">
				{docpack.owner.charAt(0).toUpperCase()}
			</div>
			<div class="flex-1 min-w-0">
				<h3 class="font-semibold text-gray-900 truncate">
					<a
						href="https://github.com/{docpack.owner}/{docpack.repo}"
						target="_blank"
						rel="noopener noreferrer"
						class="hover:text-blue-600 transition-colors"
					>
						{docpack.owner}/{docpack.repo}
					</a>
				</h3>
				<p class="text-sm text-gray-500 truncate">{docpack.key}</p>
			</div>
		</div>
	</div>

	<!-- Stats -->
	<div class="px-4 py-3 bg-gray-50 grid grid-cols-2 gap-4 text-sm">
		<div>
			<span class="text-gray-500">Size:</span>
			<span class="font-medium text-gray-900 ml-1">{formatBytes(docpack.size)}</span>
		</div>
		<div>
			<span class="text-gray-500">Updated:</span>
			<span class="font-medium text-gray-900 ml-1">{formatDate(docpack.lastModified)}</span>
		</div>
	</div>

	<!-- Actions -->
	<div class="p-4 flex gap-2">
		<a
			href={docpack.url}
			download="{docpack.repo}.docpack"
			class="flex-1 inline-flex items-center justify-center px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors font-medium text-sm"
		>
			<svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					stroke-width="2"
					d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"
				/>
			</svg>
			Download
		</a>
		<button
			onclick={copyUrl}
			class="px-3 py-2 border border-gray-300 rounded-md hover:bg-gray-50 transition-colors text-gray-700"
			title="Copy URL"
		>
			{#if copied}
				<svg class="w-5 h-5 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
				</svg>
			{:else}
				<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
					/>
				</svg>
			{/if}
		</button>
		<a
			href="https://github.com/{docpack.owner}/{docpack.repo}"
			target="_blank"
			rel="noopener noreferrer"
			class="px-3 py-2 border border-gray-300 rounded-md hover:bg-gray-50 transition-colors text-gray-700"
			title="View on GitHub"
		>
			<svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
				<path
					fill-rule="evenodd"
					d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
					clip-rule="evenodd"
				/>
			</svg>
		</a>
	</div>
</div>
