# Memory Management Strategy for CPU-Only Embedding Worker

## Problem

When processing large codebases (1000+ files like numpy, ort), the embedding worker would crash due to memory exhaustion. This happened because:

1. **Large batches**: Attempting to embed too many chunks at once
2. **Memory accumulation**: Intermediate tensors not being freed
3. **No backpressure**: System would try to load everything into memory
4. **Host machine crashes**: Would literally freeze/crash the development machine

## Solution: Intelligent Memory-Aware Batching

### Core Components

#### 1. Memory Monitoring
```python
self.process = psutil.Process()
self.max_memory_bytes = psutil.virtual_memory().total * 0.7  # 70% of RAM
```

- Tracks process memory usage in real-time
- Sets a safe upper limit (70% of system RAM by default)
- Checks before processing each batch

#### 2. Adaptive Batch Sizing
```python
min_batch_size: 8   # Start small
max_batch_size: 64  # Conservative upper limit
```

**Dynamic Adjustment**:
- Starts at minimum safe size (8)
- Gradually increases on successful batches (+4 per success)
- Halves on memory pressure (÷2 on pressure)
- Never exceeds configured max (64)

**Why these numbers?**
- 8: Small enough to always fit in memory
- 64: Large enough for efficiency without memory spikes
- Much smaller than GPU settings (which used 1024) because CPU has limited RAM bandwidth

#### 3. Chunked Processing
```python
for i in range(0, total_texts, self.current_batch_size):
    chunk = texts[i:i + self.current_batch_size]
    # Process chunk
    # Check memory
    # GC if needed
```

Large batches are automatically split into smaller chunks and processed sequentially:
- Each chunk processed independently
- Memory checked between chunks
- Garbage collection every 10 chunks
- Can dynamically reduce chunk size if memory pressure detected

#### 4. Aggressive Garbage Collection
```python
if not is_safe:
    gc.collect()  # Force Python GC
    
if chunk_num % 10 == 0:
    gc.collect()  # Periodic cleanup
```

- Forces garbage collection when memory is tight
- Regular periodic cleanup (every 10 chunks)
- Final cleanup after processing complete batch

#### 5. Graceful Degradation
```python
if not is_safe:
    self._adjust_batch_size(success=False)
    logger.warning("Retrying with smaller batch size")
    return self.embed(chunk)  # Recursive retry
```

Instead of crashing:
- Detects memory pressure early
- Reduces batch size
- Retries the operation
- Continues processing successfully

### Configuration

Environment variables for tuning:

```bash
EMBEDDING_MIN_BATCH_SIZE=8           # Minimum safe batch size
EMBEDDING_MAX_BATCH_SIZE=64          # Maximum batch size
EMBEDDING_MAX_MEMORY_PERCENT=70.0    # Max RAM usage (%)
EMBEDDING_ADAPTIVE_BATCHING=true     # Enable dynamic adjustment
EMBEDDING_ONNX_THREADS=8             # CPU threads (capped to prevent thrashing)
```

### Execution Strategy

**ONNX Runtime Settings**:
```python
sess_options.execution_mode = ort.ExecutionMode.ORT_SEQUENTIAL
```

- **Sequential** instead of Parallel
- More memory-efficient (lower peak usage)
- Prevents memory spikes from concurrent operations
- Trade-off: slightly slower, but won't crash

**Thread Limiting**:
```python
onnx_threads: int = min(os.cpu_count() or 4, 8)  # Cap at 8
```

- Caps threads at 8 even on high-core CPUs
- Prevents thread thrashing
- Reduces memory overhead from thread pools
- Better stability on resource-constrained systems

## Performance Characteristics

### Small Codebases (< 100 files)
- Batch size quickly scales to 64
- Minimal overhead from memory checks
- ~Same performance as before

### Medium Codebases (100-500 files)
- Adaptive batching finds optimal size (32-64)
- Occasional GC pauses
- Slight overhead but stable

### Large Codebases (500-2000 files)
- Processes in chunks with cleanup
- Batch size may reduce to 16-32 under pressure
- Slower but NEVER crashes
- Predictable memory usage

### Giant Codebases (2000+ files, numpy/ort scale)
- Heavy chunking (8-16 per chunk)
- Frequent GC cycles
- Much slower, but completes successfully
- Host machine remains responsive

## Cost-Benefit Analysis

### Why CPU over GPU?

**GPU Pod Costs (RunPod)**:
- RTX 4090: ~$0.50-1.00/hour
- A4000: ~$0.30-0.60/hour
- **Runtime**: 2 weeks at 24/7 = $168-336

