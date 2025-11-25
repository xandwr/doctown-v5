# Vercel Deployment Setup

## Environment Variables

The website needs to connect to your RunPod serverless backend in production. You must set these environment variables in Vercel:

### Required Environment Variables

Go to your Vercel project → Settings → Environment Variables and add:

```
BUILDER_API_URL=https://api.runpod.ai/v2/YOUR_BUILDER_ENDPOINT_ID/run
RUNPOD_API_KEY=your_runpod_api_key_here
```

**To get your configuration:**
1. Go to https://www.runpod.io/console/serverless
2. Find your **Builder** serverless endpoint
3. Copy the endpoint ID (e.g., `abc123def456`)
4. Set `BUILDER_API_URL` to: `https://api.runpod.ai/v2/abc123def456/run`
5. Get your RunPod API key from https://www.runpod.io/console/user/settings
6. Set `RUNPOD_API_KEY` to your API key

**Important:** Do NOT use the `VITE_` prefix for these variables - they are server-side only and should not be exposed to the browser. The frontend calls internal API routes (`/api/submit-build` and `/api/build-status`) which securely communicate with RunPod.

**Note:** The Builder serverless endpoint handles the entire pipeline (ingest → embedding → assembly) internally, so you only need one endpoint URL.

### Required Storage Variables (for R2)

```
BUCKET_NAME=your-bucket-name
BUCKET_ACCESS_KEY_ID=your-access-key
BUCKET_SECRET_ACCESS_KEY=your-secret-key
BUCKET_S3_ENDPOINT=https://your-account.r2.cloudflarestorage.com
```

## Project Settings

### Root Directory
Set to: `website`

This is required for monorepo support.

### Build Settings
- **Framework Preset**: SvelteKit
- **Build Command**: `npm run build` (auto-detected)
- **Output Directory**: `.svelte-kit` (auto-detected)
- **Install Command**: `npm install` (auto-detected)

### Git Integration
- **Production Branch**: `main`
- Ensure GitHub integration is connected and webhooks are working

## Deploy

After setting environment variables:
1. Trigger a new deployment (push to main or use Vercel dashboard)
2. Check deployment logs for any errors
3. Test the deployed site

## Troubleshooting

### Job Fails or Times Out
If jobs fail or timeout:
- Check your RunPod serverless endpoint is deployed and active
- Verify your `VITE_RUNPOD_API_KEY` is correct
- Check RunPod logs in the serverless console for errors
- Default timeout is 10 minutes - adjust if needed for large repos

### Missing Environment Variables
- `BUILDER_API_URL` and `RUNPOD_API_KEY` are server-side only (no `VITE_` prefix)
- Server-side variables (like `BUCKET_*` and RunPod variables) are NOT exposed to the browser
- Client-side variables (with `VITE_` prefix) are embedded in the build and publicly visible
- After adding variables, you must **redeploy** for them to take effect
- Check build logs to verify environment variables are set

### Build Fails
- Ensure Root Directory is set to `website`
- Check that `package.json` has all required dependencies
- Review build logs in Vercel dashboard
