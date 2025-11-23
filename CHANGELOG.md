# Doctown v5 - Changelog

## 2025-11-23 - Initial Development Setup

### Added
- **Event Log Component** (`EventLog.svelte`)
  - Fixed performance issue with rendering thousands of individual event DOM elements
  - Terminal-style scrollable log viewer with auto-scroll functionality
  - Color-coded event types (started=blue, completed=green, failed=red, etc.)
  - Event summaries instead of full JSON dumps
  - Auto-scroll toggle with smart detection (disables when user scrolls up)
  - Live indicator when streaming events
  - Event count display
  - Jump to bottom button

### Changed
- **Main Page** (`+page.svelte`)
  - Replaced individual event cards with unified EventLog component
  - Reduced console logging (only logs started/completed events)
  - Auto-stops loading state when `completed` event received
  - Much better performance with large event streams

### Fixed
- **Tailwind CSS PostCSS Plugin**
  - Installed `@tailwindcss/postcss` for Tailwind CSS v4 compatibility
  - Updated `postcss.config.js` to use new plugin format

- **Backend API CORS & Endpoints**
  - Added GET endpoint support to `/ingest` (EventSource only supports GET)
  - Backend now accepts both POST (JSON) and GET (query params)
  - CORS properly configured for `http://localhost:5173`

### Performance Improvements
- Event log uses single scrollable container instead of N DOM elements
- Events stored in array but rendered efficiently in single container
- Reduced console logging spam (was logging every chunk creation)
- Auto-scroll only when user is at bottom (doesn't fight manual scrolling)

### Development Environment
- Created `builder/` binary crate for running API server
- API server runs on port 3000
- Frontend dev server on port 5173
- Created `SETUP_COMPLETE.md` with full documentation
- Updated `TODO.md` to reflect current progress (~85% of Milestone 1)

## Current Status

### Working Features
✅ Full ingest pipeline with GitHub repo fetching
✅ Tree-sitter parsing for Rust and Python
✅ Symbol extraction (functions, structs, classes, methods, etc.)
✅ Chunk creation with stable IDs
✅ SSE event streaming (backend → frontend)
✅ Real-time event log viewer
✅ Auto-reconnection on connection loss
✅ Error handling and cancellation

### Known Issues
- Event log can grow unbounded in memory (no event limit yet)
- No persistence of job results
- No visual symbol tree or file explorer yet

### Next Steps
1. Add event statistics summary (file count, chunk count, language breakdown)
2. Create symbol tree visualization
3. Add file explorer UI
4. Implement result persistence
5. Deploy to RunPod + Vercel
