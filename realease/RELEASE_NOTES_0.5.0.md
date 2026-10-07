# Release Notes v0.5.0

## ✨ What's New

### Time-series Charts UI — Multi-Chart Dashboard

A new `/dashboards` route ships a full multi-chart dashboard system built around three ideas: latency **percentiles derived from histogram buckets**, interactive **SVG charts with brush-zoom + pan**, and **layout persistence in localStorage** so users can build, save, export, and import their own dashboards.

The existing single-chart `Metrics.vue` view is left untouched and remains the entry point for raw-metric / Prometheus workflows.

### Backend: Latency percentiles from histogram buckets

The historical metrics store already recorded histogram observations (`MetricValue::Histogram { le, count }`) but `query_aggregated` skipped them with `// Skip histograms for now` (line 230 of `services/metrics_store.rs`). v0.5.0 lifts that limitation.

- **New module** [`crates/kairos-rs/src/services/percentile.rs`](/Users/vongthai/workspace/kairos-rs/crates/kairos-rs/src/services/percentile.rs)
  - `PercentilePoint { timestamp, percentiles: BTreeMap<String, f64> }` — keyed by percentile number as a string with trailing zeros stripped (`"50"`, `"95"`, `"99.9"`).
  - `compute_percentiles(points, interval, percentiles)` — groups observations into `interval`-sized windows, builds a CDF from the histogram buckets within each window, and linearly interpolates between adjacent bucket boundaries to find each requested percentile.
- **New method** [`MetricsStore::query_latency_percentiles`](/Users/vongthai/workspace/kairos-rs/crates/kairos-rs/src/services/metrics_store.rs)
  - Signature: `query_latency_percentiles(name, start, end, percentiles: &[f64], interval) -> Vec<PercentilePoint>`.
  - Returns chronologically; empty when the metric is unknown, has no histogram observations, or every bucket reports zero count.
- **New endpoint** `GET /api/metrics/latency/percentiles`
  - Query: `name`, `start`, `end`, `interval` (`one_minute` | `five_minutes` | `one_hour` | `one_day`), optional comma-separated `percentiles` (defaults to `50,95,99`).
  - Returns `Vec<PercentilePoint>` as JSON.
- **Tests** [`crates/kairos-rs/tests/percentile_tests.rs`](/Users/vongthai/workspace/kairos-rs/crates/kairos-rs/tests/percentile_tests.rs) — 13 cases covering empty stores, single bucket, all-zero counts, mixed Counter+Histogram, uniform + heavy-tail distributions, p0/p100 boundaries, fractional percentiles (`75`, `90`, `99.9`), time grouping into 1-minute and 5-minute windows, chronological ordering, and stable key formatting.

### Frontend: `/dashboards` view + 3 default dashboards

- **Types** added to [`types/index.ts`](/Users/vongthai/workspace/kairos-rs/frontend/kairos-ui/src/types/index.ts):
  - `PercentilePoint`, `LatencyPercentileQuery`, `DashboardLayout`, `ChartConfig`, `TimeRangePreset`, `CustomTimeRange`.
  - `ChartConfig.gridX/gridY/gridW/gridH` encode a 12-column CSS grid position so layouts can be saved and restored without canvas math.
- **API** [`getLatencyPercentiles`](/Users/vongthai/workspace/kairos-rs/frontend/kairos-ui/src/services/api.ts) wrapping the new endpoint; **route** `/dashboards` registered in [`router/index.ts`](/Users/vongthai/workspace/kairos-rs/frontend/kairos-ui/src/router/index.ts).
- **Composables**:
  - [`useDashboards.ts`](/Users/vongthai/workspace/kairos-rs/frontend/kairos-ui/src/composables/useDashboards.ts) — `layouts` + `activeId` persisted under `kairos.dashboards` / `kairos.activeDashboard`. Seeds 3 starter dashboards on first run, supports `createDashboard`, `renameDashboard`, `deleteDashboard`, `saveLayout`, `exportLayout` (JSON), `importLayout`, and `resetToDefaults`. Never leaves the storage with zero dashboards — re-seeds defaults if everything is deleted.
  - [`useChartData.ts`](/Users/vongthai/workspace/kairos-rs/frontend/kairos-ui/src/composables/useChartData.ts) — `fetchHistorical` + `fetchPercentiles` with shared query builder, plus `toCsvHistorical` / `toCsvPercentiles` that trigger a browser download via Blob.
- **Components** under [`components/dashboards/`](/Users/vongthai/workspace/kairos-rs/frontend/kairos-ui/src/components/dashboards/):
  - **`DashboardsView.vue`** — page header (New Dashboard / Import / Settings buttons), horizontal tab strip, body mounting the active grid.
  - **`DashboardGrid.vue`** — 12-column CSS grid that positions each `ChartConfig` according to its `gridX/Y/W/H`.
  - **`ChartCard.vue`** — self-contained SVG chart supporting `requests`, `error_rate`, `active_connections`, `custom`, and `latency_percentiles` chart types. Includes brush-zoom (Shift+drag), pan (drag), double-click to reset, reset button, CSV export, loading / error / empty states, and a colour-coded legend for percentile lines.
  - **`TimeRangePicker.vue`** — six preset buttons (`5m`, `15m`, `1h`, `6h`, `24h`, `7d`) plus a Custom mode with two `<input type="datetime-local">` fields.
  - **`PercentileSelector.vue`** — toggle pills for `[50, 75, 90, 95, 99, 99.9]` (customizable via `options` prop). Emits a sorted array.
  - **`DashboardSettings.vue`** — Teleport-based modal with rename / export / delete (when a `layout` prop is provided) or import-paste/file-picker (when no layout).
