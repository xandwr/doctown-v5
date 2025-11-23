#!/bin/bash
# Build script for Doctown ingest worker Docker image

set -e

IMAGE_NAME="xandwrp/doctown-builder"
TAG="${1:-latest}"

echo "Building Doctown ingest worker Docker image..."
echo "Image: ${IMAGE_NAME}:${TAG}"

# Build from the root directory (needed for workspace context)
cd "$(dirname "$0")/.."

docker build -f builder/Dockerfile -t "${IMAGE_NAME}:${TAG}" .

echo ""
echo "✅ Build complete!"
echo ""
echo "Image: ${IMAGE_NAME}:${TAG}"
echo ""
echo "Next steps:"
echo "  1. Test locally:  ./builder/test-local.sh"
echo "  2. Push to hub:   docker push ${IMAGE_NAME}:${TAG}"
echo "  3. Deploy:        ./builder/deploy.sh"
