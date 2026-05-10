<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { apiService } from '../services/api'
import type { Router, RouteBackend } from '../types'

const routes = ref<Router[]>([])
const loading = ref(true)

const showModal = ref(false)
const isEditing = ref(false)
const currentRoute = ref<Router>({
  external_path: '',
  internal_path: '',
  protocol: 'http',
  host: '',
  port: 80,
  methods: ['GET'],
  auth_required: false,
})

const methodOptions = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH']

const loadRoutes = async () => {
  loading.value = true
  try {
    routes.value = await apiService.getRoutes()
  } catch (e) {
    console.error(e)
  } finally {
    loading.value = false
  }
}

onMounted(loadRoutes)

const openCreateModal = () => {
  isEditing.value = false
  currentRoute.value = {
    external_path: '',
    internal_path: '',
    protocol: 'http',
    host: 'localhost',
    port: 80,
    methods: ['GET'],
    auth_required: false,
  }
  showModal.value = true
}

const openEditModal = (route: Router) => {
  isEditing.value = true
  // Create a deep copy to edit
  currentRoute.value = JSON.parse(JSON.stringify(route))
  // Fallback defaults
  if (!currentRoute.value.host && !currentRoute.value.backends) {
     currentRoute.value.host = 'localhost'
     currentRoute.value.port = 80
  }
  showModal.value = true
}

const closeModal = () => {
  showModal.value = false
}

const toggleMethod = (m: string) => {
  const idx = currentRoute.value.methods.indexOf(m)
  if (idx === -1) {
    currentRoute.value.methods.push(m)
  } else {
    currentRoute.value.methods.splice(idx, 1)
  }
}

const saveRoute = async () => {
  try {
    // If external_path doesn't start with / add it
    if (!currentRoute.value.external_path.startsWith('/')) {
      currentRoute.value.external_path = '/' + currentRoute.value.external_path
    }
    
    if (isEditing.value) {
      await apiService.updateRoute(currentRoute.value.external_path, currentRoute.value)
    } else {
      await apiService.createRoute(currentRoute.value)
    }
    showModal.value = false
    await loadRoutes()
  } catch(e) {
    alert("Failed to save route:\n" + e)
  }
}

const confirmDelete = async (route: Router) => {
  if (confirm(`Are you sure you want to delete ${route.external_path}?`)) {
    try {
      await apiService.deleteRoute(route.external_path)
      await loadRoutes()
    } catch(e) {
      alert("Failed to delete route:\n" + e)
    }
  }
}
</script>

