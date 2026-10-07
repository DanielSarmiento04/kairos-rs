<script setup lang="ts">
import { computed } from 'vue';
import type { StatusCodeMapping } from '@/types';

const props = defineProps<{
  modelValue: StatusCodeMapping;
  readonly?: boolean;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: StatusCodeMapping];
  'remove': [];
}>();

const isValidCode = (n: number) => Number.isInteger(n) && n >= 100 && n <= 599;

const fromValid = computed(() => isValidCode(props.modelValue.from));
const toValid = computed(() => isValidCode(props.modelValue.to));

function update(field: keyof StatusCodeMapping, value: number | string | null) {
  emit('update:modelValue', { ...props.modelValue, [field]: value });
}
</script>

<template>
  <div class="mapping-row">
    <label class="code-label">
      From
      <input
        class="form-control"
        :class="{ invalid: !fromValid }"
        type="number"
        min="100"
        max="599"
        :value="modelValue.from"
        :disabled="readonly"
        @input="update('from', Number(($event.target as HTMLInputElement).value))"
      />
    </label>

    <span class="arrow">→</span>

    <label class="code-label">
      To
      <input
        class="form-control"
        :class="{ invalid: !toValid }"
        type="number"
        min="100"
        max="599"
        :value="modelValue.to"
        :disabled="readonly"
        @input="update('to', Number(($event.target as HTMLInputElement).value))"
      />
    </label>

    <label class="condition-label">
      Condition (optional)
      <input
        class="form-control"
        type="text"
        placeholder="path == '/health'"
        :value="modelValue.condition ?? ''"
        :disabled="readonly"
        @input="update('condition', ($event.target as HTMLInputElement).value || null)"
      />
    </label>

    <button
      v-if="!readonly"
      class="btn-row-remove"
      type="button"
      title="Remove mapping"
      @click="emit('remove')"
    >
      🗑
    </button>
  </div>
</template>

<style scoped>
.mapping-row {
  display: flex;
  gap: 10px;
  align-items: flex-end;
  background: #f8fafc;
  padding: 10px 12px;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
  margin-bottom: 8px;
  flex-wrap: wrap;
}
.code-label,
.condition-label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 0.74rem;
  font-weight: 600;
  color: #475569;
  text-transform: uppercase;
  letter-spacing: 0.4px;
}
.code-label { flex: 0 0 110px; }
.condition-label { flex: 1 1 200px; min-width: 160px; }
.arrow {
  font-size: 1.2rem;
  color: #64748b;
  padding-bottom: 8px;
}
.invalid {
  border-color: #dc2626 !important;
  box-shadow: 0 0 0 2px rgba(220, 38, 38, 0.1);
}
.btn-row-remove {
  height: 38px;
  padding: 0 12px;
  background: #fee2e2;
  color: #dc2626;
  border: 1px solid #fecaca;
  border-radius: 6px;
  cursor: pointer;
}
</style>