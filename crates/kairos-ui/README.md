# Kairos UI Development

## 🚀 Running the Enhanced Leptos UI

The Kairos UI is built with **Leptos 0.6** and provides a modern web-based admin interface for the Kairos API Gateway.

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

### Development Mode

Start the UI development server with hot reload:

```bash
# Terminal 2: Start UI dev server
cd crates/kairos-ui
cargo leptos serve
```

The UI will be available at: **http://localhost:3000**

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