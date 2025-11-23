#!/usr/bin/env bash

# Doctown v5 Development Environment Stop Script
# Stops all running development services

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

success() {
    echo -e "${GREEN}✓${NC} $1"
}

info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

echo -e "${BLUE}Stopping Doctown v5 development services...${NC}"
echo ""

# Stop backend
if [ -f ".backend.pid" ]; then
    BACKEND_PID=$(cat .backend.pid)
    if kill -0 $BACKEND_PID 2>/dev/null; then
        kill $BACKEND_PID
        success "Backend stopped (PID: $BACKEND_PID)"
    else
        info "Backend not running"
    fi
    rm .backend.pid
fi

# Stop frontend
if [ -f ".frontend.pid" ]; then
    FRONTEND_PID=$(cat .frontend.pid)
    if kill -0 $FRONTEND_PID 2>/dev/null; then
        kill $FRONTEND_PID
        success "Frontend stopped (PID: $FRONTEND_PID)"
    else
        info "Frontend not running"
    fi
    rm .frontend.pid
fi

# Clean up any remaining processes on the ports
for PORT in 3000 5173; do
    PID=$(lsof -ti:$PORT 2>/dev/null || true)
    if [ ! -z "$PID" ]; then
        kill -9 $PID 2>/dev/null || true
        success "Cleaned up process on port $PORT"
    fi
done

echo ""
success "All services stopped"
