# CPU-Only Embedding Worker - Deployment Checklist

## Pre-Deployment Testing

### Local Build & Test
- [ ] Build Docker image
  ```bash
  cd workers/embedding
  docker build -t doctown-embedding-worker:cpu .
  ```

- [ ] Run container locally
  ```bash
  docker run -p 8000:8000 doctown-embedding-worker:cpu
  ```

- [ ] Test health endpoint
  ```bash
  curl http://localhost:8000/health
  # Expected: {"status": "healthy"}
  ```

- [ ] Test small batch (< 10 chunks)
  ```bash
  curl -X POST http://localhost:8000/embed \
    -H "Content-Type: application/json" \
    -d '{
      "batch_id": "test_small",
      "chunks": [
        {"chunk_id": "1", "content": "def hello(): return \"world\""},
        {"chunk_id": "2", "content": "class Parser: pass"}
      ]
    }'
  # Expected: 200 OK with embeddings
  ```

- [ ] Test medium batch (~100 chunks)
  - Use your test script
  - Watch logs for batch size adjustments
  - Verify no memory warnings

- [ ] Test large batch (500+ chunks)
  - Simulate a large repo
  - Watch for chunking behavior
  - Monitor memory usage: `docker stats`
  - Should see "Processing X texts in Y chunks"

- [ ] Monitor logs during test
  ```bash
  docker logs -f <container_id>
  ```
  - Look for: "Memory limit set to X GB"
  - Look for: "ONNX model loaded (CPU-only)"
  - No errors or crashes

## Production Deployment

### Build for Production
- [ ] Tag image with version
  ```bash
  docker tag doctown-embedding-worker:cpu your-registry/doctown-embedding-worker:cpu-v2
  ```

- [ ] Push to container registry
  ```bash
  docker push your-registry/doctown-embedding-worker:cpu-v2
  ```

### RunPod Setup
- [ ] Create or update CPU pod
  - Select: Shared CPU or dedicated CPU
  - Recommended: 16GB+ RAM
  - Port: 8000
  - Image: your-registry/doctown-embedding-worker:cpu-v2

- [ ] Configure environment (if needed)
  ```
  EMBEDDING_MAX_MEMORY_PERCENT=70.0
  EMBEDDING_MAX_BATCH_SIZE=64
  EMBEDDING_ADAPTIVE_BATCHING=true
  ```

- [ ] Wait for pod to start
  - Check logs for successful startup
  - Look for "ONNX model loaded"

- [ ] Test health from external URL
  ```bash
  curl https://your-pod-url.runpod.net/health
  ```

### Integration Testing
- [ ] Update ingest worker with new embedding URL
  - In your `.env` or config:
    ```
    EMBEDDING_WORKER_URL=https://your-pod-url.runpod.net
    ```

- [ ] Test full pipeline with small repo
  ```bash
  # Submit a small repo through your UI/API
  # Watch for embedding events
  ```

- [ ] Test with medium repo (~200 files)
  - Monitor embedding worker logs
  - Verify adaptive batching works
  - Check processing time

- [ ] Test with large repo (1000+ files)
  - This is the critical test
  - Should NOT crash
  - May be slow but should complete
  - Watch memory usage in RunPod dashboard

## Monitoring

### Initial 24 Hours
- [ ] Check logs every few hours
  ```bash
  # Via RunPod dashboard or:
  docker logs <container_id> --tail 100
  ```

- [ ] Monitor memory usage
  - RunPod dashboard shows memory graph
  - Should stay below 70% most of the time

- [ ] Check for errors
  ```bash
  docker logs <container_id> | grep ERROR
  docker logs <container_id> | grep CRITICAL
  ```

- [ ] Verify batch size adaptation
  ```bash
  docker logs <container_id> | grep "batch size"
  # Should see increases on small batches
  # Should see decreases if memory pressure
  ```

### Ongoing Monitoring
- [ ] Set up alerting (if available)
  - Alert on pod restarts
  - Alert on high error rate
  - Alert on memory > 90%

- [ ] Weekly checks
  - Review logs for patterns
  - Check memory usage trends
  - Adjust settings if needed

## Performance Validation

### Benchmarks
- [ ] Small repo (< 50 files)
  - Time: ___ seconds
  - Memory peak: ___ GB
  - Notes: ___

- [ ] Medium repo (100-200 files)
  - Time: ___ seconds
  - Memory peak: ___ GB
  - Notes: ___

- [ ] Large repo (500-1000 files)
  - Time: ___ seconds
  - Memory peak: ___ GB
  - Notes: ___

- [ ] Giant repo (2000+ files)
  - Time: ___ seconds
  - Memory peak: ___ GB
  - Notes: ___

### Acceptance Criteria
- ✅ No crashes on any repo size
- ✅ Memory stays below 80% threshold
- ✅ Completes within reasonable time (even if slow)
- ✅ System remains responsive
- ✅ Logs show intelligent batching behavior

## Rollback Plan

If issues occur:

### Quick Rollback
- [ ] Revert to previous Docker image
  ```bash
  # In RunPod, change image to previous version
  # Or pull old image:
  docker pull your-registry/doctown-embedding-worker:cpu-v1
  ```

### Configuration Adjustment
- [ ] Reduce memory threshold
  ```
  EMBEDDING_MAX_MEMORY_PERCENT=60.0
  ```

- [ ] Reduce max batch size
  ```
  EMBEDDING_MAX_BATCH_SIZE=32
  ```

- [ ] Disable adaptive batching (fixed size)
  ```
  EMBEDDING_ADAPTIVE_BATCHING=false
  EMBEDDING_MAX_BATCH_SIZE=16
  ```

### Code Rollback
- [ ] Git revert changes
  ```bash
  git log  # Find commit hash
  git revert <commit-hash>
  git push
  ```

## Success Metrics

### Technical Success
- [ ] Zero crashes in first week
- [ ] Memory usage predictable
- [ ] All repo sizes complete successfully
- [ ] Adaptive batching works as expected

### Business Success
- [ ] Cost: $0.07/hour × 24 × 7 = $11.76/week
- [ ] Runway: 10+ weeks on budget
- [ ] User satisfaction: Repos process without errors
- [ ] System stability: 99%+ uptime

## Next Steps After Deployment

- [ ] Document actual performance numbers
- [ ] Update cost tracking spreadsheet
- [ ] Monitor user feedback on processing times
- [ ] Consider optimizations if needed:
  - Caching for repeated repos
  - Parallel workers if volume increases
  - Premium GPU tier for paid users

## Notes & Observations

### Deployment Date: ___________

### Performance Notes:
- 
- 
- 

### Issues Encountered:
- 
- 
- 

### Optimizations Made:
- 
- 
- 

## Sign-Off

- [ ] Local testing complete
- [ ] Production deployed
- [ ] Integration testing complete
- [ ] Monitoring in place
- [ ] Documentation updated
- [ ] Team notified

**Deployed by:** ___________  
**Date:** ___________  
**Status:** ___________
