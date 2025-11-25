#!/bin/bash
# Build the GPU-enabled embedding serverless image

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"

cd "$PROJECT_ROOT"

echo "Building GPU embedding serverless image..."
docker build \
    -f workers/embedding/Dockerfile.gpu \
    -t xandwrp/doctown-embedder-gpu:latest \
    .

echo "Done! Image: xandwrp/doctown-embedder-gpu:latest"
echo ""
echo "To push: docker push xandwrp/doctown-embedder-gpu:latest"
