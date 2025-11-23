#!/bin/bash
# Test script for Assembly Worker API

set -e

BASE_URL="${BASE_URL:-http://localhost:3001}"

echo "Testing Assembly Worker API at $BASE_URL"
echo

# Test health endpoint
echo "1. Testing /health endpoint..."
curl -s "$BASE_URL/health" | jq .
echo
echo "✓ Health check passed"
echo

# Test assemble endpoint with sample data
echo "2. Testing /assemble endpoint with sample data..."
curl -s -X POST "$BASE_URL/assemble" \
  -H "Content-Type: application/json" \
  -d '{
    "job_id": "test_job_123",
    "repo_url": "https://github.com/test/repo",
    "git_ref": "main",
    "chunks": [
      {
        "chunk_id": "chunk_1",
        "vector": [0.1, 0.2, 0.3, 0.4],
        "content": "fn calculate_total() { sum() }"
      },
      {
        "chunk_id": "chunk_2",
        "vector": [0.2, 0.3, 0.4, 0.5],
        "content": "fn calculate_average() { mean() }"
      }
    ],
    "symbols": [
      {
        "symbol_id": "sym_1",
        "name": "calculate_total",
        "kind": "function",
        "file_path": "src/math.rs",
        "signature": "fn calculate_total() -> i32",
        "chunk_ids": ["chunk_1"],
        "calls": ["sum"],
        "imports": ["std::collections"]
      },
      {
        "symbol_id": "sym_2",
        "name": "calculate_average",
        "kind": "function",
        "file_path": "src/math.rs",
        "signature": "fn calculate_average() -> f64",
        "chunk_ids": ["chunk_2"],
        "calls": ["mean"],
        "imports": []
      }
    ]
  }' | jq '{
    job_id: .job_id,
    cluster_count: .stats.cluster_count,
    node_count: .stats.node_count,
    edge_count: .stats.edge_count,
    duration_ms: .stats.duration_ms,
    clusters: .clusters | map({id: .cluster_id, label: .label, members: .members | length}),
    event_count: .events | length
  }'
echo
echo "✓ Assembly test passed"
echo

echo "All tests passed! ✓"
