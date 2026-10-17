# Development & Contributing

We welcome contributions to Kairos Gateway! Whether it's reporting a bug, proposing a new feature, or submitting a Pull Request, your input is highly valued.

This guide provides instructions on how to set up the development environment, our coding standards, and how the repository is structured.

## Local Setup

### Prerequisites
- **Rust Toolchain**: `1.95.0` or newer (`1.99.0` recommended; required by `rig-core 0.44.0`). Install via [rustup](https://rustup.rs/).
- **Cargo Make** (Optional but recommended): `cargo install cargo-make`
- **Docker** and **Docker Compose**: For running test backends and end-to-end examples.

### Building the Project

The project is structured as a Cargo Workspace. To build the entire workspace:

```bash
git clone https://github.com/DanielSarmiento04/kairos-rs.git
cd kairos-rs
cargo build
```

To build just the main Gateway binary for testing:
```bash
cargo build --bin kairos-gateway
```

### Running Tests

Kairos Gateway maintains a comprehensive test suite of **420 automated tests** (`198` Rust unit/integration tests + `101` Rust doctests + `121` Vitest UI tests):

```bash
# Run all unit, integration, and doc tests across the Rust workspace (299 tests)
cargo test --workspace

# Run Rust documentation tests only (101 doctests)
cargo test --doc -p kairos-rs

# Run specific integration test suites
cargo test -p kairos-rs --test ai_service_tests
cargo test -p kairos-rs --test cache_integration_tests
cargo test -p kairos-rs --test compression_tests
cargo test -p kairos-rs --test percentile_endpoint_tests
cargo test -p kairos-rs --test transform_integration_tests
cargo test -p kairos-rs --test jwt_integration_test
cargo test -p kairos-rs --test websocket_integration_tests

# Run Frontend Vitest unit & component tests (121 tests across 14 suites)
cd frontend/kairos-ui && npm run test:unit -- --run
```

## Running the Components Locally

### 1. Kairos Gateway
Modify the `config.json` at the root directory to point to your desired backends, or use one from `examples/`.

```bash
cargo run --bin kairos-gateway
```
The server will start at `http://localhost:5900` by default.

### 2. Kairos UI (Vue 3 + TypeScript)

The admin dashboard is a modern decoupled Single Page Application located in `frontend/kairos-ui`, built with **Vue 3**, **TypeScript**, and **Vite 8**.

Prerequisites: Node.js 18+ and npm.

```bash
cd frontend/kairos-ui

# Install dependencies
npm install

# Run the dev server with hot module replacement (HMR)
npm run dev

# Run unit tests
npm run test:unit -- --run

# Run type checks and production build
npm run build
```

The Vite dev server runs at `http://localhost:5173` and automatically proxies `/api`, `/health`, `/metrics`, and `/ws` to the backend gateway at `http://127.0.0.1:5900`.

### 3. Kairos CLI

```bash
cd crates/kairos-cli
cargo run -- --help
```

## Coding Guidelines

Throughout the repository, we enforce Rust best practices as defined in our internal rules. Please ensure your contributions align with the following:

- **Formatting**: Run `cargo fmt` before committing.
- **Linting**: No warnings should be thrown by Clippy.
  ```bash
  cargo clippy --all-targets --all-features -- -D warnings
  ```
- **Error Handling**: Use `thiserror` for library error types (e.g., inside `kairos-rs` and `kairos-client`) and `anyhow` for top-level binary diagnostics (`kairos-gateway` `main.rs`). Avoid `unwrap()` / `expect()` unless absolutely necessary in test code or impossible-to-fail scenarios.
- **Documentation**: All public structs, traits, and functions must be documented (`///`).
- **Asynchronous Code**: We heavily rely on `tokio` and `actix-web`. Do not write blocking code in async contexts; use `tokio::task::spawn_blocking` if computationally heavy tasks are necessary.

## Committing and Pull Requests

1. Fork the repository and create your feature branch: `git checkout -b feature/my-new-feature`
2. Implement your changes.
3. Write/update unit tests.
4. Update the related `.md` documentation in `docs/src` if your change affects system configuration or user instructions.
5. Create a Pull Request against the `main` branch.

### Conventional Commits
We encourage using [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) format for commit messages (e.g., `feat: added redis caching`, `fix: null pointer in ws handler`, `docs: updated architecture schema`). This helps automatically generate accurate release notes.