**CPU Pod Costs**:
- Shared CPU: $0.07/hour
- **Runtime**: 10+ weeks at 24/7 = $117
- **Savings**: $51-219 (30-65% cheaper)

**Critical for Bootstrapping**:
- 2 weeks → Not enough time to get funded
- 10+ weeks → Real runway to get customers and funding
- Makes difference between success and failure

### Performance Trade-offs

**GPU Benefits** (lost):
- 10-50x faster batch processing
- Can handle 1024-token batches
- Sub-second response times

**CPU with Memory Management** (gained):
- Never crashes (stable)
- Predictable costs
- Still fast enough (1-5s for typical batches)
- Can process arbitrarily large codebases
- Host machine stays responsive

**User Experience**:
- Small repos: No noticeable difference
- Large repos: Slower but completes (vs crashing)
- Better to be slow than broken

## Monitoring

### Log Messages

**Healthy Operation**:
```
INFO: ONNX model loaded (CPU-only) with 8 threads
INFO: Memory limit set to 11.20 GB (70% of system RAM)
DEBUG: Processing chunk 1/45 (64 texts)
DEBUG: Memory after chunk 10: 5.23 GB
```

**Memory Pressure**:
```
WARNING: Memory usage high (9.87 GB) before processing
WARNING: Memory pressure detected (10.34 GB), forcing GC
WARNING: Reduced batch size to 32 due to memory pressure
INFO: Retrying chunk 23/45 with smaller batch size
```

**Success**:
```
INFO: Successfully processed 2847 texts in 45 chunks
DEBUG: Increased batch size to 36
```

### Metrics to Watch

1. **Current batch size**: Should stabilize after warmup
2. **Memory usage**: Should stay below 70% threshold
3. **GC frequency**: More frequent = memory pressure
4. **Chunk count**: More chunks = larger workload

## Future Optimizations

### Possible Improvements

1. **Streaming embeddings**: Return results as they're computed
2. **Persistent cache**: Cache embeddings for unchanged files
3. **Better memory estimation**: Predict required memory before processing
4. **Quantization**: Use int8 quantized models (smaller memory footprint)
5. **Multi-worker**: Multiple worker processes with queue

### When to Consider GPU Again

- When funded and profitable
- For premium tier customers
- As optional "fast lane" service
- For real-time use cases

## Testing

### Verify Memory Management

```bash
# Small batch (should scale to 64)
curl -X POST http://localhost:8000/embed \
  -H "Content-Type: application/json" \
  -d '{
    "batch_id": "small",
    "chunks": [/* 50 chunks */]
  }'

# Large batch (should chunk automatically)
curl -X POST http://localhost:8000/embed \
  -H "Content-Type: application/json" \
  -d '{
    "batch_id": "large",
    "chunks": [/* 1000 chunks */]
  }'

# Giant batch (should handle gracefully)
curl -X POST http://localhost:8000/embed \
  -H "Content-Type: application/json" \
  -d '{
    "batch_id": "giant",
    "chunks": [/* 5000 chunks */]
  }'
```

### Monitor During Processing

```bash
# Watch memory usage
watch -n 1 'ps aux | grep python | grep -v grep'

# Watch logs
docker logs -f <container-id>

# System memory
watch -n 1 free -h
```

### Expected Behavior

✅ **Good Signs**:
- Memory stays below threshold
- Batch size stabilizes
- Completes without errors
- System remains responsive

❌ **Bad Signs**:
- Memory keeps growing
- Frequent OOM errors
- System becomes unresponsive
- Process gets killed

## Troubleshooting

### "Memory usage high" warnings
**Solution**: Normal, system is managing memory correctly. No action needed.

### Frequent batch size reductions
**Solution**: Reduce `EMBEDDING_MAX_MEMORY_PERCENT` to 60% or lower.

### Still running out of memory
**Solution**: 
1. Reduce `EMBEDDING_MAX_BATCH_SIZE` to 32 or 16
2. Reduce `EMBEDDING_MAX_MEMORY_PERCENT` to 50%
3. Increase system RAM
4. Reduce concurrent requests

### Too slow for large repos
**Solution**:
1. Increase `EMBEDDING_MAX_BATCH_SIZE` if memory allows
2. Increase `EMBEDDING_MAX_MEMORY_PERCENT` to 80%
3. Consider caching strategies
4. Consider upgrading to GPU for specific workloads

## Conclusion

This memory management strategy prioritizes **stability over speed**:

- **Never crash** > Be blazing fast
- **Complete successfully** > Finish quickly  
- **Predictable costs** > Maximum performance
- **Long runway** > Short-term optimization

For a bootstrapped startup, this is the right trade-off. We can always optimize later when we have revenue.
