<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  type: 'status' | 'method' | 'protocol' | 'auth';
  value: string | number | boolean;
}>();

const badgeClass = computed(() => {
  if (props.type === 'method') {
    return `method-${String(props.value).toLowerCase()}`;
  }
  if (props.type === 'protocol') {
    return `protocol-${String(props.value).toLowerCase()}`;
  }
  if (props.type === 'auth') {
    return props.value ? 'auth-required' : 'auth-public';
  }
  const val = Number(props.value);
  if (val >= 200 && val < 300) return 'status-2xx';
  if (val >= 300 && val < 400) return 'status-3xx';
  if (val >= 400 && val < 500) return 'status-4xx';
  if (val >= 500) return 'status-5xx';
  return 'status-unknown';
});

const displayText = computed(() => {
  if (props.type === 'auth') {
    return props.value ? '🔒 JWT' : '🔓 Public';
  }
  if (props.type === 'protocol') {
    return String(props.value).toUpperCase();
  }
  return String(props.value);
});
</script>

<template>
  <span class="badge" :class="badgeClass">
    {{ displayText }}
  </span>
</template>

<style scoped>
.badge {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  font-size: 0.75rem;
  font-weight: 700;
  border-radius: 4px;
  letter-spacing: 0.3px;
  text-transform: uppercase;
}

/* Status colors */
.status-2xx {
  background: #ecfdf5;
  color: #059669;
  border: 1px solid #a7f3d0;
}
.status-3xx {
  background: #eff6ff;
  color: #2563eb;
  border: 1px solid #bfdbfe;
}
.status-4xx {
  background: #fffbeb;
  color: #d97706;
  border: 1px solid #fde68a;
}
.status-5xx {
  background: #fef2f2;
  color: #dc2626;
  border: 1px solid #fecaca;
}
.status-unknown {
  background: #f1f5f9;
  color: #64748b;
  border: 1px solid #e2e8f0;
}

/* Method colors */
.method-get {
  background: #eff6ff;
  color: #2563eb;
  border: 1px solid #bfdbfe;
}
.method-post {
  background: #ecfdf5;
  color: #059669;
  border: 1px solid #a7f3d0;
}
.method-put {
  background: #fffbeb;
  color: #d97706;
  border: 1px solid #fde68a;
}
.method-delete {
  background: #fef2f2;
  color: #dc2626;
  border: 1px solid #fecaca;
}
.method-patch {
  background: #faf5ff;
  color: #9333ea;
  border: 1px solid #e9d5ff;
}

/* Protocol colors */
.protocol-http {
  background: #e0f2fe;
  color: #0284c7;
  border: 1px solid #bae6fd;
}
.protocol-websocket {
  background: #ede9fe;
  color: #7c3aed;
  border: 1px solid #ddd6fe;
}
.protocol-ftp {
  background: #ffedd5;
  color: #ea580c;
  border: 1px solid #fed7aa;
}
.protocol-dns {
  background: #ccfbf1;
  color: #0d9488;
  border: 1px solid #99f6e4;
}

/* Auth badge */
.auth-required {
  background: #fef3c7;
  color: #b45309;
  border: 1px solid #fde68a;
}
.auth-public {
  background: #f1f5f9;
  color: #64748b;
  border: 1px solid #cbd5e1;
}
</style>
