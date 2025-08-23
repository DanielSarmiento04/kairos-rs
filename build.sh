#!/bin/bash

# Build script for optimized Rust Docker images
# Usage: ./build.sh [standard|chef]

set -e

APP_NAME="Ben"
IMAGE_NAME="ben-app"
RUST_VERSION="1.87.0"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

print_section() {
    echo -e "${GREEN}=== $1 ===${NC}"
}

print_info() {
    echo -e "${YELLOW}INFO: $1${NC}"
}

print_error() {
    echo -e "${RED}ERROR: $1${NC}"
}

# Function to build with standard optimized Dockerfile
build_standard() {
    print_section "Building with Standard Optimized Dockerfile"
    
    docker build \
        --build-arg RUST_VERSION=${RUST_VERSION} \
        --build-arg APP_NAME=${APP_NAME} \
        --tag ${IMAGE_NAME}:standard \
        --file Dockerfile \
        .
    
    print_info "Standard build completed"
}

# Function to build with cargo-chef optimized Dockerfile
build_chef() {
    print_section "Building with Cargo Chef Optimized Dockerfile"
    
    docker build \
        --build-arg RUST_VERSION=${RUST_VERSION} \
        --build-arg APP_NAME=${APP_NAME} \
        --tag ${IMAGE_NAME}:chef \
        --file Dockerfile.chef \
        .
    
    print_info "Chef build completed"
}

# Function to show image sizes
show_sizes() {
    print_section "Image Size Comparison"
    
    echo "Image sizes:"
    docker images | grep ${IMAGE_NAME} | awk '{print $1":"$2 " - " $7$8}'
}

# Function to run security scan
security_scan() {
    print_section "Security Scan Results"
    
    if command -v dive &> /dev/null; then
        print_info "Running dive analysis..."
        dive ${IMAGE_NAME}:standard --ci
    else
        print_info "Install 'dive' for detailed image analysis: https://github.com/wagoodman/dive"
    fi
}

# Main execution
case "${1:-standard}" in
    "standard")
        build_standard
        show_sizes
        ;;
    "chef")
        build_chef
        show_sizes
        ;;
    "both")
        build_standard
        build_chef
        show_sizes
        ;;
    "scan")
        security_scan
        ;;
    *)
        echo "Usage: $0 [standard|chef|both|scan]"
        echo "  standard - Build with standard optimized Dockerfile"
        echo "  chef     - Build with cargo-chef optimized Dockerfile"  
        echo "  both     - Build both versions for comparison"
        echo "  scan     - Run security scan on standard image"
        exit 1
        ;;
esac

print_section "Build Process Complete"
print_info "To run the container: docker run -p 5900:5900 ${IMAGE_NAME}:standard"
