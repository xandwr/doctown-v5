#!/bin/bash
# Build the combined Docker image

set -e

IMAGE_NAME="xandwrp/doctown-combined"
TAG="${1:-latest}"

echo "Building Doctown Combined Image..."
echo "Image: ${IMAGE_NAME}:${TAG}"
echo ""

# Build the image
docker build -f Dockerfile.combined -t "${IMAGE_NAME}:${TAG}" .

echo ""
echo "✅ Build complete!"
echo ""
echo "Image: ${IMAGE_NAME}:${TAG}"
echo ""
echo "Next steps:"
echo "  1. Test locally:  docker run -p 3000:3000 -p 8000:8000 ${IMAGE_NAME}:${TAG}"
echo "  2. Push to registry: docker push ${IMAGE_NAME}:${TAG}"
echo "  3. Deploy to RunPod CPU Pod"
