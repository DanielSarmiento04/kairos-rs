# Development & Contributing

We welcome contributions to Kairos Gateway! Whether it's reporting a bug, proposing a new feature, or submitting a Pull Request, your input is highly valued.

This guide provides instructions on how to set up the development environment, our coding standards, and how the repository is structured.

## Local Setup

### Prerequisites
- **Rust Toolchain**: `1.75.0` or newer. Install via [rustup](https://rustup.rs/).
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

We expect all core components to have a robust set of tests. To execute them:

```bash
# Run unit and integration tests across the workspace
cargo test --workspace

# Run tests mapped to specific modules (e.g. AI logic)
cargo test -p kairos-rs -- ai_service_tests
```

## Running the Components Locally

### 1. Kairos Gateway
Modify the `config.json` at the root directory to point to your desired backends, or use one from `examples/`.

```bash
cargo run --bin kairos-gateway
```
The server will start at `http://localhost:5900` by default.

### 2. Kairos UI (Leptos)

Working on the UI requires `cargo-leptos` and the WASM toolchain.

```bash
# Install toolchain and cargo-leptos
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos

# Run the UI in development mode with HMR
cd crates/kairos-ui
cargo leptos serve
```
The UI dashboard will be available at `http://localhost:3000`. Keep the gateway running in a separate terminal so the UI can draw metrics (`http://localhost:5900/metrics/*`).

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