# Quick Setup Guide: R2 Storage for Docpacks

## What Changed?

Your Doctown pipeline now uploads finished `.docpack` files to Cloudflare R2 storage instead of keeping everything in browser memory. This fixes memory issues on small devices and provides permanent storage.

## Setup Steps

### 1. Install Dependencies (Already Done)
```bash
cd website
npm install
```

Dependencies installed:
- `@aws-sdk/client-s3` - For uploading to R2
- `@types/node` - Node.js types

### 2. Environment Variables (Already Configured)

Your `.env` file already has the R2 credentials:
```bash
BUCKET_NAME=doctown-central
BUCKET_ACCESS_KEY_ID=b221d330e5ce290d8e7d4be5b620cffd
BUCKET_SECRET_ACCESS_KEY=0b71e9b5a12c61d309d9846e35043fe8c67a673c8772bfd69b2ba7226be1186e
BUCKET_S3_ENDPOINT=https://b7ad51fbe0bb3cdb415760370578c9d0.r2.cloudflarestorage.com
```

⚠️ **Important:** Your `.env` file contains sensitive credentials. Make sure it's in `.gitignore` (it already is).

### 3. Test the Implementation

Start the dev server:
```bash
cd website
npm run dev
```

Then:
1. Open http://localhost:5173
2. Submit a GitHub repository URL
3. Watch the pipeline progress through 4 stages:
   - Stage 1/4: Ingesting files
   - Stage 2/4: Generating embeddings
   - Stage 3/4: Building graph and clusters
   - **Stage 4/4: Uploading .docpack to R2** (NEW!)
4. When complete, you'll see a green success banner with:
   - Download .docpack button
   - Copy URL button
   - The R2 URL where the docpack is stored

## Where Are Docpacks Stored?

R2 Bucket: `doctown-central`

Path structure:
```
doctown-central/
  docpacks/
    facebook/
      react.docpack
    vercel/
      next.js.docpack
    your-username/
      your-repo.docpack
```

Pattern: `docpacks/[repo_owner]/[repo_name].docpack`

## What's in a .docpack File?

A `.docpack` is a JSON file containing:
- **manifest.json** - Metadata, checksums, statistics
- **graph.json** - Semantic graph with nodes and edges
- **nodes.json** - Symbol table with AI-generated documentation
- **clusters.json** - Semantic groupings of related code
- **source_map.json** - Maps chunks back to original source files

See `specs/docpack.md` for the complete specification.

## Memory Savings

**Before:** Everything stored in browser cache
- ❌ Large repos crashed on mobile/tablets
- ❌ Embeddings consumed 100s of MBs
- ❌ No way to share or persist results

**After:** Only essentials in memory, rest on R2
- ✅ Lightweight client memory usage
- ✅ Works on all devices
- ✅ Results persisted permanently
- ✅ Shareable via R2 URLs

## New Files Created

1. `website/src/lib/docpack.ts` - Docpack packaging utilities
2. `website/src/routes/api/upload-docpack/+server.ts` - R2 upload endpoint
3. `website/src/hooks.server.ts` - Environment variable loader
4. `website/.env.example` - Environment template
5. `R2_STORAGE_IMPLEMENTATION.md` - Detailed implementation docs

## Modified Files

1. `website/src/routes/+page.svelte` - Added upload stage and download UI
2. `website/package.json` - Added AWS SDK dependencies

## Troubleshooting

### "Failed to upload docpack" Error
- Check that R2 credentials in `.env` are correct
- Verify the bucket exists in Cloudflare R2 dashboard
- Check the endpoint URL matches your account

### Memory Still High?
- The implementation now clears embeddings and chunks after upload
- If you still see high memory, check browser DevTools → Memory
- Large repositories may still require significant memory during processing

### Download Link Not Appearing?
- Check browser console for errors
- Verify all 4 pipeline stages completed successfully
- Check that docpackUrl is set in the component state

## Next Steps

1. Test with a small repository first
2. Verify the docpack appears in your R2 bucket
3. Download and inspect the .docpack JSON file
4. Try with larger repositories to verify memory improvements

## Questions?

See the detailed implementation docs in `R2_STORAGE_IMPLEMENTATION.md`
