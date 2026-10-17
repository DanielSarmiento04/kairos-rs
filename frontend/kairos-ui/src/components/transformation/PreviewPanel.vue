<script setup lang="ts">
import { computed, reactive } from 'vue';
import type { RequestTransformation, ResponseTransformation } from '@/types';
import { useTransformPreview } from '@/composables/useTransformPreview';

const props = defineProps<{
  requestTransformation?: RequestTransformation | null;
  responseTransformation?: ResponseTransformation | null;
}>();

const { preview } = useTransformPreview();

const input = reactive<{
  method: string;
  path: string;
  headers: Record<string, string>;
  query: Record<string, string>;
  simulatedStatus: number;
}>({
  method: 'GET',
  path: '/api/v1/users/123',
  headers: {
    Authorization: 'Bearer xxx',
    Accept: 'application/json',
    'User-Agent': 'demo/1.0',
  },
  query: { debug: '1', region: 'us' },
  simulatedStatus: 200,
});

const result = computed(() =>
  preview(input, props.requestTransformation ?? null, props.responseTransformation ?? null, input.simulatedStatus),
);

function addHeader() {
  const key = `X-Custom-${Object.keys(input.headers).length}`;
  input.headers[key] = '';
}

function removeHeader(name: string) {
  delete input.headers[name];
}

function addQuery() {
  const key = `param${Object.keys(input.query).length}`;
  input.query[key] = '';
}

function removeQuery(name: string) {
  delete input.query[name];
}
</script>

<template>
  <section class="preview-panel">
    <header class="preview-header">
      <h3>🔍 Live Preview</h3>
      <span class="text-muted">Sample request simulated through current transformation rules.</span>
    </header>

    <div class="preview-grid">
      <!-- INPUT -->
      <div class="preview-col input-col">
        <h4>Sample Request</h4>

        <label class="mini-label">
          Method
          <select v-model="input.method" class="form-control">
            <option>GET</option><option>POST</option><option>PUT</option>
            <option>PATCH</option><option>DELETE</option>
          </select>
        </label>
        <label class="mini-label">
          Path
          <input v-model="input.path" class="form-control mono" type="text" />
        </label>

        <div class="mini-label">
            Headers
            <div v-for="(_, name) in input.headers" :key="name" class="kv-row">
              <code class="kv-key">{{ name }}</code>
              <input
                v-model="input.headers[name]"
                class="form-control kv-val"
                type="text"
              />
              <button class="kv-remove" type="button" @click="removeHeader(name)">×</button>
            </div>
            <button class="btn-sub-action" type="button" @click="addHeader">+ Add header</button>
        </div>

        <div class="mini-label">
            Query Params
            <div v-for="(_, name) in input.query" :key="name" class="kv-row">
              <code class="kv-key">{{ name }}</code>
              <input
                v-model="input.query[name]"
                class="form-control kv-val"
                type="text"
              />
              <button class="kv-remove" type="button" @click="removeQuery(name)">×</button>
            </div>
            <button class="btn-sub-action" type="button" @click="addQuery">+ Add query</button>
        </div>

        <label class="mini-label">
          Simulated response status
          <input v-model.number="input.simulatedStatus" class="form-control" type="number" min="100" max="599" />
        </label>
      </div>

      <!-- OUTPUT -->
      <div class="preview-col output-col">
        <h4>After Transformation</h4>

        <div class="kv-block">
          <div class="kv-line"><span class="kv-key">Path</span> <code class="mono">{{ result.request.path }}</code></div>
        </div>

        <div class="kv-block">
            Headers
          <div v-for="(v, k) in result.request.headers" :key="k" class="kv-line">
            <code class="kv-key">{{ k }}</code>: <code class="mono">{{ v }}</code>
          </div>
        </div>

        <div class="kv-block">
            Query
          <div v-for="(v, k) in result.request.query" :key="k" class="kv-line">
            <code class="kv-key">{{ k }}</code>: <code class="mono">{{ v }}</code>
          </div>
        </div>

        <div class="kv-block response-block">
            Response
          <div class="kv-line">
            <span class="kv-key">Status</span>:
            <code class="mono" :class="`status-${Math.floor(result.response.status / 100)}xx`">
              {{ result.response.status }}
            </code>
          </div>
          <div v-for="(v, k) in result.response.headers" :key="k" class="kv-line">
            <code class="kv-key">{{ k }}</code>: <code class="mono">{{ v }}</code>
          </div>
        </div>

        <div v-if="result.changes.length > 0" class="changes">
          <h4>Changes ({{ result.changes.length }})</h4>
          <ul>
            <li v-for="(c, i) in result.changes" :key="i">
              <code>{{ c.field }}</code>: <s class="diff-before">{{ c.before }}</s> → <code class="diff-after">{{ c.after }}</code>
            </li>
          </ul>
        </div>
        <div v-else class="changes empty">
          <em>No changes will be applied.</em>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.preview-panel {
  margin-top: 24px;
  border-top: 1px solid #e2e8f0;
  padding-top: 20px;
}
.preview-header h3 {
  margin: 0 0 4px;
  font-size: 1rem;
  color: #0f172a;
}
.preview-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
  margin-top: 16px;
}
@media (max-width: 900px) {
  .preview-grid { grid-template-columns: 1fr; }
}
.preview-col {
  background: #f8fafc;
  padding: 14px;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
}
.preview-col h4 {
  margin: 0 0 12px;
  font-size: 0.85rem;
  color: #334155;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.mini-label {
  display: block;
  font-size: 0.74rem;
  font-weight: 600;
  color: #475569;
  text-transform: uppercase;
  letter-spacing: 0.4px;
  margin-bottom: 10px;
}
.kv-row {
  display: flex;
  gap: 6px;
  align-items: center;
  margin-bottom: 6px;
}
.kv-key {
  flex: 0 0 130px;
  font-family: 'Menlo', 'Monaco', monospace;
  font-size: 0.78rem;
  color: #334155;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.kv-val { flex: 1; font-family: 'Menlo', 'Monaco', monospace; font-size: 0.82rem; }
.kv-remove {
  background: transparent;
  border: 1px solid #fecaca;
  color: #dc2626;
  border-radius: 4px;
  cursor: pointer;
  width: 24px;
  height: 28px;
}
.kv-block {
  background: #ffffff;
  padding: 8px 10px;
  border-radius: 6px;
  border: 1px solid #e2e8f0;
  margin-bottom: 12px;
  font-size: 0.82rem;
}
.kv-line {
  margin: 3px 0;
  font-family: 'Menlo', 'Monaco', monospace;
  color: #0f172a;
}
.mono { font-family: 'Menlo', 'Monaco', monospace; font-size: 0.85rem; }
.status-2xx { color: #059669; font-weight: 700; }
.status-3xx { color: #2563eb; font-weight: 700; }
.status-4xx { color: #d97706; font-weight: 700; }
.status-5xx { color: #dc2626; font-weight: 700; }
.changes {
  background: #fefce8;
  border: 1px solid #fde047;
  border-radius: 6px;
  padding: 8px 12px;
  font-size: 0.8rem;
  margin-top: 8px;
}
.changes h4 { margin: 0 0 6px; font-size: 0.8rem; color: #854d0e; }
.changes ul { margin: 0; padding-left: 18px; }
.changes li { margin: 3px 0; }
.diff-before { color: #b91c1c; text-decoration: line-through; }
.diff-after { color: #15803d; }
.empty { background: #f1f5f9; border-color: #cbd5e1; color: #64748b; }
.response-block { border-left: 3px solid #2563eb; }
</style>