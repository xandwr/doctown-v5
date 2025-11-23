#!/bin/bash
# Test the deployed RunPod endpoint

set -e

# Load environment variables
if [ -f "$(dirname "$0")/../.env" ]; then
    export $(grep -v '^#' "$(dirname "$0")/../.env" | xargs)
fi

if [ -z "$RUNPOD_BUILDER_ID" ]; then
    echo "❌ RUNPOD_BUILDER_ID not set in .env"
    exit 1
fi

if [ -z "$RUNPOD_API_KEY" ]; then
    echo "❌ RUNPOD_API_KEY not set in .env"
    exit 1
fi

TEST_REPO="${1:-https://github.com/rust-lang/rust-by-example}"

echo "Testing RunPod endpoint..."
echo "Endpoint ID: ${RUNPOD_BUILDER_ID}"
echo "Test repo: ${TEST_REPO}"
echo ""

# Create a job
RESPONSE=$(curl -s -X POST "https://api.runpod.ai/v2/${RUNPOD_BUILDER_ID}/run" \
  -H "Authorization: Bearer ${RUNPOD_API_KEY}" \
  -H "Content-Type: application/json" \
  -d "{
    \"input\": {
      \"repo_url\": \"${TEST_REPO}\",
      \"git_ref\": \"master\"
    }
  }")

echo "Response:"
echo "$RESPONSE" | jq '.'
echo ""

# Extract job ID
JOB_ID=$(echo "$RESPONSE" | jq -r '.id')

if [ "$JOB_ID" = "null" ] || [ -z "$JOB_ID" ]; then
    echo "❌ Failed to create job"
    exit 1
fi

echo "Job created: ${JOB_ID}"
echo ""
echo "Polling for status..."
echo ""

# Poll for status
MAX_ATTEMPTS=60
ATTEMPT=0
while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
    STATUS_RESPONSE=$(curl -s "https://api.runpod.ai/v2/${RUNPOD_BUILDER_ID}/status/${JOB_ID}" \
      -H "Authorization: Bearer ${RUNPOD_API_KEY}")
    
    STATUS=$(echo "$STATUS_RESPONSE" | jq -r '.status')
    
    echo "[$ATTEMPT] Status: $STATUS"
    
    if [ "$STATUS" = "COMPLETED" ]; then
        echo ""
        echo "✅ Job completed successfully!"
        echo ""
        echo "Full response:"
        echo "$STATUS_RESPONSE" | jq '.'
        exit 0
    fi
    
    if [ "$STATUS" = "FAILED" ]; then
        echo ""
        echo "❌ Job failed"
        echo ""
        echo "Full response:"
        echo "$STATUS_RESPONSE" | jq '.'
        exit 1
    fi
    
    ATTEMPT=$((ATTEMPT + 1))
    sleep 5
done

echo ""
echo "⚠️  Job did not complete within timeout"
echo ""
echo "Check status manually:"
echo "  curl -H 'Authorization: Bearer \$RUNPOD_API_KEY' https://api.runpod.ai/v2/${RUNPOD_BUILDER_ID}/status/${JOB_ID}"
