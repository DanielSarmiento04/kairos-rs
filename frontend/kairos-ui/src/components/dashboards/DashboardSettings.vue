<script setup lang="ts">
/**
 * DashboardSettings — small modal to rename, delete, export, or import
 * a dashboard. Designed to be dropped into the Dashboards view header.
 */
import { ref } from 'vue';
import type { DashboardLayout } from '@/types';

const props = defineProps<{
  layout?: DashboardLayout;
  open: boolean;
}>();

const emit = defineEmits<{
  'close': [];
  'rename': [id: string, name: string];
  'delete': [id: string];
  'export': [id: string];
  'import': [json: string];
}>();

const renameValue = ref('');
const importValue = ref('');
const fileInputRef = ref<HTMLInputElement>();

function start() {
  renameValue.value = props.layout?.name ?? '';
  importValue.value = '';
}

function submitRename() {
  if (props.layout && renameValue.value.trim()) {
    emit('rename', props.layout.id, renameValue.value.trim());
    emit('close');
  }
}

function submitDelete() {
  if (props.layout && confirm(`Delete dashboard "${props.layout.name}"?`)) {
    emit('delete', props.layout.id);
    emit('close');
  }
}

function submitExport() {
  if (props.layout) {
    emit('export', props.layout.id);
  }
}

function submitImport() {
  if (importValue.value.trim()) {
    try {
      emit('import', importValue.value);
      emit('close');
    } catch (e) {
      alert(`Invalid JSON: ${e instanceof Error ? e.message : String(e)}`);
    }
  }
}

function pickFile() {
  fileInputRef.value?.click();
}

function onFile(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0];
  if (!file) return;
  const reader = new FileReader();
  reader.onload = () => {
    importValue.value = String(reader.result ?? '');
  };
  reader.readAsText(file);
}
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="settings-backdrop" @click.self="emit('close')">
      <div class="settings-modal">
        <header>
          <h3>Dashboard settings</h3>
          <button class="close-btn" @click="start()">×</button>
        </header>
        <div class="settings-body" @vue:mounted="start">
          <template v-if="layout">
            <h4>Rename</h4>
            <div class="row">
              <input v-model="renameValue" class="form-control" type="text" @keyup.enter="submitRename" />
              <button class="btn-sub-action" @click="submitRename">Save</button>
            </div>

            <h4>Export</h4>
            <p class="text-muted">Download this dashboard as a JSON file you can share or back up.</p>
            <button class="btn-sub-action" @click="submitExport">⤓ Download JSON</button>

            <h4 class="danger">Danger zone</h4>
            <button class="btn-danger" @click="submitDelete">🗑 Delete dashboard</button>
          </template>

          <template v-else>
            <h4>Import</h4>
            <p class="text-muted">Paste a previously exported dashboard JSON, or pick a file from disk.</p>
            <textarea
              v-model="importValue"
              class="form-control"
              rows="8"
              placeholder='{ "name": "My dashboard", "charts": [...] }'
            />
            <div class="row">
              <button class="btn-sub-action" @click="pickFile">📂 Choose file</button>
              <input ref="fileInputRef" type="file" accept="application/json" hidden @change="onFile" />
              <button class="btn-primary" :disabled="!importValue.trim()" @click="submitImport">Import</button>
            </div>
          </template>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.settings-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}
.settings-modal {
  background: #ffffff;
  border-radius: 12px;
  width: min(560px, 95vw);
  max-height: 90vh;
  overflow: auto;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
}
header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid #e2e8f0;
}
header h3 { margin: 0; font-size: 1.1rem; color: #0f172a; }
.close-btn {
  background: transparent;
  border: none;
  font-size: 1.4rem;
  color: #94a3b8;
  cursor: pointer;
}
.settings-body {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.settings-body h4 {
  margin: 8px 0 0;
  font-size: 0.85rem;
  color: #475569;
  text-transform: uppercase;
  letter-spacing: 0.4px;
}
.settings-body h4.danger { color: #b91c1c; }
.row {
  display: flex;
  gap: 8px;
  align-items: center;
}
.form-control {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  font-size: 0.85rem;
}
textarea.form-control {
  font-family: 'Menlo', 'Monaco', monospace;
  font-size: 0.8rem;
  resize: vertical;
}
.btn-sub-action,
.btn-primary,
.btn-danger {
  padding: 8px 14px;
  font-size: 0.85rem;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid transparent;
}
.btn-sub-action {
  background: #eff6ff;
  color: #2563eb;
  border-color: #bfdbfe;
}
.btn-primary {
  background: #2563eb;
  color: #ffffff;
  border: none;
}
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
.btn-danger {
  background: #fee2e2;
  color: #b91c1c;
  border-color: #fecaca;
}
.text-muted { color: #94a3b8; font-size: 0.8rem; margin: 0 0 4px; }
</style>