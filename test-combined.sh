#!/bin/bash
# Test the combined Docker image locally

set -e

IMAGE_NAME="xandwrp/doctown-combined"
TAG="${1:-latest}"
CONTAINER_NAME="doctown-combined-test"

echo "Testing Doctown Combined Image..."
echo "Image: ${IMAGE_NAME}:${TAG}"
echo ""

# Stop and remove existing container if it exists
docker stop ${CONTAINER_NAME} 2>/dev/null || true
docker rm ${CONTAINER_NAME} 2>/dev/null || true

# Start the container
echo "Starting container..."
docker run -d \
  --name ${CONTAINER_NAME} \
  -p 3000:3000 \
  -p 8000:8000 \
  ${IMAGE_NAME}:${TAG}

echo "Container started: ${CONTAINER_NAME}"
echo ""

# Wait for services to be ready
echo "Waiting for services to start..."
sleep 5

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
echo "✅ Both services are running!"
echo ""
echo "Test URLs:"
echo "  Builder:          http://localhost:3000"
echo "  Embedding Worker: http://localhost:8000"
echo ""
echo "View logs: docker logs -f ${CONTAINER_NAME}"
echo "Stop:      docker stop ${CONTAINER_NAME}"
echo ""
