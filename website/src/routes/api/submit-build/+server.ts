import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const BUILDER_API_URL = process.env.BUILDER_API_URL?.trim();
const RUNPOD_API_KEY = process.env.RUNPOD_API_KEY?.trim();

export const POST: RequestHandler = async ({ request }) => {
	if (!BUILDER_API_URL || !RUNPOD_API_KEY) {
		return json(
			{ error: 'Server not configured: Missing BUILDER_API_URL or RUNPOD_API_KEY' },
			{ status: 500 }
		);
	}

	try {
		const { repo_url, job_id } = await request.json();

		if (!repo_url) {
			return json({ error: 'Missing repo_url' }, { status: 400 });
		}

		// Submit job to RunPod serverless
		const response = await fetch(BUILDER_API_URL, {
			method: 'POST',
			headers: {
				'Authorization': `Bearer ${RUNPOD_API_KEY}`,
				'Content-Type': 'application/json'
			},
			body: JSON.stringify({
				input: {
					repo_url,
					git_ref: 'main',
					job_id: job_id || `job_${Date.now()}`
				}
			})
		});

		if (!response.ok) {
			const errorText = await response.text();
			return json(
				{ error: `RunPod API error: ${response.status} ${errorText}` },
				{ status: response.status }
			);
		}

		const result = await response.json();
		return json(result);
	} catch (error: any) {
		console.error('Submit build error:', error);
		return json({ error: error.message }, { status: 500 });
	}
};
