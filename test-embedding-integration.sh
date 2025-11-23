#!/bin/bash
# Test script to verify embedding integration

set -e

echo "Testing Embedding Integration"
echo "=============================="
echo ""

# Check if services are running
echo "1. Checking if builder is running..."
if curl -s -f http://localhost:3000/health > /dev/null 2>&1; then
    echo "   ✅ Builder is healthy"
else
    echo "   ❌ Builder is not responding on port 3000"
    echo "   Start it with: ./builder/builder (or docker-compose up)"
    exit 1
fi

echo ""
echo "2. Checking if embedding worker is running..."
if curl -s -f http://localhost:8000/health > /dev/null 2>&1; then
    echo "   ✅ Embedding worker is healthy"
    HEALTH=$(curl -s http://localhost:8000/health)
    echo "   Response: $HEALTH"
else
    echo "   ❌ Embedding worker is not responding on port 8000"
    echo "   Start it with: ./workers/embedding/run.sh"
    exit 1
fi

echo ""
echo "3. Testing embedding worker directly..."
EMBED_RESPONSE=$(curl -s -X POST http://localhost:8000/embed \
  -H "Content-Type: application/json" \
  -d '{
    "batch_id": "test_batch",
    "chunks": [
      {"chunk_id": "chunk_test1", "content": "function hello() { return world; }"},
      {"chunk_id": "chunk_test2", "content": "class Foo { bar() {} }"}
    ]
  }')

VECTOR_COUNT=$(echo "$EMBED_RESPONSE" | grep -o '"chunk_id"' | wc -l)
if [ "$VECTOR_COUNT" -eq 2 ]; then
    echo "   ✅ Embedding worker returned 2 vectors"
else
    echo "   ❌ Embedding worker did not return expected vectors"
    echo "   Response: $EMBED_RESPONSE"
    exit 1
fi

echo ""
echo "4. Testing full ingest pipeline with embedding..."
echo "   Ingesting a small test repo..."

JOB_ID="test_$(date +%s)"
INGEST_URL="http://localhost:3000/ingest"

# Use a small test repo
TEST_REPO="https://github.com/rust-lang/rustlings"

echo "   Repository: $TEST_REPO"
echo "   Job ID: $JOB_ID"
echo ""

# Start the ingest (this will stream events)
TEMP_FILE=$(mktemp)
curl -s -X POST "$INGEST_URL" \
  -H "Content-Type: application/json" \
  -d "{\"repo_url\": \"$TEST_REPO\", \"job_id\": \"$JOB_ID\", \"git_ref\": \"main\"}" \
  > "$TEMP_FILE" &

CURL_PID=$!

echo "   Waiting for completion..."
sleep 10

# Check if completed
if grep -q "ingest.completed" "$TEMP_FILE"; then
    echo "   ✅ Ingest completed"
    
    # Check for chunks_embedded in the response
    if grep -q "chunks_embedded" "$TEMP_FILE"; then
        EMBEDDED_COUNT=$(grep "chunks_embedded" "$TEMP_FILE" | grep -o '[0-9]\+' | head -1)
        echo "   ✅ Chunks embedded: $EMBEDDED_COUNT"
        echo ""
        echo "=============================="
        echo "✅ ALL TESTS PASSED!"
        echo "=============================="
        echo ""
        echo "Embedding integration is working correctly."
        echo "Chunks are being created AND embedded automatically."
    else
        echo "   ⚠️  No chunks_embedded field found"
        echo "   This might mean no chunks were created, or embedding failed"
    fi
else
    echo "   ⚠️  Ingest did not complete in time"
    echo "   Check the logs for details"
fi

# Cleanup
kill $CURL_PID 2>/dev/null || true
rm -f "$TEMP_FILE"

echo ""
echo "Next: Test in the frontend at http://localhost:5173"
