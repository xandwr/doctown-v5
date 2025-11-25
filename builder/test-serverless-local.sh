#!/bin/bash
# Test the serverless builder locally using RunPod test harness

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Default test repo
REPO_URL="${1:-https://github.com/xandwr/localdoc}"
GIT_REF="${2:-main}"

echo "Testing serverless builder with:"
echo "  Repo: $REPO_URL"
echo "  Ref:  $GIT_REF"
echo ""

# Build the image first
cd "$PROJECT_ROOT"
echo "Building serverless image..."
docker build -f builder/Dockerfile.serverless -t doctown-builder-serverless:test . 2>&1 | tail -5

echo ""
echo "Running test..."

# Run with runpod test input format
docker run --rm \
    -e RUNPOD_DEBUG_LEVEL=DEBUG \
    -e SKIP_EMBEDDING=true \
    doctown-builder-serverless:test \
    python3 -c "
import json
import sys
sys.path.insert(0, '/app')
from handler import handler

result = handler({
    'id': 'test-job-123',
    'input': {
        'repo_url': '$REPO_URL',
        'git_ref': '$GIT_REF',
        'job_id': 'job_test_$(date +%s)'
    }
})

print(json.dumps(result, indent=2))
"

echo ""
echo "Test complete!"
