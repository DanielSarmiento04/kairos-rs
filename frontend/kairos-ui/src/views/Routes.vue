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
  <div class="routes">
    <h1>Routes Configuration</h1>
    
    <div v-if="loading">Loading...</div>
    <div v-else class="routes-list">
      <div v-for="route in routes" :key="route.external_path" class="route-card">
        <h3>{{ route.external_path }} &rarr; {{ route.internal_path }}</h3>
        <div class="route-details">
          <span><strong>Methods:</strong> {{ route.methods.join(', ') }}</span>
          <span><strong>Auth:</strong> {{ route.auth_required ? 'Yes' : 'No' }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.routes h1 { margin-bottom: 20px; }
.routes-list { display: flex; flex-direction: column; gap: 15px; }
.route-card {
  background: white;
  padding: 15px;
  border-radius: 8px;
  border: 1px solid #e2e8f0;
}
.route-details {
  margin-top: 10px;
  display: flex;
  gap: 20px;
  font-size: 0.9em;
  color: #64748b;
}
</style>
