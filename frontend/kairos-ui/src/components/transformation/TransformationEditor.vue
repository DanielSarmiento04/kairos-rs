<script setup lang="ts">
import { computed } from 'vue';
import type {
  RequestTransformation,
  ResponseTransformation,
  HeaderTransformation,
  QueryTransformation,
  StatusCodeMapping,
} from '@/types';
import HeaderRuleRow from './HeaderRuleRow.vue';
import QueryRuleRow from './QueryRuleRow.vue';
import PathRuleEditor from './PathRuleEditor.vue';
import StatusCodeMappingRow from './StatusCodeMappingRow.vue';
import PresetTemplates from './PresetTemplates.vue';
import PreviewPanel from './PreviewPanel.vue';
import type { TransformationPreset } from './presets';

const props = defineProps<{
  requestTransformation?: RequestTransformation | null;
  responseTransformation?: ResponseTransformation | null;
  readonly?: boolean;
}>();

const emit = defineEmits<{
  'update:requestTransformation': [value: RequestTransformation | null];
  'update:responseTransformation': [value: ResponseTransformation | null];
}>();

/* ---------- empty-state handling ---------- */
const anyConfigured = computed(
  () => !!props.requestTransformation || !!props.responseTransformation,
);

function enableTransformations() {
  emit('update:requestTransformation', { headers: [], path: null, query_params: [] });
  emit('update:responseTransformation', { headers: [], status_code_mappings: [] });
}

/* ---------- request transformation state ---------- */
const req = computed<RequestTransformation>(() =>
  props.requestTransformation ?? { headers: [], path: null, query_params: [] },
);
const reqHeaders = computed<HeaderTransformation[]>(() => req.value.headers ?? []);
const reqQueryParams = computed<QueryTransformation[]>(() => req.value.query_params ?? []);

function commitReq(next: RequestTransformation) {
  emit('update:requestTransformation', next);
}

function addHeaderRule() {
  commitReq({
    ...req.value,
    headers: [
      ...reqHeaders.value,
      { action: 'add', name: '', value: '', pattern: null, replacement: null },
    ],
  });
}

function updateHeaderAt(idx: number, value: HeaderTransformation) {
  const next = [...reqHeaders.value];
  next[idx] = value;
  commitReq({ ...req.value, headers: next });
}

function removeHeaderAt(idx: number) {
  const next = reqHeaders.value.filter((_, i) => i !== idx);
  commitReq({ ...req.value, headers: next });
}

function updatePath(value: import('@/types').PathTransformation | null) {
  commitReq({ ...req.value, path: value });
}

function addQueryRule() {
  commitReq({
    ...req.value,
    query_params: [...reqQueryParams.value, { action: 'add', name: '', value: '' }],
  });
}

function updateQueryAt(idx: number, value: QueryTransformation) {
  const next = [...reqQueryParams.value];
  next[idx] = value;
  commitReq({ ...req.value, query_params: next });
}

function removeQueryAt(idx: number) {
  const next = reqQueryParams.value.filter((_, i) => i !== idx);
  commitReq({ ...req.value, query_params: next });
}

/* ---------- response transformation state ---------- */
const res = computed<ResponseTransformation>(() =>
  props.responseTransformation ?? { headers: [], status_code_mappings: [] },
);
const resHeaders = computed<HeaderTransformation[]>(() => res.value.headers ?? []);
const resMappings = computed<StatusCodeMapping[]>(() => res.value.status_code_mappings ?? []);

function commitRes(next: ResponseTransformation) {
  emit('update:responseTransformation', next);
}

function addResHeaderRule() {
  commitRes({
    ...res.value,
    headers: [
      ...resHeaders.value,
      { action: 'remove', name: '', value: null, pattern: null, replacement: null },
    ],
  });
}

function updateResHeaderAt(idx: number, value: HeaderTransformation) {
  const next = [...resHeaders.value];
  next[idx] = value;
  commitRes({ ...res.value, headers: next });
}

function removeResHeaderAt(idx: number) {
  const next = resHeaders.value.filter((_, i) => i !== idx);
  commitRes({ ...res.value, headers: next });
}

function addStatusRule() {
  commitRes({
    ...res.value,
    status_code_mappings: [...resMappings.value, { from: 404, to: 200, condition: null }],
  });
}

function updateMappingAt(idx: number, value: StatusCodeMapping) {
  const next = [...resMappings.value];
  next[idx] = value;
  commitRes({ ...res.value, status_code_mappings: next });
}

function removeMappingAt(idx: number) {
  const next = resMappings.value.filter((_, i) => i !== idx);
  commitRes({ ...res.value, status_code_mappings: next });
}

/* ---------- preset application (dedupe by action+name) ---------- */
function applyReqPreset(preset: TransformationPreset) {
  if (!preset.request) return;
  const incoming = preset.request;
  const next: RequestTransformation = {
    ...req.value,
    headers: [
      ...reqHeaders.value,
      ...(incoming.headers ?? []).filter(
        (h) => !reqHeaders.value.some((x) => x.action === h.action && x.name === h.name),
      ),
    ],
    query_params: [
      ...reqQueryParams.value,
      ...(incoming.query_params ?? []).filter(
        (q) => !reqQueryParams.value.some((x) => x.action === q.action && x.name === q.name),
      ),
    ],
    path: incoming.path ?? req.value.path ?? null,
  };
  commitReq(next);
}

