<script setup lang="ts">
/**
 * DashboardsView — top-level container for the /dashboards route.
 *
 * Renders:
 *  - header with "New dashboard" + "Import" + per-active-dashboard settings
 *  - horizontal tab list of all dashboards
 *  - the active dashboard's grid of charts
 *
 * State management and persistence is delegated to `useDashboards`.
 */
import { ref } from 'vue';
import DashboardGrid from './DashboardGrid.vue';
import DashboardSettings from './DashboardSettings.vue';
import { useDashboards } from '@/composables/useDashboards';
import type { DashboardLayout } from '@/types';

const {
  layouts,
  activeId,
  activeLayout,
  setActive,
  createDashboard,
  deleteDashboard,
  saveLayout,
  exportLayout,
  importLayout,
  renameDashboard,
} = useDashboards();

const newName = ref('');
const showCreate = ref(false);
const settingsOpen = ref(false);
const settingsMode = ref<'manage' | 'import'>('manage');

function onCreate() {
  const name = newName.value.trim() || 'Untitled Dashboard';
  createDashboard(name);
  newName.value = '';
  showCreate.value = false;
}

function onImport(json: string) {
  try {
    importLayout(json);
  } catch (e) {
    alert(`Import failed: ${e instanceof Error ? e.message : String(e)}`);
  }
}

function onExport(id: string) {
  const json = exportLayout(id);
  const layout = layouts.value.find((l) => l.id === id);
  const filename = `${(layout?.name ?? id).replace(/\s+/g, '_')}.dashboard.json`;
  const blob = new Blob([json], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

function onUpdate(layout: DashboardLayout) {
  saveLayout(layout);
}

function onRemoveChart(chartId: string) {
  if (!activeLayout.value) return;
  const next: DashboardLayout = {
    ...activeLayout.value,
    charts: activeLayout.value.charts.filter((c) => c.id !== chartId),
  };
  saveLayout(next);
}
</script>

<template>
  <div class="dashboards-view">
    <div class="page-header">
      <div>
        <h1>📊 Dashboards</h1>
        <p class="subtitle">Multi-chart time-series with zoom, percentiles, and CSV export.</p>
      </div>
      <div class="header-actions">
        <button class="btn-primary" @click="showCreate = !showCreate">+ New Dashboard</button>
        <button class="btn-secondary" @click="settingsMode = 'import'; settingsOpen = true">⤓ Import</button>
        <button class="btn-secondary" :disabled="!activeLayout" @click="settingsMode = 'manage'; settingsOpen = true">⚙ Settings</button>
      </div>
    </div>

    <div v-if="showCreate" class="create-bar">
      <input
        v-model="newName"
        class="form-control"
        type="text"
        placeholder="Dashboard name…"
        @keyup.enter="onCreate"
      />
      <button class="btn-primary" @click="onCreate">Create</button>
      <button class="btn-secondary" @click="showCreate = false">Cancel</button>
    </div>

    <nav class="dashboard-tabs" v-if="layouts.length > 0">
      <button
        v-for="layout in layouts"
        :key="layout.id"
        class="tab"
        :class="{ active: activeId === layout.id }"
        @click="setActive(layout.id)"
      >
        {{ layout.name }}
      </button>
    </nav>

    <div v-if="!activeLayout" class="empty-state">
      <p>No dashboards yet. Create one to get started.</p>
    </div>
    <DashboardGrid
      v-else
      :layout="activeLayout"
      @update:layout="onUpdate"
      @remove-chart="onRemoveChart"
    />

    <DashboardSettings
      v-if="settingsMode === 'manage' && activeLayout"
      :layout="activeLayout"
      :open="settingsOpen"
      @close="settingsOpen = false"
      @rename="(id, name) => renameDashboard(id, name)"
      @delete="(id) => deleteDashboard(id)"
      @export="onExport"
    />
    <DashboardSettings
      v-else
      :open="settingsOpen"
      @close="settingsOpen = false"
      @import="onImport"
    />
  </div>
</template>

<style scoped>
.dashboards-view {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 12px;
}
.page-header h1 {
  margin: 0 0 4px;
  font-size: 1.85rem;
  color: #0f172a;
}
.subtitle {
  color: #64748b;
  font-size: 0.95rem;
  margin: 0;
}
.header-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.create-bar {
  display: flex;
  gap: 8px;
  padding: 12px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
}
.create-bar .form-control {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
}
.dashboard-tabs {
  display: flex;
  gap: 4px;
  border-bottom: 1px solid #e2e8f0;
  overflow-x: auto;
}
.tab {
  padding: 10px 16px;
  border: none;
  background: transparent;
  color: #475569;
  font-weight: 600;
  font-size: 0.85rem;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  white-space: nowrap;
}
.tab.active {
  color: #2563eb;
  border-bottom-color: #2563eb;
}
.empty-state {
  padding: 60px 20px;
  text-align: center;
  color: #94a3b8;
  border: 2px dashed #cbd5e1;
  border-radius: 10px;
}
.btn-primary {
  padding: 8px 14px;
  background: #2563eb;
  color: #ffffff;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
}
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-secondary {
  padding: 8px 14px;
  background: #f1f5f9;
  color: #475569;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
}
.btn-secondary:disabled { opacity: 0.5; cursor: not-allowed; }
</style>