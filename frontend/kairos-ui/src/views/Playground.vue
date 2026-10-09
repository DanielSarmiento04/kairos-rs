<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { apiService } from '../services/api';
import type { Router, PlaygroundResponse } from '../types';
import StatusBadge from '../components/StatusBadge.vue';

const routes = ref<Router[]>([]);
const selectedRoute = ref<string>('');
const method = ref<string>('GET');
const path = ref<string>('/api/routes');
const headersText = ref<string>('{\n  "Content-Type": "application/json"\n}');
const requestBody = ref<string>('{\n  "prompt": "Hello Kairos Gateway AI, how do you optimize routing?"\n}');

const loading = ref(false);
const responseData = ref<PlaygroundResponse | null>(null);
const rawJwtToken = ref<string>('');
const isStreaming = ref<boolean>(false);
const streamTtftMs = ref<number | null>(null);
const streamChunkCount = ref<number>(0);

onMounted(async () => {
  try {
    routes.value = await apiService.getRoutes();
    const firstRoute = routes.value[0];
    if (firstRoute) {
      selectedRoute.value = firstRoute.external_path;
      path.value = firstRoute.external_path;
      method.value = firstRoute.methods[0] || 'GET';
      if (firstRoute.ai_policy?.streaming) {
        isStreaming.value = true;
      }
    }
  } catch (err: unknown) {
    console.error('Failed to load routes in playground:', err);
  }
});

const onRouteSelect = () => {
  if (selectedRoute.value) {
    path.value = selectedRoute.value;
    const found = routes.value.find(r => r.external_path === selectedRoute.value);
    if (found && found.methods.length > 0 && found.methods[0]) {
      method.value = found.methods[0];
    }
    if (found?.ai_policy?.streaming) {
      isStreaming.value = true;
    }
  }
};

const addBearerAuth = () => {
  try {
    const parsed = JSON.parse(headersText.value);
    parsed['Authorization'] = `Bearer ${rawJwtToken.value || 'test.jwt.token'}`;
    headersText.value = JSON.stringify(parsed, null, 2);
  } catch {
    headersText.value = `{\n  "Content-Type": "application/json",\n  "Authorization": "Bearer ${rawJwtToken.value || 'test.jwt.token'}"\n}`;
  }
};

const sendRequest = async () => {
  loading.value = true;
  responseData.value = null;
  streamTtftMs.value = null;
  streamChunkCount.value = 0;

  let parsedHeaders: Record<string, string> = {};
  try {
    parsedHeaders = JSON.parse(headersText.value);
  } catch {
    alert('Headers must be valid JSON');
    loading.value = false;
    return;
  }

  if (isStreaming.value) {
    responseData.value = {
      status: 200,
      statusText: 'Streaming...',
      latency_ms: 0,
      headers: {},
      body: '',
    };
    const startTime = performance.now();

    try {
      if (!parsedHeaders['Accept']) {
        parsedHeaders['Accept'] = 'text/event-stream';
      }
      const response = await fetch(path.value, {
        method: method.value,
        headers: parsedHeaders,
        body: method.value !== 'GET' && method.value !== 'HEAD' ? requestBody.value : undefined,
      });

      const responseHeaders: Record<string, string> = {};
      response.headers.forEach((val, key) => {
        responseHeaders[key] = val;
      });

      responseData.value.status = response.status;
      responseData.value.statusText = response.statusText;
      responseData.value.headers = responseHeaders;

      if (!response.body) {
        responseData.value.body = 'No response body stream received from server.';
        loading.value = false;
        return;
      }

      const reader = response.body.getReader();
      const decoder = new TextDecoder();
      let done = false;

      while (!done) {
        const { value, done: streamDone } = await reader.read();
        done = streamDone;
        if (value) {
          if (streamTtftMs.value === null) {
            streamTtftMs.value = Math.round(performance.now() - startTime);
          }
          streamChunkCount.value++;
          const textChunk = decoder.decode(value, { stream: !done });
          responseData.value.body += textChunk;
          responseData.value.latency_ms = Math.round(performance.now() - startTime);
        }
      }
    } catch (err: unknown) {
      const errStr = err instanceof Error ? err.message : String(err);
      if (responseData.value) {
        responseData.value.status = 0;
        responseData.value.statusText = 'Streaming Error';
        responseData.value.body = errStr;
      }
    } finally {
      loading.value = false;
      if (responseData.value) {
        responseData.value.latency_ms = Math.round(performance.now() - startTime);
      }
    }
    return;
  }

  const result = await apiService.sendPlaygroundRequest({
    method: method.value,
    path: path.value,
    headers: parsedHeaders,
    body: method.value !== 'GET' && method.value !== 'HEAD' ? requestBody.value : undefined,
  });

  responseData.value = result;
  loading.value = false;
};
</script>

