import { error, json, type RequestHandler } from '@sveltejs/kit';
import { S3Client, PutObjectCommand } from '@aws-sdk/client-s3';

// Load environment variables and trim whitespace
const BUCKET_NAME = process.env.BUCKET_NAME?.trim();
const BUCKET_ACCESS_KEY_ID = process.env.BUCKET_ACCESS_KEY_ID?.trim();
const BUCKET_SECRET_ACCESS_KEY = process.env.BUCKET_SECRET_ACCESS_KEY?.trim();
const BUCKET_S3_ENDPOINT = process.env.BUCKET_S3_ENDPOINT?.trim();
const BUCKET_PUBLIC_URL = process.env.BUCKET_PUBLIC_URL?.trim(); // e.g., https://pub-xxx.r2.dev or your custom domain

if (!BUCKET_NAME || !BUCKET_ACCESS_KEY_ID || !BUCKET_SECRET_ACCESS_KEY || !BUCKET_S3_ENDPOINT) {
	console.error('Missing required R2 environment variables');
}

// Initialize S3 client for R2
// Note: Cloudflare R2 is S3-compatible but has specific requirements
const s3Client = new S3Client({
	region: 'auto', // R2 uses 'auto' as the region
	endpoint: BUCKET_S3_ENDPOINT,
	credentials: {
		accessKeyId: BUCKET_ACCESS_KEY_ID!,
		secretAccessKey: BUCKET_SECRET_ACCESS_KEY!
	},
	forcePathStyle: true // Required for R2 - prevents virtual-hosted-style addressing
});

export const POST: RequestHandler = async ({ request }) => {
	try {
		const { repoOwner, repoName, docpackData } = await request.json();

		if (!repoOwner || !repoName || !docpackData) {
			throw error(400, 'Missing required fields: repoOwner, repoName, docpackData');
		}

		// Verify credentials are loaded
		if (!BUCKET_NAME || !BUCKET_ACCESS_KEY_ID || !BUCKET_SECRET_ACCESS_KEY || !BUCKET_S3_ENDPOINT) {
			console.error('R2 credentials not properly configured');
			throw error(500, 'R2 storage not configured. Check environment variables.');
		}

		console.log('Uploading to R2:', {
			bucket: BUCKET_NAME,
			endpoint: BUCKET_S3_ENDPOINT,
			keyId: BUCKET_ACCESS_KEY_ID?.substring(0, 8) + '...' // Log first 8 chars only
		});

		// Construct R2 key: docpacks/[repo_owner]/[repo_name].docpack
		const key = `docpacks/${repoOwner}/${repoName}.docpack`;

		// Convert docpack data to JSON string
		const docpackJson = JSON.stringify(docpackData, null, 2);
		const buffer = Buffer.from(docpackJson, 'utf-8');

		// Upload to R2
		const command = new PutObjectCommand({
			Bucket: BUCKET_NAME,
			Key: key,
			Body: buffer,
			ContentType: 'application/json',
			Metadata: {
				'docpack-version': '1.0',
				'generated-at': new Date().toISOString(),
				'repo-owner': repoOwner,
				'repo-name': repoName
			}
		});

		await s3Client.send(command);

		// Construct public URL - use BUCKET_PUBLIC_URL if set, otherwise fall back to r2.dev pattern
		const publicUrl = BUCKET_PUBLIC_URL 
			? `${BUCKET_PUBLIC_URL}/${key}`
			: `https://${BUCKET_NAME}.r2.dev/${key}`; // Fallback (may not work without public access enabled)

		return json({
			success: true,
			key,
			size: buffer.length,
			url: publicUrl
		});
	} catch (err: any) {
		console.error('Error uploading to R2:', err);
		const message = err.message || err.Code || err.name || 'Unknown error';
		throw error(500, `Failed to upload docpack: ${message}`);
	}
};
