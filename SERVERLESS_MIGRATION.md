# Serverless Migration Plan

## Overview

Migrate from persistent CPU pod to serverless RunPod endpoints:

1. **Builder Serverless** - CPU endpoint for ingest + assembly
2. **Embedder Serverless** - GPU endpoint for ONNX embeddings
3. **Async Job Polling** - Replace SSE with job-based polling

## Architecture Changes

### Before (Persistent Pod)
```
Website → SSE → Persistent Builder Pod → Calls Embedding Pod → Returns via SSE
```

### After (Serverless)
```
Website → Submit Job → Builder Serverless
                           ↓
                      Calls Embedder Serverless (async, polls for completion)
                           ↓
                      Runs Assembly
                           ↓
Website ← Poll Status ← Returns Complete Result
```

---

## Phase 1: Builder Serverless Handler

### Changes to `builder/handler.py`

```python
# NEW: Async job submission to embedder
async def call_embedder_serverless(batch_id: str, chunks: List[Dict]) -> Dict:
    """Submit embedding job to RunPod serverless and poll for completion."""
    
    # Submit job
    response = requests.post(
        f"https://api.runpod.ai/v2/{EMBEDDER_ENDPOINT_ID}/run",
        headers={"Authorization": f"Bearer {RUNPOD_API_KEY}"},
        json={
            "input": {
                "batch_id": batch_id,
                "chunks": chunks
            }
        }
    )
    job_id = response.json()["id"]
    
    # Poll for completion
    while True:
        status = requests.get(
            f"https://api.runpod.ai/v2/{EMBEDDER_ENDPOINT_ID}/status/{job_id}",
            headers={"Authorization": f"Bearer {RUNPOD_API_KEY}"}
        )
        result = status.json()
        
        if result["status"] == "COMPLETED":
            return result["output"]
        elif result["status"] == "FAILED":
            raise Exception(f"Embedding job failed: {result.get('error')}")
        
        await asyncio.sleep(0.5)  # Poll every 500ms
```

### New Handler Flow

1. Receive job with `repo_url`, `git_ref`
2. Run ingest pipeline (Rust binary) - produces chunks
3. Submit chunks to Embedder Serverless in batches
4. Poll for each batch completion
5. Run assembly (clustering, graph building)
6. Return complete result (no SSE)

---

## Phase 2: Embedder Serverless Handler

### New File: `workers/embedding/runpod_handler.py`

```python
import runpod
from app.model import get_model

def handler(job):
    """RunPod serverless handler for GPU embedding."""
    input_data = job["input"]
    batch_id = input_data["batch_id"]
    chunks = input_data["chunks"]
    
    # Load model (cached after first load)
    model = get_model()
    
    # Extract texts
    texts = [c["content"] for c in chunks]
    
    # Embed on GPU
    embeddings = model.embed(texts)
    
    # Return vectors
    return {
        "batch_id": batch_id,
        "vectors": [
            {"chunk_id": c["chunk_id"], "vector": embeddings[i].tolist()}
            for i, c in enumerate(chunks)
        ]
    }

runpod.serverless.start({"handler": handler})
```

### Dockerfile Changes

```dockerfile
# Use GPU base image
FROM runpod/pytorch:2.2.0-py3.10-cuda12.1.0-devel

# Install ONNX Runtime GPU
RUN pip install onnxruntime-gpu

# ... rest of setup
```

---

## Phase 3: Pipeline Refactor

### Remove SSE from Rust Pipeline

The Rust ingest pipeline currently streams events via SSE. For serverless:

1. **Option A**: Keep SSE internally, collect all events in handler
   - Minimal Rust changes
   - Handler collects events, returns final result

2. **Option B**: Refactor to return result directly
   - More Rust changes
   - Cleaner architecture

**Recommendation**: Start with Option A (current approach), then refactor if needed.

### Builder Handler Changes

```python
def handler(job):
    """New serverless handler without SSE."""
    repo_url = job["input"]["repo_url"]
    git_ref = job["input"].get("git_ref", "main")
    job_id = job["input"].get("job_id", str(uuid.uuid4()))
    
    # 1. Run ingest (produces chunks)
    chunks = run_ingest(repo_url, git_ref, job_id)
    
    # 2. Embed chunks via serverless embedder
    embeddings = await embed_chunks_serverless(chunks)
    
    # 3. Run assembly
    result = run_assembly(chunks, embeddings)
    
    # 4. Return complete result
    return {
        "status": "success",
        "job_id": job_id,
        "chunks_created": len(chunks),
        "embeddings_created": len(embeddings),
        "clusters": result["clusters"],
        "graph": result["graph"],
        "symbol_contexts": result["symbol_contexts"]
    }
```

---

## File Changes Required

### Builder Changes

| File | Change |
|------|--------|
| `builder/handler.py` | Add async embedder calls, remove SSE streaming |
| `builder/Dockerfile` | Add async dependencies |
| `builder/src/main.rs` | Add `--no-embed` flag for serverless mode |

