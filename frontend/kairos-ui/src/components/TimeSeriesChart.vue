<!--
  TimeSeriesChart.vue
  Interactive SVG time-series chart for the kairos-rs metrics dashboard.

  Supports:
    * HistoricalMetricPoint[]  (raw points, single line)
    * AggregatedMetricPoint[] (avg line + min/max band)

  Features:
    * Responsive via SVG viewBox (no JS resize listener)
    * X-axis with smart time labels (HH:MM:SS / HH:MM / MM/DD depending on range)
    * Y-axis with 5 nice-rounded tick labels
    * Hover tooltip + crosshair + point highlight
    * Empty state ("no data in selected window")
    * Zero JS dependencies (pure Vue 3 + SVG)

  Strict typing: 0 `any`. All API surfaces typed.
-->
<script setup lang="ts">
import { computed, ref } from 'vue';
import type { HistoricalMetricPoint, AggregatedMetricPoint } from '../types';

const props = withDefaults(
  defineProps<{
    data: (HistoricalMetricPoint | AggregatedMetricPoint)[];
    metricName?: string;
    width?: number;
    height?: number;
    color?: string;
  }>(),
  {
    metricName: '',
    width: 900,
    height: 320,
    color: '#2563eb',
  },
);

// ─── Layout ────────────────────────────────────────────────────────────────
const PAD = { top: 16, right: 24, bottom: 36, left: 64 };
const innerW = computed(() => Math.max(10, props.width - PAD.left - PAD.right));
const innerH = computed(() => Math.max(10, props.height - PAD.top - PAD.bottom));

// ─── Data shape ───────────────────────────────────────────────────────────
const isAggregated = computed(
  () => props.data.length > 0 && props.data[0] !== undefined && 'avg' in props.data[0],
);

const points = computed(() => {
  if (props.data.length === 0) return [] as Array<{
    ts: number; value: number; min: number | null; max: number | null;
  }>;
  return props.data.map((d) => {
    if ('avg' in d) {
      const a = d as AggregatedMetricPoint;
      return { ts: new Date(a.timestamp).getTime(), value: a.avg, min: a.min, max: a.max };
    }
    const r = d as HistoricalMetricPoint;
    return { ts: new Date(r.timestamp).getTime(), value: r.value, min: null, max: null };
  });
});

// ─── Extents ──────────────────────────────────────────────────────────────
const xExtent = computed(() => {
  if (points.value.length === 0) {
    const now = Date.now();
    return { min: now - 3600_000, max: now };
  }
  const first = points.value[0]!;
  const last = points.value[points.value.length - 1]!;
  return { min: first.ts, max: last.ts };
});

const yExtent = computed(() => {
  if (points.value.length === 0) return { min: 0, max: 1, ticks: [0, 0.25, 0.5, 0.75, 1] };
  let lo = Infinity, hi = -Infinity;
  for (const p of points.value) {
    if (p.value < lo) lo = p.value;
    if (p.value > hi) hi = p.value;
    if (p.min !== null && p.min < lo) lo = p.min;
    if (p.max !== null && p.max > hi) hi = p.max;
  }
  if (lo === hi) { lo -= 1; hi += 1; }
  // Pad 10% top/bottom
  const span = hi - lo;
  lo = Math.max(0, lo - span * 0.1);
  hi = hi + span * 0.1;
  // Nice rounded ticks (5 steps)
  const niceLo = niceFloor(lo);
  const niceHi = niceCeil(hi);
  const step = niceStep((niceHi - niceLo) / 4);
  const ticks: number[] = [];
  for (let v = niceLo; v <= niceHi + 1e-9; v += step) ticks.push(Number(v.toFixed(10)));
  return { min: niceLo, max: niceHi, ticks };
});

// ─── Nice number rounding (1, 2, 5 × 10^k ladder) ─────────────────────────
function niceStep(raw: number): number {
  if (raw <= 0 || !isFinite(raw)) return 1;
  const pow = Math.pow(10, Math.floor(Math.log10(raw)));
  const norm = raw / pow;
  let nice: number;
  if (norm < 1.5) nice = 1;
  else if (norm < 3) nice = 2;
  else if (norm < 7) nice = 5;
  else nice = 10;
  return nice * pow;
}
function niceFloor(v: number): number { return Math.floor(v); }
function niceCeil(v: number): number { return Math.ceil(v); }

