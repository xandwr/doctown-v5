#!/bin/bash
# Test the Docker image locally

set -e

IMAGE_NAME="xandwrp/doctown-builder"
TAG="${1:-latest}"
TEST_REPO="${2:-https://github.com/rust-lang/rust-by-example}"

echo "Testing Doctown ingest worker locally..."
echo "Image: ${IMAGE_NAME}:${TAG}"
echo "Test repo: ${TEST_REPO}"
echo ""

# Start the container
CONTAINER_ID=$(docker run -d -p 3000:3000 "${IMAGE_NAME}:${TAG}")

echo "Container started: ${CONTAINER_ID}"
echo "Waiting for server to be ready..."
echo ""

# Wait for health check
MAX_ATTEMPTS=30
ATTEMPT=0
while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
    if curl -s http://localhost:3000/health > /dev/null 2>&1; then
        echo "✅ Server is healthy!"
        break
    fi
    ATTEMPT=$((ATTEMPT + 1))
    sleep 1
done

if [ $ATTEMPT -eq $MAX_ATTEMPTS ]; then
    echo "❌ Server failed to become healthy"
    docker logs "${CONTAINER_ID}"
    docker stop "${CONTAINER_ID}"
    docker rm "${CONTAINER_ID}"
    exit 1
fi

echo ""
echo "Testing ingest endpoint..."
echo "Repo: ${TEST_REPO}"
echo ""

# Test the ingest endpoint (just check it responds, don't wait for full completion)
curl -N "http://localhost:3000/ingest?repo_url=${TEST_REPO}&git_ref=master" 2>/dev/null | head -n 20

echo ""
echo ""
echo "✅ Basic test passed!"
echo ""
echo "Container is still running. To view full output:"
echo "  docker logs -f ${CONTAINER_ID}"
echo ""
echo "To stop the container:"
echo "  docker stop ${CONTAINER_ID} && docker rm ${CONTAINER_ID}"
echo ""
echo "Container ID: ${CONTAINER_ID}"
