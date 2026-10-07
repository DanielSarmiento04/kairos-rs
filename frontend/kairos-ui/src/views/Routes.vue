<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { apiService } from '../services/api';
import type { Router, RouteBackend, Protocol, LoadBalancingStrategy, RetryConfig, AiPolicy } from '../types';
import StatusBadge from '../components/StatusBadge.vue';
import { TransformationEditor } from '../components/transformation';

const routes = ref<Router[]>([]);
const loading = ref(true);
const error = ref<string | null>(null);

const showModal = ref(false);
const isEditing = ref(false);
const activeTab = ref<'general' | 'backends' | 'retry' | 'ai' | 'transform'>('general');

const validationError = ref<string | null>(null);
const validationWarnings = ref<string[]>([]);
const validating = ref(false);

const methodOptions = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS'];
const protocolOptions: Protocol[] = ['http', 'websocket', 'ftp', 'dns'];
const strategyOptions: { label: string; value: LoadBalancingStrategy }[] = [
  { label: 'Round Robin', value: 'round_robin' },
  { label: 'Least Connections', value: 'least_connections' },
  { label: 'Random', value: 'random' },
  { label: 'Weighted', value: 'weighted' },
  { label: 'IP Hash (Sticky)', value: 'ip_hash' },
];

const currentRoute = ref<Router>({
  external_path: '',
  internal_path: '',
  protocol: 'http',
  methods: ['GET'],
  auth_required: false,
  backends: [{ host: 'http://localhost', port: 8080, weight: 1 }],
  load_balancing_strategy: 'round_robin',
});

const loadRoutes = async () => {
  loading.value = true;
  error.value = null;
  try {
    routes.value = await apiService.getRoutes();
  } catch (err: unknown) {
    error.value = err instanceof Error ? err.message : String(err);
  } finally {
    loading.value = false;
  }
};

onMounted(loadRoutes);

const openCreateModal = () => {
  isEditing.value = false;
  activeTab.value = 'general';
  validationError.value = null;
  validationWarnings.value = [];
  currentRoute.value = {
    external_path: '/api/v1/resource',
    internal_path: '/resource',
    protocol: 'http',
    methods: ['GET'],
    auth_required: false,
    backends: [{ host: 'http://localhost', port: 8080, weight: 1, health_check_path: '/health' }],
    load_balancing_strategy: 'round_robin',
    retry: {
      max_retries: 3,
      initial_backoff_ms: 100,
      max_backoff_ms: 5000,
      backoff_multiplier: 2.0,
      retry_on_status_codes: [502, 503, 504],
      retry_on_connection_error: true,
    },
    ai_policy: {
      enabled: false,
      strategy: 'content_analysis',
      provider: 'openai',
      fallback_backend_index: 0,
    },
  };
  showModal.value = true;
};

const openEditModal = (route: Router) => {
  isEditing.value = true;
  activeTab.value = 'general';
  validationError.value = null;
  validationWarnings.value = [];
  currentRoute.value = JSON.parse(JSON.stringify(route));

  // Ensure backends array exists for editing
  if (!currentRoute.value.backends || currentRoute.value.backends.length === 0) {
    currentRoute.value.backends = [
      {
        host: currentRoute.value.host || 'http://localhost',
        port: currentRoute.value.port || 8080,
        weight: 1,
      },
    ];
  }
  showModal.value = true;
};

const closeModal = () => {
  showModal.value = false;
};

const toggleMethod = (m: string) => {
  const idx = currentRoute.value.methods.indexOf(m);
  if (idx === -1) {
    currentRoute.value.methods.push(m);
  } else {
    currentRoute.value.methods.splice(idx, 1);
  }
};

const addBackend = () => {
  if (!currentRoute.value.backends) {
    currentRoute.value.backends = [];
  }
  currentRoute.value.backends.push({
    host: 'http://localhost',
    port: 8080,
    weight: 1,
  });
};