// ─── Scales ────────────────────────────────────────────────────────────────
function xScale(ts: number): number {
  const r = xExtent.value.max - xExtent.value.min || 1;
  return ((ts - xExtent.value.min) / r) * innerW.value;
}
function yScale(v: number): number {
  const r = yExtent.value.max - yExtent.value.min || 1;
  return innerH.value - ((v - yExtent.value.min) / r) * innerH.value;
}

// ─── Coordinates ──────────────────────────────────────────────────────────
const coords = computed(() =>
  points.value.map((p) => ({ x: xScale(p.ts), y: yScale(p.value), p })),
);

const linePath = computed(() => {
  if (coords.value.length === 0) return '';
  return coords.value
    .map((c, i) => `${i === 0 ? 'M' : 'L'} ${c.x.toFixed(1)} ${c.y.toFixed(1)}`)
    .join(' ');
});

const areaPath = computed(() => {
  if (coords.value.length === 0) return '';
  const pts = coords.value;
  const firstX = pts[0]!.x.toFixed(1);
  const lastX = pts[pts.length - 1]!.x.toFixed(1);
  const top = pts.map((c) => `${c.x.toFixed(1)} ${c.y.toFixed(1)}`).join(' L ');
  return `M ${firstX} ${innerH.value} L ${top} L ${lastX} ${innerH.value} Z`;
});

const bandPath = computed(() => {
  if (!isAggregated.value || points.value.length === 0) return '';
  // Build polygon: min forward, max reverse
  const fwd = points.value
    .map((p) => `${xScale(p.ts).toFixed(1)} ${yScale(p.min ?? p.value).toFixed(1)}`)
    .join(' L ');
  const rev = points.value
    .slice()
    .reverse()
    .map((p) => `${xScale(p.ts).toFixed(1)} ${yScale(p.max ?? p.value).toFixed(1)}`)
    .join(' L ');
  return `M ${fwd} L ${rev} Z`;
});

// ─── Axis ticks ───────────────────────────────────────────────────────────
const xTickCount = computed(() => {
  const span = xExtent.value.max - xExtent.value.min;
  if (span <= 60 * 60_000) return 6;          // ≤ 1h → 6 ticks
  if (span <= 6 * 60 * 60_000) return 6;      // ≤ 6h → 6 ticks
  return 5;                                    // longer → 5 ticks
});

const xTicks = computed(() => {
  const n = xTickCount.value;
  const out: Array<{ x: number; label: string }> = [];
  const min = xExtent.value.min;
  const max = xExtent.value.max;
  for (let i = 0; i <= n; i++) {
    const t = min + ((max - min) * i) / n;
    out.push({ x: xScale(t), label: formatTime(new Date(t), max - min) });
  }
  return out;
});

const yTicks = computed(() =>
  yExtent.value.ticks.map((v) => ({ y: yScale(v), label: formatValue(v) })),
);

