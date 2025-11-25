#!/bin/bash
# Start all Doctown services: Builder, Assembly, Embedding, and Generation workers

set -e

echo "Starting Doctown Combined Services..."
echo "======================================"

# Check for required OPENAI_API_KEY
if [ -z "$OPENAI_API_KEY" ]; then
    echo "WARNING: OPENAI_API_KEY not set. Generation worker will fail."
    echo "Set it in RunPod environment variables."
fi

# Start builder in background
echo "Starting Builder API on port 3000..."
/app/builder &
BUILDER_PID=$!
echo "Builder PID: $BUILDER_PID"

# Give builder a moment to start
sleep 2

# Start assembly worker in background
echo "Starting Assembly Worker on port 8002..."
/app/assembly-server &
ASSEMBLY_PID=$!
echo "Assembly Worker PID: $ASSEMBLY_PID"

# Give assembly a moment to start
sleep 1

# Start embedding worker in background
echo "Starting Embedding Worker on port 8000..."
cd /app/embedding
python3 -m uvicorn app.main:app --host 0.0.0.0 --port 8000 &
EMBEDDING_PID=$!
echo "Embedding Worker PID: $EMBEDDING_PID"

# Wait for embedding worker to be ready (model loading takes time)
echo "Waiting for Embedding Worker to be ready..."
for i in {1..30}; do
    if curl -s http://localhost:8000/health > /dev/null 2>&1; then
        echo "Embedding Worker is ready!"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "WARNING: Embedding Worker not responding after 30 seconds"
    fi
    sleep 1
done

# Start generation worker in background
echo "Starting Generation Worker on port 8003..."
cd /app/generation
python3 -m uvicorn app.main:app --host 0.0.0.0 --port 8003 &
GENERATION_PID=$!
echo "Generation Worker PID: $GENERATION_PID"

echo "======================================"
echo "✓ All services started"
echo "  Builder:            http://0.0.0.0:3000"
echo "  Assembly Worker:    http://0.0.0.0:8002"
echo "  Embedding Worker:   http://0.0.0.0:8000"
echo "  Generation Worker:  http://0.0.0.0:8003"
echo "======================================"

# Function to handle shutdown
cleanup() {
    echo ""
    echo "Shutting down services..."
    kill $BUILDER_PID $ASSEMBLY_PID $EMBEDDING_PID $GENERATION_PID 2>/dev/null || true
    exit 0
}

trap cleanup SIGTERM SIGINT

# Wait for all processes
wait $BUILDER_PID $ASSEMBLY_PID $EMBEDDING_PID $GENERATION_PID
