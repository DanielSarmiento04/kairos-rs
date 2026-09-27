<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(
  defineProps<{
    data: number[];
    width?: number;
    height?: number;
    color?: string;
    fillColor?: string;
    strokeWidth?: number;
    unit?: string;
  }>(),
  {
    width: 280,
    height: 60,
    color: '#3b82f6',
    fillColor: 'rgba(59, 130, 246, 0.12)',
    strokeWidth: 2,
    unit: '',
  }
);

const points = computed(() => {
  if (!props.data || props.data.length < 2) return '';
  const min = Math.min(...props.data, 0);
  const max = Math.max(...props.data, 1);
  const range = max - min || 1;

  const step = props.width / (props.data.length - 1);
  return props.data
    .map((val, idx) => {
      const x = (idx * step).toFixed(1);
      const y = (props.height - ((val - min) / range) * (props.height - 8) - 4).toFixed(1);
      return `${x},${y}`;
    })
    .join(' ');
});

const areaPoints = computed(() => {
  if (!points.value) return '';
  const firstX = '0';
  const lastX = props.width.toString();
  const bottomY = props.height.toString();
  return `${firstX},${bottomY} ${points.value} ${lastX},${bottomY}`;
});

const latestValue = computed(() => {
  if (!props.data || props.data.length === 0) return 0;
  return props.data[props.data.length - 1];
});
</script>

<template>
  <div class="sparkline-wrapper">
    <svg :viewBox="`0 0 ${width} ${height}`" class="sparkline-svg" preserveAspectRatio="none">
      <defs>
        <linearGradient :id="`grad-${color.replace('#', '')}`" x1="0" y1="0" x2="0" y2="1">
          <stop offset="0%" :stop-color="color" stop-opacity="0.25" />
          <stop offset="100%" :stop-color="color" stop-opacity="0.0" />
        </linearGradient>
      </defs>
      <polygon v-if="areaPoints" :points="areaPoints" :fill="`url(#grad-${color.replace('#', '')})`" />
      <polyline
        v-if="points"
        :points="points"
        fill="none"
        :stroke="color"
        :stroke-width="strokeWidth"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
    <div class="sparkline-current" :style="{ color }">
      {{ latestValue }}{{ unit }}
    </div>
  </div>
</template>

<style scoped>
.sparkline-wrapper {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
}

.sparkline-svg {
  width: 100%;
  height: 100%;
  overflow: visible;
}

.sparkline-current {
  position: absolute;
  top: 4px;
  right: 6px;
  font-size: 0.75rem;
  font-weight: 700;
  font-family: monospace;
}
</style>