<template>
  <div class="playground-page">
    <div class="page-header">
      <div>
        <h1>API & AI Route Playground</h1>
        <p class="subtitle">Directly test, benchmark, and debug gateway routes, AI routing policies, and JWT security.</p>
      </div>
    </div>

    <div class="playground-grid">
      <!-- Request Configurator Panel -->
      <div class="panel request-panel">
        <div class="panel-header">
          <h3>Request Builder</h3>
          <div class="route-select-wrapper">
            <span class="route-select-label">Preset Route:</span>
            <select v-model="selectedRoute" class="select-preset" @change="onRouteSelect">
              <option value="">Custom Path</option>
              <option v-for="r in routes" :key="r.external_path" :value="r.external_path">
                {{ r.external_path }} ({{ (r.protocol || 'http').toUpperCase() }})
              </option>
            </select>
          </div>
        </div>

        <div class="panel-body">
          <!-- Method & Path Bar -->
          <div class="url-bar">
            <select v-model="method" class="method-select">
              <option value="GET">GET</option>
              <option value="POST">POST</option>
              <option value="PUT">PUT</option>
              <option value="DELETE">DELETE</option>
              <option value="PATCH">PATCH</option>
            </select>
            <input v-model="path" type="text" placeholder="/api/routes" class="path-input" />
            <button class="btn-send" :disabled="loading" @click="sendRequest">
              {{ loading ? 'Executing...' : '🚀 Send Request' }}
            </button>
          </div>

          <!-- Quick Auth Helper -->
          <div class="auth-helper-row">
            <input
              v-model="rawJwtToken"
              type="text"
              placeholder="Paste JWT token here..."
              class="token-input"
            />
            <button class="btn-auth-add" @click="addBearerAuth">+ Insert Bearer Token</button>
          </div>

          <!-- Streaming & Mode Options -->
          <div class="stream-option-row">
            <label class="stream-checkbox-label">
              <input v-model="isStreaming" type="checkbox" />
              <span class="stream-label-text">⚡ Enable Server-Sent Events (SSE) Streaming</span>
            </label>
            <span v-if="isStreaming" class="stream-hint">Bypasses response cache & yields tokens in real time</span>
          </div>

          <!-- Headers Editor -->
          <div class="editor-section">
            <label>Request Headers (JSON)</label>
            <textarea v-model="headersText" rows="4" class="code-textarea"></textarea>
          </div>

          <!-- Body Editor -->
          <div v-if="method !== 'GET' && method !== 'HEAD'" class="editor-section">
            <label>Request Payload / AI Prompt (JSON/Text)</label>
            <textarea v-model="requestBody" rows="7" class="code-textarea font-mono"></textarea>
          </div>
        </div>
      </div>

      <!-- Response Panel -->
      <div class="panel response-panel">
        <div class="panel-header">
          <h3>Gateway Response</h3>
          <div v-if="responseData" class="response-meta">
            <StatusBadge type="status" :value="responseData.status" />
            <span v-if="streamTtftMs !== null" class="ttft-badge">
              ⏱️ TTFT: {{ streamTtftMs }} ms
            </span>
            <span v-if="streamChunkCount > 0" class="chunks-badge">
              📦 {{ streamChunkCount }} chunks
            </span>
            <span class="latency-badge" :class="{ fast: responseData.latency_ms < 50 }">
              ⚡ {{ responseData.latency_ms }} ms
            </span>
          </div>
        </div>

        <div class="panel-body response-body">
          <div v-if="loading" class="response-placeholder">
            <div class="pulse-loader"></div>
            <span>Forwarding request through Kairos API Gateway...</span>
          </div>

          <div v-else-if="!responseData" class="response-placeholder">
            <span class="empty-icon">🧪</span>
            <span>Configure a request on the left and click "Send Request" to test live upstream forwarding.</span>
          </div>

          <div v-else class="response-display">
            <!-- Headers toggle or details -->
            <details class="headers-accordion">
              <summary>Response Headers ({{ Object.keys(responseData.headers).length }})</summary>
              <div class="headers-table">
                <div v-for="(v, k) in responseData.headers" :key="k" class="header-line">
                  <span class="header-name">{{ k }}:</span>
                  <span class="header-val">{{ v }}</span>
                </div>
              </div>
            </details>

            <!-- Formatted Body -->
            <div class="body-box">
              <div class="body-box-header">
                <span>Response Body</span>
              </div>
              <pre class="body-content"><code>{{ responseData.body }}</code></pre>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.playground-page {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.page-header h1 {
  font-size: 1.85rem;
  font-weight: 700;
  color: #0f172a;
  margin: 0 0 4px 0;
}

.subtitle {
  color: #64748b;
  font-size: 0.95rem;
  margin: 0;
}

.playground-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
}

@media (max-width: 1024px) {
  .playground-grid {
    grid-template-columns: 1fr;
  }
}

