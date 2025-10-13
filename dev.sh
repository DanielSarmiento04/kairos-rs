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

# Check if cargo-leptos is installed
if ! command -v cargo-leptos &> /dev/null; then
    echo -e "${YELLOW}⚠️  cargo-leptos not found. Installing...${NC}"
    cargo install cargo-leptos
fi

# Check if wasm32 target is installed
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    echo -e "${YELLOW}⚠️  WebAssembly target not found. Installing...${NC}"
    rustup target add wasm32-unknown-unknown
fi

# Function to run gateway
start_gateway() {
    echo -e "${GREEN}📡 Starting Kairos Gateway on port 5900...${NC}"
    cd "$(dirname "$0")"
    cargo run --bin kairos-gateway
}

# Function to run UI
start_ui() {
    echo -e "${GREEN}🎨 Starting Kairos UI on port 3000...${NC}"
    cd "$(dirname "$0")/crates/kairos-ui"
    
    # First, try to build the WASM version
    echo -e "${BLUE}📦 Building WASM components...${NC}"
    if cargo build --lib --target wasm32-unknown-unknown; then
        echo -e "${GREEN}✅ WASM build successful!${NC}"
        
        # Try cargo leptos serve
        echo -e "${BLUE}🚀 Attempting to start Leptos server...${NC}"
        if timeout 10s cargo leptos serve 2>/dev/null; then
            echo -e "${GREEN}✅ Leptos server started successfully!${NC}"
        else
            echo -e "${YELLOW}⚠️  Leptos server failed, using fallback Python server...${NC}"
            
            # Build the site directory if needed
            if [ ! -d "target/site" ]; then
                echo -e "${BLUE}📦 Building site directory...${NC}"
                cargo leptos build 2>/dev/null || true
            fi
            
            # Use Python fallback server
            if [ -f "serve.py" ]; then
                echo -e "${BLUE}🐍 Starting Python server...${NC}"
                python3 serve.py
            else
                echo -e "${BLUE}🌐 Starting simple HTTP server...${NC}"
                cd target/site 2>/dev/null || mkdir -p target/site
                python3 -m http.server 3000
            fi
        fi
    else
        echo -e "${RED}❌ WASM build failed. Please check your code.${NC}"
        return 1
    fi
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
    echo "  ui          Start only the UI server (port 3000)"  
    echo "  both        Start both gateway and UI (default)"
    echo "  help        Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0              # Start both services"
    echo "  $0 gateway      # Start only gateway"
    echo "  $0 ui           # Start only UI"
    echo ""
    echo "URLs:"
    echo "  Gateway:  http://localhost:5900"
    echo "  UI:       http://localhost:3000"
    echo "  Health:   http://localhost:5900/health"
    echo "  Metrics:  http://localhost:5900/metrics"
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
    "both")
        echo -e "${BLUE}🔄 Starting both Gateway and UI...${NC}"
        echo -e "${YELLOW}💡 Use Ctrl+C to stop both services${NC}"
        echo ""
        
        # Start gateway in background
        start_gateway &
        GATEWAY_PID=$!
        
        # Wait a moment for gateway to start
        echo -e "${BLUE}⏳ Waiting for gateway to start...${NC}"
        sleep 3
        
        # Start UI in background
        start_ui &
        UI_PID=$!
        
        echo ""
        echo -e "${GREEN}✅ Both services started!${NC}"
        echo -e "${BLUE}📡 Gateway: http://localhost:5900${NC}"
        echo -e "${BLUE}🎨 UI:      http://localhost:3000${NC}"
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