<template>
  <div class="routes-page">
    <div class="page-header">
      <div>
        <h1>Routing Configuration</h1>
        <p class="subtitle">Manage gateway boundaries and AI routing paths.</p>
      </div>
      <button class="btn-primary" @click="openCreateModal">+ Create Route</button>
    </div>
    
    <div v-if="loading" class="loading-state">Loading routes...</div>
    
    <div v-else class="table-container">
      <table class="data-table">
        <thead>
          <tr>
            <th>Pattern (External Path)</th>
            <th>Upstream (Internal Path)</th>
            <th>Protocol</th>
            <th>Methods</th>
            <th>Secure</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr v-if="routes.length === 0">
            <td colspan="6" class="text-center empty-state">No routes configured.</td>
          </tr>
          <tr v-for="route in routes" :key="route.external_path">
            <td><code class="path-badge">{{ route.external_path }}</code></td>
            <td><span class="upstream-badge">
              <span v-if="route.backends && route.backends.length > 0">
                Multiple Backends ({{ route.backends.length }})
              </span>
              <span v-else>
                {{ route.host ? route.host + ':' + route.port : 'N/A' }} ➡ {{ route.internal_path }}
              </span>
            </span></td>
            <td>
              <span class="tag protocol-tag">{{ (route.protocol || 'http').toUpperCase() }}</span>
            </td>
            <td>
              <div class="method-tags">
                <span v-for="m in route.methods" :key="m" class="tag method-tag" :class="m.toLowerCase()">{{ m }}</span>
              </div>
            </td>
            <td>
              <span v-if="route.auth_required" class="tag auth-yes">🔐 Auth</span>
              <span v-else class="tag auth-no">Public</span>
            </td>
            <td>
              <button class="btn-icon" title="Edit" @click="openEditModal(route)">✏️</button>
              <button class="btn-icon delete-icon" title="Delete" @click="confirmDelete(route)">🗑️</button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- Modal Form -->
    <div v-if="showModal" class="modal-overlay" @click.self="closeModal">
      <div class="modal-content">
        <div class="modal-header">
          <h2>{{ isEditing ? 'Edit Route' : 'Create New Route' }}</h2>
          <button class="btn-close" @click="closeModal">×</button>
        </div>
        <div class="modal-body">
          <div class="form-group">
            <label>External Path</label>
            <input type="text" v-model="currentRoute.external_path" placeholder="/api/users" :readonly="isEditing" />
            <small v-if="isEditing">External path cannot be changed. Delete and recreate if needed.</small>
          </div>
          <div class="form-group">
            <label>Internal Path</label>
            <input type="text" v-model="currentRoute.internal_path" placeholder="/" />
          </div>
          <div class="form-row">
            <div class="form-group">
              <label>Upstream Host</label>
              <input type="text" v-model="currentRoute.host" placeholder="localhost or http://backend" />
            </div>
            <div class="form-group">
              <label>Port</label>
              <input type="number" v-model="currentRoute.port" />
            </div>
          </div>
          <div class="form-row">
            <div class="form-group">
              <label>Protocol</label>
              <select v-model="currentRoute.protocol">
                <option value="http">HTTP</option>
                <option value="websocket">WebSocket</option>
                <option value="kairos">Kairos (AI)</option>
              </select>
            </div>
            <div class="form-group">
              <label>Authentication</label>
              <label class="checkbox-label">
                <input type="checkbox" v-model="currentRoute.auth_required" />
                Require JWT Auth
              </label>
            </div>
          </div>
          <div class="form-group">
            <label>Allowed Methods</label>
            <div class="method-toggles">
              <button 
                v-for="m in methodOptions" 
                :key="m"
                class="method-toggle"
                :class="{ active: currentRoute.methods.includes(m) }"
                @click="toggleMethod(m)"
              >
                {{ m }}
              </button>
            </div>
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn-secondary" @click="closeModal">Cancel</button>
          <button class="btn-primary" @click="saveRoute">Save Route</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 32px;
}
.page-header h1 {
  font-size: 1.8rem;
  color: #0f172a;
  margin-bottom: 4px;
}
.subtitle { color: #64748b; font-size: 0.95rem; }

.btn-primary {
  background: #2563eb;
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.2s;
}
.btn-primary:hover:not(:disabled) { background: #1d4ed8; }
.btn-primary:disabled { background: #94a3b8; cursor: not-allowed; }

.btn-secondary {
  background: #f1f5f9;
  color: #334155;
  border: 1px solid #cbd5e1;
  padding: 10px 20px;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
}
.btn-secondary:hover { background: #e2e8f0; }

.table-container {
  background: white;
  border-radius: 12px;
  border: 1px solid #e2e8f0;
  box-shadow: 0 1px 3px rgba(0,0,0,0.04);
  overflow: hidden;
}

.data-table {
  width: 100%;
  border-collapse: collapse;
}
.data-table th {
  background: #f8fafc;
  padding: 16px 24px;
  text-align: left;
  font-size: 0.85rem;
  font-weight: 600;
  color: #475569;
  text-transform: uppercase;
  border-bottom: 1px solid #cbd5e1;
}
.data-table td {
  padding: 16px 24px;
  border-bottom: 1px solid #e2e8f0;
  vertical-align: middle;
}
.data-table tr:hover { background: #f8fafc; }

.empty-state {
  padding: 40px !important;
  color: #94a3b8;
  font-style: italic;
}

.text-center { text-align: center; }

/* Badges & Tags */
.path-badge {
  background: #f1f5f9;
  color: #334155;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 0.9em;
  font-family: monospace;
}
.upstream-badge {
  color: #64748b;
  font-size: 0.9em;
}

.tag {
  display: inline-block;
  padding: 4px 8px;
  border-radius: 20px;
  font-size: 0.75rem;
  font-weight: 600;
}
.protocol-tag {
  background: #e0e7ff;
  color: #1e40af;
}

.method-tags {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}
.method-tag.get { background: #dcfce7; color: #166534; }
.method-tag.post { background: #fef9c3; color: #b45309; }
.method-tag.put { background: #ffedd5; color: #854d0e; }
.method-tag.delete { background: #fee2e2; color: #991b1b; }
.method-tag.patch { background: #f3e8ff; color: #7e22ce; }

.auth-yes { background: #f1f5f9; border: 1px solid #cbd5e1; color: #475569; }
.auth-no { font-weight: normal; color: #94a3b8; }

.btn-icon {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 1.2rem;
  opacity: 0.6;
  transition: opacity 0.2s;
  padding: 4px;
}
.btn-icon:hover { opacity: 1; }
.delete-icon:hover { opacity: 1; filter: sepia(1%) hue-rotate(180deg) saturate(300%) opacity(1) brightness(0.5) contrast(200%); color: red; } /* hacky hover color */

/* Modal */
.modal-overlay {
  position: fixed;
  top: 0; left: 0; right: 0; bottom: 0;
  background: rgba(15, 23, 42, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal-content {
  background: white;
  border-radius: 12px;
  width: 100%;
  max-width: 500px;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
  display: flex;
  flex-direction: column;
}

.modal-header {
  padding: 20px 24px;
  border-bottom: 1px solid #e2e8f0;
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.modal-header h2 { margin: 0; font-size: 1.25rem; color: #0f172a; }
.btn-close { background: none; border: none; font-size: 1.5rem; cursor: pointer; color: #64748b; }

.modal-body {
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-row {
  display: flex;
  gap: 16px;
}
.form-row .form-group {
  flex: 1;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.form-group label {
  font-size: 0.9rem;
  font-weight: 600;
  color: #475569;
}
.form-group input[type="text"],
.form-group input[type="number"],
.form-group select {
  padding: 10px;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  font-size: 0.95rem;
}
.form-group input[readonly] {
  background: #f1f5f9;
  color: #94a3b8;
  cursor: not-allowed;
}

.form-group small {
  color: #94a3b8;
  font-size: 0.8rem;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: normal !important;
  cursor: pointer;
  margin-top: 8px;
}

.method-toggles {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.method-toggle {
  background: white;
  border: 1px solid #cbd5e1;
  padding: 6px 12px;
  border-radius: 20px;
  cursor: pointer;
  font-size: 0.85rem;
  font-weight: 600;
  color: #64748b;
  transition: all 0.2s;
}
.method-toggle.active {
  background: #eff6ff;
  border-color: #3b82f6;
  color: #1d4ed8;
}

.modal-footer {
  padding: 16px 24px;
  border-top: 1px solid #e2e8f0;
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  background: #f8fafc;
  border-bottom-left-radius: 12px;
  border-bottom-right-radius: 12px;
}
</style>
