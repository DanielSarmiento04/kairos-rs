<script setup lang="ts">
/**
 * DashboardGrid — 12-column CSS grid that lays out charts based on their
 * `gridX/gridY/gridW/gridH` config. Each cell hosts a `ChartCard` which
 * can emit a config update (e.g. when the user picks a new time range
 * inside the card) or a remove event.
 */
import type { ChartConfig, DashboardLayout } from '@/types';
import ChartCard from './ChartCard.vue';

const props = defineProps<{
  layout: DashboardLayout;
  readonly?: boolean;
}>();

const emit = defineEmits<{
  'update:layout': [value: DashboardLayout];
  'remove-chart': [chartId: string];
}>();

function updateChart(updated: ChartConfig) {
  const next: DashboardLayout = {
    ...props.layout,
    charts: props.layout.charts.map((c) => (c.id === updated.id ? updated : c)),
  };
  emit('update:layout', next);
}
</script>

<template>
  <div class="dashboard-grid">
    <div
      v-for="chart in layout.charts"
      :key="chart.id"
      class="dashboard-grid__cell"
      :style="{
        gridColumn: `${(chart.gridX ?? 0) + 1} / span ${chart.gridW ?? 6}`,
        gridRow: `${(chart.gridY ?? 0) + 1} / span ${chart.gridH ?? 1}`,
      }"
    >
      <ChartCard
        :config="chart"
        @update:config="updateChart"
        @remove="emit('remove-chart', chart.id)"
      />
    </div>
    <div v-if="layout.charts.length === 0" class="empty-grid">
      <p>This dashboard is empty. Add a chart to get started.</p>
    </div>
  </div>
</template>

<style scoped>
.dashboard-grid {
  display: grid;
  grid-template-columns: repeat(12, 1fr);
  grid-auto-rows: minmax(280px, auto);
  gap: 16px;
  padding: 16px 0;
}
.dashboard-grid__cell {
  min-width: 0;
  min-height: 0;
}
.empty-grid {
  grid-column: 1 / -1;
  padding: 60px 20px;
  text-align: center;
  color: #94a3b8;
  border: 2px dashed #cbd5e1;
  border-radius: 10px;
}
.empty-grid p { margin: 0; font-size: 0.9rem; }
</style>