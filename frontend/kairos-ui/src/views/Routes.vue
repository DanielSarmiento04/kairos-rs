<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { apiService } from '../services/api'
import type { Router } from '../types'

const routes = ref<Router[]>([])
const loading = ref(true)

onMounted(async () => {
  try {
    routes.value = await apiService.getRoutes()
  } catch (e) {
    console.error(e)
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="routes-page">
    <div class="page-header">
      <div>
        <h1>Routing Configuration</h1>
        <p class="subtitle">Manage gateway boundaries and AI routing paths.</p>
      </div>
      <button class="btn-primary" disabled>+ Create Route</button>
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
            <td><span class="upstream-badge">{{ route.internal_path }}</span></td>
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
              <button class="btn-icon">⚙️</button>
            </td>
          </tr>
        </tbody>
      </table>
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
.data-table tr:last-child td { border-bottom: none; }
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

.auth-yes { background: #f1f5f9; border: 1px solid #cbd5e1; color: #475569; }
.auth-no { font-weight: normal; color: #94a3b8; }

.btn-icon {
  background: none;
  border: none;
  cursor: pointer;
  font-size: 1.2rem;
  opacity: 0.6;
  transition: opacity 0.2s;
}
.btn-icon:hover { opacity: 1; }
</style>