const removeBackend = (index: number) => {
  if (currentRoute.value.backends && currentRoute.value.backends.length > 1) {
    currentRoute.value.backends.splice(index, 1);
  }
};

const validateCurrentRoute = async (): Promise<boolean> => {
  validating.value = true;
  validationError.value = null;
  validationWarnings.value = [];

  try {
    const res = await apiService.validateRoute(currentRoute.value);
    if (!res.valid) {
      validationError.value = res.error || 'Validation failed';
      return false;
    }
    if (res.warnings && res.warnings.length > 0) {
      validationWarnings.value = res.warnings;
    }
    return true;
  } catch (err: unknown) {
    validationError.value = err instanceof Error ? err.message : String(err);
    return false;
  } finally {
    validating.value = false;
  }
};

const saveRoute = async () => {
  if (!currentRoute.value.external_path.startsWith('/')) {
    currentRoute.value.external_path = '/' + currentRoute.value.external_path;
  }
  if (!currentRoute.value.internal_path.startsWith('/')) {
    currentRoute.value.internal_path = '/' + currentRoute.value.internal_path;
  }

  // Pre-validate
  const isValid = await validateCurrentRoute();
  if (!isValid) return;

  try {
    if (isEditing.value) {
      await apiService.updateRoute(currentRoute.value.external_path, currentRoute.value);
    } else {
      await apiService.createRoute(currentRoute.value);
    }
    showModal.value = false;
    await loadRoutes();
  } catch (err: unknown) {
    validationError.value = err instanceof Error ? err.message : String(err);
  }
};

const confirmDelete = async (route: Router) => {
  if (confirm(`Are you sure you want to delete route ${route.external_path}?`)) {
    try {
      await apiService.deleteRoute(route.external_path);
      await loadRoutes();
    } catch (err: unknown) {
      alert(`Failed to delete route: ${err instanceof Error ? err.message : String(err)}`);
    }
  }
};
</script>

