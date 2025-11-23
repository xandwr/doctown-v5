#!/bin/bash
set -e

# Smoke test script for RunPod deployment
# Tests the full pipeline: RunPod handler -> Ingest API -> SSE streaming

RUNPOD_API_KEY="${RUNPOD_API_KEY:-}"
RUNPOD_ENDPOINT_ID="${RUNPOD_ENDPOINT_ID:-}"
TEST_REPO="${TEST_REPO:-https://github.com/rust-lang/regex}"
TEST_REF="${TEST_REF:-main}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Doctown RunPod Smoke Test ===${NC}"
echo ""

# Check required environment variables
if [ -z "$RUNPOD_API_KEY" ]; then
    echo -e "${RED}Error: RUNPOD_API_KEY environment variable is not set${NC}"
    echo "Please set it in your .env file or export it:"
    echo "  export RUNPOD_API_KEY=your-api-key-here"
    exit 1
fi

if [ -z "$RUNPOD_ENDPOINT_ID" ]; then
    echo -e "${RED}Error: RUNPOD_ENDPOINT_ID environment variable is not set${NC}"
    echo "Please set it in your .env file or export it:"
    echo "  export RUNPOD_ENDPOINT_ID=your-endpoint-id-here"
    exit 1
fi

echo -e "${YELLOW}Configuration:${NC}"
echo "  Endpoint ID: $RUNPOD_ENDPOINT_ID"
echo "  Test Repo:   $TEST_REPO"
echo "  Git Ref:     $TEST_REF"
echo ""

# Generate a unique job ID for this test
JOB_ID="smoke-test-$(date +%s)"
echo -e "${YELLOW}Job ID: $JOB_ID${NC}"
echo ""

# Step 1: Health check (direct to endpoint if available)
echo -e "${BLUE}Step 1: Testing /health endpoint...${NC}"
HEALTH_URL="https://api.runpod.ai/v2/${RUNPOD_ENDPOINT_ID}/health"
echo "  GET $HEALTH_URL"

HEALTH_RESPONSE=$(curl -s -w "\n%{http_code}" \
    -H "Authorization: Bearer ${RUNPOD_API_KEY}" \
    "$HEALTH_URL")

HEALTH_BODY=$(echo "$HEALTH_RESPONSE" | head -n -1)
HEALTH_CODE=$(echo "$HEALTH_RESPONSE" | tail -n 1)

if [ "$HEALTH_CODE" = "200" ]; then
    echo -e "${GREEN}✓ Health check passed${NC}"
    echo "  Response: $HEALTH_BODY"
else
    echo -e "${YELLOW}⚠ Health endpoint returned $HEALTH_CODE (might not be exposed)${NC}"
    echo "  This is okay - continuing with job submission..."
fi
echo ""

# Step 2: Submit job to RunPod
echo -e "${BLUE}Step 2: Submitting ingest job to RunPod...${NC}"
RUN_URL="https://api.runpod.ai/v2/${RUNPOD_ENDPOINT_ID}/run"
echo "  POST $RUN_URL"

REQUEST_PAYLOAD=$(cat <<EOF
{
  "input": {
    "repo_url": "$TEST_REPO",
    "git_ref": "$TEST_REF",
    "job_id": "$JOB_ID"
  }
}
EOF
)

echo "  Payload: $REQUEST_PAYLOAD"

SUBMIT_RESPONSE=$(curl -s -w "\n%{http_code}" \
    -X POST \
    -H "Authorization: Bearer ${RUNPOD_API_KEY}" \
    -H "Content-Type: application/json" \
    -d "$REQUEST_PAYLOAD" \
    "$RUN_URL")

SUBMIT_BODY=$(echo "$SUBMIT_RESPONSE" | head -n -1)
SUBMIT_CODE=$(echo "$SUBMIT_RESPONSE" | tail -n 1)

echo "  HTTP Status: $SUBMIT_CODE"
echo "  Response: $SUBMIT_BODY"

if [ "$SUBMIT_CODE" != "200" ]; then
    echo -e "${RED}✗ Job submission failed${NC}"
    exit 1
fi

# Extract RunPod job ID
RUNPOD_JOB_ID=$(echo "$SUBMIT_BODY" | grep -o '"id":"[^"]*"' | head -1 | cut -d'"' -f4)

if [ -z "$RUNPOD_JOB_ID" ]; then
    echo -e "${RED}✗ Failed to extract RunPod job ID from response${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Job submitted successfully${NC}"
echo "  RunPod Job ID: $RUNPOD_JOB_ID"
echo ""

# Step 3: Poll for job status
echo -e "${BLUE}Step 3: Polling for job completion...${NC}"
STATUS_URL="https://api.runpod.ai/v2/${RUNPOD_ENDPOINT_ID}/status/${RUNPOD_JOB_ID}"

