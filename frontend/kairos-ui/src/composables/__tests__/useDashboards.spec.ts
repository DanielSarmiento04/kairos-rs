import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { nextTick } from 'vue';
import { useDashboards } from '../useDashboards';

describe('useDashboards', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  afterEach(() => {
    localStorage.clear();
  });

  it('seeds three default dashboards on first use', () => {
    const { layouts, activeLayout } = useDashboards();
    expect(layouts.value).toHaveLength(3);
    expect(layouts.value.map((l) => l.id)).toEqual([
      'overview',
      'latency-deep-dive',
      'per-route',
    ]);
    expect(activeLayout.value?.name).toBe('Overview');
  });

  it('persists layouts to localStorage', async () => {
    const { layouts } = useDashboards();
    layouts.value[0]!.name = 'Renamed';
    await nextTick();
    const raw = localStorage.getItem('kairos.dashboards');
    expect(raw).toBeTruthy();
    const parsed = JSON.parse(raw!);
    expect(parsed[0].name).toBe('Renamed');
  });

  it('rehydrates layouts from localStorage on second use', () => {
    localStorage.setItem(
      'kairos.dashboards',
      JSON.stringify([
        { id: 'custom', name: 'My Custom', charts: [], createdAt: '', updatedAt: '' },
      ]),
    );
    const { layouts, activeLayout } = useDashboards();
    expect(layouts.value).toHaveLength(1);
    expect(layouts.value[0]!.id).toBe('custom');
    expect(activeLayout.value?.name).toBe('My Custom');
  });

  it('createDashboard adds and activates a new dashboard', () => {
    const { layouts, activeId, createDashboard } = useDashboards();
    const initialCount = layouts.value.length;
    const created = createDashboard('Test Dashboard');
    expect(layouts.value).toHaveLength(initialCount + 1);
    expect(layouts.value.find((l) => l.id === created.id)).toBeDefined();
    expect(activeId.value).toBe(created.id);
    expect(created.name).toBe('Test Dashboard');
    expect(created.charts).toEqual([]);
  });

  it('createDashboard defaults empty name to "Untitled Dashboard"', () => {
    const { createDashboard } = useDashboards();
    const created = createDashboard('   ');
    expect(created.name).toBe('Untitled Dashboard');
  });

  it('renameDashboard updates name and updatedAt', async () => {
    const { layouts, renameDashboard } = useDashboards();
    const id = layouts.value[0]!.id;
    const beforeUpdated = layouts.value[0]!.updatedAt;
    await new Promise((r) => setTimeout(r, 5));
    renameDashboard(id, 'New Name');
    expect(layouts.value[0]!.name).toBe('New Name');
    expect(layouts.value[0]!.updatedAt).not.toBe(beforeUpdated);
  });

  it('deleteDashboard removes dashboard and shifts active', () => {
    const { layouts, activeId, createDashboard, deleteDashboard } = useDashboards();
    const newOne = createDashboard('Temp');
    expect(activeId.value).toBe(newOne.id);
    deleteDashboard(newOne.id);
    expect(layouts.value.find((l) => l.id === newOne.id)).toBeUndefined();
    expect(activeId.value).not.toBe(newOne.id);
  });

  it('deleteDashboard re-seeds defaults if last dashboard removed', () => {
    const { layouts, deleteDashboard } = useDashboards();
    for (const l of [...layouts.value!]) deleteDashboard(l.id);
    expect(layouts.value!).toHaveLength(3);
    expect(layouts.value[0]!.id).toBe('overview');
  });

  it('exportLayout returns valid JSON', () => {
    const { layouts, exportLayout } = useDashboards();
    const json = exportLayout(layouts.value![0]!.id);
    const parsed = JSON.parse(json);
    expect(parsed.id).toBe(layouts.value![0]!.id);
    expect(Array.isArray(parsed.charts)).toBe(true);
  });

  it('exportLayout throws for unknown id', () => {
    const { exportLayout } = useDashboards();
    expect(() => exportLayout('does-not-exist')).toThrow();
  });

  it('importLayout adds a new dashboard from JSON', () => {
    const { layouts, importLayout } = useDashboards();
    const before = layouts.value!.length;
    const json = JSON.stringify({
      name: 'Imported',
      charts: [{ id: 'c1', type: 'requests', metricName: 'foo', timeRange: '1h', aggregation: 'raw', gridX: 0, gridY: 0, gridW: 6, gridH: 1 }],
    });
    const imported = importLayout(json);
    expect(layouts.value).toHaveLength(before + 1);
    expect(imported.name).toBe('Imported');
    expect(imported.charts).toHaveLength(1);
  });

  it('importLayout throws on invalid JSON', () => {
    const { importLayout } = useDashboards();
    expect(() => importLayout('{}')).toThrow(/charts array/);
  });

  it('saveLayout updates updatedAt timestamp', async () => {
    const { layouts, saveLayout } = useDashboards();
    const layout = layouts.value![0]!;
    const before = layout.updatedAt;
    await new Promise((r) => setTimeout(r, 5));
    saveLayout({ ...layout, name: 'Saved' });
    expect(layouts.value![0]!.name).toBe('Saved');
    expect(layouts.value![0]!.updatedAt).not.toBe(before);
  });

  it('resetToDefaults restores the three seeded dashboards', () => {
    const { layouts, createDashboard, resetToDefaults } = useDashboards();
    createDashboard('Extra');
    expect(layouts.value!.length).toBeGreaterThan(3);
    resetToDefaults();
    expect(layouts.value!).toHaveLength(3);
  });
});