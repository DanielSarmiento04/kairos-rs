<script setup lang="ts">
import { computed, ref } from 'vue';
import type { TransformationPreset } from './presets';
import { presetsForScope } from './presets';

const props = defineProps<{
  scope: 'request' | 'response';
}>();

const emit = defineEmits<{
  apply: [preset: TransformationPreset];
}>();

const selectedId = ref<string>('');

const presets = computed(() => presetsForScope(props.scope));

function applyPreset() {
  const preset = presets.value.find((p) => p.id === selectedId.value);
  if (preset) {
    emit('apply', preset);
    selectedId.value = '';
  }
}
</script>

<template>
  <div class="preset-templates">
    <select v-model="selectedId" class="form-control preset-select" :disabled="presets.length === 0">
      <option value="" disabled>{{ presets.length === 0 ? 'No presets' : 'Apply example preset…' }}</option>
      <option v-for="p in presets" :key="p.id" :value="p.id">{{ p.name }}</option>
    </select>
    <button
      class="btn-sub-action"
      type="button"
      :disabled="!selectedId"
      @click="applyPreset"
    >
      Apply
    </button>
  </div>
</template>

<style scoped>
.preset-templates {
  display: flex;
  gap: 8px;
  align-items: center;
}
.preset-select {
  font-size: 0.85rem;
  padding: 6px 10px;
  min-width: 240px;
}
</style>