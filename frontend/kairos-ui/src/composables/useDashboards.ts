/**
 * `useDashboards` — manage the user's dashboard layouts.
 *
 * Layouts are persisted to `localStorage` under `kairos.dashboards`.
 * On first run (storage empty) we seed three default dashboards so the
 * `/dashboards` view is never empty.
 *
 * The composable is framework-agnostic enough to be unit-tested without
 * mounting a Vue component: callers can drive `layouts.value` directly.
 */
import { computed, ref, watch } from 'vue';
import type { ComputedRef } from 'vue';
import type { ChartConfig, DashboardLayout, TimeRangePreset } from '@/types';

const STORAGE_KEY = 'kairos.dashboards';
const ACTIVE_KEY = 'kairos.activeDashboard';

function nowIso(): string {
  return new Date().toISOString();
}

function uid(prefix: string): string {
  return `${prefix}_${Date.now().toString(36)}_${Math.random().toString(36).slice(2, 8)}`;
}

function chart(
  type: ChartConfig['type'],
  metricName: string | undefined,
  timeRange: TimeRangePreset,
  aggregation: ChartConfig['aggregation'],
  grid: { x: number; y: number; w: number; h: number },
  percentiles?: number[],
  title?: string,
): ChartConfig {
  return {
    id: uid('chart'),
    type,
    metricName,
    percentiles,
    timeRange,
    aggregation,
    gridX: grid.x,
    gridY: grid.y,
    gridW: grid.w,
    gridH: grid.h,
    title,
  };
}

function buildDefaultDashboards(): DashboardLayout[] {
  const created = nowIso();
  return [
    {
      id: 'overview',
      name: 'Overview',
      createdAt: created,
      updatedAt: created,
      charts: [
        chart('requests', 'requests_total', '1h', 'one_minute', { x: 0, y: 0, w: 6, h: 1 }, undefined, 'Requests / sec'),
        chart('error_rate', 'requests_error', '1h', 'one_minute', { x: 6, y: 0, w: 6, h: 1 }, undefined, 'Errors / sec'),
        chart('active_connections', 'active_connections', '1h', 'one_minute', { x: 0, y: 1, w: 6, h: 1 }, undefined, 'Active connections'),
        chart('requests', 'response_time_avg', '1h', 'one_minute', { x: 6, y: 1, w: 6, h: 1 }, undefined, 'Avg response time (ms)'),
      ],
    },
    {
      id: 'latency-deep-dive',
      name: 'Latency Deep Dive',
      createdAt: created,
      updatedAt: created,
      charts: [
        chart(
          'latency_percentiles',
          'response_time',
          '1h',
          'one_minute',
          { x: 0, y: 0, w: 12, h: 2 },
          [50, 95, 99, 99.9],
          'Response time percentiles',
        ),
        chart('requests', 'requests_total', '1h', 'one_minute', { x: 0, y: 2, w: 12, h: 1 }, undefined, 'Requests / sec (correlate)'),
      ],
    },
    {
      id: 'per-route',
      name: 'Per-route breakdown',
      createdAt: created,
      updatedAt: created,
      charts: [
        chart('requests', 'requests_total', '1h', 'one_minute', { x: 0, y: 0, w: 12, h: 2 }, undefined, 'Per-route requests / sec'),
      ],
    },
  ];
}

function readStorage<T>(key: string, fallback: T): T {
  if (typeof localStorage === 'undefined') return fallback;
  try {
    const raw = localStorage.getItem(key);
    if (raw === null) return fallback;
    return JSON.parse(raw) as T;
  } catch {
    return fallback;
  }
}

function writeStorage<T>(key: string, value: T): void {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(key, JSON.stringify(value));
  } catch {
    // quota exceeded / private mode — ignore
  }
}

