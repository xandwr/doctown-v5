#!/bin/bash
# Start both Builder (Rust) and Embedding Worker (Python) services

set -e

echo "Starting Doctown Combined Services..."
echo "======================================"

# Start builder in background
echo "Starting Builder API on port 3000..."
/app/builder &
BUILDER_PID=$!
echo "Builder PID: $BUILDER_PID"

# Give builder a moment to start
sleep 2

# Start embedding worker in background
echo "Starting Embedding Worker on port 8000..."
cd /app/embedding
python3 -m uvicorn app.main:app --host 0.0.0.0 --port 8000 &
EMBEDDING_PID=$!
echo "Embedding Worker PID: $EMBEDDING_PID"

echo "======================================"
echo "✓ Both services started"
echo "  Builder:          http://0.0.0.0:3000"
echo "  Embedding Worker: http://0.0.0.0:8000"
echo "======================================"

# Wait for both processes
wait $BUILDER_PID $EMBEDDING_PID