<template>
  <div class="routes-page">
    <div class="page-header">
      <div>
        <h1>Routing Configuration</h1>
        <p class="subtitle">Manage multi-backend load balancing, AI policies, and upstream proxies.</p>
      </div>
      <button class="btn-primary" @click="openCreateModal">+ Create New Route</button>
    </div>

    <div v-if="error" class="error-banner">
      <strong>Error:</strong> {{ error }}
    </div>

    <div v-if="loading" class="loading-state">
      <div class="spinner"></div> Loading routing configurations...
    </div>

    <div v-else class="table-card">
      <div class="table-container">
        <table class="data-table">
          <thead>
            <tr>
              <th>External Path</th>
              <th>Protocol</th>
              <th>Upstream Backends</th>
              <th>Load Balancing</th>
              <th>Methods</th>
              <th>Security</th>
              <th>AI Policy</th>
              <th class="text-right">Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-if="routes.length === 0">
              <td colspan="8" class="empty-state">No routes currently configured. Click "+ Create New Route" above.</td>
            </tr>
            <tr v-for="route in routes" :key="route.external_path">
              <td>
                <code class="path-badge">{{ route.external_path }}</code>
              </td>
              <td>
                <StatusBadge type="protocol" :value="route.protocol || 'http'" />
              </td>
              <td>
                <div v-if="route.backends && route.backends.length > 0 && route.backends[0]" class="backends-summary">
                  <span class="backend-count">{{ route.backends.length }} backend{{ route.backends.length > 1 ? 's' : '' }}</span>
                  <span class="backend-first">{{ route.backends[0].host }}:{{ route.backends[0].port }} ➡ {{ route.internal_path }}</span>
                </div>
                <div v-else class="backends-summary">
                  <span>{{ route.host ? `${route.host}:${route.port}` : 'None' }} ➡ {{ route.internal_path }}</span>
                </div>
              </td>
              <td>
                <span v-if="route.backends && route.backends.length > 1" class="lb-badge">
                  ⚖️ {{ route.load_balancing_strategy || 'round_robin' }}
                </span>
                <span v-else class="text-muted">Direct</span>
              </td>
              <td>
                <div class="method-tags">
                  <StatusBadge v-for="m in route.methods" :key="m" type="method" :value="m" />
                </div>
              </td>
              <td>
                <StatusBadge type="auth" :value="route.auth_required" />
              </td>
              <td>
                <span v-if="route.ai_policy && route.ai_policy.enabled" class="ai-pill">
                  🤖 {{ typeof route.ai_policy.strategy === 'string' ? route.ai_policy.strategy : 'AI' }}
                </span>
                <span v-else class="text-muted">Disabled</span>
              </td>
              <td class="text-right">
                <button class="btn-action btn-edit" @click="openEditModal(route)">Edit</button>
                <button class="btn-action btn-delete" @click="confirmDelete(route)">Delete</button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Create / Edit Modal -->
    <div v-if="showModal" class="modal-overlay" @click.self="closeModal">
      <div class="modal-container">
        <div class="modal-header">
          <h2>{{ isEditing ? 'Edit Route' : 'Create New Route' }}</h2>
          <button class="close-btn" @click="closeModal">✕</button>
        </div>

        <!-- Modal Navigation Tabs -->
        <div class="modal-tabs">
          <button
            class="tab-btn"
            :class="{ active: activeTab === 'general' }"
            @click="activeTab = 'general'"
          >
            General & Protocol
          </button>
          <button
            class="tab-btn"
            :class="{ active: activeTab === 'backends' }"
            @click="activeTab = 'backends'"
          >
            Backends & Load Balancer
          </button>
          <button
            class="tab-btn"
            :class="{ active: activeTab === 'retry' }"
            @click="activeTab = 'retry'"
          >
            Retry Policy
          </button>
          <button
            class="tab-btn"
            :class="{ active: activeTab === 'ai' }"
            @click="activeTab = 'ai'"
          >
            AI Routing Policy
          </button>
          <button
            class="tab-btn"
            :class="{ active: activeTab === 'transform' }"
            @click="activeTab = 'transform'"
          >
            🔄 Transformations
          </button>
        </div>

        <div class="modal-body">
          <!-- Validation Banner -->
          <div v-if="validationError" class="validation-banner error">
            <strong>Validation Error:</strong> {{ validationError }}
          </div>
          <div v-if="validationWarnings.length > 0" class="validation-banner warn">
            <strong>Warning:</strong>
            <ul>
              <li v-for="w in validationWarnings" :key="w">{{ w }}</li>
            </ul>
          </div>

          <!-- Tab 1: General & Protocol -->
          <div v-if="activeTab === 'general'" class="tab-pane">
            <div class="form-group">
              <label>External Path Pattern (Gateway Route)</label>
              <input
                v-model="currentRoute.external_path"
                type="text"
                placeholder="/api/v1/users/{id}"
                class="form-control"
              />
              <span class="help-text">Dynamic parameters supported with {param} syntax.</span>
            </div>

            <div class="form-group">
              <label>Internal Path (Upstream Destination)</label>
              <input
                v-model="currentRoute.internal_path"
                type="text"
                placeholder="/users/{id}"
                class="form-control"
              />
              <span class="help-text">Destination path forwarded to the backend service.</span>
            </div>

            <div class="form-row">
              <div class="form-group half">
                <label>Protocol</label>
                <select v-model="currentRoute.protocol" class="form-control">
                  <option v-for="proto in protocolOptions" :key="proto" :value="proto">
                    {{ proto.toUpperCase() }}
                  </option>
                </select>
              </div>

              <div class="form-group half flex-center-y">
                <label class="checkbox-label">
                  <input v-model="currentRoute.auth_required" type="checkbox" />
                  <span>Require JWT Authentication</span>
                </label>
              </div>
            </div>

            <div class="form-group">
              <label>Allowed HTTP Methods</label>
              <div class="method-selector">
                <button
                  v-for="m in methodOptions"
                  :key="m"
                  type="button"
                  class="btn-method"
                  :class="{ active: currentRoute.methods.includes(m) }"
                  @click="toggleMethod(m)"
                >
                  {{ m }}
                </button>
              </div>
            </div>
          </div>

          <!-- Tab 2: Backends & Load Balancer -->
          <div v-if="activeTab === 'backends'" class="tab-pane">
            <div class="form-group">
              <label>Load Balancing Strategy</label>
              <select v-model="currentRoute.load_balancing_strategy" class="form-control">
                <option v-for="opt in strategyOptions" :key="opt.value" :value="opt.value">
                  {{ opt.label }}
                </option>
              </select>
            </div>

            <div class="backends-list">
              <div class="backends-list-header">
                <label>Backend Upstreams</label>
                <button type="button" class="btn-sub-action" @click="addBackend">+ Add Backend</button>
              </div>

              <div
                v-for="(b, idx) in currentRoute.backends"
                :key="idx"
                class="backend-row"
              >
                <div class="form-group host-input">
                  <label>Host URL</label>
                  <input v-model="b.host" type="text" placeholder="http://10.0.0.1" class="form-control" />
                </div>
                <div class="form-group port-input">
                  <label>Port</label>
                  <input v-model.number="b.port" type="number" min="1" max="65535" class="form-control" />
                </div>
                <div class="form-group weight-input">
                  <label>Weight</label>
                  <input v-model.number="b.weight" type="number" min="1" class="form-control" />
                </div>
                <button
                  v-if="currentRoute.backends && currentRoute.backends.length > 1"
                  type="button"
                  class="btn-row-remove"
                  @click="removeBackend(idx)"
                >
                  ✕
                </button>
              </div>
            </div>
          </div>

          <!-- Tab 3: Retry Policy -->
          <div v-if="activeTab === 'retry'" class="tab-pane">
            <div v-if="!currentRoute.retry" class="empty-state">
              <button
                type="button"
                class="btn-primary"
                @click="
                  currentRoute.retry = {
                    max_retries: 3,
                    initial_backoff_ms: 100,
                    max_backoff_ms: 5000,
                    backoff_multiplier: 2.0,
                    retry_on_status_codes: [502, 503, 504],
                    retry_on_connection_error: true,
                  }
                "
              >
                + Enable Retry Policy
              </button>
            </div>

            <div v-else class="retry-form">
              <div class="form-row">
                <div class="form-group half">
                  <label>Max Retries</label>
                  <input v-model.number="currentRoute.retry.max_retries" type="number" min="1" max="10" class="form-control" />
                </div>
                <div class="form-group half">
                  <label>Multiplier (Exponential)</label>
                  <input v-model.number="currentRoute.retry.backoff_multiplier" type="number" step="0.5" min="1.0" class="form-control" />
                </div>
              </div>

              <div class="form-row">
                <div class="form-group half">
                  <label>Initial Backoff (ms)</label>
                  <input v-model.number="currentRoute.retry.initial_backoff_ms" type="number" min="10" class="form-control" />
                </div>
                <div class="form-group half">
                  <label>Max Backoff (ms)</label>
                  <input v-model.number="currentRoute.retry.max_backoff_ms" type="number" min="100" class="form-control" />
                </div>
              </div>

              <div class="form-group">
                <label class="checkbox-label">
                  <input v-model="currentRoute.retry.retry_on_connection_error" type="checkbox" />
                  <span>Retry on network/connection errors</span>
                </label>
              </div>
            </div>
          </div>

          <!-- Tab 4: AI Routing Policy -->
          <div v-if="activeTab === 'ai'" class="tab-pane">
            <div v-if="!currentRoute.ai_policy" class="empty-state">
              <button
                type="button"
                class="btn-primary"
                @click="
                  currentRoute.ai_policy = {
                    enabled: true,
                    strategy: 'content_analysis',
                    provider: 'openai',
                    fallback_backend_index: 0,
                  }
                "
              >
                + Enable AI Routing Policy
              </button>
            </div>

            <div v-else class="ai-policy-form">
              <div class="form-group">
                <label class="checkbox-label">
                  <input v-model="currentRoute.ai_policy.enabled" type="checkbox" />
                  <span>Enable AI Intelligent Routing for this Route</span>
                </label>
              </div>

              <div class="form-group">
                <label>AI Strategy</label>
                <select v-model="currentRoute.ai_policy.strategy" class="form-control">
                  <option value="content_analysis">Content Analysis (Context & Intent Routing)</option>
                  <option value="latency_prediction">Latency Prediction (Dynamic Fast-Path)</option>
                  <option value="anomaly_detection">Anomaly Detection (Security Inspection)</option>
                </select>
              </div>

              <div class="form-row">
                <div class="form-group half">
                  <label>AI Provider</label>
                  <select v-model="currentRoute.ai_policy.provider" class="form-control">
                    <option value="openai">OpenAI</option>
                    <option value="anthropic">Anthropic</option>
                    <option value="cohere">Cohere</option>
                    <option value="groq">Groq</option>
                    <option value="mistral">Mistral</option>
                    <option value="perplexity">Perplexity</option>
                  </select>
                </div>

                <div class="form-group half">
                  <label>Fallback Backend Index</label>
                  <input
                    v-model.number="currentRoute.ai_policy.fallback_backend_index"
                    type="number"
                    min="0"
                    class="form-control"
                  />
                </div>
              </div>
            </div>
          </div>
          <!-- Tab 5: Transformations -->
          <div v-if="activeTab === 'transform'" class="tab-pane">
            <TransformationEditor
              :request-transformation="currentRoute.request_transformation ?? null"
              :response-transformation="currentRoute.response_transformation ?? null"
              @update:request-transformation="(v) => (currentRoute.request_transformation = v)"
              @update:response-transformation="(v) => (currentRoute.response_transformation = v)"
            />
          </div>
        </div>

        <div class="modal-footer">
          <button type="button" class="btn-secondary" @click="closeModal">Cancel</button>
          <button type="button" class="btn-validate" :disabled="validating" @click="validateCurrentRoute">
            {{ validating ? 'Validating...' : '✓ Validate' }}
          </button>
          <button type="button" class="btn-primary" @click="saveRoute">
            {{ isEditing ? 'Update Route' : 'Create Route' }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.routes-page {
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

.btn-primary {
  background: #2563eb;
  color: #ffffff;
  border: none;
  padding: 10px 20px;
  border-radius: 8px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  box-shadow: 0 2px 4px rgba(37, 99, 235, 0.2);
}

.btn-primary:hover {
  background: #1d4ed8;
}

.table-card {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.02);
  overflow: hidden;
}

.table-container {
  overflow-x: auto;
}

.data-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.9rem;
  text-align: left;
}

