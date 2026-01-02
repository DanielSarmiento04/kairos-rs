# Release Notes v0.2.15

## 🚀 New Features

### 📊 Real-time Metrics Dashboard
- **WebSocket Integration**: Replaced polling-based metrics with a real-time WebSocket connection (`/ws/admin/metrics`).
- **Live Updates**: The Admin UI now receives metrics updates every second with zero latency.
- **Prometheus Parsing**: Implemented a robust Prometheus text format parser in the UI to correctly visualize all gateway metrics.
- **Enhanced Visualization**: Real-time charts for CPU, Memory, Active Connections, and Request Rates.

### 🧠 AI & Transformation Support in UI
- **Router Models**: Updated UI data models to support `AiPolicy` and `RequestTransformation` configurations.
- **Configuration UI**: Added support for configuring AI-driven routing and request/response transformations directly from the dashboard.

## 🛠 Improvements & Fixes

### Backend
- **Concurrency Optimization**: Refactored `websocket_admin.rs` to use `tokio::select!`, eliminating race conditions and improving resource usage for WebSocket connections.
- **Configuration Stability**: Fixed the initialization order in `main.rs` to ensure `metrics` and `websocket_admin` services are configured consistently.
- **Bug Fix**: Resolved a critical `RouteHandler` extraction error in the `/metrics` endpoint that was causing 500 errors.

### Frontend
- **WASM Thread Safety**: Fixed compilation issues related to `Send/Sync` traits in WASM closures using `send_wrapper`.
- **Data Accuracy**: Fixed an issue where the UI was displaying "0" for all metrics due to incorrect parsing of the Prometheus response.

## 📦 Dependency Updates
- Updated `kairos-ui` dependencies for better WASM support.
- Optimized `kairos-gateway` for high-concurrency scenarios.

## 🔜 What's Next?
- **AI Routing Logic**: Full implementation of the AI orchestration layer.
- **Visual Transformation Editor**: A drag-and-drop interface for configuring request/response transformations.
