<script setup lang="ts">
import { computed } from 'vue';
import { RouterLink, RouterView, useRoute } from 'vue-router';
import { useGatewayStore } from './stores/gateway';

const route = useRoute();
const gatewayStore = useGatewayStore();

const routeTitle = computed(() => {
  switch (route.name) {
    case 'dashboard':
      return 'System Dashboard';
    case 'routes':
      return 'Route Management & Load Balancing';
    case 'metrics':
      return 'Metrics & Observability';
    case 'playground':
      return 'API & AI Route Playground';
    case 'clients':
      return 'Security & Client API Keys';
    case 'config':
      return 'Global Gateway Configuration';
    default:
      return 'Admin Console';
  }
});
</script>

<template>
  <div class="app-layout">
    <!-- Sidebar Navigation -->
    <aside class="sidebar">
      <div class="logo">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="26"
          height="26"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          class="logo-icon"
        >
          <polygon points="12 2 2 7 12 12 22 7 12 2"></polygon>
          <polyline points="2 17 12 22 22 17"></polyline>
          <polyline points="2 12 12 17 22 12"></polyline>
        </svg>
        <div class="logo-text-group">
          <span class="logo-title">Kairos Gateway</span>
          <span class="logo-sub">AI & Multi-Protocol</span>
        </div>
      </div>

      <nav>
        <RouterLink to="/">
          <span class="nav-icon">📊</span>
          <span>Dashboard</span>
        </RouterLink>

        <RouterLink to="/routes">
          <span class="nav-icon">🛣️</span>
          <span>Routes</span>
        </RouterLink>

        <RouterLink to="/metrics">
          <span class="nav-icon">📈</span>
          <span>Metrics</span>
        </RouterLink>

        <RouterLink to="/playground">
          <span class="nav-icon">🧪</span>
          <span>Playground</span>
        </RouterLink>

        <RouterLink to="/clients">
          <span class="nav-icon">🔑</span>
          <span>Clients & Auth</span>
        </RouterLink>

        <RouterLink to="/config">
          <span class="nav-icon">⚙️</span>
          <span>Config</span>
        </RouterLink>
      </nav>

      <div class="sidebar-footer">
        <div class="status-indicator-row">
          <span
            class="status-dot"
            :class="{ online: gatewayStore.wsConnected }"
          ></span>
          <span class="status-label">
            {{ gatewayStore.wsConnected ? 'Backend Connected' : 'Connecting...' }}
          </span>
        </div>
        <span class="version-label">v0.4.0 Production</span>
      </div>
    </aside>

    <!-- Main Content Area -->
    <main class="main-content">
      <header class="top-nav">
        <div class="breadcrumb">
          <span class="breadcrumb-root">Kairos Gateway</span>
          <span class="breadcrumb-separator">/</span>
          <span class="breadcrumb-current">{{ routeTitle }}</span>
        </div>

        <div class="top-nav-actions">
          <a
            href="/metrics"
            target="_blank"
            class="nav-link-btn"
            title="Open Prometheus endpoint"
          >
            Prometheus Metrics ↗
          </a>
          <a
            href="/health"
            target="_blank"
            class="nav-link-btn"
            title="Open Health Probe"
          >
            Health Probe ↗
          </a>
        </div>
      </header>

      <div class="page-container">
        <RouterView />
      </div>
    </main>
  </div>
</template>

<style>
/* Layout resets */
* {
  box-sizing: border-box;
}

body {
  margin: 0;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  color: #0f172a;
  background-color: #f8fafc;
}

.app-layout {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

/* Sidebar */
.sidebar {
  width: 250px;
  background: #0f172a;
  color: #e2e8f0;
  display: flex;
  flex-direction: column;
  box-shadow: 2px 0 10px rgba(0, 0, 0, 0.08);
  z-index: 20;
}

.logo {
  padding: 20px 20px 18px 20px;
  display: flex;
  align-items: center;
  gap: 12px;
  border-bottom: 1px solid #1e293b;
}

.logo-icon {
  color: #38bdf8;
  flex-shrink: 0;
}

.logo-text-group {
  display: flex;
  flex-direction: column;
}

.logo-title {
  font-size: 1.15rem;
  font-weight: 700;
  color: #f8fafc;
  line-height: 1.2;
}

.logo-sub {
  font-size: 0.72rem;
  color: #94a3b8;
  letter-spacing: 0.5px;
  text-transform: uppercase;
}

.sidebar nav {
  flex: 1;
  padding: 16px 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  overflow-y: auto;
}

.sidebar nav a {
  display: flex;
  align-items: center;
  padding: 10px 14px;
  color: #94a3b8;
  text-decoration: none;
  border-radius: 8px;
  font-weight: 600;
  font-size: 0.9rem;
  transition: all 0.2s ease;
}

.sidebar nav a .nav-icon {
  margin-right: 12px;
  font-size: 1.1rem;
}

.sidebar nav a:hover {
  background: #1e293b;
  color: #f8fafc;
}

.sidebar nav a.router-link-active {
  background: #2563eb;
  color: #ffffff;
  box-shadow: 0 4px 6px -1px rgba(37, 99, 235, 0.25);
}

.sidebar-footer {
  padding: 16px 20px;
  border-top: 1px solid #1e293b;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.status-indicator-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.8rem;
  color: #94a3b8;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #ef4444;
}

.status-dot.online {
  background: #10b981;
  box-shadow: 0 0 6px #10b981;
}

.version-label {
  font-size: 0.75rem;
  color: #64748b;
  font-family: monospace;
}

/* Main Content */
.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: #f8fafc;
}

.top-nav {
  height: 60px;
  background: #ffffff;
  border-bottom: 1px solid #e2e8f0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 32px;
}

.breadcrumb {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.9rem;
}

.breadcrumb-root {
  color: #64748b;
  font-weight: 500;
}

.breadcrumb-separator {
  color: #cbd5e1;
}

.breadcrumb-current {
  color: #0f172a;
  font-weight: 700;
}

.top-nav-actions {
  display: flex;
  gap: 12px;
}

.nav-link-btn {
  font-size: 0.8rem;
  font-weight: 600;
  color: #475569;
  text-decoration: none;
  padding: 6px 12px;
  border: 1px solid #cbd5e1;
  border-radius: 6px;
  background: #f8fafc;
  transition: all 0.2s;
}

.nav-link-btn:hover {
  background: #e2e8f0;
  color: #0f172a;
}

.page-container {
  flex: 1;
  padding: 28px 32px;
  overflow-y: auto;
}
</style>
