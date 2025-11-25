# Vercel Deployment Setup

## Environment Variables

The website needs to connect to your RunPod backend in production. You must set these environment variables in Vercel:

### Required Environment Variables

Go to your Vercel project → Settings → Environment Variables and add:

```
VITE_EMBEDDING_API_URL=https://YOUR-POD-ID-8000.proxy.runpod.net
VITE_INGEST_API_URL=https://YOUR-POD-ID-3000.proxy.runpod.net
VITE_ASSEMBLY_API_URL=https://YOUR-POD-ID-8002.proxy.runpod.net
```

**To get your RunPod URLs:**
1. Go to https://www.runpod.io/console/pods
2. Find your active pod
3. Look for the proxy URLs (format: `{pod-id}-{port}.proxy.runpod.net`)
4. Replace `YOUR-POD-ID` with your actual pod ID

### Optional Storage Variables (if using R2)

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

### CORS Errors
If you see CORS errors in production:
- Verify your RunPod URLs are correct
- Check that your RunPod services are running
- Ensure RunPod ports (3000, 8000, 3001) are exposed

### Still Using localhost:8000
- Environment variables in Vercel must be prefixed with `VITE_`
- After adding variables, you must **redeploy** for them to take effect
- Check build logs to verify environment variables are set

### Build Fails
- Ensure Root Directory is set to `website`
- Check that `package.json` has all required dependencies
- Review build logs in Vercel dashboard
