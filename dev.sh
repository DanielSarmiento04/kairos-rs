#!/bin/bash

# Kairos-rs Development Startup Script

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Starting Kairos-rs Development Environment${NC}"
echo ""

# Function to run gateway
start_gateway() {
    echo -e "${GREEN}📡 Starting Kairos Gateway on port 5900...${NC}"
    cd "$(dirname "$0")"
    cargo run --bin kairos-gateway
}

# Function to run UI
start_ui() {
    echo -e "${GREEN}🎨 Starting Kairos Vite UI (http://localhost:5173)...${NC}"
    cd "$(dirname "$0")/frontend/kairos-ui"
    npm run dev
}

# Function to test project
run_tests() {
    echo -e "${BLUE}🧪 Running Rust Workspace Tests (cargo test --workspace)...${NC}"
    cd "$(dirname "$0")"
    cargo test --workspace
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Rust tests failed.${NC}"
        return 1
    fi
    echo -e "${GREEN}✅ Rust tests passed!${NC}"
    echo ""
    echo -e "${BLUE}🧪 Running Frontend Tests (Vitest + Vue-TSC)...${NC}"
    cd "$(dirname "$0")/frontend/kairos-ui"
    npm run test:unit -- --run && npm run type-check
    if [ $? -ne 0 ]; then
        echo -e "${RED}❌ Frontend tests failed.${NC}"
        return 1
    fi
    echo -e "${GREEN}✅ All frontend tests passed!${NC}"
}

# Function to show help
show_help() {
    echo "Kairos-rs Development Startup Script"
    echo ""
    echo "Usage:"
    echo "  $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  gateway     Start only the API gateway (port 5900)"
    echo "  ui          Start only the Vite UI server (port 5173)"
    echo "  both        Start both gateway and UI (default)"
    echo "  test        Run full test suite (Rust workspace + Vite UI)"
    echo "  help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0              # Start both services"
    echo "  $0 gateway      # Start only gateway"
    echo "  $0 ui           # Start only UI"
    echo "  $0 test         # Run all tests"
    echo ""
    echo "URLs:"
    echo "  Gateway:    http://localhost:5900"
    echo "  UI:         http://localhost:5173"
    echo "  Health:     http://localhost:5900/health"
    echo "  Metrics:    http://localhost:5900/metrics"
    echo "  Playground: http://localhost:5173/playground"
    echo ""
}

# Function to cleanup background processes
cleanup() {
    echo ""
    echo -e "${YELLOW}🛑 Stopping services...${NC}"
    if [ ! -z "$GATEWAY_PID" ]; then
        kill $GATEWAY_PID 2>/dev/null || true
    fi
    if [ ! -z "$UI_PID" ]; then
        kill $UI_PID 2>/dev/null || true
    fi
    exit 0
}

# Set up signal handling
trap cleanup SIGINT SIGTERM

# Parse command line arguments
case "${1:-both}" in
    "gateway")
        start_gateway
        ;;
    "ui")
        start_ui
        ;;
    "test")
        run_tests
        ;;
    "both")
        echo -e "${BLUE}🔄 Starting both Gateway and UI...${NC}"
        echo -e "${YELLOW}💡 Use Ctrl+C to stop both services${NC}"
        echo ""
        
        # Start gateway in background
        start_gateway &
        GATEWAY_PID=$!
        
        # Wait a moment for gateway to start
        echo -e "${BLUE}⏳ Waiting for gateway to start...${NC}"
        sleep 2
        
        # Start UI in background
        start_ui &
        UI_PID=$!
        
        echo ""
        echo -e "${GREEN}✅ Both services started!${NC}"
        echo -e "${BLUE}📡 Gateway: http://localhost:5900${NC}"
        echo -e "${BLUE}🎨 UI:      http://localhost:5173${NC}"
        echo ""
        echo -e "${YELLOW}💡 Press Ctrl+C to stop both services${NC}"
        
        # Wait for both processes
        wait $GATEWAY_PID $UI_PID
        ;;
    "help"|"-h"|"--help")
        show_help
        ;;
    *)
        echo -e "${RED}❌ Unknown command: $1${NC}"
        echo ""
        show_help
        exit 1
        ;;
esac