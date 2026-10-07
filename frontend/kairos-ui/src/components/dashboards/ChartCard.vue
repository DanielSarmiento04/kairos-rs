<script setup lang="ts">
/**
 * ChartCard — renders a single chart inside a dashboard grid cell.
 *
 * Self-contained SVG implementation (no external chart lib) so the
 * component works in isolation. Supports:
 *  - `requests` / `error_rate` / `active_connections` / `custom`:
 *    single-line time-series from `/api/metrics/history`
 *  - `latency_percentiles`: multi-line (one per percentile) from
 *    `/api/metrics/latency/percentiles`
 *  - Brush zoom: drag a region on the x-axis to zoom
 *  - Pan: drag inside the plot area
 *  - Reset: double-click to reset zoom
 *  - CSV export via `useChartData.toCsvHistorical` / `toCsvPercentiles`
 */
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import type {
  AggregatedMetricPoint,
  ChartConfig,
  HistoricalMetricPoint,
  PercentilePoint,
} from '@/types';
import { useChartData } from '@/composables/useChartData';

const props = defineProps<{
  config: ChartConfig;
  readonly?: boolean;
}>();

const emit = defineEmits<{
  'update:config': [value: ChartConfig];
  'remove': [];
}>();

const {
  resolveTimeRange,
  fetchHistorical,
  fetchPercentiles,
  toCsvHistorical,
  toCsvPercentiles,
} = useChartData();

type Row = HistoricalMetricPoint | AggregatedMetricPoint;
const rows = ref<Row[]>([]);
const percentileRows = ref<PercentilePoint[]>([]);
const loading = ref(false);
const error = ref<string | null>(null);

const W = 720;
const H = 220;
const PAD = { top: 14, right: 14, bottom: 24, left: 48 };
const innerW = W - PAD.left - PAD.right;
const innerH = H - PAD.top - PAD.bottom;

// Zoom state (in seconds).
const zoom = ref<{ min: number; max: number } | null>(null);
const svgRef = ref<SVGSVGElement>();