.data-table th {
  padding: 12px 16px;
  background: #f8fafc;
  color: #64748b;
  font-weight: 600;
  font-size: 0.8rem;
  text-transform: uppercase;
  letter-spacing: 0.4px;
  border-bottom: 1px solid #e2e8f0;
}

.data-table td {
  padding: 14px 16px;
  border-bottom: 1px solid #f1f5f9;
  vertical-align: middle;
}

.path-badge {
  background: #f1f5f9;
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 0.85rem;
  color: #1e293b;
  font-weight: 600;
}

.backends-summary {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.backend-count {
  font-size: 0.8rem;
  font-weight: 700;
  color: #3b82f6;
}

.backend-first {
  font-size: 0.85rem;
  color: #475569;
}

.lb-badge {
  font-size: 0.75rem;
  padding: 2px 6px;
  background: #eff6ff;
  color: #1e40af;
  border-radius: 4px;
  font-weight: 600;
}

.method-tags {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.ai-pill {
  font-size: 0.75rem;
  padding: 2px 8px;
  background: #f3e8ff;
  color: #7e22ce;
  border-radius: 12px;
  font-weight: 600;
}

.btn-action {
  padding: 6px 12px;
  font-size: 0.8rem;
  font-weight: 600;
  border-radius: 6px;
  border: 1px solid transparent;
  cursor: pointer;
  margin-left: 6px;
  transition: all 0.2s;
}

.btn-edit {
  background: #eff6ff;
  color: #1d4ed8;
  border-color: #bfdbfe;
}
.btn-edit:hover { background: #dbeafe; }

.btn-delete {
  background: #fef2f2;
  color: #dc2626;
  border-color: #fecaca;
}
.btn-delete:hover { background: #fee2e2; }

/* Modal */
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

.modal-container {
  background: #ffffff;
  border-radius: 14px;
  width: 100%;
  max-width: 680px;
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.2);
}

.modal-header {
  padding: 20px 24px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid #e2e8f0;
}

.modal-header h2 {
  font-size: 1.35rem;
  font-weight: 700;
  color: #0f172a;
  margin: 0;
}

.close-btn {
  background: transparent;
  border: none;
  font-size: 1.25rem;
  color: #94a3b8;
  cursor: pointer;
}
.close-btn:hover { color: #0f172a; }

.modal-tabs {
  display: flex;
  background: #f8fafc;
  border-bottom: 1px solid #e2e8f0;
  padding: 0 16px;
}

.tab-btn {
  padding: 12px 16px;
  border: none;
  background: transparent;
  font-size: 0.85rem;
  font-weight: 600;
  color: #64748b;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  transition: all 0.2s;
}

.tab-btn.active {
  color: #2563eb;
  border-bottom-color: #2563eb;
}

.modal-body {
  padding: 24px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 14px;
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

.form-row {
  display: flex;
  gap: 16px;
}

.half { flex: 1; }
.flex-center-y { display: flex; align-items: center; justify-content: flex-start; }

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.9rem;
  color: #1e293b;
  cursor: pointer;
}

.help-text {
  font-size: 0.75rem;
  color: #64748b;
}

.method-selector {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.btn-method {
  padding: 6px 12px;
  font-size: 0.8rem;
  font-weight: 600;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  background: #ffffff;
  color: #64748b;
  cursor: pointer;
}

.btn-method.active {
  background: #2563eb;
  border-color: #2563eb;
  color: #ffffff;
}

/* Backends list */
.backends-list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.btn-sub-action {
  font-size: 0.8rem;
  padding: 4px 10px;
  background: #eff6ff;
  color: #2563eb;
  border: 1px solid #bfdbfe;
  border-radius: 6px;
  cursor: pointer;
  font-weight: 600;
}

.backend-row {
  display: flex;
  gap: 10px;
  align-items: flex-end;
  background: #f8fafc;
  padding: 12px;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
  margin-bottom: 8px;
}

.host-input { flex: 3; margin: 0; }
.port-input { flex: 1; margin: 0; }
.weight-input { flex: 1; margin: 0; }

.btn-row-remove {
  height: 38px;
  padding: 0 12px;
  background: #fee2e2;
  color: #dc2626;
  border: 1px solid #fecaca;
  border-radius: 6px;
  cursor: pointer;
}

/* Validation banner */
.validation-banner {
  padding: 12px 16px;
  border-radius: 6px;
  font-size: 0.85rem;
}
.validation-banner.error {
  background: #fef2f2;
  border-left: 4px solid #ef4444;
  color: #b91c1c;
}
.validation-banner.warn {
  background: #fffbeb;
  border-left: 4px solid #f59e0b;
  color: #b45309;
}

.modal-footer {
  padding: 16px 24px;
  border-top: 1px solid #e2e8f0;
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.btn-secondary {
  padding: 9px 18px;
  background: #f1f5f9;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  font-weight: 600;
  color: #475569;
  cursor: pointer;
}

.btn-validate {
  padding: 9px 18px;
  background: #f0fdf4;
  border: 1px solid #bbf7d0;
  border-radius: 6px;
  font-weight: 600;
  color: #166534;
  cursor: pointer;
}

.text-muted { color: #94a3b8; }
.text-right { text-align: right; }
</style>
