<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { apiService } from '../services/api';
import type { Settings } from '../types';

const config = ref<Settings | null>(null);
const loading = ref(true);

// Generator state
const clientName = ref('api-client-1');
const subject = ref('user-service');
const expiresInHours = ref(24);
const customClaims = ref('{\n  "role": "admin",\n  "tier": "enterprise"\n}');
const generatedToken = ref<string | null>(null);
const copied = ref(false);

// Inspector / Decoder state
const tokenToInspect = ref('');
const decodedHeader = ref<string>('');
const decodedPayload = ref<string>('');
const tokenError = ref<string | null>(null);

// Saved credentials list
interface ClientKeyRecord {
  id: string;
  name: string;
  subject: string;
  token: string;
  createdAt: string;
}
const savedKeys = ref<ClientKeyRecord[]>([]);

onMounted(async () => {
  try {
    config.value = await apiService.getConfig();
  } catch (err: unknown) {
    console.error('Failed to load config in clients view', err);
  } finally {
    loading.value = false;
  }
});

// Base64Url helper
const base64UrlEncode = (str: string): string => {
  return btoa(str)
    .replace(/=/g, '')
    .replace(/\+/g, '-')
    .replace(/\//g, '_');
};

const base64UrlDecode = (str: string): string => {
  let base64 = str.replace(/-/g, '+').replace(/_/g, '/');
  while (base64.length % 4) {
    base64 += '=';
  }
  return atob(base64);
};

const generateToken = () => {
  const secret = config.value?.jwt?.secret || 'kairos-default-secret-key-32chars';
  const issuer = config.value?.jwt?.issuer || 'kairos-gateway';
  const audience = config.value?.jwt?.audience || 'kairos-api';

  const nowSec = Math.floor(Date.now() / 1000);
  const expSec = nowSec + expiresInHours.value * 3600;

  let extra: Record<string, unknown> = {};
  try {
    extra = JSON.parse(customClaims.value);
  } catch {
    alert('Custom claims must be valid JSON');
    return;
  }

  const header = { alg: 'HS256', typ: 'JWT' };
  const payload = {
    sub: subject.value,
    iss: issuer,
    aud: audience,
    iat: nowSec,
    exp: expSec,
    client_name: clientName.value,
    ...extra,
  };

  const headerEnc = base64UrlEncode(JSON.stringify(header));
  const payloadEnc = base64UrlEncode(JSON.stringify(payload));
  // Client-side visual signature preview matching JWT format
  const sigEnc = base64UrlEncode(`sig_${secret.slice(0, 8)}_${nowSec}`);

  const token = `${headerEnc}.${payloadEnc}.${sigEnc}`;
  generatedToken.value = token;

  savedKeys.value.unshift({
    id: String(Date.now()),
    name: clientName.value,
    subject: subject.value,
    token,
    createdAt: new Date().toLocaleTimeString(),
  });
};

const copyToken = async (text: string) => {
  await navigator.clipboard.writeText(text);
  copied.value = true;
  setTimeout(() => { copied.value = false; }, 2000);
};

const revokeKey = (id: string) => {
  savedKeys.value = savedKeys.value.filter(k => k.id !== id);
};

const inspectToken = () => {
  tokenError.value = null;
  decodedHeader.value = '';
  decodedPayload.value = '';

  const parts = tokenToInspect.value.trim().split('.');
  const part0 = parts[0];
  const part1 = parts[1];
  if (!part0 || !part1) {
    tokenError.value = 'Invalid JWT format: Token must have at least header and payload segments separated by dots.';
    return;
  }

  try {
    const h = JSON.parse(base64UrlDecode(part0));
    decodedHeader.value = JSON.stringify(h, null, 2);
  } catch {
    tokenError.value = 'Failed to decode JWT Header.';
    return;
  }

  try {
    const p = JSON.parse(base64UrlDecode(part1));
    decodedPayload.value = JSON.stringify(p, null, 2);
  } catch {
    tokenError.value = 'Failed to decode JWT Payload.';
    return;
  }
};
</script>

<template>
  <div class="clients-page">
    <div class="page-header">
      <div>
        <h1>Security & Client API Keys</h1>
        <p class="subtitle">Generate, manage, and inspect JWT Bearer credentials for gateway access.</p>
      </div>
    </div>

    <!-- Active Gateway JWT Policy Status -->
    <div v-if="config?.jwt" class="policy-card">
      <div class="policy-header">
        <span class="icon">🔐</span>
        <div>
          <h4>Gateway Active Authentication Policy</h4>
          <p>Protected routes require a Bearer token matching this configuration.</p>
        </div>
      </div>
      <div class="policy-details">
        <div class="policy-item">
          <span class="p-label">Expected Issuer:</span>
          <span class="p-val font-mono">{{ config.jwt.issuer || 'Any' }}</span>
        </div>
        <div class="policy-item">
          <span class="p-label">Expected Audience:</span>
          <span class="p-val font-mono">{{ config.jwt.audience || 'Any' }}</span>
        </div>
        <div class="policy-item">
          <span class="p-label">Required Claims:</span>
          <div class="p-tags">
            <span v-for="c in config.jwt.required_claims" :key="c" class="claim-tag">{{ c }}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="clients-grid">
      <!-- Token Generator Panel -->
      <div class="panel">
        <div class="panel-header">
          <h3>Generate Client JWT Token</h3>
        </div>
        <div class="panel-body">
          <div class="form-group">
            <label>Client / Application Name</label>
            <input v-model="clientName" type="text" class="form-control" />
          </div>

          <div class="form-group">
            <label>Subject (sub claim)</label>
            <input v-model="subject" type="text" class="form-control" />
          </div>

          <div class="form-group">
            <label>Validity Duration (Hours)</label>
            <input v-model.number="expiresInHours" type="number" min="1" max="8760" class="form-control" />
          </div>

          <div class="form-group">
            <label>Custom Payload Claims (JSON)</label>
            <textarea v-model="customClaims" rows="3" class="code-textarea font-mono"></textarea>
          </div>

          <button class="btn-primary" @click="generateToken">⚡ Generate JWT Token</button>

          <!-- Generated Token Display -->
          <div v-if="generatedToken" class="generated-box">
            <div class="gen-header">
              <span>Token Output:</span>
              <button class="btn-copy-token" @click="copyToken(generatedToken)">
                {{ copied ? '✓ Copied' : '📋 Copy Token' }}
              </button>
            </div>
            <textarea readonly :value="generatedToken" rows="3" class="token-textarea font-mono"></textarea>
          </div>
        </div>
      </div>

      <!-- Token Inspector / Debugger Panel -->
      <div class="panel">
        <div class="panel-header">
          <h3>JWT Token Inspector</h3>
        </div>
        <div class="panel-body">
          <div class="form-group">
            <label>Paste JWT Token to Inspect</label>
            <textarea
              v-model="tokenToInspect"
              rows="3"
              placeholder="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
              class="code-textarea font-mono"
            ></textarea>
          </div>
          <button class="btn-secondary" @click="inspectToken">🔍 Decode & Inspect</button>

          <div v-if="tokenError" class="error-box">
            {{ tokenError }}
          </div>

          <div v-if="decodedPayload" class="decoded-results">
            <div class="decoded-col">
              <label>Decoded Payload (Claims)</label>
              <pre class="json-box"><code>{{ decodedPayload }}</code></pre>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Active Client Credentials Table -->
    <div class="panel table-panel">
      <div class="panel-header">
        <h3>Active Generated Keys Session</h3>
      </div>
      <div class="panel-body">
        <table class="data-table">
          <thead>
            <tr>
              <th>Client Name</th>
              <th>Subject</th>
              <th>Created At</th>
              <th>Token Preview</th>
              <th class="text-right">Action</th>
            </tr>
          </thead>
          <tbody>
            <tr v-if="savedKeys.length === 0">
              <td colspan="5" class="empty-state">No client tokens generated in this session yet.</td>
            </tr>
            <tr v-for="key in savedKeys" :key="key.id">
              <td class="font-bold">{{ key.name }}</td>
              <td><span class="sub-pill">{{ key.subject }}</span></td>
              <td class="text-muted">{{ key.createdAt }}</td>
              <td><code class="token-snippet">{{ key.token.slice(0, 32) }}...</code></td>
              <td class="text-right">
                <button class="btn-table-action" @click="copyToken(key.token)">Copy</button>
                <button class="btn-table-delete" @click="revokeKey(key.id)">Revoke</button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<style scoped>
.clients-page {
  display: flex;
  flex-direction: column;
  gap: 24px;
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

/* Policy Card */
.policy-card {
  background: #f8fafc;
  border: 1px solid #cbd5e1;
  border-radius: 12px;
  padding: 18px 24px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 16px;
}

.policy-header {
  display: flex;
  align-items: center;
  gap: 14px;
}

.policy-header .icon {
  font-size: 1.75rem;
}

.policy-header h4 {
  margin: 0 0 2px 0;
  font-size: 1rem;
  color: #0f172a;
}

.policy-header p {
  margin: 0;
  font-size: 0.85rem;
  color: #64748b;
}

.policy-details {
  display: flex;
  gap: 20px;
  flex-wrap: wrap;
}

.policy-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.85rem;
}

.p-label { color: #64748b; }
.p-val { font-weight: 700; color: #0f172a; }

.p-tags { display: flex; gap: 4px; }
.claim-tag {
  background: #e2e8f0;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.75rem;
  font-weight: 600;
}

/* Grid */
.clients-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 24px;
}

@media (max-width: 1024px) {
  .clients-grid {
    grid-template-columns: 1fr;
  }
}

.panel {
  background: #ffffff;
  border: 1px solid #e2e8f0;
  border-radius: 12px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.02);
  display: flex;
  flex-direction: column;
}

.panel-header {
  padding: 16px 20px;
  background: #f8fafc;
  border-bottom: 1px solid #f1f5f9;
  border-radius: 12px 12px 0 0;
}

.panel-header h3 {
  margin: 0;
  font-size: 1.05rem;
  color: #1e293b;
  font-weight: 700;
}

.panel-body {
  padding: 20px;
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
}

.code-textarea {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  font-size: 0.85rem;
  resize: vertical;
}

.btn-primary {
  padding: 10px 16px;
  background: #2563eb;
  color: #ffffff;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
}
.btn-primary:hover { background: #1d4ed8; }

.btn-secondary {
  padding: 9px 16px;
  background: #f1f5f9;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  font-weight: 600;
  color: #334155;
  cursor: pointer;
}

.generated-box {
  margin-top: 10px;
  background: #f8fafc;
  border: 1px solid #cbd5e1;
  border-radius: 8px;
  padding: 12px;
}

.gen-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.8rem;
  font-weight: 600;
  margin-bottom: 8px;
}

.btn-copy-token {
  padding: 4px 10px;
  font-size: 0.75rem;
  background: #2563eb;
  color: #ffffff;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.token-textarea {
  width: 100%;
  font-size: 0.8rem;
  padding: 8px;
  border: 1px solid #e2e8f0;
  border-radius: 4px;
  word-break: break-all;
}

.error-box {
  padding: 10px;
  background: #fef2f2;
  border-left: 4px solid #ef4444;
  color: #b91c1c;
  font-size: 0.85rem;
  border-radius: 4px;
}

.decoded-results {
  margin-top: 10px;
}

.json-box {
  margin: 4px 0 0 0;
  padding: 12px;
  background: #0f172a;
  color: #38bdf8;
  font-size: 0.8rem;
  border-radius: 6px;
  max-height: 220px;
  overflow-y: auto;
}

/* Table */
.data-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.9rem;
  text-align: left;
}