function applyResPreset(preset: TransformationPreset) {
  if (!preset.response) return;
  const incoming = preset.response;
  const next: ResponseTransformation = {
    ...res.value,
    headers: [
      ...resHeaders.value,
      ...(incoming.headers ?? []).filter(
        (h) => !resHeaders.value.some((x) => x.action === h.action && x.name === h.name),
      ),
    ],
    status_code_mappings: [
      ...resMappings.value,
      ...(incoming.status_code_mappings ?? []).filter(
        (m) => !resMappings.value.some((x) => x.from === m.from && x.to === m.to),
      ),
    ],
  };
  commitRes(next);
}
</script>

<template>
  <section class="transform-editor">
    <div v-if="!anyConfigured && !readonly" class="empty-state">
      <div>
        <h3>No transformations configured</h3>
        <p class="text-muted">
          Add request/response transformations to modify headers, query params, paths, or status codes.
        </p>
      </div>
      <button class="btn-primary" type="button" @click="enableTransformations">
        Enable Transformations
      </button>
    </div>

    <template v-else>
      <!-- REQUEST SECTION -->
      <fieldset class="section">
        <div class="section-header">
          <legend>📤 Request Transformation</legend>
          <PresetTemplates scope="request" @apply="applyReqPreset" />
        </div>

        <div class="sub-section">
          <h5>Headers</h5>
          <HeaderRuleRow
            v-for="(h, i) in reqHeaders"
            :key="`req-h-${i}`"
            :model-value="h"
            :readonly="readonly"
            @update:model-value="updateHeaderAt(i, $event)"
            @remove="removeHeaderAt(i)"
          />
          <button
            v-if="!readonly"
            class="btn-add"
            type="button"
            @click="addHeaderRule"
          >
            + Add header rule
          </button>
        </div>

        <div class="sub-section">
          <h5>Path Rewriting</h5>
          <PathRuleEditor
            :model-value="req.path"
            :readonly="readonly"
            @update:model-value="updatePath"
          />
        </div>

        <div class="sub-section">
          <h5>Query Params</h5>
          <QueryRuleRow
            v-for="(q, i) in reqQueryParams"
            :key="`req-q-${i}`"
            :model-value="q"
            :readonly="readonly"
            @update:model-value="updateQueryAt(i, $event)"
            @remove="removeQueryAt(i)"
          />
          <button
            v-if="!readonly"
            class="btn-add"
            type="button"
            @click="addQueryRule"
          >
            + Add query rule
          </button>
        </div>
      </fieldset>

      <!-- RESPONSE SECTION -->
      <fieldset class="section">
        <div class="section-header">
          <legend>📥 Response Transformation</legend>
          <PresetTemplates scope="response" @apply="applyResPreset" />
        </div>

        <div class="sub-section">
          <h5>Headers</h5>
          <HeaderRuleRow
            v-for="(h, i) in resHeaders"
            :key="`res-h-${i}`"
            :model-value="h"
            :readonly="readonly"
            @update:model-value="updateResHeaderAt(i, $event)"
            @remove="removeResHeaderAt(i)"
          />
          <button
            v-if="!readonly"
            class="btn-add"
            type="button"
            @click="addResHeaderRule"
          >
            + Add header rule
          </button>
        </div>

        <div class="sub-section">
          <h5>Status Code Mappings</h5>
          <StatusCodeMappingRow
            v-for="(m, i) in resMappings"
            :key="`res-m-${i}`"
            :model-value="m"
            :readonly="readonly"
            @update:model-value="updateMappingAt(i, $event)"
            @remove="removeMappingAt(i)"
          />
          <button
            v-if="!readonly"
            class="btn-add"
            type="button"
            @click="addStatusRule"
          >
            + Add status mapping
          </button>
        </div>
      </fieldset>

      <PreviewPanel
        :request-transformation="props.requestTransformation"
        :response-transformation="props.responseTransformation"
      />
    </template>
  </section>
</template>

<style scoped>
.transform-editor {
  display: flex;
  flex-direction: column;
  gap: 18px;
}
.empty-state {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  padding: 24px;
  background: #f8fafc;
  border: 2px dashed #cbd5e1;
  border-radius: 10px;
}
.empty-state h3 { margin: 0 0 6px; color: #0f172a; font-size: 1rem; }
.empty-state p { margin: 0; font-size: 0.85rem; }
.section {
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 16px;
  background: #ffffff;
}
.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 12px;
  margin-bottom: 14px;
}
.section-header legend {
  font-weight: 700;
  font-size: 0.95rem;
  color: #0f172a;
  padding: 0;
}
.sub-section { margin-bottom: 16px; }
.sub-section:last-child { margin-bottom: 0; }
.sub-section h5 {
  margin: 0 0 8px;
  font-size: 0.78rem;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: #475569;
}
.btn-add {
  background: #eff6ff;
  color: #2563eb;
  border: 1px dashed #bfdbfe;
  border-radius: 6px;
  padding: 8px 14px;
  font-weight: 600;
  font-size: 0.82rem;
  cursor: pointer;
  width: 100%;
  margin-top: 4px;
}
.btn-add:hover { background: #dbeafe; }
.btn-primary {
  padding: 10px 18px;
  background: #2563eb;
  border: none;
  color: #ffffff;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
}
.text-muted { color: #94a3b8; }
</style>