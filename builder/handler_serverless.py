#!/usr/bin/env python3
"""
RunPod Serverless Handler for Doctown Builder (Ingest + Assembly)

This is the new serverless handler that:
1. Runs the ingest pipeline to extract chunks
2. Calls the Embedder serverless endpoint for embeddings
3. Runs the assembly pipeline
4. Returns the complete result (no SSE streaming)

Replaces the old SSE-based handler.py for serverless architecture.
"""

import asyncio
import json
import os
import subprocess
import sys
import time
import signal
import logging
from typing import Any, Dict, List, Optional
from dataclasses import dataclass

import runpod
import requests

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger(__name__)

# Configuration from environment
RUNPOD_API_KEY = os.environ.get("RUNPOD_API_KEY", "")
EMBEDDER_ENDPOINT_ID = os.environ.get("EMBEDDER_ENDPOINT_ID", "")
ASSEMBLY_URL = os.environ.get("ASSEMBLY_URL", "http://localhost:3001")

# Batch configuration
EMBEDDING_BATCH_SIZE = int(os.environ.get("EMBEDDING_BATCH_SIZE", "64"))
EMBEDDING_MAX_CONCURRENT = int(os.environ.get("EMBEDDING_MAX_CONCURRENT", "4"))
EMBEDDING_POLL_INTERVAL = float(os.environ.get("EMBEDDING_POLL_INTERVAL", "0.5"))
EMBEDDING_TIMEOUT = int(os.environ.get("EMBEDDING_TIMEOUT", "300"))  # 5 minutes


@dataclass
class EmbeddingJob:
    """Tracks an embedding job submitted to RunPod."""
    job_id: str
    batch_id: str
    chunk_ids: List[str]
    status: str = "pending"
    result: Optional[Dict] = None
    error: Optional[str] = None


class EmbedderClient:
    """Client for calling the Embedder serverless endpoint."""
    
    def __init__(self, endpoint_id: str, api_key: str):
        self.endpoint_id = endpoint_id
        self.api_key = api_key
        self.base_url = f"https://api.runpod.ai/v2/{endpoint_id}"
        self.headers = {"Authorization": f"Bearer {api_key}"}
    
    def submit_job(self, batch_id: str, chunks: List[Dict]) -> str:
        """Submit an embedding job and return the job ID."""
        response = requests.post(
            f"{self.base_url}/run",
            headers=self.headers,
            json={
                "input": {
                    "batch_id": batch_id,
                    "chunks": chunks
                }
            },
            timeout=30
        )
        response.raise_for_status()
        return response.json()["id"]
    
    def get_status(self, job_id: str) -> Dict:
        """Get the status of a job."""
        response = requests.get(
            f"{self.base_url}/status/{job_id}",
            headers=self.headers,
            timeout=30
        )
        response.raise_for_status()
        return response.json()
    
    def cancel_job(self, job_id: str) -> None:
        """Cancel a running job."""
        try:
            requests.post(
                f"{self.base_url}/cancel/{job_id}",
                headers=self.headers,
                timeout=10
            )
        except Exception as e:
            logger.warning(f"Failed to cancel job {job_id}: {e}")


