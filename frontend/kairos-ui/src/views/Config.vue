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
  <div class="config">
    <h1>Global Configuration</h1>
    
    <div v-if="loading">Loading...</div>
    <div v-else-if="config" class="config-sections">
      <div class="card">
        <h3>Version</h3>
        <p>{{ config.version }}</p>
      </div>

      <div class="card" v-if="config.metrics">
        <h3>Metrics</h3>
        <pre>{{ JSON.stringify(config.metrics, null, 2) }}</pre>
      </div>
      
      <div class="card" v-if="config.ai">
        <h3>AI Provider</h3>
        <pre>{{ JSON.stringify(config.ai, null, 2) }}</pre>
      </div>
    </div>
  </div>
</template>

<style scoped>
.config h1 { margin-bottom: 20px; }
.config-sections { display: flex; flex-direction: column; gap: 20px; }
.card {
  background: white;
  padding: 20px;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
}
pre {
  background: #f8fafc;
  padding: 10px;
  border-radius: 4px;
  overflow-x: auto;
}
</style>
