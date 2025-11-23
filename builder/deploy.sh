#!/bin/bash
# Deploy script for pushing to Docker Hub

set -e

IMAGE_NAME="xandwrp/doctown-builder"
TAG="${1:-latest}"

echo "Pushing Doctown ingest worker to Docker Hub..."
echo "Image: ${IMAGE_NAME}:${TAG}"
echo ""

# Check if logged in to Docker Hub
if ! docker info | grep -q "Username"; then
    echo "⚠️  Not logged in to Docker Hub"
    echo "Run: docker login"
    exit 1
fi

# Push the image
docker push "${IMAGE_NAME}:${TAG}"

echo ""
echo "✅ Push complete!"
echo ""
echo "Image available at: ${IMAGE_NAME}:${TAG}"
echo ""
echo "Next steps:"
echo "  1. Update RunPod serverless endpoint to use this image"
echo "  2. Test via RunPod: ./builder/test-runpod.sh"