async function load() {
  loading.value = true;
  error.value = null;
  try {
    const { start, end } = resolveTimeRange(props.config.timeRange);
    if (props.config.type === 'latency_percentiles') {
      const ps = props.config.percentiles ?? [50, 95, 99];
      percentileRows.value = await fetchPercentiles(
        props.config.metricName ?? 'response_time',
        props.config.timeRange,
        props.config.aggregation === 'raw' ? 'one_minute' : props.config.aggregation,
        ps,
      );
      rows.value = [];
    } else {
      const metric =
        props.config.type === 'error_rate'
          ? 'requests_error'
          : props.config.type === 'active_connections'
          ? 'active_connections'
          : props.config.metricName ?? 'requests_total';
      rows.value = await fetchHistorical(
        metric,
        props.config.timeRange,
        props.config.aggregation,
      );
      percentileRows.value = [];
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
    rows.value = [];
    percentileRows.value = [];
  } finally {
    loading.value = false;
  }
}

onMounted(load);
watch(() => [props.config.timeRange, props.config.aggregation, props.config.metricName, props.config.type, props.config.percentiles?.join(',')], load, { deep: true });

/* ─── Coordinate helpers ────────────────────────────────────────────── */
type Sample = { ts: number; values: number[] };

const samples = computed<Sample[]>(() => {
  if (props.config.type === 'latency_percentiles') {
    return percentileRows.value.map((p) => ({
      ts: new Date(p.timestamp).getTime() / 1000,
      values: (props.config.percentiles ?? [50, 95, 99]).map(
        (pp) => p.percentiles[String(pp)] ?? Number.NaN,
      ),
    }));
  }
  return rows.value.map((r) => {
    const v = 'avg' in r ? r.avg : r.value;
    return { ts: new Date(r.timestamp).getTime() / 1000, values: [v] };
  });
});

const tsExtent = computed<[number, number]>(() => {
  if (samples.value.length === 0) {
    const now = Date.now() / 1000;
    return [now - 3600, now];
  }
  const first = samples.value[0]!.ts;
  const last = samples.value[samples.value.length - 1]!.ts;
  return first <= last ? [first, last] : [last, first];
});

const viewExtent = computed<[number, number]>(() => zoom.value ?? tsExtent.value);

const valExtent = computed<[number, number]>(() => {
  const v = samples.value
    .filter((s) => s.ts >= viewExtent.value[0] && s.ts <= viewExtent.value[1])
    .flatMap((s) => s.values)
    .filter((x) => Number.isFinite(x));
  if (v.length === 0) return [0, 1];
  const min = Math.min(...v);
  const max = Math.max(...v);
  if (min === max) return [min - 0.5, max + 0.5];
  const pad = (max - min) * 0.1;
  return [min - pad, max + pad];
});

function xScale(t: number): number {
  const [tMin, tMax] = viewExtent.value;
  if (tMax === tMin) return PAD.left;
  return PAD.left + ((t - tMin) / (tMax - tMin)) * innerW;
}
function yScale(v: number): number {
  const [vMin, vMax] = valExtent.value;
  if (vMax === vMin) return PAD.top + innerH / 2;
  return PAD.top + innerH - ((v - vMin) / (vMax - vMin)) * innerH;
}

const percentileKeys = computed(() =>
  props.config.type === 'latency_percentiles'
    ? (props.config.percentiles ?? [50, 95, 99])
    : [],
);

const COLORS = ['#2563eb', '#f59e0b', '#dc2626', '#10b981', '#8b5cf6', '#ec4899'];
function colorFor(i: number): string {
  return COLORS[i % COLORS.length] ?? '#2563eb';
}

const linePaths = computed<string[]>(() => {
  const seriesCount =
    props.config.type === 'latency_percentiles' ? percentileKeys.value.length : 1;
  const paths: string[] = [];
  for (let s = 0; s < seriesCount; s++) {
    const pts = samples.value
      .filter((p) => Number.isFinite(p.values[s] ?? NaN))
      .filter((p) => p.ts >= viewExtent.value[0] && p.ts <= viewExtent.value[1]);
    if (pts.length === 0) {
      paths.push('');
      continue;
    }
    const d = pts
      .map((p, i) => `${i === 0 ? 'M' : 'L'} ${xScale(p.ts).toFixed(2)} ${yScale(p.values[s]!).toFixed(2)}`)
      .join(' ');
    paths.push(d);
  }
  return paths;
});

const yTicks = computed(() => {
  const [vMin, vMax] = valExtent.value;
  const step = (vMax - vMin) / 4;
  return [0, 1, 2, 3, 4].map((i) => vMin + step * i);
});

function formatTime(sec: number): string {
  const d = new Date(sec * 1000);
  return `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
}

/* ─── Brush zoom + pan ─────────────────────────────────────────────── */
const dragStart = ref<number | null>(null);
const dragRect = ref<{ x: number; w: number } | null>(null);
const panLastX = ref<number | null>(null);

function onMouseDown(e: MouseEvent) {
  if (zoom.value && e.shiftKey) {
    // Reset
    zoom.value = null;
    return;
  }
  if (e.shiftKey) {
    dragStart.value = e.offsetX;
  } else {
    panLastX.value = e.offsetX;
  }
}
function onMouseMove(e: MouseEvent) {
  if (dragStart.value !== null) {
    const x0 = Math.min(dragStart.value, e.offsetX);
    const x1 = Math.max(dragStart.value, e.offsetX);
    dragRect.value = { x: x0, w: x1 - x0 };
  } else if (panLastX.value !== null) {
    const dx = e.offsetX - panLastX.value;
    panLastX.value = e.offsetX;
    const [tMin, tMax] = viewExtent.value;
    const dt = -(dx / innerW) * (tMax - tMin);
    const newMin = tMin + dt;
    const newMax = tMax + dt;
    zoom.value = { min: newMin, max: newMax };
  }
}
function onMouseUp() {
  if (dragStart.value !== null && dragRect.value) {
    const { x, w } = dragRect.value;
    if (w > 4) {
      const tMin = viewExtent.value[0] + ((x - PAD.left) / innerW) * (viewExtent.value[1] - viewExtent.value[0]);
      const tMax = tMin + (w / innerW) * (viewExtent.value[1] - viewExtent.value[0]);
      zoom.value = { min: tMin, max: tMax };
    }
  }
  dragStart.value = null;
  dragRect.value = null;
  panLastX.value = null;
}
function onDoubleClick() {
  zoom.value = null;
}

/* ─── Export ───────────────────────────────────────────────────────── */
function exportCsv() {
  const base = props.config.title?.replace(/\s+/g, '_') ?? props.config.id;
  if (props.config.type === 'latency_percentiles') {
    toCsvPercentiles(percentileRows.value, `${base}.csv`);
  } else {
    toCsvHistorical(rows.value, `${base}.csv`);
  }
}
</script>

<template>
  <div class="chart-card" :class="{ empty: !loading && samples.length === 0 }">
    <header class="chart-card__header">
      <h4>{{ config.title ?? config.metricName ?? config.type }}</h4>
      <div class="chart-card__actions">
        <button class="icon-btn" :disabled="!zoom" title="Reset zoom (or double-click chart)" @click="zoom = null">⟲</button>
        <button class="icon-btn" title="Export CSV" @click="exportCsv">⤓</button>
        <button v-if="!readonly" class="icon-btn icon-btn--danger" title="Remove chart" @click="emit('remove')">×</button>
      </div>
    </header>

    <div v-if="loading" class="chart-card__state">Loading…</div>
    <div v-else-if="error" class="chart-card__state error">⚠ {{ error }}</div>
    <div v-else-if="samples.length === 0" class="chart-card__state">No data in selected window</div>

    <svg
      v-show="!loading && !error && samples.length > 0"
      ref="svgRef"
      class="chart-card__svg"
      :viewBox="`0 0 ${W} ${H}`"
      :class="{ zooming: dragStart !== null }"
      @mousedown="onMouseDown"
      @mousemove="onMouseMove"
      @mouseup="onMouseUp"
      @mouseleave="onMouseUp"
      @dblclick="onDoubleClick"
    >
      <!-- y axis ticks -->
      <g class="grid">
        <line
          v-for="(t, i) in yTicks"
          :key="`y-${i}`"
          :x1="PAD.left"
          :x2="W - PAD.right"
          :y1="yScale(t)"
          :y2="yScale(t)"
        />
        <text
          v-for="(t, i) in yTicks"
          :key="`yt-${i}`"
          :x="PAD.left - 4"
          :y="yScale(t)"
          text-anchor="end"
          dominant-baseline="middle"
          font-size="10"
          fill="#64748b"
        >{{ t.toFixed(0) }}</text>
      </g>

      <!-- x axis labels -->
      <g class="axis">
        <text
          :x="xScale(viewExtent[0])"
          :y="H - 6"
          font-size="10"
          fill="#64748b"
        >{{ formatTime(viewExtent[0]) }}</text>
        <text
          :x="xScale(viewExtent[1])"
          :y="H - 6"
          font-size="10"
          fill="#64748b"
          text-anchor="end"
        >{{ formatTime(viewExtent[1]) }}</text>
      </g>

      <!-- lines -->
      <path
        v-for="(d, i) in linePaths"
        :key="`line-${i}`"
        :d="d"
        fill="none"
        :stroke="colorFor(i)"
        stroke-width="2"
        stroke-linejoin="round"
      />

      <!-- brush rectangle -->
      <rect
        v-if="dragRect"
        :x="dragRect.x"
        :y="PAD.top"
        :width="dragRect.w"
        :height="innerH"
        fill="rgba(37, 99, 235, 0.1)"
        stroke="#2563eb"
        stroke-dasharray="4 2"
      />
    </svg>

    <footer v-if="config.type === 'latency_percentiles' && percentileKeys.length > 0" class="chart-card__legend">
      <span
        v-for="(p, i) in percentileKeys"
        :key="p"
        class="legend-item"
      >
        <span class="dot" :style="{ background: colorFor(i) }" />
        p{{ p }}
      </span>
    </footer>
    <p v-else-if="!loading" class="chart-card__hint">Shift+drag to zoom · drag to pan · double-click to reset</p>
  </div>
</template>

<style scoped>
.chart-card {
  display: flex;
  flex-direction: column;
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  padding: 12px;
  min-height: 260px;
}
.chart-card__header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}
.chart-card__header h4 {
  margin: 0;
  font-size: 0.85rem;
  color: #0f172a;
  font-weight: 600;
}
.chart-card__actions {
  display: flex;
  gap: 4px;
}
.icon-btn {
  width: 26px;
  height: 26px;
  border: 1px solid #e2e8f0;
  background: #ffffff;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.85rem;
  color: #475569;
}
.icon-btn:hover { background: #f1f5f9; }
.icon-btn:disabled { opacity: 0.4; cursor: not-allowed; }
.icon-btn--danger { color: #dc2626; }
.chart-card__state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #94a3b8;
  font-size: 0.85rem;
}
.chart-card__state.error { color: #b91c1c; }
.chart-card__svg {
  width: 100%;
  height: auto;
  cursor: crosshair;
  user-select: none;
}
.chart-card__svg.zooming { cursor: ew-resize; }
.chart-card__legend {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-top: 6px;
  font-size: 0.75rem;
  color: #475569;
}
.legend-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  display: inline-block;
}
.chart-card__hint {
  margin: 4px 0 0;
  font-size: 0.7rem;
  color: #94a3b8;
}
</style>