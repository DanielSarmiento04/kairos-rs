<script setup lang="ts">
import { computed } from 'vue';
import type { HeaderTransformation, TransformAction } from '@/types';

const props = defineProps<{
  modelValue: HeaderTransformation;
  readonly?: boolean;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: HeaderTransformation];
  'remove': [];
}>();

const ACTIONS: { value: TransformAction; label: string }[] = [
  { value: 'add', label: 'Add (if absent)' },
  { value: 'set', label: 'Set (override)' },
  { value: 'remove', label: 'Remove' },
  { value: 'replace', label: 'Replace (regex)' },
];

const showValue = computed(
  () => props.modelValue.action === 'add' || props.modelValue.action === 'set',
);
const showReplace = computed(() => props.modelValue.action === 'replace');

const patternValid = computed(() => {
  if (!showReplace.value || !props.modelValue.pattern) return true;
  try {
    new RegExp(props.modelValue.pattern);
    return true;
  } catch {
    return false;
  }
});

function update(field: keyof HeaderTransformation, value: unknown) {
  emit('update:modelValue', { ...props.modelValue, [field]: value });
}

function onActionChange(action: TransformAction) {
  // Clear action-specific fields when switching actions
  emit('update:modelValue', {
    ...props.modelValue,
    action,
    value: action === 'remove' ? null : (props.modelValue.value ?? null),
    pattern: action === 'replace' ? (props.modelValue.pattern ?? '') : null,
    replacement: action === 'replace' ? (props.modelValue.replacement ?? '') : null,
  });
}
</script>

<template>
  <div class="rule-row">
    <select
      class="form-control action-select"
      :value="modelValue.action"
      :disabled="readonly"
      @change="onActionChange(($event.target as HTMLSelectElement).value as TransformAction)"
    >
      <option v-for="a in ACTIONS" :key="a.value" :value="a.value">{{ a.label }}</option>
    </select>

    <input
      class="form-control name-input"
      type="text"
      placeholder="Header name (e.g. Authorization)"
      :value="modelValue.name"
      :disabled="readonly"
      @input="update('name', ($event.target as HTMLInputElement).value)"
    />

    <input
      v-if="showValue"
      class="form-control value-input"
      type="text"
      placeholder="Value"
      :value="modelValue.value ?? ''"
      :disabled="readonly"
      @input="update('value', ($event.target as HTMLInputElement).value)"
    />

    <template v-if="showReplace">
      <input
        class="form-control pattern-input"
        :class="{ invalid: !patternValid }"
        type="text"
        placeholder="Regex pattern"
        :value="modelValue.pattern ?? ''"
        :disabled="readonly"
        :title="patternValid ? '' : 'Invalid regex pattern'"
        @input="update('pattern', ($event.target as HTMLInputElement).value)"
      />
      <input
        class="form-control replacement-input"
        type="text"
        placeholder="Replacement (e.g. /$1)"
        :value="modelValue.replacement ?? ''"
        :disabled="readonly"
        @input="update('replacement', ($event.target as HTMLInputElement).value)"
      />
    </template>

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
  flex-wrap: wrap;
}
.action-select { flex: 0 0 160px; }
.name-input { flex: 1 1 160px; min-width: 140px; }
.value-input { flex: 2 1 200px; min-width: 160px; }
.pattern-input { flex: 1 1 180px; min-width: 140px; }
.replacement-input { flex: 1 1 160px; min-width: 140px; }
.btn-row-remove { flex: 0 0 auto; }

.invalid {
  border-color: #dc2626 !important;
  box-shadow: 0 0 0 2px rgba(220, 38, 38, 0.1);
}
</style>