async def embed_chunks_serverless(
    chunks: List[Dict],
    embedder_client: EmbedderClient,
    job_prefix: str
) -> List[Dict]:
    """
    Embed chunks using the serverless embedder with async polling.
    
    Args:
        chunks: List of {"chunk_id": str, "content": str}
        embedder_client: Client for the embedder endpoint
        job_prefix: Prefix for batch IDs
        
    Returns:
        List of {"chunk_id": str, "vector": List[float]}
    """
    if not chunks:
        return []
    
    logger.info(f"Embedding {len(chunks)} chunks in batches of {EMBEDDING_BATCH_SIZE}")
    
    # Split into batches
    batches = [
        chunks[i:i + EMBEDDING_BATCH_SIZE]
        for i in range(0, len(chunks), EMBEDDING_BATCH_SIZE)
    ]
    
    # Submit all jobs
    jobs: List[EmbeddingJob] = []
    for i, batch in enumerate(batches):
        batch_id = f"{job_prefix}_batch_{i}"
        try:
            job_id = embedder_client.submit_job(batch_id, batch)
            jobs.append(EmbeddingJob(
                job_id=job_id,
                batch_id=batch_id,
                chunk_ids=[c["chunk_id"] for c in batch]
            ))
            logger.info(f"Submitted batch {i+1}/{len(batches)} (job_id={job_id})")
        except Exception as e:
            logger.error(f"Failed to submit batch {i+1}: {e}")
            jobs.append(EmbeddingJob(
                job_id="",
                batch_id=batch_id,
                chunk_ids=[c["chunk_id"] for c in batch],
                status="failed",
                error=str(e)
            ))
    
    # Poll for completion with concurrency limit
    pending_jobs = [j for j in jobs if j.status == "pending"]
    start_time = time.time()
    
    while pending_jobs and (time.time() - start_time) < EMBEDDING_TIMEOUT:
        # Check a limited number of jobs per iteration
        check_jobs = pending_jobs[:EMBEDDING_MAX_CONCURRENT]
        
        for job in check_jobs:
            try:
                status_response = embedder_client.get_status(job.job_id)
                status = status_response.get("status", "UNKNOWN")
                
                if status == "COMPLETED":
                    job.status = "completed"
                    job.result = status_response.get("output", {})
                    logger.info(f"Batch {job.batch_id} completed")
                elif status == "FAILED":
                    job.status = "failed"
                    job.error = status_response.get("error", "Unknown error")
                    logger.error(f"Batch {job.batch_id} failed: {job.error}")
                elif status in ("CANCELLED", "TIMED_OUT"):
                    job.status = "failed"
                    job.error = f"Job {status}"
                    logger.error(f"Batch {job.batch_id}: {status}")
                # else: still IN_QUEUE or IN_PROGRESS
            except Exception as e:
                logger.warning(f"Error checking status for {job.batch_id}: {e}")
        
        # Update pending list
        pending_jobs = [j for j in jobs if j.status == "pending"]
        
        if pending_jobs:
            await asyncio.sleep(EMBEDDING_POLL_INTERVAL)
    
    # Handle timeouts
    for job in pending_jobs:
        job.status = "failed"
        job.error = "Timeout"
        embedder_client.cancel_job(job.job_id)
    
    # Collect results
    all_vectors = []
    failed_count = 0
    
    for job in jobs:
        if job.status == "completed" and job.result:
            vectors = job.result.get("vectors", [])
            all_vectors.extend(vectors)
        else:
            failed_count += 1
    
    logger.info(f"Embedding complete: {len(all_vectors)} vectors, {failed_count} failed batches")
    
    if failed_count > 0:
        logger.warning(f"{failed_count} batches failed during embedding")
    
    return all_vectors


