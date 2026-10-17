<script setup lang="ts">
import type { QueryTransformation, TransformAction } from '@/types';

const props = defineProps<{
  modelValue: QueryTransformation;
  readonly?: boolean;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: QueryTransformation];
  'remove': [];
}>();

const ACTIONS: { value: Exclude<TransformAction, 'replace'>; label: string }[] = [
  { value: 'add', label: 'Add (if absent)' },
  { value: 'set', label: 'Set (override)' },
  { value: 'remove', label: 'Remove' },
];

function update(field: keyof QueryTransformation, value: unknown) {
  emit('update:modelValue', { ...props.modelValue, [field]: value });
}

function onActionChange(action: QueryTransformation['action']) {
  emit('update:modelValue', {
    ...props.modelValue,
    action,
    value: action === 'remove' ? null : (props.modelValue.value ?? null),
  });
}
</script>

<template>
  <div class="rule-row">
    <select
      class="form-control action-select"
      :value="modelValue.action"
      :disabled="readonly"
      @change="onActionChange(($event.target as HTMLSelectElement).value as QueryTransformation['action'])"
    >
      <option v-for="a in ACTIONS" :key="a.value" :value="a.value">{{ a.label }}</option>
    </select>

    <input
      class="form-control name-input"
      type="text"
      placeholder="Param name (e.g. api_key)"
      :value="modelValue.name"
      :disabled="readonly"
      @input="update('name', ($event.target as HTMLInputElement).value)"
    />

    <input
      v-if="modelValue.action !== 'remove'"
      class="form-control value-input"
      type="text"
      placeholder="Value"
      :value="modelValue.value ?? ''"
      :disabled="readonly"
      @input="update('value', ($event.target as HTMLInputElement).value)"
    />

    <button
      v-if="!readonly"
      class="btn-row-remove"
      type="button"
      title="Remove rule"
      @click="emit('remove')"
    >
      🗑
    </button>
  </div>
</template>

<style scoped>
.rule-row {
  display: flex;
  gap: 8px;
  align-items: center;
  background: #f8fafc;
  padding: 10px 12px;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
  margin-bottom: 8px;
}
.action-select { flex: 0 0 160px; }
.name-input { flex: 1 1 160px; min-width: 140px; }
.value-input { flex: 2 1 200px; min-width: 160px; }
.btn-row-remove { flex: 0 0 auto; }
</style>