# R2 Storage Implementation

## Overview

Modified the Doctown v5 pipeline to package output files as `.docpack` format and upload them to Cloudflare R2 storage instead of storing everything in browser cache. This prevents memory issues on small devices and provides persistent storage for analysis results.

## Changes Made

### 1. Created Docpack Packaging Utility
**File:** `website/src/lib/docpack.ts`

- Implements `.docpack` format according to `specs/docpack.md`
- Creates manifest with metadata, checksums, and statistics
- Packages graph, nodes, clusters, and source maps
- Includes `parseRepoUrl()` helper to extract owner/name from GitHub URLs

### 2. Created R2 Upload API Endpoint
**File:** `website/src/routes/api/upload-docpack/+server.ts`

- SvelteKit server endpoint that handles docpack uploads
- Uses AWS S3 SDK to communicate with Cloudflare R2
- Stores docpacks at: `doctown-central/docpacks/[repo_owner]/[repo_name].docpack`
- Returns signed URL for download access

### 3. Updated Frontend Pipeline
**File:** `website/src/routes/+page.svelte`

**Added:**
- New pipeline stage: `uploading` (4th stage)
- `uploadDocpack()` function that:
  - Creates docpack from assembly results
  - Uploads to R2 via API endpoint
  - Clears memory-intensive data (embeddings, chunks) after upload
  - Stores docpack URL for download

**Modified:**
- Updated progress indicator to show 4 stages (was 3)
- Added docpack download UI with green success banner
- Shows download button and URL copy button when complete
- Displays docpack URL after successful upload

### 4. Environment Configuration
**File:** `website/src/hooks.server.ts` (new)
- Loads environment variables from `.env` on server startup

**File:** `website/.env.example` (new)
- Documents required environment variables
- Provides template for R2 credentials

### 5. Dependencies
**Added to `package.json`:**
- `@aws-sdk/client-s3` - For R2 (S3-compatible) uploads
- `@types/node` - Node.js type definitions

## Environment Variables Required

```bash
# R2 Storage Configuration
BUCKET_NAME=doctown-central
BUCKET_ACCESS_KEY_ID=your_access_key_id
BUCKET_SECRET_ACCESS_KEY=your_secret_access_key
BUCKET_S3_ENDPOINT=https://your-account-id.r2.cloudflarestorage.com
```

## Storage Path Structure

Docpacks are stored in R2 with the following structure:
```
doctown-central/
  docpacks/
    [repo_owner]/
      [repo_name].docpack
```

Example: `doctown-central/docpacks/facebook/react.docpack`

## Docpack Format

Each `.docpack` is a JSON file containing:
- `manifest.json` - Metadata, checksums, statistics
- `graph.json` - Semantic graph (nodes and edges)
- `nodes.json` - Symbol table with documentation
- `clusters.json` - Semantic clusters
- `source_map.json` - Maps chunks to source files

See `specs/docpack.md` for complete specification.

## Memory Management

The implementation now:
1. ✅ Stores only essential data in memory during pipeline
2. ✅ Clears `embeddings` Map after upload (largest data structure)
3. ✅ Clears `chunks` array after upload
4. ✅ Keeps only event summaries (not full event data)
5. ✅ Downloads docpack from R2 instead of browser cache

This significantly reduces memory usage on client devices, especially important for:
- Mobile devices
- Tablets
- Low-memory laptops
- Large repositories with many files

## Usage Flow

1. User submits repository URL
2. **Stage 1:** Ingest - Extract symbols and chunks
3. **Stage 2:** Embedding - Generate embeddings for chunks
4. **Stage 3:** Assembly - Create graph and clusters
5. **Stage 4:** Upload - Package as `.docpack` and upload to R2
6. **Complete:** Display download link and clear memory

## Testing

To test the implementation:

```bash
cd website
npm install
npm run dev
```

Then:
1. Enter a GitHub repository URL
2. Wait for all 4 pipeline stages to complete
3. Verify docpack download link appears
4. Click "Download .docpack" button
5. Verify file is downloaded from R2

## Future Enhancements

- [ ] Add docpack versioning (allow multiple versions per repo)
- [ ] Add docpack expiration/TTL for cost management
- [ ] Implement incremental updates (only re-analyze changed files)
- [ ] Add docpack browsing UI (view stored docpacks)
- [ ] Add docpack sharing via unique URLs
- [ ] Implement CDN caching for popular docpacks
