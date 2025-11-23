# Doctown Ingest Worker - Deployment Guide

This directory contains the Docker image and RunPod handler for deploying the Doctown ingest worker as a serverless endpoint.

## Overview

The ingest worker is packaged as a Docker container that runs on RunPod serverless. It consists of:
- **Rust API Server** (`builder` binary): Handles repository ingestion with SSE streaming
- **Python Handler** (`handler.py`): RunPod wrapper that interfaces with the Rust server

## Quick Start

### 1. Build the Docker Image

```bash
./build.sh
```

This builds the image as `xandwrp/doctown-builder:latest`.

### 2. Test Locally

```bash
./test-local.sh
```

This:
- Starts the container locally
- Waits for health check
- Tests the `/ingest` endpoint with a sample repo
- Leaves the container running for inspection

To test with a specific repo:
```bash
./test-local.sh latest https://github.com/your/repo
```

### 3. Push to Docker Hub

First, ensure you're logged in:
```bash
docker login
```

Then push:
```bash
./deploy.sh
```

### 4. Deploy to RunPod

1. Go to [RunPod Serverless](https://www.runpod.io/console/serverless)
2. Find your endpoint (ID in `.env` as `RUNPOD_BUILDER_ID`)
3. Update the endpoint to use `xandwrp/doctown-builder:latest`
4. Save changes

### 5. Test the RunPod Endpoint

```bash
./test-runpod.sh
```

This submits a test job to your RunPod endpoint and polls for completion.

To test with a specific repo:
```bash
./test-runpod.sh https://github.com/your/repo
```

## Architecture

### Request Flow

1. RunPod receives job request with `repo_url` and optional `git_ref`
2. Handler starts the Rust API server (`builder`)
3. Handler waits for server health check
4. Handler calls the `/ingest` endpoint with SSE streaming
5. Handler collects all events from the stream
6. Handler returns summary and event list

### API Endpoints

The Rust server exposes:

- `GET /health` - Health check endpoint
  ```json
  {"status": "ok"}
  ```

- `GET /ingest?repo_url=...&git_ref=...` - Ingest endpoint (SSE stream)
  - Streams events in Server-Sent Events format
  - Returns events like `ingest.started.v1`, `ingest.file_detected.v1`, etc.

### RunPod Handler Interface

**Input:**
```json
{
  "repo_url": "https://github.com/user/repo",
  "git_ref": "main"
}
```

**Output (Success):**
```json
{
  "status": "success",
  "summary": {
    "status": "success",
    "files_detected": 42,
    "files_processed": 38,
    "files_skipped": 4,
    "chunks_created": 156,
    "duration_ms": 1234,
    "language_breakdown": {
      "Rust": 30,
      "Python": 8
    }
  },
  "events": [...]
}
```

**Output (Error):**
```json
{
  "status": "error",
  "message": "Error description",
  "events": [...]
}
```

## Development

### Building Without Cache

```bash
docker build --no-cache -f Dockerfile -t xandwrp/doctown-builder:latest ..
```

### Viewing Logs

When testing locally:
```bash
# Find container ID
docker ps

# View logs
docker logs -f <container-id>
```

### Debugging

To run the container interactively:
```bash
docker run -it -p 3000:3000 xandwrp/doctown-builder:latest /bin/bash
```

Then manually start components:
```bash
# Start the Rust server
/app/builder

# In another terminal, test health
curl http://localhost:3000/health
```

## Configuration

### Environment Variables

The handler and Rust server don't require environment variables for basic operation, but you can configure:

- `RUST_LOG`: Set log level (e.g., `debug`, `info`)
- `RUST_BACKTRACE`: Enable backtraces on panic

### Port Configuration

- **Container Port**: 3000 (exposed in Dockerfile)
- **Local Testing**: Maps to 3000 on host
- **RunPod**: Managed by RunPod infrastructure

## Troubleshooting

### Server Won't Start

Check the logs:
```bash
docker logs <container-id>
```

Common issues:
- Binary not found: Ensure build completed successfully
- Port already in use: Stop other instances on port 3000
- Missing dependencies: Rebuild with `--no-cache`

### Health Check Fails

The health check requires:
- Server listening on port 3000
- `/health` endpoint responding with 200 OK

Test manually:
```bash
curl http://localhost:3000/health
```

### RunPod Job Times Out

The handler has a 5-minute timeout. For large repos:
- Monitor job status via RunPod console
- Check worker logs in RunPod dashboard
- Consider increasing timeout in `handler.py`

### Events Not Streaming

The SSE stream requires:
- Proper `Accept: text/event-stream` header (handled by handler)
- Connection kept alive until completion
- Events formatted as `data: {...}\n\n`

## Monitoring

### Health Checks

Docker health check runs every 30s:
```bash
docker inspect <container-id> | jq '.[0].State.Health'
```

### Metrics

Track in RunPod dashboard:
- Job completion rate
- Average duration
- Error rate
- Resource usage (CPU, memory)

## Next Steps

After M1.12.1 deployment:
- [ ] M1.12.2: Deploy website to Vercel pointing to RunPod endpoint
- [ ] M1.12.3: Complete ship gates with end-to-end testing

## Resources

- [RunPod Docs](https://docs.runpod.io/)
- [Docker Best Practices](https://docs.docker.com/develop/dev-best-practices/)
- [Server-Sent Events Spec](https://html.spec.whatwg.org/multipage/server-sent-events.html)
