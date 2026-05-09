<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { apiService } from '../services/api'

const health = ref<{ status: string } | null>(null)
const error = ref<string | null>(null)

onMounted(async () => {
  try {
    health.value = await apiService.getHealth()
  } catch (err: any) {
    error.value = err.message
  }
})
</script>

<template>
  <div class="dashboard">
    <h1>Dashboard</h1>
    
    <div class="metrics-grid">
      <div class="card">
        <h3>Gateway Status</h3>
        <p v-if="health" class="status ok">{{ health.status }}</p>
        <p v-else-if="error" class="status error">{{ error }}</p>
        <p v-else>Loading...</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dashboard h1 {
  margin-bottom: 20px;
}
.card {
  background: white;
  padding: 20px;
  border-radius: 8px;
  box-shadow: 0 1px 3px rgba(0,0,0,0.1);
}
.status.ok { color: #10b981; font-weight: bold; font-size: 1.2rem; }
.status.error { color: #ef4444; }
</style>