- **Default dashboards** seeded by `useDashboards`:
  - **Overview** — four 6×1 charts: `requests_total`, `requests_error`, `active_connections`, `response_time_avg`.
  - **Latency Deep Dive** — one 12×2 chart of p50/p95/p99/99.9 plus one 12×1 chart of `requests_total` to correlate.
  - **Per-route breakdown** — placeholder for the future per-route filter (intentionally a single full-width chart to make room for the picker when it lands).

### Operational notes

- **`vue-tsc` strict mode** — caught a `WritableComputedRef` / `ComputedRef` mismatch in `useDashboards.ts` (Vue 3.5 ships stricter `Ref` inference) and several `noUncheckedIndexedAccess` warnings in `useDashboards.spec.ts`. Both fixed by tightening the interface to `ComputedRef<DashboardLayout | undefined>` and adding `!` assertions at the call sites.
- **`vite build`** failed silently with `Unexpected token` because the new route in `router/index.ts` was missing its closing `},`. The strict oxc parser used by rolldown rejects unclosed object literals — vue-tsc and vitest were both happy because they use a more permissive parser. Fixed; replays a clean build.
- **`package.json`** — `type-check` script changed from `vue-tsc --build` to `vue-tsc --noEmit`. The `--build` flag requires `tsconfig.json` project references to be set up for declaration emission; this project never produced `.d.ts`, so the flag only ever added noise.

## 📦 Architecture

```
crates/kairos-rs/src/
├── percentile.rs              (NEW) PercentilePoint + CDF algorithm
├── metrics_store.rs           + query_latency_percentiles()
├── routes/metrics.rs          + get_latency_percentiles() + route
└── tests/percentile_tests.rs  (NEW) 13 cases

frontend/kairos-ui/src/
├── types/index.ts                       + 6 dashboard/percentile types
├── services/api.ts                      + getLatencyPercentiles()
├── composables/
│   ├── useDashboards.ts       (NEW) localStorage layouts + CRUD + import/export
│   └── useChartData.ts        (NEW) fetch + CSV + query builder
├── components/dashboards/     (NEW) 7 files:
│   ├── index.ts                          barrel
│   ├── DashboardsView.vue                header / tabs / body
│   ├── DashboardGrid.vue                 12-col grid wrapper
│   ├── ChartCard.vue                     SVG chart + zoom/pan
│   ├── TimeRangePicker.vue               presets + custom range
│   ├── PercentileSelector.vue            percentile chips
│   └── DashboardSettings.vue             rename/delete/export/import modal
└── views/Dashboards.vue       (NEW) thin wrapper delegating to DashboardsView
```

## 🧪 Testing

- **Backend** — `cargo test percentile` runs 13 cases, all pass. `cargo build --release` succeeds with only pre-existing async-trait warnings unrelated to this release.
- **Frontend** — `npx vitest run` runs **119 tests across 16 files, all pass** (up from 35 before this release). New tests:
  - `useDashboards.spec.ts` — 14 cases (seed, persist, rehydrate, CRUD, export, import, reset).
  - `useChartData.spec.ts` — 11 cases (time range resolution, query builder, fetch wrappers, CSV formatter).
  - `ChartCard.spec.ts` — 11 cases (title fallback, metric mapping, percentile chart, loading / empty / error states).
  - `TimeRangePicker.spec.ts` — 6 cases (preset buttons, active state, custom inputs, click → emit).
  - `PercentileSelector.spec.ts` — 6 cases (default / custom options, active class, toggle on/off).
  - `Dashboards.spec.ts` — 10 cases (header, tabs, create bar, settings modal, import modal, localStorage persistence, rehydration).

## 🛠 Quality

- **`vue-tsc --noEmit`** — exits 0 (strict mode + `noUncheckedIndexedAccess`).
- **`npm run build`** — `vite: build ok`, zero warnings. Bundle impact: dashboards add ~9 KB pre-gzip; no chart-library dependency was added (used self-contained SVG to skip the `uPlot` install).

## 🔜 What's Next?

- **Per-route analytics** — wire the placeholder chart on the "Per-route breakdown" dashboard to the existing `MetricsStore` once a per-route counter exists.
- **Drag-and-drop layout editor** — the 12-column grid already encodes `gridX/Y/W/H`; the next step is letting users reposition cards by dragging them.
- **Custom dashboard sharing** — `exportLayout` produces a JSON blob; layering a "copy link" UX on top is a small follow-up.

---

**Full diff:** see git log for commits tagged `v0.5.0`.