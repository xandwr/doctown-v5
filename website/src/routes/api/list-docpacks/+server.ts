import { json, type RequestHandler } from '@sveltejs/kit';
import { S3Client, ListObjectsV2Command, HeadObjectCommand } from '@aws-sdk/client-s3';

// Load environment variables and trim whitespace
const BUCKET_NAME = process.env.BUCKET_NAME?.trim();
const BUCKET_ACCESS_KEY_ID = process.env.BUCKET_ACCESS_KEY_ID?.trim();
const BUCKET_SECRET_ACCESS_KEY = process.env.BUCKET_SECRET_ACCESS_KEY?.trim();
const BUCKET_S3_ENDPOINT = process.env.BUCKET_S3_ENDPOINT?.trim();
const BUCKET_PUBLIC_URL = process.env.BUCKET_PUBLIC_URL?.trim();

// Initialize S3 client for R2
const s3Client = new S3Client({
	region: 'auto',
	endpoint: BUCKET_S3_ENDPOINT,
	credentials: {
		accessKeyId: BUCKET_ACCESS_KEY_ID!,
		secretAccessKey: BUCKET_SECRET_ACCESS_KEY!
	},
	forcePathStyle: true
});

export interface DocpackInfo {
	key: string;
	owner: string;
	repo: string;
	size: number;
	lastModified: string;
	url: string;
}

export const GET: RequestHandler = async () => {
	try {
		// Verify credentials are loaded
		if (!BUCKET_NAME || !BUCKET_ACCESS_KEY_ID || !BUCKET_SECRET_ACCESS_KEY || !BUCKET_S3_ENDPOINT) {
			console.error('R2 credentials not properly configured');
			return json({ error: 'R2 storage not configured' }, { status: 500 });
		}

		// List all objects in the docpacks/ prefix
		const command = new ListObjectsV2Command({
			Bucket: BUCKET_NAME,
			Prefix: 'docpacks/'
		});

		const response = await s3Client.send(command);
		const docpacks: DocpackInfo[] = [];

		if (response.Contents) {
			for (const object of response.Contents) {
				// Skip if not a .docpack file
				if (!object.Key?.endsWith('.docpack')) continue;

				// Parse owner/repo from key: docpacks/[owner]/[repo].docpack
				const keyParts = object.Key.replace('docpacks/', '').replace('.docpack', '').split('/');
				if (keyParts.length !== 2) continue;

				const [owner, repo] = keyParts;

				// Construct public URL
				const publicUrl = BUCKET_PUBLIC_URL
					? `${BUCKET_PUBLIC_URL}/${object.Key}`
					: `https://${BUCKET_NAME}.r2.dev/${object.Key}`;

				docpacks.push({
					key: object.Key,
					owner,
					repo,
					size: object.Size || 0,
					lastModified: object.LastModified?.toISOString() || new Date().toISOString(),
					url: publicUrl
				});
			}
		}

		// Sort by last modified (newest first)
		docpacks.sort((a, b) => new Date(b.lastModified).getTime() - new Date(a.lastModified).getTime());

		return json({
			success: true,
			count: docpacks.length,
			docpacks
		});
	} catch (err: any) {
		console.error('Error listing docpacks:', err);
		const message = err.message || err.Code || err.name || 'Unknown error';
		return json({ error: `Failed to list docpacks: ${message}` }, { status: 500 });
	}
};
