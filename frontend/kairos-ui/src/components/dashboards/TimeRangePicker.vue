<script setup lang="ts">
import type { CustomTimeRange, TimeRangePreset } from '@/types';

const props = defineProps<{
  modelValue: TimeRangePreset;
  customRange?: CustomTimeRange;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: TimeRangePreset];
  'update:customRange': [value: CustomTimeRange];
}>();

const PRESETS: TimeRangePreset[] = ['5m', '15m', '1h', '6h', '24h', '7d'];

function setPreset(p: TimeRangePreset) {
  emit('update:modelValue', p);
}

function setCustom(start: string, end: string) {
  emit('update:modelValue', 'custom');
  emit('update:customRange', { start, end });
}
</script>

<template>
  <div class="time-range-picker">
    <div class="presets">
      <button
        v-for="p in PRESETS"
        :key="p"
        type="button"
        class="preset-btn"
        :class="{ active: modelValue === p }"
        @click="setPreset(p)"
      >
        {{ p }}
      </button>
    </div>
    <div class="custom">
      <button
        type="button"
        class="preset-btn"
        :class="{ active: modelValue === 'custom' }"
        @click="setCustom(customRange?.start ?? new Date(Date.now() - 3600_000).toISOString(), customRange?.end ?? new Date().toISOString())"
      >
        Custom
      </button>
      <input
        v-if="modelValue === 'custom'"
        class="form-control"
        type="datetime-local"
        :value="customRange?.start?.slice(0, 16) ?? ''"
        @change="(e) => setCustom(new Date((e.target as HTMLInputElement).value).toISOString(), customRange?.end ?? new Date().toISOString())"
      />
      <input
        v-if="modelValue === 'custom'"
        class="form-control"
        type="datetime-local"
        :value="customRange?.end?.slice(0, 16) ?? ''"
        @change="(e) => setCustom(customRange?.start ?? new Date(Date.now() - 3600_000).toISOString(), new Date((e.target as HTMLInputElement).value).toISOString())"
      />
    </div>
  </div>
</template>

<style scoped>
.time-range-picker {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}
.presets,
.custom {
  display: flex;
  gap: 4px;
  align-items: center;
  flex-wrap: wrap;
}
.preset-btn {
  padding: 6px 10px;
  font-size: 0.78rem;
  font-weight: 600;
  border: 1px solid #cbd5e1;
  background: #ffffff;
  color: #475569;
  border-radius: 6px;
  cursor: pointer;
}
.preset-btn.active {
  background: #2563eb;
  color: #ffffff;
  border-color: #2563eb;
}
.form-control {
  padding: 4px 8px;
  font-size: 0.8rem;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
}
</style>