<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { apiService } from '../services/api';
import type { Settings, AiSettings, JwtSettings, RateLimitConfig, CorsConfig, ServerConfig, MetricsConfig, CacheStats } from '../types';
import hljs from 'highlight.js/lib/core';
import json from 'highlight.js/lib/languages/json';
import 'highlight.js/styles/vs2015.css';

hljs.registerLanguage('json', json);

const config = ref<Settings | null>(null);
const cacheStats = ref<CacheStats | null>(null);
const cacheLoading = ref(false);
const loading = ref(true);
const error = ref<string | null>(null);
const successMessage = ref<string | null>(null);
const copied = ref(false);

const activeModal = ref<'ai' | 'jwt' | 'rate-limit' | 'cors' | 'server' | 'metrics' | null>(null);

// Edit form states
const aiForm = ref<AiSettings>({ provider: 'openai', model: 'gpt-4o', api_key: '' });
const jwtForm = ref<JwtSettings>({ secret: '', issuer: '', audience: '', required_claims: ['sub', 'exp'] });
const rateLimitForm = ref<RateLimitConfig>({ strategy: 'FixedWindow', max_requests: 100 });
const corsForm = ref<CorsConfig>({ allowed_origins: ['*'], allowed_methods: ['GET', 'POST', 'PUT', 'DELETE'], allowed_headers: ['Content-Type', 'Authorization'], allow_credentials: true });
const serverForm = ref<ServerConfig>({ host: '0.0.0.0', port: 5900, workers: 4, keep_alive: 75 });
const metricsForm = ref<MetricsConfig>({ endpoint: '/metrics', enable_per_route_metrics: true });

const claimsInput = ref('');
const corsOriginsInput = ref('');
const corsMethodsInput = ref('');
const corsHeadersInput = ref('');

const loadCacheStats = async () => {
  try {
    const res = await apiService.getCacheStats();
    if (res.success && res.data) {
      cacheStats.value = res.data;
    }
  } catch (e) {
    console.debug('Cache stats unavailable:', e);
  }
};

const handleFlushCache = async () => {
  try {
    cacheLoading.value = true;
    const res = await apiService.clearCache();
    successMessage.value = res.message || 'Cache cleared successfully!';
    await loadCacheStats();
    setTimeout(() => { successMessage.value = null; }, 4000);
  } catch (err: unknown) {
    error.value = err instanceof Error ? err.message : String(err);
  } finally {
    cacheLoading.value = false;
  }
};

const loadConfig = async () => {
  loading.value = true;
  error.value = null;
  try {
    const data = await apiService.getConfig();
    config.value = data;
    await loadCacheStats();
  } catch (err: unknown) {
    error.value = err instanceof Error ? err.message : String(err);
  } finally {
    loading.value = false;
  }
};

onMounted(loadConfig);

const highlightedConfig = computed(() => {
  if (!config.value) return '';
  const jsonString = JSON.stringify(config.value, null, 2);
  return hljs.highlight(jsonString, { language: 'json' }).value;
});

const copyRawConfig = async () => {
  if (!config.value) return;
  try {
    await navigator.clipboard.writeText(JSON.stringify(config.value, null, 2));
    copied.value = true;
    setTimeout(() => { copied.value = false; }, 2000);
  } catch (err: unknown) {
    console.error('Failed to copy: ', err);
  }
};

const handleReload = async () => {
  try {
    const res = await apiService.triggerReload();
    successMessage.value = res.message || 'Configuration reloaded successfully!';
    await loadConfig();
    setTimeout(() => { successMessage.value = null; }, 4000);
  } catch (err: unknown) {
    error.value = err instanceof Error ? err.message : String(err);
  }
};

// Open Modals
const openAiModal = () => {
  if (config.value?.ai) {
    aiForm.value = { ...config.value.ai, api_key: '' };
  }
  activeModal.value = 'ai';
};

const openJwtModal = () => {
  if (config.value?.jwt) {
    jwtForm.value = { ...config.value.jwt };
    claimsInput.value = (config.value.jwt.required_claims || []).join(', ');
  } else {
    jwtForm.value = { secret: '', issuer: 'kairos-gateway', audience: 'kairos-api', required_claims: ['sub', 'exp'] };
    claimsInput.value = 'sub, exp';
  }
  activeModal.value = 'jwt';
};

