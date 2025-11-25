import { json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

const BUILDER_API_URL = process.env.BUILDER_API_URL?.trim();
const RUNPOD_API_KEY = process.env.RUNPOD_API_KEY?.trim();

export const GET: RequestHandler = async ({ params }) => {
	if (!BUILDER_API_URL || !RUNPOD_API_KEY) {
		return json(
			{ error: 'Server not configured: Missing BUILDER_API_URL or RUNPOD_API_KEY' },
			{ status: 500 }
		);
	}

	const { jobId } = params;

	if (!jobId) {
		return json({ error: 'Missing jobId' }, { status: 400 });
	}

	try {
		// Get status from RunPod
		const statusUrl = BUILDER_API_URL.replace('/run', `/status/${jobId}`);
		
		const response = await fetch(statusUrl, {
			headers: {
				'Authorization': `Bearer ${RUNPOD_API_KEY}`
			}
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
		console.error('Build status error:', error);
		return json({ error: error.message }, { status: 500 });
	}
};
