#!/bin/bash
# CEREBRO Intelligence System - Startup Script
# Starts both backend (FastAPI) and frontend (React)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}"
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║              🧠 CEREBRO Intelligence System                   ║"
echo "║                  Central Trust Point                          ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Configuration
export CEREBRO_ARCH_PATH="${CEREBRO_ARCH_PATH:-/home/kernelcore/arch}"
export CEREBRO_DATA_DIR="${CEREBRO_DATA_DIR:-./data/intelligence}"

echo -e "${YELLOW}Configuration:${NC}"
echo "  ARCH_PATH: $CEREBRO_ARCH_PATH"
echo "  DATA_DIR:  $CEREBRO_DATA_DIR"
echo ""

# Create data directories
mkdir -p "$CEREBRO_DATA_DIR/embeddings"

# Function to start backend
start_backend() {
    echo -e "${GREEN}Starting Backend (FastAPI)...${NC}"
    cd "$SCRIPT_DIR"

    # Check if poetry is available
    if command -v poetry &> /dev/null; then
        poetry run uvicorn phantom.api.server:app --host 0.0.0.0 --port 8000 --reload &
    else
        python -m uvicorn phantom.api.server:app --host 0.0.0.0 --port 8000 --reload &
    fi

    BACKEND_PID=$!
    echo "Backend started with PID: $BACKEND_PID"
}

# Function to start frontend
start_frontend() {
    echo -e "${GREEN}Starting Frontend (React)...${NC}"
    cd "$SCRIPT_DIR/dashboard"

    # Check if node_modules exists
    if [ ! -d "node_modules" ]; then
        echo "Installing frontend dependencies..."
        npm install
    fi

    npm run dev &
    FRONTEND_PID=$!
    echo "Frontend started with PID: $FRONTEND_PID"
}

# Handle termination
cleanup() {
    echo ""
    echo -e "${YELLOW}Shutting down CEREBRO...${NC}"
    kill $BACKEND_PID 2>/dev/null || true
    kill $FRONTEND_PID 2>/dev/null || true
    echo -e "${GREEN}CEREBRO shutdown complete${NC}"
    exit 0
}

trap cleanup SIGINT SIGTERM

# Start services
start_backend
sleep 2
start_frontend

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}CEREBRO is running!${NC}"
echo ""
echo "  🌐 Dashboard: http://localhost:3000"
echo "  🔌 API:       http://localhost:8000"
echo "  📚 API Docs:  http://localhost:8000/docs"
echo ""
echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"

# Wait for processes
wait