const openRateLimitModal = () => {
  if (config.value?.rate_limit) {
    rateLimitForm.value = { ...config.value.rate_limit };
  }
  activeModal.value = 'rate-limit';
};

const openCorsModal = () => {
  if (config.value?.cors) {
    corsForm.value = { ...config.value.cors };
    corsOriginsInput.value = (config.value.cors.allowed_origins || []).join(', ');
    corsMethodsInput.value = (config.value.cors.allowed_methods || []).join(', ');
    corsHeadersInput.value = (config.value.cors.allowed_headers || []).join(', ');
  }
  activeModal.value = 'cors';
};

const openServerModal = () => {
  if (config.value?.server) {
    serverForm.value = { ...config.value.server };
  }
  activeModal.value = 'server';
};

const openMetricsModal = () => {
  if (config.value?.metrics) {
    metricsForm.value = { ...config.value.metrics };
  }
  activeModal.value = 'metrics';
};

const closeModal = () => {
  activeModal.value = null;
  error.value = null;
};

// Save handlers
const saveAiConfig = async () => {
  try {
    const res = await apiService.updateAiConfig(aiForm.value);
    successMessage.value = res.message;
    closeModal();
    await loadConfig();
    setTimeout(() => { successMessage.value = null; }, 4000);
  } catch (err: unknown) {
    error.value = err instanceof Error ? err.message : String(err);
  }
};

const saveJwtConfig = async () => {
  try {
    jwtForm.value.required_claims = claimsInput.value.split(',').map(s => s.trim()).filter(Boolean);
    const res = await apiService.updateJwtConfig(jwtForm.value);
    successMessage.value = res.message;
    closeModal();
    await loadConfig();
    setTimeout(() => { successMessage.value = null; }, 4000);
  } catch (err: unknown) {
    error.value = err instanceof Error ? err.message : String(err);
  }
};

const saveRateLimitConfig = async () => {
  try {
    const res = await apiService.updateRateLimitConfig(rateLimitForm.value);
    successMessage.value = res.message;
    closeModal();
    await loadConfig();
    setTimeout(() => { successMessage.value = null; }, 4000);
  } catch (err: unknown) {
    error.value = err instanceof Error ? err.message : String(err);
  }
};

const saveCorsConfig = async () => {
  try {
    corsForm.value.allowed_origins = corsOriginsInput.value.split(',').map(s => s.trim()).filter(Boolean);
    corsForm.value.allowed_methods = corsMethodsInput.value.split(',').map(s => s.trim()).filter(Boolean);
    corsForm.value.allowed_headers = corsHeadersInput.value.split(',').map(s => s.trim()).filter(Boolean);
    const res = await apiService.updateCorsConfig(corsForm.value);
    successMessage.value = res.message;
    closeModal();
    await loadConfig();
    setTimeout(() => { successMessage.value = null; }, 4000);
  } catch (err: unknown) {
    error.value = err instanceof Error ? err.message : String(err);
  }
};

const saveServerConfig = async () => {
  try {
    const res = await apiService.updateServerConfig(serverForm.value);
    successMessage.value = res.message;
    closeModal();
    await loadConfig();
    setTimeout(() => { successMessage.value = null; }, 4000);
  } catch (err: unknown) {
    error.value = err instanceof Error ? err.message : String(err);
  }
};

const saveMetricsConfig = async () => {
  try {
    const res = await apiService.updateMetricsConfig(metricsForm.value);
    successMessage.value = res.message;
    closeModal();
    await loadConfig();
    setTimeout(() => { successMessage.value = null; }, 4000);
  } catch (err: unknown) {
    error.value = err instanceof Error ? err.message : String(err);
  }
};
</script>

