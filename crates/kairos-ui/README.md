# Kairos UI Development

## 🚀 Running the Enhanced Leptos UI

The Kairos UI is built with **Leptos 0.6** and provides a modern web-based admin interface for the Kairos API Gateway.

### Quick Start (TL;DR)

```bash
# 1. Start the gateway (in one terminal)
cargo run --bin kairos-gateway

# 2. Start the UI dev server (in another terminal)
cd crates/kairos-ui
cargo leptos serve

# 3. Open http://localhost:3000
```

### Prerequisites

1. **Gateway must be running** on port 5900:
   ```bash
   # Terminal 1: Start the gateway
   cargo run --bin kairos-gateway
   ```

2. **Install cargo-leptos** (if not already installed):
   ```bash
   cargo install cargo-leptos
   ```

3. **Install wasm-bindgen-cli** (if building manually):
   ```bash
   cargo install wasm-bindgen-cli
   ```

### Development Mode

Start the UI development server with hot reload:

```bash
# Terminal 2: Start UI dev server
cd crates/kairos-ui
cargo leptos serve
```

The UI will be available at: **http://localhost:3000**

### Building for Production

**Option 1: Using cargo-leptos (Recommended)**
```bash
cargo leptos build --release
./target/release/kairos-ui
```

**Option 2: Manual build**
```bash
# Make the build script executable
chmod +x build.sh

# Run the build
./build.sh

# Run the server
./target/release/kairos-ui
```

**⚠️ Important:** 
- The WASM client must be built with `csr` feature (default)
- The server binary must be built with `ssr` feature
- **Never** build WASM with `--features ssr` - this will fail because server features like Tokio networking are incompatible with WASM

### Features Implemented

✅ **Real-time Dashboard**
- Live metrics from gateway API
- Auto-refresh every 30 seconds
- Error handling and loading states
- Responsive metric cards

✅ **Modern Design**
- Professional CSS styling
- Responsive layout
- Status indicators with animations
- Consistent color scheme

✅ **Navigation**
- Sidebar navigation with active states
- Breadcrumb navigation
- Route-based page titles

✅ **API Integration**
- Connects to gateway health endpoint
- Fetches metrics data
- Graceful error handling when gateway is offline

### UI Components

- **Dashboard**: Real-time metrics and system status
- **Routes**: Route management (placeholder)  
- **Metrics**: Detailed metrics view (placeholder)
- **Config**: Configuration management (placeholder)
- **Health**: System health monitoring (placeholder)

### Next Steps

- [ ] Implement remaining page components
- [ ] Add route management functionality
- [ ] Create configuration editor
- [ ] Add charts and graphs for metrics
- [ ] Implement WebSocket for real-time updates

### Troubleshooting

If you see "Failed to connect to gateway" error:
1. Make sure the gateway is running: `cargo run --bin kairos-gateway`
2. Verify it's listening on port 5900
3. Check the browser console for CORS issues

### Architecture

```
kairos-ui/
├── src/
│   ├── app.rs          # Main app component with routing
│   ├── components/     # Reusable UI components
│   │   ├── header.rs   # Top navigation bar
│   │   └── sidebar.rs  # Left sidebar navigation
│   ├── pages/          # Route-specific pages
│   │   ├── dashboard.rs # Main dashboard with live data
│   │   ├── routes.rs   # Route management
│   │   └── ...
│   └── services/       # API integration services
├── assets/
│   └── styles.css      # Custom CSS styling
└── Cargo.toml          # Leptos configuration
```

The UI uses **Client-Side Rendering (CSR)** by default, with **Server-Side Rendering (SSR)** available via the `ssr` feature flag.