def run_ingest_pipeline(repo_url: str, git_ref: str, job_id: str) -> Dict:
    """
    Run the Rust ingest pipeline and collect all events.
    
    Returns a dict with:
    - chunks: List of chunks created
    - files_processed: Number of files processed
    - files_skipped: Number of files skipped
    - symbols: Symbol metadata for assembly
    """
    logger.info(f"Running ingest for {repo_url} @ {git_ref}")
    
    # Start the Rust server
    env = os.environ.copy()
    env['PRODUCTION'] = 'true'
    env['HOST'] = '127.0.0.1'
    env['SKIP_EMBEDDING'] = 'true'  # Don't embed in Rust, we'll do it separately
    
    server_process = subprocess.Popen(
        ["/app/builder"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env
    )
    
    try:
        # Wait for server to be ready
        max_attempts = 30
        server_ready = False
        
        for _ in range(max_attempts):
            try:
                response = requests.get("http://127.0.0.1:3000/health", timeout=1)
                if response.status_code == 200:
                    server_ready = True
                    break
            except requests.exceptions.RequestException:
                pass
            
            if server_process.poll() is not None:
                stderr = server_process.stderr.read() if server_process.stderr else ""
                raise RuntimeError(f"Server crashed during startup: {stderr}")
            
            time.sleep(0.5)
        
        if not server_ready:
            raise RuntimeError("Server failed to start within timeout")
        
        logger.info("Ingest server ready, calling /ingest endpoint")
        
        # Call the ingest API
        response = requests.get(
            "http://127.0.0.1:3000/ingest",
            params={
                "repo_url": repo_url,
                "git_ref": git_ref,
                "job_id": job_id
            },
            stream=True,
            timeout=600  # 10 minute timeout
        )
        
        if response.status_code != 200:
            raise RuntimeError(f"Ingest API returned {response.status_code}: {response.text}")
        
        # Collect events from SSE stream
        events = []
        chunks = []
        symbols = []
        summary = {}
        
        for line in response.iter_lines():
            if not line:
                continue
            
            line_str = line.decode('utf-8')
            
            if line_str.startswith('data: '):
                try:
                    event = json.loads(line_str[6:])
                    events.append(event)
                    
                    event_type = event.get('event_type', '')
                    payload = event.get('payload', {})
                    
                    # Collect chunks
                    if event_type == 'ingest.chunk_created.v1':
                        chunks.append({
                            "chunk_id": payload.get('chunk_id'),
                            "content": payload.get('content'),
                            "file_path": payload.get('file_path'),
                            "language": payload.get('language'),
                            "symbol_name": payload.get('symbol_name'),
                            "symbol_kind": payload.get('symbol_kind'),
                        })
                    
                    # Collect symbols
                    elif event_type == 'ingest.symbol_extracted.v1':
                        symbols.append({
                            "symbol_id": payload.get('symbol_id'),
                            "name": payload.get('name'),
                            "kind": payload.get('kind'),
                            "file_path": payload.get('file_path'),
                            "signature": payload.get('signature'),
                            "chunk_ids": payload.get('chunk_ids', []),
                            "calls": payload.get('calls', []),
                            "imports": payload.get('imports', []),
                        })
                    
                    # Extract summary
                    elif event_type == 'ingest.completed.v1':
                        summary = {
                            "status": payload.get('status'),
                            "files_detected": payload.get('files_detected', 0),
                            "files_processed": payload.get('files_processed', 0),
                            "files_skipped": payload.get('files_skipped', 0),
                            "chunks_created": payload.get('chunks_created', 0),
                            "duration_ms": payload.get('duration_ms', 0),
                        }
                        
                except json.JSONDecodeError as e:
                    logger.warning(f"Failed to parse event: {e}")
        
        logger.info(f"Ingest complete: {len(chunks)} chunks, {len(symbols)} symbols")
        
        return {
            "chunks": chunks,
            "symbols": symbols,
            "summary": summary,
            "events": events
        }
        
    finally:
        # Clean up server process
        if server_process.poll() is None:
            server_process.terminate()
            try:
                server_process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                server_process.kill()


def run_assembly(
    chunks_with_embeddings: List[Dict],
    symbols: List[Dict],
    job_id: str,
    repo_url: str,
    git_ref: str
) -> Dict:
    """
    Run the assembly pipeline to cluster and build the graph.
    
    For now, calls the Assembly API. In the future, this could be 
    integrated directly into the builder binary.
    """
    logger.info(f"Running assembly with {len(chunks_with_embeddings)} embedded chunks")
    
    # Prepare request
    request = {
        "job_id": job_id,
        "repo_url": repo_url,
        "git_ref": git_ref,
        "chunks": chunks_with_embeddings,
        "symbols": symbols
    }
    
    # Call assembly API
    response = requests.post(
        f"{ASSEMBLY_URL}/assemble",
        json=request,
        timeout=300  # 5 minute timeout
    )
    
    if response.status_code != 200:
        raise RuntimeError(f"Assembly API returned {response.status_code}: {response.text}")
    
    result = response.json()
    logger.info(f"Assembly complete: {result.get('stats', {})}")
    
    return result


def handler(job: Dict[str, Any]) -> Dict[str, Any]:
    """
    RunPod serverless handler for the complete Build pipeline.
    
    Expected input:
    {
        "repo_url": "https://github.com/user/repo",
        "git_ref": "main" (optional),
        "job_id": "job_xxx" (optional, generated if not provided)
    }
    
    Returns:
    {
        "status": "success" | "error",
        "job_id": "...",
        "ingest_summary": {...},
        "assembly_result": {...},
        "message": "..." (if error)
    }
    """
    start_time = time.time()
    
    # Validate input
    if 'input' not in job:
        return {"status": "error", "message": "Missing 'input' field"}
    
    repo_url = job['input'].get('repo_url')
    if not repo_url:
        return {"status": "error", "message": "Missing 'repo_url' in input"}
    
    git_ref = job['input'].get('git_ref', 'main')
    job_id = job['input'].get('job_id') or f"job_{int(time.time() * 1000)}"
    # Ensure job_id starts with 'job_' prefix
    if not job_id.startswith('job_'):
        job_id = f"job_{job_id}"
    
    logger.info(f"Starting build job {job_id} for {repo_url} @ {git_ref}")
    
    try:
        # Step 1: Run ingest pipeline
        ingest_result = run_ingest_pipeline(repo_url, git_ref, job_id)
        
        if not ingest_result['chunks']:
            return {
                "status": "error",
                "job_id": job_id,
                "message": "No chunks created during ingest",
                "ingest_summary": ingest_result['summary']
            }
        
        # Step 2: Embed chunks via serverless embedder
        if EMBEDDER_ENDPOINT_ID and RUNPOD_API_KEY:
            embedder_client = EmbedderClient(EMBEDDER_ENDPOINT_ID, RUNPOD_API_KEY)
            
            # Prepare chunks for embedding
            chunks_for_embedding = [
                {"chunk_id": c["chunk_id"], "content": c["content"]}
                for c in ingest_result['chunks']
            ]
            
            # Run async embedding
            vectors = asyncio.run(
                embed_chunks_serverless(chunks_for_embedding, embedder_client, job_id)
            )
            
            # Merge embeddings with chunks
            vector_map = {v["chunk_id"]: v["vector"] for v in vectors}
            chunks_with_embeddings = [
                {
                    "chunk_id": c["chunk_id"],
                    "content": c["content"],
                    "vector": vector_map.get(c["chunk_id"], [])
                }
                for c in ingest_result['chunks']
                if c["chunk_id"] in vector_map
            ]
        else:
            logger.warning("No embedder configured, skipping embedding step")
            chunks_with_embeddings = ingest_result['chunks']
        
        # Step 3: Run assembly (optional, can be skipped)
        assembly_result = None
        if os.environ.get("RUN_ASSEMBLY", "false").lower() in ("true", "1", "yes"):
            try:
                assembly_result = run_assembly(
                    chunks_with_embeddings,
                    ingest_result['symbols'],
                    job_id,
                    repo_url,
                    git_ref
                )
            except Exception as e:
                logger.error(f"Assembly failed: {e}")
                assembly_result = {"error": str(e)}
        
        duration_ms = (time.time() - start_time) * 1000
        
        return {
            "status": "success",
            "job_id": job_id,
            "ingest_summary": ingest_result['summary'],
            "chunks_created": len(ingest_result['chunks']),
            "chunks_embedded": len(chunks_with_embeddings),
            "symbols_extracted": len(ingest_result['symbols']),
            "assembly_result": assembly_result,
            "duration_ms": duration_ms
        }
        
    except Exception as e:
        logger.exception(f"Build job failed: {e}")
        return {
            "status": "error",
            "job_id": job_id,
            "message": str(e)
        }


def cleanup_handler(signum, frame):
    """Clean up on shutdown."""
    logger.info("Shutting down...")
    sys.exit(0)


# Register signal handlers
signal.signal(signal.SIGTERM, cleanup_handler)
signal.signal(signal.SIGINT, cleanup_handler)

if __name__ == "__main__":
    logger.info("Starting RunPod serverless handler for Doctown Builder")
    runpod.serverless.start({"handler": handler})
