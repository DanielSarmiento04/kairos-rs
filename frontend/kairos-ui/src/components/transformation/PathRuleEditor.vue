<script setup lang="ts">
import { computed } from 'vue';
import type { PathTransformation } from '@/types';

const props = defineProps<{
  modelValue?: PathTransformation | null;
  readonly?: boolean;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: PathTransformation | null];
}>();

const patternValid = computed(() => {
  if (!props.modelValue?.pattern) return true;
  try {
    new RegExp(props.modelValue.pattern);
    return true;
  } catch {
    return false;
  }
});

const enabled = computed(() => props.modelValue !== null);

function enable() {
  emit('update:modelValue', { pattern: '', replacement: '' });
}

function disable() {
  emit('update:modelValue', null);
}

function update(field: keyof PathTransformation, value: string) {
  emit('update:modelValue', { ...(props.modelValue ?? { pattern: '', replacement: '' }), [field]: value });
}
</script>

<template>
  <div class="path-editor">
    <div v-if="!enabled && !readonly" class="empty-state">
      <span class="text-muted">No path rewriting configured.</span>
      <button class="btn-sub-action" type="button" @click="enable">Enable path rewriting</button>
    </div>

    <div v-else class="path-fields">
      <label class="path-label">
        Pattern (regex)
        <input
          class="form-control"
          :class="{ invalid: !patternValid }"
          type="text"
          placeholder="^/api/v1/(.+)$"
          :value="modelValue?.pattern ?? ''"
          :disabled="readonly"
          :title="patternValid ? '' : 'Invalid regex pattern'"
          @input="update('pattern', ($event.target as HTMLInputElement).value)"
        />
      </label>
      <label class="path-label">
        Replacement
        <input
          class="form-control"
          type="text"
          placeholder="/$1"
          :value="modelValue?.replacement ?? ''"
          :disabled="readonly"
          @input="update('replacement', ($event.target as HTMLInputElement).value)"
        />
      </label>
      <button
        v-if="!readonly"
        class="btn-row-remove"
        type="button"
        title="Disable path rewriting"
        @click="disable"
      >
        ✕
      </button>
    </div>
  </div>
</template>

<style scoped>
.path-editor {
  background: #f8fafc;
  padding: 12px;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
}
.empty-state {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
}
.path-fields {
  display: flex;
  gap: 12px;
  align-items: flex-end;
  flex-wrap: wrap;
}
.path-label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1 1 240px;
  font-size: 0.78rem;
  font-weight: 600;
  color: #475569;
  text-transform: uppercase;
  letter-spacing: 0.4px;
}
.path-label .form-control {
  font-family: 'Menlo', 'Monaco', monospace;
  font-size: 0.85rem;
  text-transform: none;
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