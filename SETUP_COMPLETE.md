# Doctown v5 Development Environment - Setup Complete

## ✅ What Was Fixed

### 1. Tailwind CSS PostCSS Plugin
- **Problem**: Tailwind CSS v4 moved PostCSS plugin to separate package
- **Solution**: Installed `@tailwindcss/postcss` and updated `postcss.config.js`

### 2. Backend API Endpoint Mismatch
- **Problem**: Frontend using GET requests with query params, backend only accepting POST with JSON body
- **Solution**: Added GET endpoint handler to support both POST (JSON) and GET (query params)

## 🚀 Services Running

### Backend API (Port 3000)
- **Binary**: `./target/release/builder`
- **Endpoints**:
  - `GET /health` - Health check
  - `GET /ingest?repo_url=<url>&job_id=<id>&git_ref=<ref>` - Ingest with SSE streaming
  - `POST /ingest` - Ingest with JSON body (alternative)
- **CORS**: Configured for `http://localhost:5173`

### Frontend Dev Server (Port 5173)
- **Command**: `npm run dev` (in website/ directory)
- **Framework**: SvelteKit + Vite
- **Styling**: Tailwind CSS v4

## 📝 Quick Start Commands

### Start Everything
```bash
# Terminal 1 - Backend
cd /home/xander/Documents/doctown-v5
RUST_LOG=info ./target/release/builder

# Terminal 2 - Frontend
cd /home/xander/Documents/doctown-v5/website
npm run dev
```

### Test Backend
```bash
# Health check
curl http://localhost:3000/health

# Test ingest endpoint (will process real GitHub repo)
curl -N "http://localhost:3000/ingest?repo_url=https://github.com/xandwr/localdoc&job_id=job_test123"
```

## 🔧 Development Workflow

1. **Start backend**: Run the builder binary on port 3000
2. **Start frontend**: Run npm dev server on port 5173
3. **Make changes**:
   - Rust changes: Rebuild with `cargo build --release` (or in builder/ dir)
   - Frontend changes: Hot reload automatically
4. **Test**: Access http://localhost:5173 in browser

## 📦 Project Structure

```
doctown-v5/
├── builder/                 # Binary crate for API server
│   └── src/main.rs         # Entry point (port 3000)
├── crates/
│   ├── doctown-common/     # Shared types and IDs
│   ├── doctown-events/     # Event envelope system
│   └── doctown-ingest/     # Core ingest logic + API
│       └── src/api.rs      # API endpoints (GET & POST /ingest)
└── website/                 # SvelteKit frontend
    ├── src/
    │   ├── lib/
    │   │   └── sse-client.ts  # SSE client for event streaming
    │   └── routes/
    │       └── +page.svelte    # Main UI
    └── postcss.config.js       # Updated for @tailwindcss/postcss
```

## 🐛 Troubleshooting

### Backend Issues
- **Port 3000 in use**: Kill process with `pkill -f builder`
- **Build errors**: Run `cargo clean` then rebuild
- **CORS errors**: Check builder/src/main.rs CORS configuration

### Frontend Issues
- **Tailwind not working**: Verify `@tailwindcss/postcss` is installed
- **SSE connection fails**: Ensure backend is running on port 3000
- **Hot reload not working**: Restart Vite dev server

## 📊 What's Working

✅ Backend API server on port 3000
✅ Frontend dev server on port 5173  
✅ CORS configured correctly
✅ SSE streaming from backend to frontend
✅ GitHub repo ingestion pipeline
✅ Tree-sitter parsing (Rust, Python)
✅ Symbol extraction and chunking
✅ Event envelope system

## 🎯 Next Steps

1. Test full repo ingestion flow in browser
2. Add error handling UI for failed ingests
3. Display parsed symbols in frontend
4. Add progress indicators for long operations
5. Implement job status persistence

## 📝 Notes

- Job IDs must start with `job_` and be 8-64 chars (alphanumeric + underscore)
- SSE events follow the doctown-events envelope format
- Pipeline supports Rust and Python files (more languages can be added)
- Unsupported files are skipped with `file_skipped` events
