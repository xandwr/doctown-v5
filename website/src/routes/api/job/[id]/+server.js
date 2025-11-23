import { json } from '@sveltejs/kit';
import { RUNPOD_ENDPOINT_ID, RUNPOD_API_KEY } from '$env/static/private';

export async function GET({ params }) {
  const { id } = params;

  const response = await fetch(`https://api.runpod.ai/v2/${RUNPOD_ENDPOINT_ID}/status/${id}`, {
    method: 'GET',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${RUNPOD_API_KEY}`
    }
  });

  const data = await response.json();

  return json(data);
}
