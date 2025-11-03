# Release Notes - Kairos-rs v0.2.10

**Release Date:** October 30, 2025  
**Status:** Production Ready

## 🎯 Overview

Version 0.2.10 focuses on containerization improvements, deployment automation, and enhanced developer experience. This release brings production-ready Docker multi-platform support and automated versioning workflows.

---

## ✨ New Features

### 🐳 Docker & Container Improvements

#### Multi-Platform Container Images
- **AMD64 and ARM64 Support**: Container images now build for both Intel/AMD processors and Apple Silicon (ARM64)
- **GitHub Container Registry**: Automated publishing to `ghcr.io/danielsarmiento04/kairos-rs`
- **Platform-specific optimizations**: Native performance on both architectures

#### Automated Version Tagging
- **Cargo.toml-based versioning**: Docker images automatically tagged with version from `crates/kairos-gateway/Cargo.toml`
- **Multiple tag strategies**:
  - Semantic versioning: `0.2.10`, `0.2`, `0`
  - Branch tags: `main`, `ws`
  - Commit SHA: `sha-abc123`
  - Latest tag for main branch
- **No manual version management needed**: Version bumps in Cargo.toml automatically propagate to container images

#### Distroless Debug Images
- **Minimal footprint**: Based on `gcr.io/distroless/cc-debian12:debug`
- **Shell access for debugging**: Includes busybox shell (`sh`) for troubleshooting
- **Security-first**: No package managers or unnecessary tools in production images
- **Small size**: ~28MB total image size (vs ~25MB for regular distroless)

### 📚 Documentation Enhancements

#### Organized Documentation Structure
- **docs/ directory**: Centralized documentation location
  - `WEBSOCKET_GUIDE.md`: Comprehensive WebSocket proxy documentation
  - `POSTMAN_GUIDE.md`: API testing guide
- **realease/ directory**: Version-specific release notes
- **Clear navigation**: Updated README with links to all documentation

#### Docker Deployment Guide
- **Quick Start with Docker**: Step-by-step container deployment
- **Docker Compose examples**: Production-ready compose configurations
- **Debugging instructions**: How to exec into containers and troubleshoot
- **Multi-platform guidance**: Platform-specific deployment notes

---

## 🔧 Improvements

### Build & CI/CD

#### GitHub Actions Workflows
- **docker-publish.yml**: Enhanced with Cargo.toml version extraction
- **docker-image.yml**: Updated for multi-platform builds
- **Automated tagging**: Version tags created automatically on push

#### Docker Configuration
- **Optimized Dockerfile**: Multi-stage build with layer caching
- **BuildKit support**: Faster builds with cache mounts
- **Minimal runtime**: Distroless base for security and size
- **.dockerignore**: Optimized to exclude unnecessary files

### Developer Experience

#### Container Debugging
- **Shell access**: `docker exec -it gateway sh` now works
- **Busybox utilities**: `ls`, `cat`, `ps`, `env`, and more available
- **Production-ready**: Debug capabilities don't compromise security

#### Version Management
- **Single source of truth**: Version defined once in Cargo.toml
- **Automatic propagation**: Version flows to Docker tags automatically
- **Clear versioning**: Semantic versioning for all releases

---

## 📦 Docker Image Details

### Available Tags

```bash
# Latest stable version
ghcr.io/danielsarmiento04/kairos-rs:latest

# Specific version
ghcr.io/danielsarmiento04/kairos-rs:0.2.10
ghcr.io/danielsarmiento04/kairos-rs:0.2
ghcr.io/danielsarmiento04/kairos-rs:0

# Branch-specific
ghcr.io/danielsarmiento04/kairos-rs:main
ghcr.io/danielsarmiento04/kairos-rs:ws

# Commit-specific
ghcr.io/danielsarmiento04/kairos-rs:sha-abc123
```

### Supported Platforms
- `linux/amd64` - Intel/AMD 64-bit
- `linux/arm64` - ARM 64-bit (Apple Silicon, ARM servers)

### Image Specifications
- **Base**: `gcr.io/distroless/cc-debian12:debug`
- **Size**: ~28MB
- **Shell**: Busybox sh
- **User**: nonroot (UID 65532)
- **Working Directory**: `/app`
- **Exposed Port**: 5900

