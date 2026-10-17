<!--
  Dashboards.vue — route entry for /dashboards.
  All view logic (header, tabs, grid, settings, import/export) lives in
  `<DashboardsView>` from `components/dashboards/`. This file exists so
  the router has a stable lazy import target and the view name mirrors
  the URL.
-->
<script setup lang="ts">
import { DashboardsView } from '@/components/dashboards';
</script>

<template>
  <DashboardsView />
</template>