### Embedder Changes

| File | Change |
|------|--------|
| `workers/embedding/runpod_handler.py` | NEW: RunPod serverless handler |
| `workers/embedding/Dockerfile.gpu` | NEW: GPU-enabled Dockerfile |
| `workers/embedding/app/model.py` | Add GPU support via onnxruntime-gpu |

### Pipeline Changes

| File | Change |
|------|--------|
| `crates/doctown-ingest/src/pipeline.rs` | Add mode for returning chunks without embedding |
| `crates/doctown-ingest/src/api.rs` | Add `/ingest-chunks` endpoint (returns JSON, no SSE) |

---

## Environment Variables

### Builder Serverless
```
RUNPOD_API_KEY=rp_xxx
EMBEDDER_ENDPOINT_ID=xxx
PRODUCTION=true
```

### Embedder Serverless
```
ONNX_USE_GPU=true
MODEL_PATH=/app/models/minilm-l6
```

---

## Deployment Order

1. Deploy Embedder Serverless first (new GPU endpoint)
2. Test Embedder independently
3. Update Builder to call Embedder Serverless
4. Deploy Builder Serverless
5. Update Website to use polling instead of SSE
6. Retire persistent pod

---

## Cost Comparison

### Persistent Pod (Current)
- 24/7 CPU pod: ~$50-100/month
- Always-on embedding worker: ~$30-50/month
- **Total**: ~$80-150/month fixed

### Serverless (New)
- Builder: ~$0.0001/second (only runs during jobs)
- Embedder GPU: ~$0.0003/second (only runs during embedding)
- **Total**: ~$5-20/month (usage-based)

**Estimated savings**: 70-90% for typical usage patterns

---

## Next Steps

1. [x] Create `workers/embedding/runpod_handler.py`
2. [x] Create `workers/embedding/Dockerfile.gpu`
3. [x] Update `builder/handler_serverless.py` with async embedder calls
4. [x] Add `SKIP_EMBEDDING` env var to Rust ingest
5. [ ] Test locally with mock embedder
6. [ ] Deploy embedder serverless
7. [ ] Deploy builder serverless
8. [ ] Update website polling logic

---

## Deployment Guide

### Step 1: Deploy Embedder Serverless (GPU)

```bash
# Build the GPU image
cd workers/embedding
./build-gpu.sh

# Push to Docker Hub
docker push xandwrp/doctown-embedder-gpu:latest

# In RunPod Console:
# 1. Go to Serverless → Endpoints
# 2. Create New Endpoint
# 3. Select "GPU" worker type
# 4. Image: xandwrp/doctown-embedder-gpu:latest
# 5. GPU: RTX 3090 or RTX 4090 (recommended for speed)
# 6. Max Workers: 3-5
# 7. Save and note the Endpoint ID
```

### Step 2: Deploy Builder Serverless (CPU)

```bash
# Build the serverless image
cd builder
./build-serverless.sh

# Push to Docker Hub
docker push xandwrp/doctown-builder-serverless:latest

# In RunPod Console:
# 1. Go to Serverless → Endpoints
# 2. Create New Endpoint
# 3. Select "CPU" worker type
# 4. Image: xandwrp/doctown-builder-serverless:latest
# 5. Environment Variables:
#    - RUNPOD_API_KEY=rp_xxx
#    - EMBEDDER_ENDPOINT_ID=xxx (from Step 1)
# 6. Max Workers: 3-5
# 7. Save and note the Endpoint ID
```

### Step 3: Test the Pipeline

```bash
# Test embedder directly
curl -X POST "https://api.runpod.ai/v2/{EMBEDDER_ENDPOINT_ID}/runsync" \
  -H "Authorization: Bearer $RUNPOD_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "input": {
      "batch_id": "test",
      "chunks": [{"chunk_id": "c1", "content": "Hello world"}]
    }
  }'

# Test builder
curl -X POST "https://api.runpod.ai/v2/{BUILDER_ENDPOINT_ID}/run" \
  -H "Authorization: Bearer $RUNPOD_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "input": {
      "repo_url": "https://github.com/xandwr/localdoc",
      "git_ref": "main"
    }
  }'
```

### Step 4: Update Website

Update the website to use async job polling instead of SSE:

```typescript
// Old (SSE streaming)
const eventSource = new EventSource(`/api/ingest?repo=${repo}`);

// New (async polling)
async function runJob(repo: string): Promise<JobResult> {
  // Submit job
  const submitRes = await fetch('/api/jobs', {
    method: 'POST',
    body: JSON.stringify({ repo_url: repo })
  });
  const { job_id } = await submitRes.json();
  
  // Poll for completion
  while (true) {
    const statusRes = await fetch(`/api/jobs/${job_id}/status`);
    const status = await statusRes.json();
    
    if (status.status === 'COMPLETED') {
      return status.output;
    } else if (status.status === 'FAILED') {
      throw new Error(status.error);
    }
    
    await sleep(1000); // Poll every second
  }
}
```