.panel {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.02);
  display: flex;
  flex-direction: column;
}

.panel-header {
  padding: 16px 20px;
  border-bottom: 1px solid #f1f5f9;
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: #f8fafc;
  border-radius: 12px 12px 0 0;
}

.panel-header h3 {
  font-size: 1.05rem;
  font-weight: 700;
  color: #1e293b;
  margin: 0;
}

.route-select-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
}

.route-select-label {
  font-size: 0.8rem;
  color: #64748b;
  font-weight: 600;
}

.select-preset {
  font-size: 0.8rem;
  padding: 4px 8px;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  background: #ffffff;
  color: #0f172a;
}

.panel-body {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  flex: 1;
}

/* URL Bar */
.url-bar {
  display: flex;
  gap: 8px;
}

.method-select {
  padding: 10px 14px;
  font-weight: 700;
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  background: #f8fafc;
  color: #0f172a;
}

.path-input {
  flex: 1;
  padding: 10px 14px;
  font-family: monospace;
  font-size: 0.95rem;
  border: 1px solid #cbd5e1;
  border-radius: 8px;
}

.btn-send {
  padding: 10px 18px;
  background: #2563eb;
  color: #ffffff;
  border: none;
  border-radius: 8px;
  font-weight: 700;
  cursor: pointer;
  white-space: nowrap;
}
.btn-send:hover { background: #1d4ed8; }
.btn-send:disabled { opacity: 0.6; cursor: not-allowed; }

/* Auth Helper */
.auth-helper-row {
  display: flex;
  gap: 8px;
}

.token-input {
  flex: 1;
  padding: 6px 12px;
  font-size: 0.85rem;
  border: 1px solid #e2e8f0;
  border-radius: 6px;
}

.btn-auth-add {
  padding: 6px 12px;
  font-size: 0.8rem;
  font-weight: 600;
  background: #eff6ff;
  color: #2563eb;
  border: 1px solid #bfdbfe;
  border-radius: 6px;
  cursor: pointer;
}

.editor-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.editor-section label {
  font-size: 0.8rem;
  font-weight: 600;
  color: #64748b;
  text-transform: uppercase;
}

.code-textarea {
  width: 100%;
  font-family: monospace;
  font-size: 0.85rem;
  padding: 12px;
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  background: #f8fafc;
  color: #0f172a;
  resize: vertical;
}

/* Response Panel */
.response-meta {
  display: flex;
  align-items: center;
  gap: 10px;
}

.latency-badge {
  font-size: 0.8rem;
  font-weight: 700;
  padding: 3px 8px;
  border-radius: 6px;
  background: #f1f5f9;
  color: #475569;
}
.latency-badge.fast {
  background: #ecfdf5;
  color: #059669;
}

.ttft-badge {
  font-size: 0.8rem;
  font-weight: 700;
  padding: 3px 8px;
  border-radius: 6px;
  background: #eff6ff;
  color: #2563eb;
  border: 1px solid #bfdbfe;
}

.chunks-badge {
  font-size: 0.8rem;
  font-weight: 700;
  padding: 3px 8px;
  border-radius: 6px;
  background: #f0fdf4;
  color: #16a34a;
  border: 1px solid #bbf7d0;
}

/* Streaming Controls */
.stream-option-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  padding: 10px 12px;
}

.stream-checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.stream-label-text {
  font-size: 0.85rem;
  font-weight: 700;
  color: #1e293b;
}

.stream-hint {
  font-size: 0.75rem;
  color: #64748b;
  margin-left: 22px;
}

.response-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 60px 20px;
  color: #94a3b8;
  text-align: center;
  font-size: 0.95rem;
}

.empty-icon {
  font-size: 2.5rem;
}

.headers-accordion {
  margin-bottom: 12px;
  border: 1px solid #e2e8f0;
  border-radius: 8px;
  padding: 8px 12px;
  font-size: 0.85rem;
  background: #f8fafc;
}

.headers-accordion summary {
  cursor: pointer;
  font-weight: 600;
  color: #475569;
}

.headers-table {
  margin-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.header-line {
  display: flex;
  gap: 8px;
  font-family: monospace;
  font-size: 0.8rem;
}

.header-name {
  color: #2563eb;
  font-weight: 600;
}

.header-val {
  color: #334155;
  word-break: break-all;
}

.body-box {
  border: 1px solid #1e293b;
  border-radius: 8px;
  overflow: hidden;
}

.body-box-header {
  background: #0f172a;
  color: #94a3b8;
  padding: 8px 14px;
  font-size: 0.8rem;
  font-weight: 600;
  text-transform: uppercase;
}

.body-content {
  margin: 0;
  padding: 16px;
  background: #0f172a;
  color: #38bdf8;
  font-family: monospace;
  font-size: 0.85rem;
  max-height: 380px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
