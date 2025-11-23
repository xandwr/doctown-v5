#!/usr/bin/env bash

# Doctown v5 Development Environment Setup Script
# This script sets up and runs the complete local development environment

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BACKEND_PORT=3000
FRONTEND_PORT=5173
BACKEND_HOST="127.0.0.1"

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Doctown v5 Development Environment   ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# Function to print colored status messages
info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check if a port is in use
port_in_use() {
    lsof -Pi :$1 -sTCP:LISTEN -t >/dev/null 2>&1
}

# Function to cleanup on exit
cleanup() {
    echo ""
    info "Shutting down services..."
    
    # Kill background jobs
    jobs -p | xargs -r kill 2>/dev/null || true
    
    success "Cleanup complete"
    exit 0
}

# Trap Ctrl+C and cleanup
trap cleanup SIGINT SIGTERM

# ============================================================
# 1. Check Prerequisites
# ============================================================
echo -e "${BLUE}[1/6]${NC} Checking prerequisites..."

if ! command_exists cargo; then
    error "Rust/Cargo not found. Please install from https://rustup.rs/"
    exit 1
fi
success "Rust/Cargo found ($(cargo --version))"

if ! command_exists node; then
    error "Node.js not found. Please install from https://nodejs.org/"
    exit 1
fi
success "Node.js found ($(node --version))"

if ! command_exists npm; then
    error "npm not found. Please install Node.js from https://nodejs.org/"
    exit 1
fi
success "npm found ($(npm --version))"

# ============================================================
# 2. Check Environment Variables
# ============================================================
echo ""
echo -e "${BLUE}[2/6]${NC} Checking environment configuration..."

if [ ! -f ".env" ]; then
    warning ".env file not found"
    if [ -f ".env.example" ]; then
        info "Creating .env from .env.example..."
        cp .env.example .env
        warning "Please edit .env with your actual API keys"
    else
        error "No .env or .env.example found"
        exit 1
    fi
fi

# Load environment variables
if [ -f ".env" ]; then
    export $(grep -v '^#' .env | xargs)
    success "Environment variables loaded from .env"
fi

# ============================================================
# 3. Check for Port Conflicts
# ============================================================
echo ""
echo -e "${BLUE}[3/6]${NC} Checking for port conflicts..."

if port_in_use $BACKEND_PORT; then
    warning "Port $BACKEND_PORT is already in use"
    read -p "Kill the process using port $BACKEND_PORT? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        lsof -ti:$BACKEND_PORT | xargs kill -9 2>/dev/null || true
        sleep 1
        success "Freed port $BACKEND_PORT"
    else
        error "Cannot start backend on port $BACKEND_PORT"
        exit 1
    fi
else
    success "Port $BACKEND_PORT is available"
fi

if port_in_use $FRONTEND_PORT; then
    warning "Port $FRONTEND_PORT is already in use"
    read -p "Kill the process using port $FRONTEND_PORT? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        lsof -ti:$FRONTEND_PORT | xargs kill -9 2>/dev/null || true
        sleep 1
        success "Freed port $FRONTEND_PORT"
    else
        error "Cannot start frontend on port $FRONTEND_PORT"
        exit 1
    fi
else
    success "Port $FRONTEND_PORT is available"
fi

# ============================================================
# 4. Install Dependencies
# ============================================================
echo ""
echo -e "${BLUE}[4/6]${NC} Installing dependencies..."

# Frontend dependencies
if [ ! -d "website/node_modules" ]; then
    info "Installing frontend dependencies..."
    cd website
    npm install
    cd ..
    success "Frontend dependencies installed"
else
    success "Frontend dependencies already installed"
fi

# Backend dependencies (cargo will handle this on build)
success "Backend dependencies will be checked during build"

# ============================================================
# 5. Build and Start Backend (Rust API Server)
# ============================================================
echo ""
echo -e "${BLUE}[5/6]${NC} Starting backend API server..."

info "Building Rust backend..."

# Build in release mode for better performance, or debug for faster compilation
if [ "$1" = "--release" ]; then
    cargo build --release -p builder
    BACKEND_BIN="target/release/builder"
else
    cargo build -p builder
    BACKEND_BIN="target/debug/builder"
fi

if [ ! -f "$BACKEND_BIN" ]; then
    error "Backend binary not found at $BACKEND_BIN"
    exit 1
fi

info "Starting backend on http://$BACKEND_HOST:$BACKEND_PORT"
# Run backend in background, redirect output to a log file
$BACKEND_BIN > backend.log 2>&1 &
BACKEND_PID=$!
echo $BACKEND_PID > .backend.pid

# Wait for backend to start
sleep 2

# Check if backend is running
if ! kill -0 $BACKEND_PID 2>/dev/null; then
    error "Backend failed to start. Check backend.log for details"
    cat backend.log
    exit 1
fi

success "Backend running (PID: $BACKEND_PID)"

# ============================================================
# 6. Start Frontend (SvelteKit)
# ============================================================
echo ""
echo -e "${BLUE}[6/6]${NC} Starting frontend development server..."

cd website

info "Starting SvelteKit dev server on http://localhost:$FRONTEND_PORT"
# Run frontend in background
npm run dev > ../frontend.log 2>&1 &
FRONTEND_PID=$!
echo $FRONTEND_PID > ../.frontend.pid

cd ..

# Wait for frontend to start
sleep 3

# Check if frontend is running
if ! kill -0 $FRONTEND_PID 2>/dev/null; then
    error "Frontend failed to start. Check frontend.log for details"
    cat frontend.log
    exit 1
fi

success "Frontend running (PID: $FRONTEND_PID)"

# ============================================================
# Development Environment Ready
# ============================================================
echo ""
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║   Development Environment Ready! 🚀   ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
echo ""
echo -e "  ${BLUE}Frontend:${NC}  http://localhost:$FRONTEND_PORT"
echo -e "  ${BLUE}Backend:${NC}   http://$BACKEND_HOST:$BACKEND_PORT"
echo -e "  ${BLUE}Health:${NC}    http://$BACKEND_HOST:$BACKEND_PORT/health"
echo ""
echo -e "${YELLOW}Logs:${NC}"
echo -e "  Backend:  tail -f backend.log"
echo -e "  Frontend: tail -f frontend.log"
echo ""
echo -e "${YELLOW}Press Ctrl+C to stop all services${NC}"
echo ""

# Tail both logs in the foreground
tail -f backend.log frontend.log
