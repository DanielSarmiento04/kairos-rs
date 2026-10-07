<script setup lang="ts">
const props = defineProps<{
  modelValue: number[];
  /** Available percentile options; defaults to common set. */
  options?: number[];
}>();

const emit = defineEmits<{
  'update:modelValue': [value: number[]];
}>();

const DEFAULTS = [50, 75, 90, 95, 99, 99.9];
const available = props.options ?? DEFAULTS;

function toggle(p: number) {
  const set = new Set(props.modelValue);
  if (set.has(p)) set.delete(p);
  else set.add(p);
  emit(
    'update:modelValue',
    available.filter((x) => set.has(x)).sort((a, b) => a - b),
  );
}
</script>

<template>
  <div class="percentile-selector">
    <label v-for="p in available" :key="p" class="chip" :class="{ active: modelValue.includes(p) }">
      <input
        type="checkbox"
        :checked="modelValue.includes(p)"
        @change="toggle(p)"
      />
      <span>p{{ p }}</span>
    </label>
  </div>
</template>

<style scoped>
.percentile-selector {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.chip {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border: 1px solid #cbd5e1;
  background: #ffffff;
  color: #475569;
  border-radius: 14px;
  font-size: 0.75rem;
  font-weight: 600;
  cursor: pointer;
}
.chip input { display: none; }
.chip.active {
  background: #eef2ff;
  color: #4338ca;
  border-color: #c7d2fe;
}
</style>