# Task: Migrate `kairos-rs` UI from Leptos to Vue 3 + TypeScript

## 1. Objective
Migrate the presentation layer of the `kairos-rs` TensorZero LLM Application Platform/Gateway from Leptos (Rust/WASM) to a decoupled Vue 3 + TypeScript Single Page Application (SPA). 
The goal is to eliminate WASM-related UI maintenance complexity, improve frontend developer experience, and maintain a high-performance dashboard for LLM telemetry and routing configuration.

## 2. Architecture Shift
*   **Current State:** Monolithic/Tightly-coupled Rust application using Leptos for the frontend and Leptos Server Functions for backend communication.
*   **Target State:** 
    *   **Backend:** Pure Rust API server (Axum/Actix) serving JSON via REST.
    *   **Frontend:** Vue 3 (Composition API) + Vite + TypeScript + Pinia + Vue Router.

## 3. Execution Phases

### Phase 1: API Boundary Decoupling (Rust Backend)
*The agent must decouple the UI logic from the Rust backend before writing Vue code.*
- [ ] **Analyze Leptos Server Functions:** Scan the current `kairos-rs` codebase for all `#[server]` macros.
- [ ] **Create REST API Contracts:** Convert these server functions into standard RESTful endpoints (e.g., `GET /api/v1/telemetry`, `POST /api/v1/config/routes`).
- [ ] **Implement CORS & Proxy:** Ensure the Rust backend is configured to accept requests from the Vite development server (typically `http://localhost:5173`) or configure a Vite proxy.

### Phase 2: Frontend Scaffolding
- [ ] Initialize a new Vite + Vue + TS project in a `/frontend` or `/ui` directory.
- [ ] Configure `vite.config.ts` to proxy `/api` requests to the Rust backend port.
- [ ] Set up Vue Router with the base layout (Sidebar navigation, Header).
- [ ] Set up Pinia for global state (e.g., current environment, authentication state if applicable).

### Phase 3: Core Feature Migration (TensorZero Gateway UI)
*The agent must replicate the existing Leptos features into Vue components using `<script setup>` syntax and strict TypeScript interfaces matching the Rust structs.*

- [ ] **Dashboard / Telemetry View:**
    - Create interfaces for LLM metrics (tokens, latency, cost).
    - Implement a data table for gateway logs. 
    - *Agent Constraint:* Use `shallowRef` instead of `ref` for arrays of telemetry data to ensure rendering performance for large log datasets.
- [ ] **Routing & Configuration View:**
    - Migrate forms for configuring LLM providers, model routing rules, and fallback logic.
    - Implement JSON/YAML validation if configurations are edited directly.
- [ ] **API Key / Client Management:**
    - Migrate the UI for generating, revoking, and viewing client API keys.
- [ ] **Playground / Request Testing (If applicable):**
    - Migrate the chat/prompt interface used to test gateway routes directly from the dashboard.

### Phase 4: Styling & Cleanup
- [ ] Replicate the existing Leptos CSS/Tailwind styling within Vue Single File Components (`<style scoped>`) or global stylesheets.
- [ ] Remove all Leptos dependencies from the Rust `Cargo.toml`.
- [ ] Remove Leptos specific build steps (e.g., `cargo-leptos`, Trunk) from the CI/CD pipeline.
- [ ] Update the `README.md` to reflect the new decoupled build instructions.

## 4. Technical Constraints & Rules for the Agent
1.  **Strict Typing:** Do not use `any` in TypeScript. All API responses must have corresponding TypeScript interfaces mapped to the Rust backend structs.
2.  **Composition API:** Strictly use Vue 3 `<script setup>` syntax. Do not use the Options API.
3.  **State Management:** Use Vue's built-in reactivity (`ref`, `computed`) for local component state, and Pinia only for truly global state.
4.  **Performance:** Keep the bundle size light. Avoid heavy charting libraries unless explicitly requested; prefer lightweight UI components.