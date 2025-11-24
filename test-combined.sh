#!/bin/bash
# Test the combined Docker image locally

set -e

IMAGE_NAME="xandwrp/doctown-combined"
TAG="${1:-latest}"
CONTAINER_NAME="doctown-combined-test"

echo "Testing Doctown Combined Image..."
echo "Image: ${IMAGE_NAME}:${TAG}"
echo ""

# Check for OPENAI_API_KEY
if [ -z "$OPENAI_API_KEY" ]; then
  echo "WARNING: OPENAI_API_KEY not set. Generation worker will not function."
  echo "Set it with: export OPENAI_API_KEY=your-key"
  OPENAI_KEY_ARG=""
else
  OPENAI_KEY_ARG="-e OPENAI_API_KEY=${OPENAI_API_KEY}"
fi

# Stop and remove existing container if it exists
docker stop ${CONTAINER_NAME} 2>/dev/null || true
docker rm ${CONTAINER_NAME} 2>/dev/null || true

# Start the container
echo "Starting container..."
docker run -d \
  --name ${CONTAINER_NAME} \
  -p 3000:3000 \
  -p 8000:8000 \
  -p 8002:8002 \
  -p 8003:8003 \
  ${OPENAI_KEY_ARG} \
  ${IMAGE_NAME}:${TAG}

echo "Container started: ${CONTAINER_NAME}"
echo ""

# Wait for services to be ready
echo "Waiting for services to start..."
sleep 10

# Test builder health
echo "Testing Builder (port 3000)..."
if curl -f http://localhost:3000/health 2>/dev/null; then
  echo "✅ Builder is healthy"
else
  echo "❌ Builder health check failed"
  docker logs ${CONTAINER_NAME}
  exit 1
fi

echo ""

# Test embedding worker health
echo "Testing Embedding Worker (port 8000)..."
if curl -f http://localhost:8000/health 2>/dev/null; then
  echo "✅ Embedding Worker is healthy"
  curl -s http://localhost:8000/health | python3 -m json.tool
else
  echo "❌ Embedding Worker health check failed"
  docker logs ${CONTAINER_NAME}
  exit 1
fi

echo ""

# Test assembly worker health
echo "Testing Assembly Worker (port 8002)..."
if curl -f http://localhost:8002/health 2>/dev/null; then
  echo "✅ Assembly Worker is healthy"
else
  echo "❌ Assembly Worker health check failed"
  docker logs ${CONTAINER_NAME}
  exit 1
fi

echo ""

# Test generation worker health
echo "Testing Generation Worker (port 8003)..."
if curl -f http://localhost:8003/health 2>/dev/null; then
  echo "✅ Generation Worker is healthy"
  curl -s http://localhost:8003/health | python3 -m json.tool
else
  echo "⚠️  Generation Worker health check failed (may need OPENAI_API_KEY)"
fi

echo ""
echo "✅ All services are running!"
echo ""
echo "Test URLs:"
echo "  Builder:            http://localhost:3000"
echo "  Embedding Worker:   http://localhost:8000"
echo "  Assembly Worker:    http://localhost:8002"
echo "  Generation Worker:  http://localhost:8003"
echo ""
echo "View logs: docker logs -f ${CONTAINER_NAME}"
echo "Stop:      docker stop ${CONTAINER_NAME}"
echo ""
