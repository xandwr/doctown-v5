#!/bin/bash
set -e

# Simple health check test for the ingest API
# Tests both local (port 3000) and RunPod deployment

API_URL="${API_URL:-http://localhost:3000}"
RUNPOD_API_KEY="${RUNPOD_API_KEY:-}"
RUNPOD_ENDPOINT_ID="${RUNPOD_ENDPOINT_ID:-}"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Health Check Test ===${NC}"
echo ""

# Test 1: Local API
echo -e "${BLUE}Test 1: Local API Health Check${NC}"
echo "  GET ${API_URL}/health"

if command -v curl >/dev/null 2>&1; then
    RESPONSE=$(curl -s -w "\n%{http_code}" "${API_URL}/health" 2>&1 || echo -e "\nERROR")
    BODY=$(echo "$RESPONSE" | head -n -1)
    CODE=$(echo "$RESPONSE" | tail -n 1)
    
    if [ "$CODE" = "200" ]; then
        echo -e "${GREEN}✓ Local API is healthy${NC}"
        echo "  Response: $BODY"
    elif [ "$CODE" = "ERROR" ]; then
        echo -e "${YELLOW}⚠ Cannot connect to local API${NC}"
        echo "  Make sure the server is running: ./dev.sh"
    else
        echo -e "${RED}✗ Health check failed with HTTP $CODE${NC}"
        echo "  Response: $BODY"
    fi
else
    echo -e "${YELLOW}⚠ curl not found, skipping local test${NC}"
fi

echo ""

# Test 2: RunPod API
if [ -n "$RUNPOD_API_KEY" ] && [ -n "$RUNPOD_ENDPOINT_ID" ]; then
    echo -e "${BLUE}Test 2: RunPod Health Check${NC}"
    RUNPOD_HEALTH_URL="https://api.runpod.ai/v2/${RUNPOD_ENDPOINT_ID}/health"
    echo "  GET ${RUNPOD_HEALTH_URL}"
    
    RESPONSE=$(curl -s -w "\n%{http_code}" \
        -H "Authorization: Bearer ${RUNPOD_API_KEY}" \
        "${RUNPOD_HEALTH_URL}" 2>&1 || echo -e "\nERROR")
    
    BODY=$(echo "$RESPONSE" | head -n -1)
    CODE=$(echo "$RESPONSE" | tail -n 1)
    
    if [ "$CODE" = "200" ]; then
        echo -e "${GREEN}✓ RunPod API is healthy${NC}"
        echo "  Response: $BODY"
    elif [ "$CODE" = "ERROR" ]; then
        echo -e "${YELLOW}⚠ Cannot connect to RunPod API${NC}"
    else
        echo -e "${YELLOW}⚠ Health endpoint returned HTTP $CODE${NC}"
        echo "  Note: Health endpoint might not be exposed by RunPod"
        echo "  This is normal - use smoke-test.sh to test the full pipeline"
    fi
else
    echo -e "${YELLOW}⚠ Skipping RunPod test (RUNPOD_API_KEY or RUNPOD_ENDPOINT_ID not set)${NC}"
fi

echo ""
echo -e "${GREEN}=== Health Check Complete ===${NC}"