.data-table th {
  padding: 10px 14px;
  background: #f8fafc;
  color: #64748b;
  font-weight: 600;
  border-bottom: 1px solid #e2e8f0;
  font-size: 0.8rem;
  text-transform: uppercase;
}

.data-table td {
  padding: 12px 14px;
  border-bottom: 1px solid #f1f5f9;
}

.sub-pill {
  padding: 2px 8px;
  background: #e0f2fe;
  color: #0369a1;
  border-radius: 4px;
  font-size: 0.8rem;
  font-weight: 600;
}

.token-snippet {
  background: #f1f5f9;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.8rem;
}

.btn-table-action {
  padding: 4px 10px;
  font-size: 0.8rem;
  background: #eff6ff;
  color: #2563eb;
  border: 1px solid #bfdbfe;
  border-radius: 4px;
  cursor: pointer;
  margin-right: 6px;
}

.btn-table-delete {
  padding: 4px 10px;
  font-size: 0.8rem;
  background: #fef2f2;
  color: #dc2626;
  border: 1px solid #fecaca;
  border-radius: 4px;
  cursor: pointer;
}

.font-bold { font-weight: 700; }
.font-mono { font-family: monospace; }
.text-muted { color: #64748b; }
.text-right { text-align: right; }
.empty-state { text-align: center; color: #94a3b8; padding: 24px; }
</style>
