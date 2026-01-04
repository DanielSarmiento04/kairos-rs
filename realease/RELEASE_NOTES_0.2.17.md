# Release Notes v0.2.17

## 🛠 Improvements & Fixes

### CI/CD & Build System
- **Docker Build Stability**: Fixed critical build failures in GitHub Actions for multi-platform builds (ARM64/QEMU). Added `sharing=locked` to Cargo cache mounts in `Dockerfile` to prevent race conditions during dependency unpacking (`os error 17`).

### Testing & Quality Assurance
- **Test Suite Repair**: Fixed multiple regression tests that were failing due to recent model changes.
  - Updated `config_settings_tests` to properly handle `Result` types.
  - Updated `route_matcher` and other unit tests to include the new `ai_policy` field in `Router` initialization.
- **Warning Resolution**: Resolved dead code warnings in `metrics_store.rs` to ensure a clean build output.

### Codebase Maintenance
- **Model Consistency**: Ensured all internal test instantiations of the `Router` struct are aligned with the latest schema including AI policy configuration.

## 📦 Dependency Updates
- Maintenance updates for build stability.

## 🔜 What's Next?
- **AI Orchestration**: Continued development on the AI routing logic and integration.
- **Documentation**: Further refinements to documentation and guides.
