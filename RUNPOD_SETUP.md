# RunPod Setup - Quick Reference

## TL;DR

**Yes, the embedding worker can run on your existing RunPod setup!** 

You have two options:

### Option 1: Combined Container (Recommended for M2)
Run both Builder + Embedding Worker in one container on a **persistent CPU Pod**.

```bash
# Build combined image
./build-combined.sh

# Test locally
./test-combined.sh

# Push to Docker Hub
docker push xandwrp/doctown-combined:latest

# Deploy to RunPod CPU Pod (see below)
```

### Option 2: Keep Them Separate
- Keep Builder as serverless (current setup)
- Deploy Embedding Worker to a persistent CPU Pod

---

## Current Setup

Your builder is deployed as **RunPod Serverless**:
- ✅ Spins up on demand
- ✅ Scales to zero when not in use
- ✅ Good for ingest operations (1-2 second jobs)

---

## Embedding Worker Needs

According to your TODO, the embedding worker should run on a **persistent Pod**:
- ✅ Always warm (no cold starts)
- ✅ Model loads once on startup
- ✅ Better for frequent embedding requests
- ✅ ONNX model loading time (~1-2s) doesn't matter

---

## Deployment: Combined Container

### 1. Build & Test Locally

```bash
# Build the combined image
./build-combined.sh

# Test it
./test-combined.sh

# Should see:
# ✅ Builder is healthy
# ✅ Embedding Worker is healthy
```

### 2. Push to Docker Hub

```bash
docker login
docker push xandwrp/doctown-combined:latest
```

### 3. Create RunPod CPU Pod

1. Go to: https://www.runpod.io/console/pods
2. Click "Deploy" (not "Serverless" - we want a persistent Pod)
3. Select **CPU** option (cheaper, no GPU needed)
4. Configuration:
   ```
   Container Image: xandwrp/doctown-combined:latest
   Container Disk: 10 GB
   Expose HTTP Ports: 3000,8000
   Volume: Not needed for now
   ```
5. Choose CPU size:
   - **4 vCPU, 8 GB RAM** is plenty (~$0.30/hour)
   - Can start with 2 vCPU, 4 GB RAM (~$0.20/hour)
6. Click "Deploy"

### 4. Get Your URLs

After deployment, RunPod gives you:
```
Builder:   https://{pod-id}-3000.proxy.runpod.net
Embedding: https://{pod-id}-8000.proxy.runpod.net
```

### 5. Test It

```bash
# Test builder
curl https://{pod-id}-3000.proxy.runpod.net/health

# Test embedding worker
curl https://{pod-id}-8000.proxy.runpod.net/health

# Should return:
# {"status":"healthy","model_loaded":true,"embedding_dim":384}
```

### 6. Update Frontend

In your frontend config, set:
```javascript
const BUILDER_URL = "https://{pod-id}-3000.proxy.runpod.net";
const EMBEDDING_URL = "https://{pod-id}-8000.proxy.runpod.net";
```

---

## Cost Estimate

### Development (Combined CPU Pod)
- **4 vCPU, 8 GB RAM**: ~$0.30/hour = ~$216/month
- **2 vCPU, 4 GB RAM**: ~$0.20/hour = ~$144/month

### Production Options
Later, you can optimize:
- Builder: Keep as serverless (pay per use)
- Embedding: Persistent Pod (always warm)
- Add caching layer to reduce embedding calls

---

## Troubleshooting

### Container Won't Start
Check logs in RunPod console:
```
Logs tab → Look for:
- "Starting Builder API on port 3000..."
- "Starting Embedding Worker on port 8000..."
- "Model loaded successfully"
```

### Health Check Fails
Common issues:
1. Model files not copied properly → Check Dockerfile.combined
2. Python dependencies missing → Rebuild image
3. Ports not exposed → Check RunPod port config

### One Service Works, Other Doesn't
SSH into Pod and check:
```bash
curl localhost:3000/health  # Builder
curl localhost:8000/health  # Embedding
```

---

## Alternative: Separate Containers

If you prefer to keep them separate:

### Builder (Serverless - Current Setup)
Keep as-is using `xandwrp/doctown-builder:latest`

### Embedding Worker (CPU Pod)
```bash
cd workers/embedding
docker build -t xandwrp/doctown-embedding:latest .
docker push xandwrp/doctown-embedding:latest
```

Deploy to separate CPU Pod:
- Container: `xandwrp/doctown-embedding:latest`
- Port: 8000 only
- CPU: 2-4 vCPU

---

## Next Steps After Deployment

1. ✅ Deploy combined container to CPU Pod
2. ✅ Get Pod URLs from RunPod
3. ✅ Test both services
4. → Update frontend to use new URLs
5. → Test end-to-end flow (ingest → embed)
6. → Monitor logs and performance
7. → Move to M2.3 (Semantic Assembly)

---

## Questions?

- **"Can I use the same Pod for both?"** - Yes! That's the combined container approach.
- **"Should I use GPU?"** - No, CPU is fine for ONNX embeddings and much cheaper.
- **"What about the old builder?"** - You can replace it or keep both (serverless for builder, Pod for embedding).
- **"How do I scale?"** - Start with one Pod. If needed, add more Pods + load balancer later.

---

## Summary

✅ **Recommended for M2**: Use combined container on CPU Pod
✅ **Cost**: ~$0.20-0.30/hour
✅ **Ports**: 3000 (builder), 8000 (embedding)
✅ **Scripts**: `./build-combined.sh` and `./test-combined.sh`
✅ **Ready to go!**