export interface UseDashboardsApi {
  layouts: ReturnType<typeof ref<DashboardLayout[]>>;
  activeId: ReturnType<typeof ref<string>>;
  activeLayout: ComputedRef<DashboardLayout | undefined>;
  setActive: (id: string) => void;
  createDashboard: (name: string) => DashboardLayout;
  renameDashboard: (id: string, name: string) => void;
  deleteDashboard: (id: string) => void;
  saveLayout: (layout: DashboardLayout) => void;
  exportLayout: (id: string) => string;
  importLayout: (json: string) => DashboardLayout;
  resetToDefaults: () => void;
}

export function useDashboards(): UseDashboardsApi {
  const initial = readStorage<DashboardLayout[] | null>(STORAGE_KEY, null);
  const layouts = ref<DashboardLayout[]>(
    initial && initial.length > 0 ? initial : buildDefaultDashboards(),
  );

  // If storage was empty, immediately persist the seeded defaults.
  if (initial === null || initial.length === 0) {
    writeStorage(STORAGE_KEY, layouts.value);
  }

  const activeId = ref<string>(readStorage<string>(ACTIVE_KEY, layouts.value[0]?.id ?? 'overview'));

  // Persist on any change.
  watch(layouts, (val) => writeStorage(STORAGE_KEY, val), { deep: true });
  watch(activeId, (val) => writeStorage(ACTIVE_KEY, val));

  const activeLayout = computed<DashboardLayout | undefined>(() =>
    layouts.value.find((l) => l.id === activeId.value),
  );

  function setActive(id: string) {
    activeId.value = id;
  }

  function createDashboard(name: string): DashboardLayout {
    const layout: DashboardLayout = {
      id: uid('dash'),
      name: name.trim() || 'Untitled Dashboard',
      createdAt: nowIso(),
      updatedAt: nowIso(),
      charts: [],
    };
    layouts.value = [...layouts.value, layout];
    activeId.value = layout.id;
    return layout;
  }

  function renameDashboard(id: string, name: string) {
    layouts.value = layouts.value.map((l) =>
      l.id === id ? { ...l, name: name.trim() || l.name, updatedAt: nowIso() } : l,
    );
  }

  function deleteDashboard(id: string) {
    const next = layouts.value.filter((l) => l.id !== id);
    if (next.length === 0) {
      // Never leave the user with zero dashboards — re-seed defaults.
      const defaults = buildDefaultDashboards();
      layouts.value = defaults;
      activeId.value = defaults[0]!.id;
      return;
    }
    layouts.value = next;
    if (activeId.value === id) activeId.value = next[0]!.id;
  }

  function saveLayout(layout: DashboardLayout) {
    layouts.value = layouts.value.map((l) =>
      l.id === layout.id ? { ...layout, updatedAt: nowIso() } : l,
    );
  }

  function exportLayout(id: string): string {
    const layout = layouts.value.find((l) => l.id === id);
    if (!layout) throw new Error(`Dashboard ${id} not found`);
    return JSON.stringify(layout, null, 2);
  }

  function importLayout(json: string): DashboardLayout {
    const parsed = JSON.parse(json) as Partial<DashboardLayout>;
    if (!parsed || typeof parsed !== 'object' || !Array.isArray(parsed.charts)) {
      throw new Error('Invalid dashboard JSON: missing charts array');
    }
    const layout: DashboardLayout = {
      id: uid('dash'),
      name: parsed.name ?? 'Imported Dashboard',
      charts: parsed.charts as ChartConfig[],
      createdAt: nowIso(),
      updatedAt: nowIso(),
    };
    layouts.value = [...layouts.value, layout];
    activeId.value = layout.id;
    return layout;
  }

  function resetToDefaults() {
    layouts.value = buildDefaultDashboards();
    activeId.value = layouts.value[0]!.id;
  }

  return {
    layouts,
    activeId,
    activeLayout,
    setActive,
    createDashboard,
    renameDashboard,
    deleteDashboard,
    saveLayout,
    exportLayout,
    importLayout,
    resetToDefaults,
  };
}