<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { apiService } from '../services/api'
import type { Settings } from '../types'

const config = ref<Settings | null>(null)
const loading = ref(true)

onMounted(async () => {
  try {
    config.value = await apiService.getConfig()
  } catch (e) {
    console.error(e)
  } finally {
    loading.value = false
  }
})
</script>

<template>
  <div class="config-page">
    <div class="page-header">
      <div>
        <h1>Global Configuration</h1>
        <p class="subtitle">System settings, AI providers, and middleware.</p>
      </div>
      <button class="btn-primary" @click="apiService.triggerReload()">Reload Backend Config</button>
    </div>
    
    <div v-if="loading" class="loading-state">Loading configuration...</div>
    
    <div v-else-if="config" class="config-grid">
      <!-- AI Configuration -->
      <div class="config-card" v-if="config.ai">
        <div class="card-header">
          <h3>🧠 AI Provider</h3>
        </div>
        <div class="card-body">
          <div class="config-row">
            <span class="label">Provider</span>
            <span class="value badge">{{ config.ai.provider }}</span>
          </div>
          <div class="config-row">
            <span class="label">Model</span>
            <span class="value font-mono">{{ config.ai.model }}</span>
          </div>
        </div>
      </div>

      <!-- JWT Configuration -->
      <div class="config-card" v-if="config.jwt">
        <div class="card-header">
          <h3>🔐 JWT Authentication</h3>
        </div>
        <div class="card-body">
          <div class="config-row">
            <span class="label">Issuer</span>
            <span class="value">{{ config.jwt.issuer || 'N/A' }}</span>
          </div>
          <div class="config-row">
            <span class="label">Audience</span>
            <span class="value">{{ config.jwt.audience || 'N/A' }}</span>
          </div>
          <div class="config-row">
            <span class="label">Required Claims</span>
            <div class="value tags">
              <span v-for="c in config.jwt.required_claims" :key="c" class="tag">{{ c }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Rate Limiting -->
      <div class="config-card" v-if="config.rate_limit">
        <div class="card-header">
          <h3>⏳ Rate Limiting</h3>
        </div>
        <div class="card-body">
          <div class="config-row">
            <span class="label">Status</span>
            <span class="value tag" :class="config.rate_limit.enabled ? 'enabled' : 'disabled'">
              {{ config.rate_limit.enabled ? 'Enabled' : 'Disabled' }}
            </span>
          </div>
          <div class="config-row">
            <span class="label">Strategy</span>
            <span class="value">{{ config.rate_limit.strategy }}</span>
          </div>
          <div class="config-row">
            <span class="label">Rate</span>
            <span class="value">{{ config.rate_limit.requests_per_second }} req/s</span>
          </div>
        </div>
      </div>

      <!-- Raw Fallback -->
      <div class="config-card full-width">
         <div class="card-header">
          <h3>⚙️ Raw Configuration Object</h3>
        </div>
        <div class="card-body">
           <pre>{{ JSON.stringify(config, null, 2) }}</pre>
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
  background: #0f172a;
  color: white;
  border: none;
  padding: 10px 20px;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.2s;
}
.btn-primary:hover { background: #334155; }

.config-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
  gap: 24px;
}

.config-card {
  background: white;
  border-radius: 12px;
  border: 1px solid #e2e8f0;
  box-shadow: 0 1px 3px rgba(0,0,0,0.04);
  overflow: hidden;
}
.full-width { grid-column: 1 / -1; }

.card-header {
  background: #f8fafc;
  padding: 16px 24px;
  border-bottom: 1px solid #e2e8f0;
}
.card-header h3 {
  font-size: 1.1rem;
  color: #334155;
  margin: 0;
}

.card-body {
  padding: 24px;
}

.config-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
  border-bottom: 1px solid #f1f5f9;
}
.config-row:last-child {
  border-bottom: none;
  padding-bottom: 0;
}

.label {
  color: #64748b;
  font-weight: 500;
  font-size: 0.9rem;
}
.value {
  color: #0f172a;
  font-weight: 600;
}

.badge {
  background: #f1f5f9;
  padding: 4px 8px;
  border-radius: 4px;
  border: 1px solid #cbd5e1;
}
.font-mono { font-family: monospace; }

.tags {
  display: flex;
  gap: 8px;
}
.tag {
  background: #e0e7ff;
  color: #1e40af;
  padding: 4px 10px;
  border-radius: 20px;
  font-size: 0.8rem;
}
.tag.enabled { background: #dcfce7; color: #166534; }
.tag.disabled { background: #fee2e2; color: #991b1b; }

pre {
  background: #0f172a;
  color: #e2e8f0;
  padding: 20px;
  border-radius: 8px;
  overflow-x: auto;
  font-size: 0.85rem;
  line-height: 1.5;
}
</style>
