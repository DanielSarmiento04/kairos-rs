<script setup lang="ts">
import { RouterLink, RouterView } from 'vue-router'
</script>

<template>
  <div class="app-layout">
    <aside class="sidebar">
      <div class="logo">
        <h2>Kairos Gateway</h2>
      </div>
      <nav>
        <RouterLink to="/">Dashboard</RouterLink>
        <RouterLink to="/routes">Routes</RouterLink>
        <RouterLink to="/config">Config</RouterLink>
      </nav>
    </aside>

    <main class="main-content">
      <RouterView />
    </main>
  </div>
</template>

<style>
/* Reset basic margins */
body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  background-color: #f5f7fa;
  color: #333;
}

.app-layout {
  display: flex;
  min-height: 100vh;
}

.sidebar {
  width: 250px;
  background: #1e293b;
  color: #fff;
  display: flex;
  flex-direction: column;
}

.sidebar .logo {
  padding: 20px;
  background: #0f172a;
  text-align: center;
}

.sidebar nav {
  display: flex;
  flex-direction: column;
  padding: 10px 0;
}

.sidebar nav a {
  padding: 15px 20px;
  color: #cbd5e1;
  text-decoration: none;
  transition: all 0.2s;
}

.sidebar nav a:hover,
.sidebar nav a.router-link-active {
  background: #334155;
  color: #fff;
}

.main-content {
  flex: 1;
  padding: 30px;
  max-width: 1200px;
  width: 100%;
}
</style>