<template>
  <div class="config-page">
    <div class="page-header">
      <div>
        <h1>Global Configuration</h1>
        <p class="subtitle">Gateway runtime settings, AI models, security, and rate limiting.</p>
      </div>
      <button class="btn-primary" @click="handleReload">🔄 Hot-Reload Config</button>
    </div>

    <div v-if="successMessage" class="banner success">
      ✓ {{ successMessage }}
    </div>
    <div v-if="error" class="banner error">
      ✕ {{ error }}
    </div>

    <div v-if="loading" class="loading-state">
      Loading configuration...
    </div>

    <div v-else-if="config" class="config-grid">
      <!-- AI Configuration Card -->
      <div class="config-card">
        <div class="card-header">
          <h3>🧠 AI Provider & Models</h3>
          <button class="btn-card-edit" @click="openAiModal">Configure</button>
        </div>
        <div class="card-body">
          <div class="config-row">
            <span class="label">Provider</span>
            <span class="value font-bold text-primary">{{ config.ai?.provider || 'None' }}</span>
          </div>
          <div class="config-row">
            <span class="label">Default Model</span>
            <span class="value font-mono">{{ config.ai?.model || 'None' }}</span>
          </div>
          <div class="config-row">
            <span class="label">API Key</span>
            <span class="value text-muted">•••••••• (Hidden for security)</span>
          </div>
        </div>
      </div>

      <!-- JWT Configuration Card -->
      <div class="config-card">
        <div class="card-header">
          <h3>🔐 JWT Authentication</h3>
          <button class="btn-card-edit" @click="openJwtModal">Configure</button>
        </div>
        <div class="card-body">
          <div class="config-row">
            <span class="label">Issuer</span>
            <span class="value">{{ config.jwt?.issuer || 'N/A' }}</span>
          </div>
          <div class="config-row">
            <span class="label">Audience</span>
            <span class="value">{{ config.jwt?.audience || 'N/A' }}</span>
          </div>
          <div class="config-row">
            <span class="label">Required Claims</span>
            <div class="value tags">
              <span v-for="c in (config.jwt?.required_claims || [])" :key="c" class="tag">{{ c }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Rate Limiting Card -->
      <div class="config-card">
        <div class="card-header">
          <h3>⏳ Rate Limiting</h3>
          <button class="btn-card-edit" @click="openRateLimitModal">Configure</button>
        </div>
        <div class="card-body">
          <div class="config-row">
            <span class="label">Status</span>
            <span class="value tag enabled">Enabled</span>
          </div>
          <div class="config-row">
            <span class="label">Strategy</span>
            <span class="value">{{ config.rate_limit?.strategy || 'FixedWindow' }}</span>
          </div>
          <div class="config-row">
            <span class="label">Max Requests</span>
            <span class="value font-bold">{{ config.rate_limit?.max_requests || 100 }} req</span>
          </div>
        </div>
      </div>

      <!-- Server & Network Card -->
      <div class="config-card">
        <div class="card-header">
          <h3>🖥️ Server & Network</h3>
          <button class="btn-card-edit" @click="openServerModal">Configure</button>
        </div>
        <div class="card-body">
          <div class="config-row">
            <span class="label">Bind Host</span>
            <span class="value font-mono">{{ config.server?.host || '0.0.0.0' }}</span>
          </div>
          <div class="config-row">
            <span class="label">Bind Port</span>
            <span class="value font-mono font-bold">{{ config.server?.port || 5900 }}</span>
          </div>
          <div class="config-row">
            <span class="label">Workers</span>
            <span class="value font-mono">{{ config.server?.workers || 4 }} Threads</span>
          </div>
        </div>
      </div>

      <!-- CORS Policy Card -->
      <div class="config-card">
        <div class="card-header">
          <h3>🌐 CORS Policy</h3>
          <button class="btn-card-edit" @click="openCorsModal">Configure</button>
        </div>
        <div class="card-body">
          <div class="config-row">
            <span class="label">Allowed Origins</span>
            <span class="value font-mono text-small">{{ (config.cors?.allowed_origins || ['*']).join(', ') }}</span>
          </div>
          <div class="config-row">
            <span class="label">Credentials</span>
            <span class="value">{{ config.cors?.allow_credentials ? 'Allowed' : 'Disallowed' }}</span>
          </div>
        </div>
      </div>

      <!-- Metrics Settings Card -->
      <div class="config-card">
        <div class="card-header">
          <h3>📊 Observability</h3>
          <button class="btn-card-edit" @click="openMetricsModal">Configure</button>
        </div>
        <div class="card-body">
          <div class="config-row">
            <span class="label">Endpoint</span>
            <span class="value font-mono">{{ config.metrics?.endpoint || '/metrics' }}</span>
          </div>
          <div class="config-row">
            <span class="label">Per-Route Metrics</span>
            <span class="value">{{ config.metrics?.enable_per_route_metrics ? 'Enabled' : 'Disabled' }}</span>
          </div>
        </div>
      </div>

      <!-- In-Memory Response Cache Card -->
      <div class="config-card">
        <div class="card-header">
          <h3>⚡ In-Memory Response Cache</h3>
          <button class="btn-card-edit" :disabled="cacheLoading" @click="handleFlushCache">
            {{ cacheLoading ? 'Flushing...' : 'Flush Cache' }}
          </button>
        </div>
        <div class="card-body">
          <div class="config-row">
            <span class="label">Hit Ratio</span>
            <span class="value font-bold text-primary">{{ cacheStats ? `${cacheStats.hit_ratio.toFixed(1)}%` : 'Active' }}</span>
          </div>
          <div class="config-row">
            <span class="label">Hits / Misses</span>
            <span class="value font-mono">{{ cacheStats ? `${cacheStats.hits} / ${cacheStats.misses}` : '0 / 0' }}</span>
          </div>
          <div class="config-row">
            <span class="label">Stored Entries</span>
            <span class="value font-mono">{{ cacheStats ? `${cacheStats.current_entries} / ${cacheStats.max_entries}` : '0 / 5,000' }}</span>
          </div>
          <div class="config-row">
            <span class="label">Evictions</span>
            <span class="value font-mono">{{ cacheStats?.evictions ?? 0 }}</span>
          </div>
        </div>
      </div>

      <!-- Raw Configuration JSON Object -->
      <div class="config-card full-width">
        <div class="card-header raw-header">
          <h3>⚙️ Raw Configuration Object (config.json)</h3>
          <button class="btn-copy" @click="copyRawConfig" :class="{ copied }">
            {{ copied ? '✓ Copied to Clipboard' : '📋 Copy JSON' }}
          </button>
        </div>
        <div class="card-body raw-body">
          <pre class="json-display"><code class="hljs" v-html="highlightedConfig"></code></pre>
        </div>
      </div>
    </div>

    <!-- Modals for Editing -->
    <!-- AI Modal -->
    <div v-if="activeModal === 'ai'" class="modal-overlay" @click.self="closeModal">
      <div class="modal-box">
        <div class="modal-header">
          <h2>Configure AI Provider</h2>
          <button class="close-btn" @click="closeModal">✕</button>
        </div>
        <div class="modal-content">
          <div class="form-group">
            <label>AI Provider</label>
            <select v-model="aiForm.provider" class="form-control">
              <option value="openai">OpenAI</option>
              <option value="anthropic">Anthropic</option>
              <option value="cohere">Cohere</option>
              <option value="groq">Groq</option>
              <option value="mistral">Mistral</option>
              <option value="perplexity">Perplexity</option>
            </select>
          </div>
          <div class="form-group">
            <label>Default Model</label>
            <input v-model="aiForm.model" type="text" placeholder="gpt-4o" class="form-control" />
          </div>
          <div class="form-group">
            <label>API Key (Leave empty to keep existing)</label>
            <input v-model="aiForm.api_key" type="password" placeholder="sk-..." class="form-control" />
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn-secondary" @click="closeModal">Cancel</button>
          <button class="btn-primary" @click="saveAiConfig">Save AI Settings</button>
        </div>
      </div>
    </div>

    <!-- JWT Modal -->
    <div v-if="activeModal === 'jwt'" class="modal-overlay" @click.self="closeModal">
      <div class="modal-box">
        <div class="modal-header">
          <h2>Configure JWT Authentication</h2>
          <button class="close-btn" @click="closeModal">✕</button>
        </div>
        <div class="modal-content">
          <div class="form-group">
            <label>Secret Key (Must be at least 32 characters)</label>
            <input v-model="jwtForm.secret" type="password" placeholder="your-secure-32-char-jwt-secret-key" class="form-control" />
            <span class="help-text">Length: {{ jwtForm.secret.length }}/32 characters min</span>
          </div>
          <div class="form-group">
            <label>Expected Issuer (iss)</label>
            <input v-model="jwtForm.issuer" type="text" placeholder="kairos-gateway" class="form-control" />
          </div>
          <div class="form-group">
            <label>Expected Audience (aud)</label>
            <input v-model="jwtForm.audience" type="text" placeholder="kairos-api" class="form-control" />
          </div>
          <div class="form-group">
            <label>Required Claims (comma-separated)</label>
            <input v-model="claimsInput" type="text" placeholder="sub, exp, role" class="form-control" />
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn-secondary" @click="closeModal">Cancel</button>
          <button class="btn-primary" @click="saveJwtConfig">Save JWT Settings</button>
        </div>
      </div>
    </div>

    <!-- Rate Limit Modal -->
    <div v-if="activeModal === 'rate-limit'" class="modal-overlay" @click.self="closeModal">
      <div class="modal-box">
        <div class="modal-header">
          <h2>Configure Rate Limiting</h2>
          <button class="close-btn" @click="closeModal">✕</button>
        </div>
        <div class="modal-content">
          <div class="form-group">
            <label>Strategy</label>
            <select v-model="rateLimitForm.strategy" class="form-control">
              <option value="FixedWindow">Fixed Window</option>
              <option value="SlidingWindow">Sliding Window</option>
              <option value="TokenBucket">Token Bucket</option>
            </select>
          </div>
          <div class="form-group">
            <label>Max Requests per Window</label>
            <input v-model.number="rateLimitForm.max_requests" type="number" min="1" class="form-control" />
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn-secondary" @click="closeModal">Cancel</button>
          <button class="btn-primary" @click="saveRateLimitConfig">Save Rate Limit</button>
        </div>
      </div>
    </div>

    <!-- CORS Modal -->
    <div v-if="activeModal === 'cors'" class="modal-overlay" @click.self="closeModal">
      <div class="modal-box">
        <div class="modal-header">
          <h2>Configure CORS Policy</h2>
          <button class="close-btn" @click="closeModal">✕</button>
        </div>
        <div class="modal-content">
          <div class="form-group">
            <label>Allowed Origins (comma-separated)</label>
            <input v-model="corsOriginsInput" type="text" placeholder="*, http://localhost:3000" class="form-control" />
          </div>
          <div class="form-group">
            <label>Allowed Methods (comma-separated)</label>
            <input v-model="corsMethodsInput" type="text" placeholder="GET, POST, PUT, DELETE" class="form-control" />
          </div>
          <div class="form-group">
            <label>Allowed Headers (comma-separated)</label>
            <input v-model="corsHeadersInput" type="text" placeholder="Content-Type, Authorization" class="form-control" />
          </div>
          <div class="form-group">
            <label class="checkbox-label">
              <input v-model="corsForm.allow_credentials" type="checkbox" />
              <span>Allow Credentials (cookies, auth headers)</span>
            </label>
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn-secondary" @click="closeModal">Cancel</button>
          <button class="btn-primary" @click="saveCorsConfig">Save CORS</button>
        </div>
      </div>
    </div>

    <!-- Server Modal -->
    <div v-if="activeModal === 'server'" class="modal-overlay" @click.self="closeModal">
      <div class="modal-box">
        <div class="modal-header">
          <h2>Configure Server Binding</h2>
          <button class="close-btn" @click="closeModal">✕</button>
        </div>
        <div class="modal-content">
          <div class="form-group">
            <label>Host Address</label>
            <input v-model="serverForm.host" type="text" placeholder="0.0.0.0" class="form-control" />
          </div>
          <div class="form-group">
            <label>Port</label>
            <input v-model.number="serverForm.port" type="number" min="1" max="65535" class="form-control" />
          </div>
          <div class="form-group">
            <label>Worker Threads</label>
            <input v-model.number="serverForm.workers" type="number" min="1" max="64" class="form-control" />
          </div>
          <div class="form-group">
            <label>Keep-Alive Timeout (seconds)</label>
            <input v-model.number="serverForm.keep_alive" type="number" min="1" class="form-control" />
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn-secondary" @click="closeModal">Cancel</button>
          <button class="btn-primary" @click="saveServerConfig">Save Server Settings</button>
        </div>
      </div>
    </div>

    <!-- Metrics Modal -->
    <div v-if="activeModal === 'metrics'" class="modal-overlay" @click.self="closeModal">
      <div class="modal-box">
        <div class="modal-header">
          <h2>Configure Observability</h2>
          <button class="close-btn" @click="closeModal">✕</button>
        </div>
        <div class="modal-content">
          <div class="form-group">
            <label>Prometheus Metrics Path</label>
            <input v-model="metricsForm.endpoint" type="text" placeholder="/metrics" class="form-control" />
          </div>
          <div class="form-group">
            <label class="checkbox-label">
              <input v-model="metricsForm.enable_per_route_metrics" type="checkbox" />
              <span>Enable Per-Route Granular Metrics</span>
            </label>
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn-secondary" @click="closeModal">Cancel</button>
          <button class="btn-primary" @click="saveMetricsConfig">Save Metrics Settings</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.config-page {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 16px;
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

.banner {
  padding: 12px 16px;
  border-radius: 8px;
  font-size: 0.9rem;
  font-weight: 600;
}
.banner.success {
  background: #ecfdf5;
  color: #059669;
  border: 1px solid #a7f3d0;
}
.banner.error {
  background: #fef2f2;
  color: #dc2626;
  border: 1px solid #fecaca;
}

.btn-primary {
  background: #2563eb;
  color: #ffffff;
  border: none;
  padding: 10px 18px;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}
.btn-primary:hover { background: #1d4ed8; }

.config-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
  gap: 20px;
}

.config-card {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.02);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.full-width {
  grid-column: 1 / -1;
}

.card-header {
  padding: 16px 20px;
  border-bottom: 1px solid #f1f5f9;
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: #f8fafc;
}

.card-header h3 {
  font-size: 1rem;
  font-weight: 700;
  color: #1e293b;
  margin: 0;
}

.btn-card-edit {
  font-size: 0.8rem;
  font-weight: 600;
  padding: 4px 10px;
  background: #eff6ff;
  color: #2563eb;
  border: 1px solid #bfdbfe;
  border-radius: 6px;
  cursor: pointer;
}
.btn-card-edit:hover { background: #dbeafe; }

.card-body {
  padding: 18px 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.config-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.9rem;
}

.label {
  color: #64748b;
  font-weight: 500;
}

.value {
  color: #0f172a;
}

.tags {
  display: flex;
  gap: 4px;
}

.tag {
  background: #f1f5f9;
  color: #475569;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 600;
}

.tag.enabled {
  background: #ecfdf5;
  color: #059669;
}

.raw-header {
  background: #0f172a;
}
.raw-header h3 {
  color: #f8fafc;
}

.btn-copy {
  padding: 6px 12px;
  font-size: 0.8rem;
  font-weight: 600;
  border-radius: 6px;
  border: 1px solid #334155;
  background: #1e293b;
  color: #e2e8f0;
  cursor: pointer;
}
.btn-copy:hover { background: #334155; }
.btn-copy.copied {
  background: #059669;
  border-color: #059669;
  color: #ffffff;
}

.raw-body {
  padding: 0;
  background: #0f172a;
}

.json-display {
  margin: 0;
  padding: 20px;
  font-family: monospace;
  font-size: 0.85rem;
  max-height: 400px;
  overflow-y: auto;
}

/* Modals */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.6);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.modal-box {
  background: #ffffff;
  border-radius: 12px;
  width: 100%;
  max-width: 520px;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.2);
  display: flex;
  flex-direction: column;
}

.modal-header {
  padding: 18px 24px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid #e2e8f0;
}

.modal-header h2 {
  font-size: 1.25rem;
  font-weight: 700;
  color: #0f172a;
  margin: 0;
}

.close-btn {
  background: transparent;
  border: none;
  font-size: 1.2rem;
  color: #94a3b8;
  cursor: pointer;
}

.modal-content {
  padding: 20px 24px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group label {
  font-size: 0.85rem;
  font-weight: 600;
  color: #334155;
}

.form-control {
  padding: 9px 12px;
  font-size: 0.9rem;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  background: #ffffff;
  color: #0f172a;
}
.form-control:focus {
  outline: none;
  border-color: #2563eb;
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.1);
}

.help-text {
  font-size: 0.75rem;
  color: #64748b;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.9rem;
  color: #1e293b;
  cursor: pointer;
}

.modal-footer {
  padding: 16px 24px;
  border-top: 1px solid #e2e8f0;
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.btn-secondary {
  padding: 8px 16px;
  background: #f1f5f9;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  font-weight: 600;
  color: #475569;
  cursor: pointer;
}

.font-bold { font-weight: 700; }
.font-mono { font-family: monospace; }
.text-primary { color: #2563eb; }
.text-muted { color: #94a3b8; }
.text-small { font-size: 0.8rem; }
</style>
