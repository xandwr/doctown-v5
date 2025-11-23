# Doctown v5 - Development Setup Guide

This guide will help you set up and run the complete Doctown v5 development environment locally.

## Prerequisites

Before you begin, ensure you have the following installed:

- **Rust** (latest stable) - [Install from rustup.rs](https://rustup.rs/)
- **Node.js** (v18 or later) - [Install from nodejs.org](https://nodejs.org/)
- **npm** (comes with Node.js)

## Quick Start

### 1. Clone the Repository

```bash
git clone https://github.com/xandwr/doctown-v5.git
cd doctown-v5
```

### 2. Configure Environment Variables

The repository includes a `.env` file with configuration. Update it if needed:

```bash
# Edit .env with your API keys if required
nano .env
```

### 3. Run the Development Environment

The easiest way to start everything:

```bash
./dev.sh
```

This script will:
- ✓ Check all prerequisites
- ✓ Verify environment configuration
- ✓ Check for port conflicts
- ✓ Install frontend dependencies
- ✓ Build and start the Rust backend (port 3000)
- ✓ Start the SvelteKit frontend (port 5173)

### 4. Access the Application

Once the script completes:

- **Frontend**: http://localhost:5173
- **Backend API**: http://127.0.0.1:3000
- **Health Check**: http://127.0.0.1:3000/health

## Architecture

```
doctown-v5/
├── builder/              # Rust backend entry point
├── crates/
│   ├── doctown-common/   # Shared types and utilities
│   ├── doctown-events/   # Event system
│   └── doctown-ingest/   # Main ingestion logic & API
├── website/              # SvelteKit frontend
├── dev.sh               # Development environment launcher
└── stop.sh              # Stop all services
```

## Manual Setup (Alternative)

If you prefer to run services manually:

### Backend (Rust API Server)

```bash
# Build and run
cd builder
cargo build
../target/debug/builder

# Or build in release mode for better performance
cargo build --release
../target/release/builder
```

The API server will start on `http://127.0.0.1:3000`

### Frontend (SvelteKit)

```bash
cd website
npm install
npm run dev
```

The frontend will start on `http://localhost:5173`

## Stopping Services

To stop all running development services:

```bash
./stop.sh
```

Or press `Ctrl+C` if running via `dev.sh` (will automatically cleanup).

## Viewing Logs

When running via `dev.sh`, logs are saved to:

```bash
# View backend logs
tail -f backend.log

# View frontend logs
tail -f frontend.log

# View both simultaneously (dev.sh does this automatically)
tail -f backend.log frontend.log
```

## Development Workflow

### Making Changes

**Frontend Changes** (SvelteKit):
- Edit files in `website/src/`
- Hot reload will automatically update the browser

**Backend Changes** (Rust):
- Edit files in `crates/` or `builder/`
- Stop the backend (Ctrl+C or `./stop.sh`)
- Restart via `./dev.sh`

### Running Tests

```bash
# Rust tests
cargo test

# Frontend tests
cd website
npm test
```

### Code Formatting

```bash
# Rust formatting
cargo fmt

# Frontend formatting
cd website
npm run format
```

### Linting

```bash
# Rust linting
cargo clippy

# Frontend linting
cd website
npm run lint
```

## Troubleshooting

### Port Already in Use

If you get a "port already in use" error:

```bash
# Find what's using the port
lsof -i :3000  # Backend
lsof -i :5173  # Frontend

# Kill the process
kill -9 <PID>

# Or use the stop script
./stop.sh
```

### Backend Won't Start

Check the backend logs:

```bash
cat backend.log
```

Common issues:
- Missing environment variables in `.env`
- Port 3000 already in use
- Rust compilation errors

### Frontend Won't Start

Check the frontend logs:

```bash
cat frontend.log
```

Common issues:
- Node modules not installed (`cd website && npm install`)
- Port 5173 already in use
- Missing dependencies

### CORS Errors

If you see CORS errors in the browser console:
- Ensure backend is running on port 3000
- Verify CORS origins are configured in `builder/src/main.rs`
- Check that frontend is accessing `http://localhost:3000`

## Configuration

### Backend Configuration

Edit `builder/src/main.rs` to change:
- Port (default: 3000)
- Host (default: 127.0.0.1)
- CORS origins
- Max request body size

### Frontend Configuration

Edit `website/vite.config.ts` to change Vite settings.

### Environment Variables

The `.env` file contains:
- `RUNPOD_BUILDER_ID` - RunPod builder endpoint ID
- `RUNPOD_API_KEY` - RunPod API key for remote building

## Additional Resources

- [Project Roadmap](TODO.md)
- [Architecture Documentation](README.md)
- [Event Specifications](specs/events.v1.md)

## Getting Help

If you encounter issues:
1. Check the logs (`backend.log`, `frontend.log`)
2. Review this guide's troubleshooting section
3. Check open issues on GitHub
4. Create a new issue with logs and steps to reproduce
