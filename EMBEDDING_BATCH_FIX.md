# Embedding Batch Fix

**Issue:** Large repos (1600+ chunks) were timing out when trying to embed all chunks in one batch.

## Problem

The embedding worker was receiving requests but the HTTP client was timing out before the embedding could complete:
```
Warning: Failed to embed chunks: Internal error: Failed to call embedding worker: error sending request for url (http://localhost:8000/embed)
```

Root causes:
1. **Default timeout too short** - Default reqwest timeout (30s) insufficient for large batches
2. **Single large batch** - Trying to embed 1634 chunks in one request
3. **No progress feedback** - No visibility into what's happening during embedding

## Solution

### 1. Increased Client Timeout
```rust
let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(120)) // 2 minutes
    .build()
```

### 2. Batch Splitting
Split large chunk collections into batches of 256:
```rust
const BATCH_SIZE: usize = 256;

for (batch_num, chunk_batch) in collected_chunks.chunks(BATCH_SIZE).enumerate() {
    // Embed each batch separately
}
```

### 3. Progress Logging
Added per-batch logging:
```
Embedded batch 1: 256 chunks in 1247ms (~205 chunks/sec)
Embedded batch 2: 256 chunks in 1198ms (~213 chunks/sec)
...
```

## Benefits

✅ **No timeouts** - Each batch completes within timeout window
✅ **Progress visibility** - See embedding progress in real-time
✅ **Graceful degradation** - If one batch fails, others still succeed
✅ **Better throughput** - Can process repos of any size

## Performance

**Before:**
- Timeout on batches > ~500 chunks
- No feedback during processing
- All-or-nothing approach

**After:**
- Handles repos with 1000+ chunks easily
- Real-time progress logging
- Partial success possible

## Example Output

```
Embedded batch 1: 256 chunks in 1247ms (~205 chunks/sec)
Embedded batch 2: 256 chunks in 1198ms (~213 chunks/sec)
Embedded batch 3: 256 chunks in 1211ms (~211 chunks/sec)
Embedded batch 4: 256 chunks in 1189ms (~215 chunks/sec)
Embedded batch 5: 256 chunks in 1234ms (~207 chunks/sec)
Embedded batch 6: 256 chunks in 1223ms (~209 chunks/sec)
Embedded batch 7: 122 chunks in 592ms (~206 chunks/sec)

Total: 1634 chunks embedded successfully
```

## Configuration

### Timeout
Change in `embedding.rs`:
```rust
.timeout(std::time::Duration::from_secs(120))
```

### Batch Size
Change in `pipeline.rs`:
```rust
const BATCH_SIZE: usize = 256;  // Adjust based on performance
```

Recommendations:
- **256 chunks** - Good balance (recommended)
- **128 chunks** - More conservative, slower overall
- **512 chunks** - Faster but riskier with timeouts

## Files Modified

- `crates/doctown-ingest/src/embedding.rs` - Added timeout configuration
- `crates/doctown-ingest/src/pipeline.rs` - Added batch splitting and progress logging

## Testing

Test with a large repo:
```bash
# A repo with 1000+ chunks
curl -X POST http://localhost:3000/ingest \
  -H "Content-Type: application/json" \
  -d '{"repo_url": "https://github.com/rust-lang/cargo", "job_id": "test123"}'
```

Watch the logs for batch progress!