// ─── Time formatting ──────────────────────────────────────────────────────
function formatTime(d: Date, spanMs: number): string {
  const pad = (n: number) => String(n).padStart(2, '0');
  if (spanMs <= 24 * 60 * 60_000) {
    // within a day: HH:MM:SS or HH:MM
    return `${pad(d.getHours())}:${pad(d.getMinutes())}`;
  }
  return `${pad(d.getMonth() + 1)}/${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

function formatValue(v: number): string {
  if (v === 0) return '0';
  const abs = Math.abs(v);
  if (abs >= 1_000_000) return `${(v / 1_000_000).toFixed(1)}M`;
  if (abs >= 10_000) return `${(v / 1_000).toFixed(0)}k`;
  if (abs >= 1_000) return `${(v / 1_000).toFixed(1)}k`;
  if (abs >= 10) return v.toFixed(0);
  return v.toFixed(2);
}

// ─── Hover / tooltip ──────────────────────────────────────────────────────
const hoverIdx = ref<number>(-1);
const svgRef = ref<SVGSVGElement | null>(null);

function handleMove(evt: MouseEvent) {
  if (coords.value.length === 0 || !svgRef.value) return;
  const rect = svgRef.value.getBoundingClientRect();
  // Map page px → viewBox units (account for preserveAspectRatio scaling)
  const scaleX = props.width / rect.width;
  const scaleY = props.height / rect.height;
  const localX = (evt.clientX - rect.left) * scaleX - PAD.left;
  if (localX < 0 || localX > innerW.value) {
    hoverIdx.value = -1;
    return;
  }
  // Find nearest x coord
  let best = 0;
  let bestDx = Infinity;
  for (let i = 0; i < coords.value.length; i++) {
    const c = coords.value[i];
    if (!c) continue;
    const dx = Math.abs(c.x - localX);
    if (dx < bestDx) { bestDx = dx; best = i; }
  }
  hoverIdx.value = best;
}

function handleLeave() { hoverIdx.value = -1; }

const hoverPoint = computed(() => (hoverIdx.value >= 0 ? coords.value[hoverIdx.value] : null));
const hoverTimeLabel = computed(() => {
  if (!hoverPoint.value) return '';
  return formatTime(new Date(hoverPoint.value.p.ts), xExtent.value.max - xExtent.value.min);
});
const tooltipW = 160;
const tooltipH = isAggregated.value ? 64 : 44;
</script>

<template>
  <div class="ts-chart-wrap">
    <!-- Empty state -->
    <div v-if="points.length === 0" class="ts-empty">
      <span class="ts-empty-icon">∅</span>
      <span class="ts-empty-text">No data points in selected window</span>
    </div>

    <svg
      v-else
      ref="svgRef"
      :viewBox="`0 0 ${width} ${height}`"
      class="ts-svg"
      :preserveAspectRatio="'xMidYMid meet'"
      role="img"
      :aria-label="`Time-series chart for ${metricName}`"
      @mousemove="handleMove"
      @mouseleave="handleLeave"
    >
      <defs>
        <linearGradient :id="`ts-area-grad-${metricName}`" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" :stop-color="color" stop-opacity="0.32" />
          <stop offset="100%" :stop-color="color" stop-opacity="0.0" />
        </linearGradient>
      </defs>

      <!-- Plot group, offset by padding -->
      <g :transform="`translate(${PAD.left}, ${PAD.top})`">
        <!-- Horizontal gridlines -->
        <line
          v-for="(t, i) in yTicks"
          :key="`g-${i}`"
          x1="0" :x2="innerW"
          :y1="t.y" :y2="t.y"
          stroke="#f1f5f9"
          stroke-dasharray="3 3"
          stroke-width="1"
        />

        <!-- Min/max band (only when aggregated) -->
        <path
          v-if="isAggregated"
          :d="bandPath"
          :fill="color"
          fill-opacity="0.10"
          stroke="none"
        />

        <!-- Filled area under value line -->
        <path
          :d="areaPath"
          :fill="`url(#ts-area-grad-${metricName})`"
          stroke="none"
        />

        <!-- Main value line -->
        <path
          :d="linePath"
          fill="none"
          :stroke="color"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        />

        <!-- Crosshair + tooltip -->
        <g v-if="hoverPoint" pointer-events="none">
          <line
            :x1="hoverPoint.x" :x2="hoverPoint.x"
            y1="0" :y2="innerH"
            stroke="#94a3b8"
            stroke-dasharray="4 4"
            stroke-width="1"
          />
          <circle
            :cx="hoverPoint.x" :cy="hoverPoint.y"
            r="5"
            fill="#ffffff"
            :stroke="color"
            stroke-width="2.5"
          />
          <!-- Min/max dots if aggregated -->
          <template v-if="isAggregated && hoverPoint.p.min !== null">
            <circle
              :cx="hoverPoint.x" :cy="yScale(hoverPoint.p.min)"
              r="3"
              :stroke="color"
              stroke-width="1.5"
              fill="#ffffff"
            />
          </template>
          <template v-if="isAggregated && hoverPoint.p.max !== null">
            <circle
              :cx="hoverPoint.x" :cy="yScale(hoverPoint.p.max)"
              r="3"
              :stroke="color"
              stroke-width="1.5"
              fill="#ffffff"
            />
          </template>
        </g>

        <!-- X-axis baseline -->
        <line
          x1="0" :x2="innerW"
          :y1="innerH" :y2="innerH"
          stroke="#cbd5e1"
          stroke-width="1"
        />
        <!-- Y-axis baseline -->
        <line
          x1="0" :x2="0"
          y1="0" :y2="innerH"
          stroke="#cbd5e1"
          stroke-width="1"
        />
      </g>

      <!-- Y-axis tick labels (outside plot, left) -->
      <g class="y-ticks">
        <text
          v-for="(t, i) in yTicks"
          :key="`yt-${i}`"
          :x="PAD.left - 8"
          :y="PAD.top + t.y + 4"
          text-anchor="end"
          font-size="11"
          fill="#64748b"
          font-family="ui-monospace, SFMono-Regular, Menlo, monospace"
        >{{ t.label }}</text>
      </g>

      <!-- X-axis tick labels (below plot) -->
      <g class="x-ticks">
        <text
          v-for="(t, i) in xTicks"
          :key="`xt-${i}`"
          :x="PAD.left + t.x"
          :y="height - PAD.bottom + 18"
          text-anchor="middle"
          font-size="11"
          fill="#64748b"
          font-family="ui-monospace, SFMono-Regular, Menlo, monospace"
        >{{ t.label }}</text>
      </g>

      <!-- Tooltip card (HTML overlay positioned via foreignObject) -->
      <foreignObject
        v-if="hoverPoint"
        :x="PAD.left + hoverPoint.x + 12"
        :y="Math.max(0, PAD.top + hoverPoint.y - (isAggregated ? 40 : 28))"
        :width="tooltipW"
        :height="tooltipH"
        style="pointer-events: none; overflow: visible;"
      >
        <div class="ts-tooltip">
          <div class="ts-tooltip-time">{{ hoverTimeLabel }}</div>
          <div class="ts-tooltip-val">
            <span class="ts-tooltip-dot" :style="{ background: color }"></span>
            <strong>{{ hoverPoint.p.value.toFixed(2) }}</strong>
          </div>
          <div v-if="isAggregated && hoverPoint.p.min !== null && hoverPoint.p.max !== null" class="ts-tooltip-band">
            <span>min {{ hoverPoint.p.min.toFixed(1) }}</span>
            <span>max {{ hoverPoint.p.max.toFixed(1) }}</span>
          </div>
        </div>
      </foreignObject>
    </svg>
  </div>
</template>

<style scoped>
.ts-chart-wrap {
  position: relative;
  width: 100%;
  height: 100%;
  min-height: 280px;
}
.ts-svg {
  display: block;
  width: 100%;
  height: 100%;
  cursor: crosshair;
}
.ts-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: #94a3b8;
  gap: 8px;
}
.ts-empty-icon {
  font-size: 2.4rem;
  color: #cbd5e1;
}
.ts-empty-text {
  font-size: 0.95rem;
}
.ts-tooltip {
  background: #0f172a;
  color: #f8fafc;
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 0.78rem;
  line-height: 1.4;
  box-shadow: 0 6px 16px rgba(15, 23, 42, 0.20);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  min-width: 130px;
}
.ts-tooltip-time {
  color: #94a3b8;
  font-size: 0.7rem;
  margin-bottom: 2px;
}
.ts-tooltip-val {
  display: flex;
  align-items: center;
  gap: 6px;
}
.ts-tooltip-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  display: inline-block;
}
.ts-tooltip-band {
  display: flex;
  justify-content: space-between;
  color: #94a3b8;
  font-size: 0.7rem;
  margin-top: 4px;
  border-top: 1px solid #334155;
  padding-top: 4px;
}
</style>