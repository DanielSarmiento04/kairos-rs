# Release Notes v0.4.0

## ✨ What's New

### Transformation UI — Visual Editor

Replaces manual JSON editing for request/response transformation rules with a guided visual editor integrated into the route modal.

- **New "🔄 Transformations" tab** in the route create/edit modal (`Routes.vue`) — alongside the existing General / Backends / Retry / AI tabs.
- **Full CRUD editor** for every transformation primitive supported by the Rust backend (`crates/kairos-rs/src/middleware/transform.rs`):
  - **Request headers** — `add` / `set` / `remove` / `replace (regex)` actions with live regex validation.
  - **Request path rewriting** — single regex `pattern` + capture-group `replacement` (e.g. `^/api/v1/(.+)$` → `/$1`).
  - **Request query parameters** — `add` / `set` / `remove` actions.
  - **Response headers** — same 4 actions as request headers.
  - **Status code mappings** — `from` / `to` numeric pair + optional `condition` expression; range validation (100-599).
- **Live Preview panel** — type a sample method / path / headers / query on the left, see exactly what the gateway will forward on the right, with red/green diff for every change. Pure client-side simulation (no extra API call), mirrors backend behavior 1:1.
- **Preset Templates dropdown** — 6 built-in templates for common patterns (Strip Authorization, Add X-Forwarded-By, Rewrite `/api/v1/*` → `/*`, Inject `api_key`, Remove Server header, Map 404→200 for health). One click to apply, deduped by `action+name`.
- **Empty state UX** — first-time visitors see a friendly "No transformations configured" panel with a single "Enable Transformations" button instead of empty fields.

### Schema Alignment

Frontend TypeScript types in `frontend/kairos-ui/src/types/index.ts` are now a faithful mirror of the Rust `TransformAction` / `HeaderTransformation` / `QueryTransformation` / `PathTransformation` / `StatusCodeMapping` structs. The previous record-based `{add?, remove?, set?}` shape caused silent round-trip drift; this release eliminates it.

- `HeaderTransformation[]` / `QueryTransformation[]` are now arrays of action-based rules (not string→string maps).
- `RequestTransformation.query_params` (not `query`).
- `ResponseTransformation.status_code_mappings` (not `status_code_mapping`).
- `StatusCodeMapping` includes the optional `condition` field.
- `PathTransformation` keeps `pattern` + `replacement`.

## 📦 Architecture

```
src/
├── components/transformation/
│   ├── TransformationEditor.vue   # container, v-model request + response
│   ├── HeaderRuleRow.vue          # 1 header rule (4 actions)
│   ├── QueryRuleRow.vue           # 1 query rule (3 actions)
│   ├── PathRuleEditor.vue         # 1 path rewrite
│   ├── StatusCodeMappingRow.vue   # 1 status mapping
│   ├── PresetTemplates.vue        # dropdown with apply event
│   ├── PreviewPanel.vue           # live diff sandbox
│   ├── presets.ts                 # 6 built-in templates
│   └── index.ts                   # barrel
└── composables/
    └── useTransformPreview.ts     # pure preview() + applyHeaderRule + applyQueryRule
```

`useTransformPreview` is a pure composable (no Vue reactivity required) — it can be unit-tested directly and reused outside the editor (e.g. for a future CLI/REPL).

## 🧪 Testing

- **6 new Vitest spec files** covering 58 tests:
  - `useTransformPreview.spec.ts` — 20 cases: every action × {header, query}, path rewrite success + invalid regex + missing value, status mapping, edge cases (null / undefined transformation inputs).
  - `HeaderRuleRow.spec.ts` — 7 cases: action switching, conditional field rendering, regex validity indicator, readonly mode.
  - `QueryRuleRow.spec.ts` — 6 cases.
  - `PathRuleEditor.spec.ts` — 6 cases: empty state, enable/disable, regex validity.
  - `StatusCodeMappingRow.spec.ts` — 5 cases: range validation, condition passthrough.
  - `PresetTemplates.spec.ts` — 4 cases: scope filtering, apply event, disabled state.
  - `TransformationEditor.spec.ts` — 7 cases: empty state, enable click, add/remove rules, preview panel render, readonly.
- **58/58 tests pass** (up from 35 before this release).

## 🛠 Quality

- **TypeScript strict** — frontend `tsconfig.app.json` enables `noUncheckedIndexedAccess`; all new code passes `vue-tsc --noEmit` with zero errors.
- **Production build** — `npm run build` produces clean dist; vite v8 with code splitting, no warnings.

## 🔜 What's Next?

- **Time-series charts UI** for historical metrics (next from Immediate Priorities).
- **Plugin System** — explore WASM-based plugins for custom logic.

---

**Full diff:** see git log for commits tagged `v0.4.0`.
