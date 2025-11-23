# Doctown v5 - Deployment Guide

## Overview

Doctown v5 consists of two backend services:
1. **Builder (Rust)**: Ingest API on port 3000
2. **Embedding Worker (Python)**: Embedding API on port 8000

Both can run in the same container for simplified deployment.

## RunPod Deployment Options

### Option A: Combined Container (Easiest)

Run both services in one container on a persistent RunPod CPU Pod.

**Pros:**
- Simple deployment
- One container to manage
- Both services always available
- Shared resources

**Cons:**
- Both services restart together
- Shared memory/CPU

### Option B: Separate Containers

Run builder as serverless (current setup), embedding worker as persistent Pod.

**Pros:**
- Independent scaling
- Builder scales to zero when not in use
- Embedding worker always warm

**Cons:**
- Two endpoints to manage
- Slightly more complex

---

## Quick Start: Combined Container

### 1. Create Combined Dockerfile

See `Dockerfile.combined` - builds both Rust builder and Python embedding worker.

### 2. Build Image

```bash
docker build -f Dockerfile.combined -t xandwrp/doctown-combined:latest .
```

### 3. Test Locally

```bash
docker run -p 3000:3000 -p 8000:8000 xandwrp/doctown-combined:latest
```

Test both services:
```bash
# Builder health
curl http://localhost:3000/health

# Embedding health
curl http://localhost:8000/health
```

### 4. Push to Registry

```bash
docker push xandwrp/doctown-combined:latest
```

### 5. Deploy to RunPod

#### Create a CPU Pod:
1. Go to [RunPod Pods](https://www.runpod.io/console/pods)
2. Click "Deploy"
3. Select "CPU" (not GPU - save money!)
4. Choose template or use custom:
   - Container Image: `xandwrp/doctown-combined:latest`
   - Container Disk: 10 GB
   - Expose HTTP Ports: `3000,8000`
5. Deploy!

#### Get Pod URLs:
After deployment, RunPod will give you URLs like:
- Builder: `https://{pod-id}-3000.proxy.runpod.net`
- Embedding: `https://{pod-id}-8000.proxy.runpod.net`

### 6. Test RunPod Deployment

```bash
# Test builder
curl https://{pod-id}-3000.proxy.runpod.net/health

# Test embedding worker
curl https://{pod-id}-8000.proxy.runpod.net/health
```

---

## Alternative: Keep Current Builder Serverless

If you want to keep the builder as serverless and only add the embedding worker:

### 1. Build Embedding Worker Only

```bash
cd workers/embedding
docker build -t xandwrp/doctown-embedding:latest .
docker push xandwrp/doctown-embedding:latest
```

### 2. Deploy to RunPod CPU Pod

Same as above but use `xandwrp/doctown-embedding:latest` and only expose port 8000.

### 3. Update Frontend

Point frontend to:
- Builder: Your existing RunPod serverless endpoint
- Embedding: New CPU Pod URL

---

## Environment Variables

### Builder
- `HOST`: Server host (default: 0.0.0.0)
- `PORT`: Server port (default: 3000)
- `PRODUCTION`: Production mode flag

### Embedding Worker
- `EMBEDDING_HOST`: Server host (default: 0.0.0.0)
- `EMBEDDING_PORT`: Server port (default: 8000)
- `EMBEDDING_MODEL_PATH`: Model path (default: /app/models/minilm-l6)
- `EMBEDDING_ONNX_THREADS`: Thread count (default: 4)

---

## Cost Comparison

### RunPod CPU Pod (Persistent)
- ~$0.20-0.40/hour for 4-8 vCPU
- Runs 24/7: ~$144-288/month
- Good for: Development, steady traffic

### RunPod Serverless (Pay-per-use)
- ~$0.30/GPU-minute or $0.03/CPU-minute
- Scales to zero
- Good for: Spiky traffic, production with caching

**Recommendation for M2:**
- Use persistent CPU Pod during development
- Switch to serverless + edge caching for production

---

## Next Steps

1. Choose deployment option (combined or separate)
2. Build and test locally
3. Push to Docker Hub
4. Deploy to RunPod
5. Update frontend URLs
6. Test end-to-end flow

---

## Monitoring

Both services log to stdout. View logs in RunPod console:
1. Click on your Pod/Endpoint
2. Go to "Logs" tab
3. Watch for:
   - Builder: "Ingest API server listening on..."
   - Embedding: "Model loaded successfully"
