# RunPod Deployment Guide - Updated with Generation Worker

## Quick Setup

### 1. Environment Variables

Add to your RunPod pod:

```
OPENAI_API_KEY=sk-your-openai-api-key-here
```

### 2. Docker Image

```
xandwrp/doctown-combined:latest
```

### 3. Port Mapping

Expose these ports:
- **3000** - Builder API (main endpoint)
- **8000** - Embedding Worker
- **8002** - Assembly Worker
- **8003** - Generation Worker

### 4. Container Settings

- **GPU:** Not required (CPU-only)
- **Disk:** 10GB minimum
- **Volume:** None required
- **Command:** Default (uses `start-combined.sh`)

## Complete RunPod Configuration

### Template Settings

```yaml
Docker Image: xandwrp/doctown-combined:latest
Expose HTTP Ports: 3000,8000,8002,8003
Container Disk: 10 GB
GPU: None (CPU Pod)

Environment Variables:
  OPENAI_API_KEY: sk-your-key-here
  MODEL_NAME: gpt-5-nano (optional)
  MAX_CONCURRENT_REQUESTS: 10 (optional)
```

## Service Endpoints

Once deployed, your RunPod pod will have:

```
https://your-pod-id.runpod.net:3000  → Builder API
https://your-pod-id.runpod.net:8000  → Embedding Worker
https://your-pod-id.runpod.net:8002  → Assembly Worker
https://your-pod-id.runpod.net:8003  → Generation Worker
```

## Health Checks

Verify all services are running:

```bash
RUNPOD_URL="https://your-pod-id.runpod.net"

curl ${RUNPOD_URL}:3000/health
curl ${RUNPOD_URL}:8000/health
curl ${RUNPOD_URL}:8002/health
curl ${RUNPOD_URL}:8003/health
```

Expected responses:
```json
// Builder
{"status": "healthy"}

// Embedding
{"status": "healthy", "model_loaded": true, "embedding_dim": 384}

// Assembly
{"status": "healthy", "version": "...", "service": "assembly"}

// Generation
{"status": "healthy", "model": "gpt-5-nano", "ready": true}
```

## Website Configuration

Update your SvelteKit website environment variables:

```env
# In Vercel or your deployment platform
BUILDER_URL=https://your-pod-id.runpod.net:3000
EMBEDDING_URL=https://your-pod-id.runpod.net:8000
ASSEMBLY_URL=https://your-pod-id.runpod.net:8002
GENERATION_URL=https://your-pod-id.runpod.net:8003
```

## Build & Deploy Process

### Local Build

```bash
# Build the image
./build-combined.sh

# Test locally (requires OPENAI_API_KEY)
export OPENAI_API_KEY=your-key
./test-combined.sh
```

### Push to Registry

```bash
# Login to Docker Hub
docker login

# Push
docker push xandwrp/doctown-combined:latest
```

### Deploy to RunPod

1. Create new pod from template
2. Add `OPENAI_API_KEY` to environment
3. Start pod
4. Verify health checks
5. Update website URLs

## Monitoring

### Check Logs

In RunPod web interface:
- Go to your pod
- Click "Logs" tab
- Watch for startup messages from all 4 services

Expected logs:
```
Starting Doctown Combined Services...
Starting Builder API on port 3000...
Builder PID: 123
Starting Assembly Worker on port 8002...
Assembly Worker PID: 456
Starting Embedding Worker on port 8000...
Embedding Worker PID: 789
Starting Generation Worker on port 8003...
Generation Worker PID: 012
✓ All services started
```

### Generation Worker Costs

Monitor token usage in logs:
```
Generated docs for sym_xyz: 150 input + 45 output = 195 total tokens
```

Estimated costs:
- 50 symbols: ~$0.01
- 200 symbols: ~$0.04
- 1000 symbols: ~$0.20

## Troubleshooting

### Generation Worker Not Ready

**Symptom:** `/health` returns `"ready": false`

**Solution:** Check OPENAI_API_KEY
```bash
# In RunPod, verify environment variable is set
echo $OPENAI_API_KEY
```

### Port Not Accessible

**Symptom:** Cannot reach service endpoint

**Solution:** Verify port is exposed in RunPod template:
- Edit template
- Ensure port is in "Expose HTTP Ports" list
- Restart pod

### Out of Memory

**Symptom:** Services crash with OOM errors

**Solution:** Increase pod size:
- Minimum: 4GB RAM
- Recommended: 8GB RAM for large codebases

### OpenAI Rate Limits

**Symptom:** 429 errors in logs

**Solution:** The generation worker has automatic retry with exponential backoff. If persistent:
- Reduce `MAX_CONCURRENT_REQUESTS` to 5 or lower
- Upgrade OpenAI API tier
- Contact OpenAI support

## Cost Estimation

### RunPod Costs
- CPU Pod: ~$0.20-0.40/hour
- Recommended: Shared CPU pod

### OpenAI Costs
- Per documentation job: $0.01-0.20 (depending on codebase size)
- 1000 symbols: ~$0.15-0.30

### Total Monthly (Estimate)
- RunPod (24/7): ~$150-300/month
- OpenAI (100 jobs): ~$10-30/month
- **Total: ~$160-330/month**

Can reduce by:
- Stopping pod when not in use
- Using spot instances
- Batching documentation jobs

## Security Notes

### API Key Protection

- Store `OPENAI_API_KEY` as environment variable (not in code)
- Use RunPod's secret management
- Rotate keys periodically
- Monitor usage in OpenAI dashboard

### Network Security

- RunPod pods are internet-accessible
- Add authentication in your website layer
- Use HTTPS only (RunPod provides this)
- Implement rate limiting in website

## Scaling

### Horizontal Scaling

Deploy multiple pods for higher throughput:
- Use load balancer (e.g., Cloudflare)
- Each pod handles independent jobs
- Website distributes requests

### Vertical Scaling

For larger codebases:
- Increase pod RAM to 16GB
- Use faster CPU pods
- Adjust `MAX_CONCURRENT_REQUESTS` up to 20

## Rollback

If issues occur:

```bash
# Deploy previous version
docker push xandwrp/doctown-combined:v1.0.0

# Update RunPod template to use :v1.0.0
```

Keep previous working versions tagged!

## Support

Issues with:
- **Container:** Check logs in RunPod
- **Generation:** Verify OPENAI_API_KEY and check OpenAI status
- **Performance:** Increase pod resources
- **Costs:** Monitor OpenAI dashboard

## Next Steps After Deployment

1. ✅ Verify all health checks pass
2. ✅ Test with a small repository
3. ✅ Check OpenAI usage in dashboard
4. ✅ Update website environment variables
5. ✅ Test complete pipeline end-to-end
6. ✅ Monitor costs for first 24 hours
7. ✅ Set up alerts for high usage

---

**Last Updated:** 2025-11-23  
**Version:** Combined Container with Generation Worker  
**Status:** ✅ Ready for Production
