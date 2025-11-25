#!/bin/bash
# Build the serverless builder image

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

echo "Building serverless builder image..."
docker build \
    -f builder/Dockerfile.serverless \
    -t xandwrp/doctown-builder-serverless:latest \
    .

echo "Done! Image: xandwrp/doctown-builder-serverless:latest"
echo ""
echo "To push: docker push xandwrp/doctown-builder-serverless:latest"
