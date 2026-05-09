<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
</script>

<template>
  <div class="app-layout">
    <aside class="sidebar">
      <div class="logo">
        <svg xmlns="http://www.w3.org/,100" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="margin-right: 12px; color: #38bdf8;"><polygon points="12 2 2 7 12 12 22 7 12 2"></polygon><polyline points="2 17 12 22 22 17"></polyline><polyline points="2 12 12 17 22 12"></polyline></svg>
        <span class="logo-text">Kairos Gateway</span>
      </div>
      <nav>
        <RouterLink to="/">
          <span class="icon">📊</span> Dashboard
        </RouterLink>
        <RouterLink to="/routes">
          <span class="icon">🛣️</span> Routes
        </RouterLink>
        <RouterLink to="/config">
          <span class="icon">⚙️</span> Config
        </RouterLink>
      </nav>
      
      <div class="sidebar-footer">
        <span class="version">v0.2.17</span>
      </div>
    </aside>

    <main class="main-content">
      <header class="top-nav">
        <div class="breadcrumb">
          Admin Console
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
.app-layout {
  display: flex;
  height: 100vh;
  overflow: hidden;
  background-color: #f1f5f9;
}

/* Sidebar */
.sidebar {
  width: 260px;
  background: #0f172a;
  color: #e2e8f0;
  display: flex;
  flex-direction: column;
  transition: all 0.3s;
  box-shadow: 2px 0 10px rgba(0,0,0,0.1);
  z-index: 10;
}

.logo {
  padding: 24px;
  display: flex;
  align-items: center;
  border-bottom: 1px solid #1e293b;
}

.logo-text {
  font-size: 1.25rem;
  font-weight: 700;
  letter-spacing: 0.5px;
  color: #f8fafc;
}

.sidebar nav {
  flex: 1;
  padding: 20px 10px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.sidebar nav a {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  color: #94a3b8;
  text-decoration: none;
  border-radius: 8px;
  font-weight: 600;
  transition: all 0.2s ease;
}

.sidebar nav a .icon {
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
  box-shadow: 0 4px 6px -1px rgba(37, 99, 235, 0.2);
}

.sidebar-footer {
  padding: 20px;
  border-top: 1px solid #1e293b;
  font-size: 0.8rem;
  color: #64748b;
  text-align: center;
}

/* Main Content */
.main-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.top-nav {
  height: 64px;
  background: #ffffff;
  border-bottom: 1px solid #e2e8f0;
  display: flex;
  align-items: center;
  padding: 0 32px;
  box-shadow: 0 1px 2px rgba(0,0,0,0.02);
}

.breadcrumb {
  font-weight: 600;
  color: #475569;
}

.page-container {
  flex: 1;
  padding: 32px;
  overflow-y: auto;
}
</style>
