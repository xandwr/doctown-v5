import { json } from '@sveltejs/kit';
import { RUNPOD_ENDPOINT_ID, RUNPOD_API_KEY } from '$env/static/private';

export async function POST({ request }) {
  const { repoUrl } = await request.json();

  const response = await fetch(`https://api.runpod.ai/v2/${RUNPOD_ENDPOINT_ID}/run`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${RUNPOD_API_KEY}`
    },
    body: JSON.stringify({
      input: {
        repo_url: repoUrl
      }
    })
  });

  const data = await response.json();
  
  return json(data);
}
