<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { apiService } from '../services/api'
import type { Router } from '../types'

const health = ref<{ status: string } | null>(null)
const routes = ref<Router[]>([])
const error = ref<string | null>(null)
const loading = ref(true)

onMounted(async () => {
  try {
    const [h, r] = await Promise.all([
      apiService.getHealth(),
      apiService.getRoutes()
    ])
    health.value = h
    routes.value = r
  } catch (err: any) {
    error.value = err.message
  } finally {
    loading.value = false
  }
})

const activeProtocols = computed(() => {
  if (!Array.isArray(routes.value)) return 'None'
  const protocols = new Set(routes.value.map(r => r.protocol || 'http'))
  return Array.from(protocols).join(', ')
})
</script>

<template>
  <div class="dashboard">
    <div class="page-header">
      <div>
        <h1>System Dashboard</h1>
        <p class="subtitle">Overview and health of Kairos LLM Gateway.</p>
      </div>
        <div v-if="health" class="health-badge" :class="{ ok: health.status === 'healthy' || health.status === 'ok' }">
          <span class="indicator"></span>
          {{ health.status === 'healthy' || health.status === 'ok' ? 'System Operational' : 'Degraded' }}
        </div>
    </div>

    <div v-if="error" class="error-banner">
      <strong>Error:</strong> {{ error }}
    </div>

    <div v-if="!loading && !error" class="stats-grid">
      <!-- Total Routes -->
      <div class="stat-card">
        <div class="stat-icon r-icon">🛣️</div>
        <div class="stat-details">
          <span class="stat-label">Active Routes</span>
          <span class="stat-value">{{ routes.length }}</span>
        </div>
      </div>

      <!-- Protocols -->
      <div class="stat-card">
        <div class="stat-icon p-icon">🌐</div>
        <div class="stat-details">
          <span class="stat-label">Protocols</span>
          <span class="stat-value text-capitalize">{{ activeProtocols || 'None' }}</span>
        </div>
      </div>

       <!-- Est. Latency (Placeholder until WS is implemented) -->
       <div class="stat-card">
        <div class="stat-icon l-icon">⏱️</div>
        <div class="stat-details">
          <span class="stat-label">Avg Gateway Latency</span>
          <span class="stat-value">~12ms</span>
        </div>
      </div>

       <!-- Total Connections (Placeholder) -->
       <div class="stat-card">
        <div class="stat-icon c-icon">🔗</div>
        <div class="stat-details">
          <span class="stat-label">Active Connections</span>
          <span class="stat-value">--</span>
        </div>
      </div>
    </div>

    <div v-if="!loading && !error" class="activity-feed">
      <h3>Recent Gateway Events (Live)</h3>
      <div class="feed-box">
        <p class="placeholder-text">WebSocket telemetry integration pending...</p>
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
.subtitle {
  color: #64748b;
  font-size: 0.95rem;
}

/* Health Badge */
.health-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  background: #f1f5f9;
  border: 1px solid #cbd5e1;
  border-radius: 20px;
  font-weight: 600;
  font-size: 0.9rem;
  color: #475569;
}
.health-badge.ok {
  background: #ecfdf5;
  border-color: #a7f3d0;
  color: #059669;
}
.health-badge .indicator {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #cbd5e1;
}
.health-badge.ok .indicator {
  background: #10b981;
  box-shadow: 0 0 8px rgba(16, 185, 129, 0.6);
}

/* Error State */
.error-banner {
  background: #fef2f2;
  border-left: 4px solid #ef4444;
  padding: 16px;
  color: #b91c1c;
  border-radius: 4px;
  margin-bottom: 24px;
}

/* Stats Grid */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 24px;
  margin-bottom: 32px;
}

.stat-card {
  background: #ffffff;
  padding: 24px;
  border-radius: 12px;
  border: 1px solid #e2e8f0;
  box-shadow: 0 1px 3px rgba(0,0,0,0.04);
  display: flex;
  align-items: center;
  gap: 16px;
  transition: transform 0.2s;
}
.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 6px rgba(0,0,0,0.06);
}

.stat-icon {
  font-size: 24px;
  width: 56px;
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 12px;
}
.r-icon { background: #eff6ff; }
.p-icon { background: #fdf4ff; }
.l-icon { background: #f0fdf4; }
.c-icon { background: #fffbeb; }

.stat-details {
  display: flex;
  flex-direction: column;
}
.stat-label {
  font-size: 0.85rem;
  color: #64748b;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 4px;
}
.stat-value {
  font-size: 1.8rem;
  font-weight: 800;
  color: #0f172a;
}
.text-capitalize { text-transform: capitalize; }

/* Activity Feed */
.activity-feed h3 {
  font-size: 1.1rem;
  color: #334155;
  margin-bottom: 16px;
}
.feed-box {
  background: #1e293b;
  border-radius: 8px;
  padding: 32px;
  min-height: 200px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.placeholder-text {
  color: #94a3b8;
  font-family: monospace;
}
</style>
