# Combined Container Update - Generation Worker Integration

## Overview

The **Generation Worker** has been successfully integrated into the combined Docker container (`Dockerfile.combined`). The container now runs all four core services:

1. **Builder API** (Rust) - Port 3000
2. **Assembly Worker** (Rust) - Port 8002
3. **Embedding Worker** (Python) - Port 8000
4. **Generation Worker** (Python) - Port 8003 ⭐ NEW

## What Changed

### Dockerfile.combined

**Added:**
- Assembly server binary (`/app/assembly-server`)
- Generation worker Python code (`/app/generation/`)
- OpenAI dependencies (openai, tiktoken, tenacity)
- Port 8002 and 8003 exposed
- Updated health checks for all 4 services

**Build Process:**
```bash
# Now builds both Rust binaries
cargo build --release --bin builder --bin assembly-server

# Installs dependencies for both Python workers
pip install ... # embedding deps
pip install ... # generation deps
```

### start-combined.sh

**Updated to start all 4 services:**
```bash
1. Builder API (port 3000)
2. Assembly Worker (port 8002)  
3. Embedding Worker (port 8000)
4. Generation Worker (port 8003)
```

**Features:**
- Warns if `OPENAI_API_KEY` is not set
- Proper signal handling for graceful shutdown
- Sequential startup with delays to ensure readiness

### test-combined.sh

**Updated to test all 4 services:**
- Health check for Builder
- Health check for Embedding Worker
- Health check for Assembly Worker
- Health check for Generation Worker (with warning if no API key)

### build-combined.sh

**Updated port mapping instructions:**
```bash
docker run -p 3000:3000 -p 8000:8000 -p 8002:8002 -p 8003:8003 \
  -e OPENAI_API_KEY=your-key \
  xandwrp/doctown-combined:latest
```

## Using the Combined Container

### Building

```bash
./build-combined.sh
# or with custom tag
./build-combined.sh v1.0.0
```

### Testing Locally

```bash
export OPENAI_API_KEY=your-key-here
./test-combined.sh
```

This will:
1. Start the container with all 4 services
2. Wait for startup
3. Test health endpoints
4. Display service URLs

### Running Manually

```bash
docker run -d \
  --name doctown-combined \
  -p 3000:3000 \
  -p 8000:8000 \
  -p 8002:8002 \
  -p 8003:8003 \
  -e OPENAI_API_KEY=your-openai-key \
  xandwrp/doctown-combined:latest
```

### Health Checks

The container health check now tests all 4 services:
```bash
curl http://localhost:3000/health  # Builder
curl http://localhost:8000/health  # Embedding
curl http://localhost:8002/health  # Assembly
curl http://localhost:8003/health  # Generation
```

## RunPod Deployment

### Environment Variables Required

Add to your RunPod pod environment:

```
OPENAI_API_KEY=your-openai-api-key-here
```

Optional variables (with defaults):
```
MODEL_NAME=gpt-5-nano
MAX_CONCURRENT_REQUESTS=10
INPUT_TOKEN_PRICE=0.15
OUTPUT_TOKEN_PRICE=0.60
```

### Port Mapping

When deploying to RunPod, expose these ports:

| Service | Internal Port | Description |
|---------|--------------|-------------|
| Builder | 3000 | Main API endpoint |
| Embedding | 8000 | Embedding generation |
| Assembly | 8002 | Graph assembly & clustering |
| Generation | 8003 | Documentation generation |

### Updated RunPod Template

Use this image:
```
xandwrp/doctown-combined:latest
```

**Environment Variables to Set:**
- `OPENAI_API_KEY` - **Required** for generation worker

**Ports to Expose:**
- 3000 (HTTP)
- 8000 (HTTP)
- 8002 (HTTP)
- 8003 (HTTP)

## Service Architecture

```
┌─────────────────────────────────────────────────────────┐
│                  Combined Container                      │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌──────────────┐  ┌──────────────┐                    │
│  │   Builder    │  │  Assembly    │                    │
│  │   (Rust)     │  │   (Rust)     │                    │
│  │  Port 3000   │  │  Port 8002   │                    │
│  └──────────────┘  └──────────────┘                    │
│                                                          │
│  ┌──────────────┐  ┌──────────────┐                    │
│  │  Embedding   │  │  Generation  │                    │
│  │  (Python)    │  │   (Python)   │                    │
│  │  Port 8000   │  │  Port 8003   │                    │
│  └──────────────┘  └──────────────┘                    │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## Complete Pipeline Flow

1. **Builder** (3000) - Orchestrates the entire pipeline
2. **Embedding** (8000) - Generates embeddings for code chunks
3. **Assembly** (8002) - Creates graph, clusters, and symbol contexts
4. **Generation** (8003) - Generates documentation using OpenAI

The Builder coordinates all workers to create the final `.docpack` file.

## Verification

After deployment, verify all services:

```bash
# Get your RunPod endpoint
RUNPOD_URL="https://your-pod-id.runpod.net"

# Check all health endpoints
curl ${RUNPOD_URL}:3000/health
curl ${RUNPOD_URL}:8000/health
curl ${RUNPOD_URL}:8002/health
curl ${RUNPOD_URL}:8003/health
```

All should return healthy status.

## Troubleshooting

### Generation Worker Shows Unhealthy

**Cause:** `OPENAI_API_KEY` not set

**Solution:** Add the environment variable in RunPod:
1. Go to your pod settings
2. Add environment variable: `OPENAI_API_KEY=sk-...`
3. Restart the pod

### Services Not Starting

**Check logs:**
```bash
docker logs doctown-combined

# Or in RunPod
# Use the RunPod web interface to view logs
```

### Port Already in Use (Local Testing)

Stop existing container:
```bash
docker stop doctown-combined-test
docker rm doctown-combined-test
```

## Cost Considerations

**Generation Worker Costs:**
- Input: $0.15 per 1M tokens
- Output: $0.60 per 1M tokens
- Typical cost: ~$0.0001-0.0003 per symbol

For a 200-symbol codebase:
- Estimated cost: ~$0.02-0.06 per documentation run

Monitor token usage via the generation events or response totals.

## Next Steps

1. **Build & Test Locally:**
   ```bash
   ./build-combined.sh
   export OPENAI_API_KEY=your-key
   ./test-combined.sh
   ```

2. **Push to Registry:**
   ```bash
   docker push xandwrp/doctown-combined:latest
   ```

3. **Update RunPod:**
   - Add `OPENAI_API_KEY` environment variable
   - Redeploy with new image
   - Verify all 4 health endpoints

4. **Test Full Pipeline:**
   - Submit a repository for documentation
   - Verify all workers are called
   - Check for generated documentation in output

## Files Modified

- `Dockerfile.combined` - Added assembly + generation services
- `start-combined.sh` - Updated to start all 4 services
- `test-combined.sh` - Added health checks for new services
- `build-combined.sh` - Updated port documentation

## Summary

✅ Generation worker fully integrated  
✅ All 4 services in one container  
✅ Health checks for all services  
✅ Ready for RunPod deployment  
✅ Requires only `OPENAI_API_KEY` env var  

The combined container is now a complete Doctown pipeline in a single deployable unit!
