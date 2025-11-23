<script lang="ts">
	/**
	 * Repository Input Component
	 * Handles GitHub repository URL input with validation
	 */

	interface Props {
		onSubmit: (repoUrl: string) => void;
		isLoading?: boolean;
		disabled?: boolean;
	}

	let { onSubmit, isLoading = false, disabled = false }: Props = $props();

	let repoUrl = $state('');
	let validationError = $state<string | null>(null);

	/**
	 * Validate GitHub repository URL
	 * Accepts formats like:
	 * - https://github.com/user/repo
	 * - https://github.com/user/repo.git
	 * - github.com/user/repo
	 */
	function validateGitHubUrl(url: string): string | null {
		if (!url.trim()) {
			return 'Repository URL is required';
		}

		// Remove trailing .git if present
		const cleanUrl = url.trim().replace(/\.git$/, '');

		// GitHub URL pattern
		const githubPattern = /^(https?:\/\/)?(www\.)?github\.com\/[a-zA-Z0-9_-]+\/[a-zA-Z0-9_.-]+\/?$/;

		if (!githubPattern.test(cleanUrl)) {
			return 'Invalid GitHub URL. Expected format: https://github.com/user/repo';
		}

		return null;
	}

	/**
	 * Normalize GitHub URL to standard format
	 */
	function normalizeUrl(url: string): string {
		let normalized = url.trim().replace(/\.git$/, '');

		// Add https:// if not present
		if (!normalized.startsWith('http://') && !normalized.startsWith('https://')) {
			normalized = `https://${normalized}`;
		}

		// Remove trailing slash
		normalized = normalized.replace(/\/$/, '');

		return normalized;
	}

	function handleSubmit(event: Event) {
		event.preventDefault();

		// Validate URL
		const error = validateGitHubUrl(repoUrl);
		if (error) {
			validationError = error;
			return;
		}

		// Clear error and submit
		validationError = null;
		const normalizedUrl = normalizeUrl(repoUrl);
		onSubmit(normalizedUrl);
	}

	function handleInput() {
		// Clear validation error on input
		if (validationError) {
			validationError = null;
		}
	}
</script>

<form onsubmit={handleSubmit} class="w-full max-w-2xl">
	<div class="space-y-2">
		<label for="repo-url" class="block text-sm font-medium text-gray-700">
			GitHub Repository URL
		</label>

		<div class="flex gap-2">
			<input
				id="repo-url"
				type="text"
				bind:value={repoUrl}
				oninput={handleInput}
				disabled={isLoading || disabled}
				placeholder="https://github.com/user/repo"
				class="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:bg-gray-100 disabled:cursor-not-allowed"
				class:border-red-500={validationError}
			/>

			<button
				type="submit"
				disabled={isLoading || disabled || !repoUrl.trim()}
				class="px-6 py-2 bg-blue-600 text-white font-medium rounded-lg hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors"
			>
				{#if isLoading}
					<span class="flex items-center gap-2">
						<svg
							class="animate-spin h-5 w-5"
							xmlns="http://www.w3.org/2000/svg"
							fill="none"
							viewBox="0 0 24 24"
						>
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
						Processing...
					</span>
				{:else}
					Ingest
				{/if}
			</button>
		</div>

		{#if validationError}
			<p class="text-sm text-red-600 mt-1">{validationError}</p>
		{/if}

		<p class="text-xs text-gray-500 mt-1">
			Enter a public GitHub repository URL to analyze its code structure
		</p>
	</div>
</form>