MAX_ATTEMPTS=60  # 5 minutes with 5-second intervals
ATTEMPT=0
EVENT_COUNT=0
COMPLETED=false

while [ $ATTEMPT -lt $MAX_ATTEMPTS ]; do
    ATTEMPT=$((ATTEMPT + 1))
    
    STATUS_RESPONSE=$(curl -s -w "\n%{http_code}" \
        -H "Authorization: Bearer ${RUNPOD_API_KEY}" \
        "$STATUS_URL")
    
    STATUS_BODY=$(echo "$STATUS_RESPONSE" | head -n -1)
    STATUS_CODE=$(echo "$STATUS_RESPONSE" | tail -n 1)
    
    if [ "$STATUS_CODE" != "200" ]; then
        echo -e "${RED}✗ Status check failed with HTTP $STATUS_CODE${NC}"
        echo "  Response: $STATUS_BODY"
        exit 1
    fi
    
    # Extract status
    JOB_STATUS=$(echo "$STATUS_BODY" | grep -o '"status":"[^"]*"' | head -1 | cut -d'"' -f4)
    
    # Count events if available
    NEW_EVENT_COUNT=$(echo "$STATUS_BODY" | grep -o '"event_type":' | wc -l)
    if [ "$NEW_EVENT_COUNT" -gt "$EVENT_COUNT" ]; then
        EVENT_COUNT=$NEW_EVENT_COUNT
        echo "  [Attempt $ATTEMPT] Status: $JOB_STATUS | Events received: $EVENT_COUNT"
    fi
    
    case "$JOB_STATUS" in
        "COMPLETED")
            echo -e "${GREEN}✓ Job completed successfully${NC}"
            COMPLETED=true
            break
            ;;
        "FAILED")
            echo -e "${RED}✗ Job failed${NC}"
            echo "  Response: $STATUS_BODY"
            exit 1
            ;;
        "CANCELLED")
            echo -e "${RED}✗ Job was cancelled${NC}"
            exit 1
            ;;
        "IN_QUEUE"|"IN_PROGRESS")
            # Keep polling
            sleep 5
            ;;
        *)
            echo "  [Attempt $ATTEMPT] Status: $JOB_STATUS"
            sleep 5
            ;;
    esac
done

if [ "$COMPLETED" = false ]; then
    echo -e "${RED}✗ Job did not complete within timeout period${NC}"
    exit 1
fi

echo ""

# Step 4: Validate output
echo -e "${BLUE}Step 4: Validating job output...${NC}"

# Check for expected event types
if echo "$STATUS_BODY" | grep -q '"event_type":"ingest.started.v1"'; then
    echo -e "${GREEN}✓ Found ingest.started.v1 event${NC}"
else
    echo -e "${YELLOW}⚠ Missing ingest.started.v1 event${NC}"
fi

if echo "$STATUS_BODY" | grep -q '"event_type":"ingest.completed.v1"'; then
    echo -e "${GREEN}✓ Found ingest.completed.v1 event${NC}"
else
    echo -e "${YELLOW}⚠ Missing ingest.completed.v1 event${NC}"
fi

# Count chunks and symbols
CHUNK_COUNT=$(echo "$STATUS_BODY" | grep -o '"event_type":"chunk.created.v1"' | wc -l)
SYMBOL_COUNT=$(echo "$STATUS_BODY" | grep -o '"event_type":"symbol.extracted.v1"' | wc -l)

echo "  Total events: $EVENT_COUNT"
echo "  Chunks created: $CHUNK_COUNT"
echo "  Symbols extracted: $SYMBOL_COUNT"

if [ "$CHUNK_COUNT" -gt 0 ] && [ "$SYMBOL_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✓ Pipeline produced chunks and symbols${NC}"
else
    echo -e "${YELLOW}⚠ Pipeline produced no chunks or symbols${NC}"
fi

echo ""
echo -e "${GREEN}=== Smoke Test Complete ===${NC}"
echo ""
echo "Summary:"
echo "  Job ID: $JOB_ID"
echo "  RunPod Job ID: $RUNPOD_JOB_ID"
echo "  Status: $JOB_STATUS"
echo "  Events: $EVENT_COUNT"
echo "  Chunks: $CHUNK_COUNT"
echo "  Symbols: $SYMBOL_COUNT"
echo ""

# Save full response for inspection
OUTPUT_FILE="/tmp/runpod-smoke-test-${JOB_ID}.json"
echo "$STATUS_BODY" | jq '.' > "$OUTPUT_FILE" 2>/dev/null || echo "$STATUS_BODY" > "$OUTPUT_FILE"
echo "Full response saved to: $OUTPUT_FILE"
