<script setup lang="ts">
import { ref } from 'vue';
import {
  adminKey,
  isAuthenticated,
  isAuthRequired,
  showAuthModal,
  authService,
} from '../services/auth';
import { useGatewayStore } from '../stores/gateway';

const inputKey = ref(adminKey.value);
const loading = ref(false);
const errorMessage = ref('');
const showPlainKey = ref(false);

const gatewayStore = useGatewayStore();

const handleSave = async () => {
  errorMessage.value = '';
  loading.value = true;
  try {
    const result = await authService.verify(inputKey.value);
    if (result.authenticated || !result.auth_required) {
      showAuthModal.value = false;
      // Reconnect websocket with new credentials
      gatewayStore.disconnectWebSocket();
      gatewayStore.connectWebSocket();
    } else {
      errorMessage.value = 'Invalid Admin Key. Please check the key and try again.';
    }
  } catch (err: unknown) {
    errorMessage.value = err instanceof Error ? err.message : 'Verification failed';
  } finally {
    loading.value = false;
  }
};

const handleClear = () => {
  authService.clearAdminKey();
  inputKey.value = '';
  errorMessage.value = '';
  showAuthModal.value = false;
  gatewayStore.disconnectWebSocket();
  gatewayStore.connectWebSocket();
};

const handleClose = () => {
  if (isAuthenticated.value || !isAuthRequired.value) {
    showAuthModal.value = false;
  }
};
</script>

<template>
  <div v-if="showAuthModal" class="modal-overlay" @click.self="handleClose">
    <div class="modal-card">
      <div class="modal-header">
        <div class="header-icon">🛡️</div>
        <div>
          <h3>Admin Authentication</h3>
          <p class="subtitle">
            Secure control plane endpoints with an Admin API Key
          </p>
        </div>
      </div>

      <div class="modal-body">
        <div v-if="errorMessage" class="error-banner">
          {{ errorMessage }}
        </div>

        <div class="form-group">
          <label for="adminKeyInput">Admin API Key</label>
          <div class="input-wrapper">
            <input
              id="adminKeyInput"
              v-model="inputKey"
              :type="showPlainKey ? 'text' : 'password'"
              placeholder="e.g. krs_adm_secret_key"
              autocomplete="off"
              @keydown.enter="handleSave"
            />
            <button
              type="button"
              class="toggle-vis-btn"
              @click="showPlainKey = !showPlainKey"
              :title="showPlainKey ? 'Hide key' : 'Show key'"
            >
              {{ showPlainKey ? '👁️' : '🔒' }}
            </button>
          </div>
          <span class="hint">
            Configured via <code>admin_auth.api_keys</code> in <code>config.json</code> or the <code>KAIROS_ADMIN_KEY</code> environment variable.
          </span>
        </div>
      </div>

      <div class="modal-footer">
        <button
          v-if="adminKey"
          type="button"
          class="btn-secondary"
          @click="handleClear"
          :disabled="loading"
        >
          Clear Stored Key
        </button>
        <button
          v-if="isAuthenticated || !isAuthRequired"
          type="button"
          class="btn-secondary"
          @click="handleClose"
          :disabled="loading"
        >
          Cancel
        </button>
        <button
          type="button"
          class="btn-primary"
          @click="handleSave"
          :disabled="loading || !inputKey.trim()"
        >
          {{ loading ? 'Verifying...' : 'Save & Authenticate' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background-color: rgba(15, 23, 42, 0.75);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 1.5rem;
}

.modal-card {
  background: #1e293b;
  border: 1px solid #334155;
  border-radius: 12px;
  width: 100%;
  max-width: 480px;
  box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.5), 0 8px 10px -6px rgba(0, 0, 0, 0.5);
  overflow: hidden;
}

.modal-header {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid #334155;
  background: #0f172a;
}

.header-icon {
  font-size: 1.75rem;
}

.modal-header h3 {
  margin: 0;
  font-size: 1.15rem;
  font-weight: 600;
  color: #f8fafc;
}

.subtitle {
  margin: 0.25rem 0 0 0;
  font-size: 0.85rem;
  color: #94a3b8;
}

.modal-body {
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.error-banner {
  background: rgba(239, 68, 68, 0.15);
  border: 1px solid #ef4444;
  color: #fca5a5;
  padding: 0.75rem 1rem;
  border-radius: 8px;
  font-size: 0.875rem;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.form-group label {
  font-size: 0.875rem;
  font-weight: 500;
  color: #cbd5e1;
}

.input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.input-wrapper input {
  width: 100%;
  background: #0f172a;
  border: 1px solid #475569;
  border-radius: 8px;
  padding: 0.65rem 2.5rem 0.65rem 0.85rem;
  color: #f8fafc;
  font-size: 0.95rem;
  outline: none;
  transition: border-color 0.2s;
}

.input-wrapper input:focus {
  border-color: #6366f1;
  box-shadow: 0 0 0 1px #6366f1;
}

.toggle-vis-btn {
  position: absolute;
  right: 0.5rem;
  background: transparent;
  border: none;
  cursor: pointer;
  padding: 0.25rem;
  font-size: 1rem;
  opacity: 0.75;
}

.toggle-vis-btn:hover {
  opacity: 1;
}

.hint {
  font-size: 0.8rem;
  color: #64748b;
  line-height: 1.4;
}

.hint code {
  color: #94a3b8;
  background: #0f172a;
  padding: 0.1rem 0.3rem;
  border-radius: 4px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.75rem;
  padding: 1rem 1.5rem;
  border-top: 1px solid #334155;
  background: #0f172a;
}

.btn-primary {
  background: #4f46e5;
  color: #ffffff;
  border: none;
  border-radius: 8px;
  padding: 0.6rem 1.25rem;
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s;
}

.btn-primary:hover:not(:disabled) {
  background: #4338ca;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  background: #334155;
  color: #cbd5e1;
  border: none;
  border-radius: 8px;
  padding: 0.6rem 1rem;
  font-size: 0.875rem;
  cursor: pointer;
  transition: background 0.2s;
}

.btn-secondary:hover:not(:disabled) {
  background: #475569;
}
</style>
