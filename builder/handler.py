#!/usr/bin/env python3
"""
RunPod Serverless Handler for Doctown Ingest Worker

This handler wraps the Rust ingest API server and provides a RunPod-compatible
interface for processing repository ingestion jobs.
"""

import subprocess
import threading
import time
import json
import sys
import signal
import runpod
import requests
from typing import Dict, Any, List

# Global reference to the server process
server_process = None
server_ready = False

def run_rust_server():
    """Run the Rust ingest API server in the background."""
    global server_process, server_ready
    
    print("[Handler] Starting Rust ingest API server...")
    server_process = subprocess.Popen(
        ["/app/builder"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    
    # Wait for server to be ready
    max_attempts = 30
    for attempt in range(max_attempts):
        try:
            response = requests.get("http://localhost:3000/health", timeout=1)
            if response.status_code == 200:
                server_ready = True
                print("[Handler] Rust server ready!")
                return
        except requests.exceptions.RequestException:
            pass
        
        if server_process.poll() is not None:
            # Server crashed
            stderr = server_process.stderr.read()
            print(f"[Handler] Server crashed during startup: {stderr}", file=sys.stderr)
            return
        
        time.sleep(0.5)
    
    print("[Handler] Warning: Server health check timed out", file=sys.stderr)

def handler(job: Dict[str, Any]) -> Dict[str, Any]:
    """
    RunPod job handler for repository ingestion.
    
    Expected input:
    {
        "repo_url": "https://github.com/user/repo",
        "git_ref": "main" (optional)
    }
    
    Returns:
    {
        "status": "success" | "error",
        "events": [...],  # List of all events from the ingest pipeline
        "summary": {...}, # Summary statistics
        "message": "..."  # Error message if failed
    }
    """
    global server_ready
    
    # Validate input
    if 'input' not in job:
        return {"status": "error", "message": "Missing 'input' field"}
    
    repo_url = job['input'].get('repo_url')
    if not repo_url:
        return {"status": "error", "message": "Missing 'repo_url' in input"}
    
    git_ref = job['input'].get('git_ref', 'main')
    
    print(f"[Handler] Processing job for repo: {repo_url}, ref: {git_ref}")
    
    # Start server if not already running
    if not server_ready:
        server_thread = threading.Thread(target=run_rust_server, daemon=True)
        server_thread.start()
        server_thread.join(timeout=15)
        
        if not server_ready:
            return {
                "status": "error",
                "message": "Ingest server failed to start within timeout"
            }
    
    # Call the ingest API with SSE streaming
    try:
        events = []
        summary = {}
        
        print(f"[Handler] Calling ingest API...")
        response = requests.get(
            "http://localhost:3000/ingest",
            params={
                "repo_url": repo_url,
                "git_ref": git_ref
            },
            stream=True,
            timeout=300  # 5 minute timeout
        )
        
        if response.status_code != 200:
            return {
                "status": "error",
                "message": f"Ingest API returned status {response.status_code}: {response.text}"
            }
        
        # Process SSE stream
        for line in response.iter_lines():
            if not line:
                continue
            
            line_str = line.decode('utf-8')
            
            # Parse SSE data lines
            if line_str.startswith('data: '):
                try:
                    event_json = line_str[6:]  # Remove 'data: ' prefix
                    event = json.loads(event_json)
                    events.append(event)
                    
                    # Extract summary from completed event
                    if event.get('event_type') == 'ingest.completed.v1':
                        payload = event.get('payload', {})
                        summary = {
                            "status": payload.get('status'),
                            "files_detected": payload.get('files_detected', 0),
                            "files_processed": payload.get('files_processed', 0),
                            "files_skipped": payload.get('files_skipped', 0),
                            "chunks_created": payload.get('chunks_created', 0),
                            "duration_ms": payload.get('duration_ms', 0),
                            "language_breakdown": payload.get('language_breakdown', {})
                        }
                        
                        print(f"[Handler] Ingest completed: {summary}")
                except json.JSONDecodeError as e:
                    print(f"[Handler] Failed to parse event: {e}", file=sys.stderr)
                    continue
        
        # Check if we got a completion event
        if not summary:
            return {
                "status": "error",
                "message": "Ingest did not complete - no completion event received",
                "events": events
            }
        
        if summary.get('status') == 'failed':
            return {
                "status": "error",
                "message": "Ingest failed",
                "events": events,
                "summary": summary
            }
        
        return {
            "status": "success",
            "events": events,
            "summary": summary
        }
        
    except requests.exceptions.Timeout:
        return {
            "status": "error",
            "message": "Ingest timed out after 5 minutes"
        }
    except requests.exceptions.RequestException as e:
        return {
            "status": "error",
            "message": f"Request failed: {str(e)}"
        }
    except Exception as e:
        return {
            "status": "error",
            "message": f"Unexpected error: {str(e)}"
        }

def cleanup_handler(signum, frame):
    """Clean up server process on shutdown."""
    global server_process
    print("[Handler] Shutting down...")
    if server_process and server_process.poll() is None:
        server_process.terminate()
        server_process.wait(timeout=5)
    sys.exit(0)

# Register signal handlers
signal.signal(signal.SIGTERM, cleanup_handler)
signal.signal(signal.SIGINT, cleanup_handler)

if __name__ == "__main__":
    print("[Handler] Starting RunPod serverless handler for Doctown Ingest")
    runpod.serverless.start({"handler": handler})