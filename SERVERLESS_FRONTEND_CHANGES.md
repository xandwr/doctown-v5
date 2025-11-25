# Frontend Serverless Migration Summary

## Changes Made

The frontend has been refactored to work with the new serverless architecture.

### Before (3-Service Architecture)
```
Frontend → Ingest API (SSE streaming)
         → Embedding API (batch processing)
         → Assembly API (graph building)
         → R2 Upload
```

### After (Serverless Architecture)
```
Frontend → Builder Serverless (RunPod)
              ├─ Submits job via /run endpoint
              ├─ Polls /status/{job_id} for progress
              └─ Receives complete result
         → R2 Upload
```

## Environment Variables Required

### For Vercel (Production)

**Server-side only (NO VITE_ prefix - keeps keys secure):**
- `BUILDER_API_URL` - Builder serverless endpoint URL
  - Format: `https://api.runpod.ai/v2/{ENDPOINT_ID}/run`
  - Get from: https://www.runpod.io/console/serverless
- `RUNPOD_API_KEY` - RunPod API key for authentication
  - Get from: https://www.runpod.io/console/user/settings

**Server-side storage variables:**
- `BUCKET_NAME` - R2 bucket name
- `BUCKET_ACCESS_KEY_ID` - R2 access key
- `BUCKET_SECRET_ACCESS_KEY` - R2 secret key
- `BUCKET_S3_ENDPOINT` - R2 endpoint URL
- `BUCKET_PUBLIC_URL` - Public URL for R2 bucket

### For CPU RunPod (Builder Serverless)

- `RUNPOD_API_KEY` - To call the Embedder serverless
- `EMBEDDER_ENDPOINT_ID` - GPU embedder endpoint ID
- `ASSEMBLY_URL` - *(Optional)* If assembly runs separately

### For GPU RunPod (Embedder Serverless)

- `MODEL_PATH` - Path to ONNX model (default: `/app/models/minilm-l6`)
- `ONNX_USE_GPU` - Set to `true` for GPU acceleration

## Key Code Changes

### 1. Removed Multi-Stage Pipeline
**Old:** Three separate functions calling three different services
```typescript
await runIngestStage(apiUrl, repoUrl, jobId);
await runEmbeddingStage(embeddingUrl, jobId);
await runAssemblyStage(assemblyUrl, repoUrl, jobId);
```

**New:** Single function with polling
```typescript
await runBuilderServerless(builderUrl, apiKey, repoUrl, jobId);
```

### 2. Job Submission & Polling via Server-Side API
The new `runBuilderServerless` function:
- Calls `/api/submit-build` (server-side route that securely stores API keys)
- Server submits job to RunPod's `/run` endpoint
- Frontend polls `/api/build-status/{job_id}` every 2 seconds
- Server proxies status requests to RunPod
- Updates pipeline stage based on elapsed time
- Handles COMPLETED, FAILED, and CANCELLED states
- Has 10-minute timeout

**Security:** API keys never leave the server - frontend only calls internal API routes.

### 3. No More SSE Streaming
- Real-time progress updates are now estimated based on elapsed time
- Actual events are synthesized from the final result
- Consider adding custom progress indicators if needed

## Testing

1. **Local Development:**
   - Set `.env` file in `website/` directory
   - Use local URLs if testing locally, or RunPod URLs

2. **Vercel Deployment:**
   - Set environment variables in Vercel dashboard
   - Push to main branch to trigger deployment
   - Check logs for any issues

## Migration Checklist

- [x] Refactored frontend to use single Builder endpoint
- [x] Replaced SSE streaming with polling
- [x] Updated environment variable configuration
- [x] Updated `.env.example` file
- [x] Updated `VERCEL_SETUP.md` documentation
- [ ] Test with actual RunPod serverless endpoints
- [ ] Verify R2 upload still works
- [ ] Monitor performance and adjust polling interval if needed

## Known Limitations

1. **No Real-Time Progress:** 
   - Progress stages are estimated based on time, not actual pipeline state
   - Consider implementing progress webhooks if real-time updates are critical

2. **Polling Overhead:**
   - Polls every 2 seconds (could be optimized)
   - 10-minute timeout might be too short for large repos

3. **Error Details:**
   - Limited error context from RunPod status API
   - Check RunPod logs for detailed debugging

## Future Improvements

- [ ] Add progress webhooks for real-time updates
- [ ] Implement exponential backoff for polling
- [ ] Add job cancellation support
- [ ] Display detailed progress from Builder logs
- [ ] Add retry logic for transient failures
