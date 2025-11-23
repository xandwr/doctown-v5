#!/bin/bash

# Full development setup for M2.4 pipeline
# Starts ingest, embedding, assembly workers, and website

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Starting Doctown v5 Full Pipeline...${NC}"

# Ports used by services
PORTS=(3000 8000 3001 5173)

# Function to kill processes on ports
kill_port() {
    local port=$1
    local pids=$(lsof -ti:$port 2>/dev/null || true)
    if [ -n "$pids" ]; then
        echo -e "${YELLOW}Killing process on port $port (PID: $pids)${NC}"
        kill -9 $pids 2>/dev/null || true
    fi
}

# Function to cleanup on exit
cleanup() {
    echo -e "\n${YELLOW}Shutting down services...${NC}"
    
    # Kill background jobs
    kill $(jobs -p) 2>/dev/null || true
    
    # Kill any lingering processes on our ports
    for port in "${PORTS[@]}"; do
        kill_port $port
    done
    
    echo -e "${GREEN}Server shutdown complete${NC}"
    exit
}

trap cleanup SIGINT SIGTERM

# Clean up any existing processes on our ports before starting
echo -e "${YELLOW}Checking for lingering processes...${NC}"
for port in "${PORTS[@]}"; do
    kill_port $port
done
sleep 1

# Check if embedding worker dependencies are installed
if [ ! -d "workers/embedding/.venv" ]; then
    echo -e "${YELLOW}Setting up embedding worker...${NC}"
    cd workers/embedding
    python3 -m venv .venv
    source .venv/bin/activate
    pip install -e .
    cd ../..
fi

# Check if website dependencies are installed
if [ ! -d "website/node_modules" ]; then
    echo -e "${YELLOW}Installing website dependencies...${NC}"
    cd website
    npm install
    cd ..
fi

# Start ingest worker (builder)
echo -e "${GREEN}[1/4] Starting Ingest Worker on port 3000...${NC}"
cargo build --bin builder 2>&1 | grep -v "Compiling\|Finished" || true
PORT=3000 cargo run --bin builder &
INGEST_PID=$!
sleep 2

# Start embedding worker
echo -e "${GREEN}[2/4] Starting Embedding Worker on port 8000...${NC}"
cd workers/embedding
source .venv/bin/activate
PORT=8000 uvicorn app.main:app --reload &
EMBEDDING_PID=$!
cd ../..
sleep 2

# Start assembly worker
echo -e "${GREEN}[3/4] Starting Assembly Worker on port 3001...${NC}"
PORT=3001 cargo run --bin assembly-server &
ASSEMBLY_PID=$!
sleep 2

# Start website
echo -e "${GREEN}[4/4] Starting Website on port 5173...${NC}"
cd website
npm run dev &
WEBSITE_PID=$!
cd ..

echo -e "\n${BLUE}════════════════════════════════════════${NC}"
echo -e "${GREEN}✓ All services running!${NC}"
echo -e "${BLUE}════════════════════════════════════════${NC}"
echo -e "  Ingest Worker:    ${YELLOW}http://localhost:3000${NC}"
echo -e "  Embedding Worker: ${YELLOW}http://localhost:8000${NC}"
echo -e "  Assembly Worker:  ${YELLOW}http://localhost:3001${NC}"
echo -e "  Website:          ${YELLOW}http://localhost:5173${NC}"
echo -e "${BLUE}════════════════════════════════════════${NC}"
echo -e "\nPress ${RED}Ctrl+C${NC} to stop all services\n"

# Wait for all background jobs
wait