---

## 🚀 Getting Started with Docker

### Quick Start

```bash
# Pull and run
docker pull ghcr.io/danielsarmiento04/kairos-rs:0.2.10
docker run -d -p 5900:5900 \
  -v $(pwd)/config.json:/app/config.json:ro \
  ghcr.io/danielsarmiento04/kairos-rs:0.2.10
```

### Docker Compose

```yaml
services:
  kairos-gateway:
    image: ghcr.io/danielsarmiento04/kairos-rs:0.2.10
    container_name: kairos-gateway
    restart: unless-stopped
    ports:
      - "5900:5900"
    volumes:
      - ./config.json:/app/config.json:ro
    environment:
      - RUST_LOG=info
      - KAIROS_HOST=0.0.0.0
      - KAIROS_PORT=5900
```

### Debugging

```bash
# Exec into running container
docker exec -it kairos-gateway sh

# View logs
docker logs -f kairos-gateway

# Check process
docker exec kairos-gateway ps aux
```

---

## 🔄 Migration Guide

### From 0.2.9 to 0.2.10

No breaking changes in this release. Simply update your image tag:

```bash
# Update Docker Compose
# Before:
image: ghcr.io/danielsarmiento04/kairos-rs:0.2.9

# After:
image: ghcr.io/danielsarmiento04/kairos-rs:0.2.10
# or
image: ghcr.io/danielsarmiento04/kairos-rs:latest
```

### Notable Changes
- Container base image changed from regular distroless to distroless:debug
- Shell access now available via `sh` (busybox)
- Image size increased slightly (~3MB) for debug capabilities
- Multi-platform support added (no action needed, automatic)

---

## 📊 Technical Details

### GitHub Actions Workflow Changes

**New step in docker-publish.yml:**
```yaml
- name: Extract version from Cargo.toml
  id: cargo_version
  run: |
    VERSION=$(grep -m1 '^version = ' crates/kairos-gateway/Cargo.toml | cut -d'"' -f2)
    echo "version=$VERSION" >> $GITHUB_OUTPUT
```

**Enhanced metadata extraction:**
```yaml
tags: |
  type=ref,event=branch
  type=semver,pattern={{version}},value=v${{ steps.cargo_version.outputs.version }}
  type=semver,pattern={{major}}.{{minor}},value=v${{ steps.cargo_version.outputs.version }}
  type=raw,value=latest,enable={{is_default_branch}}
```

### Dockerfile Changes

```dockerfile
# Before (0.2.9):
FROM gcr.io/distroless/cc-debian12:latest AS runtime

# After (0.2.10):
FROM gcr.io/distroless/cc-debian12:debug AS runtime
```

---

## 📝 Documentation Updates

### New/Updated Files
- ✅ `README.md`: Added Docker Quick Start section
- ✅ `README.md`: Updated version to 0.2.10
- ✅ `README.md`: Added recent completions section
- ✅ `.github/workflows/docker-publish.yml`: Automated versioning
- ✅ `Dockerfile`: Changed to distroless:debug
- ✅ `examples/websocket_routing/compose.yml`: Production-ready example

### Documentation Organization
- 📁 `docs/`: Guides and tutorials
- 📁 `realease/`: Version-specific release notes
- 📄 `README.md`: Overview and quick start
- 📄 `ROADMAP.md`: Future plans and completed features

---

## 🎯 What's Next

See [ROADMAP.md](../ROADMAP.md) for upcoming features.

**Planned for 0.3.x:**
- Response caching for improved performance
- Historical metrics and time-series data
- Distributed tracing integration
- WebSocket UI updates with real-time monitoring
- gRPC protocol support

---

## 🙏 Acknowledgments

Thanks to all contributors and users who provided feedback on container deployment and developer experience!

---

## 📞 Support

- **Issues**: https://github.com/DanielSarmiento04/kairos-rs/issues
- **Discussions**: https://github.com/DanielSarmiento04/kairos-rs/discussions
- **Docker Images**: https://github.com/DanielSarmiento04/kairos-rs/pkgs/container/kairos-rs

---

**Full Changelog**: https://github.com/DanielSarmiento04/kairos-rs/compare/v0.2.9...v0.2.10
