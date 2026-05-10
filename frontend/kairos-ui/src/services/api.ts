import type { Settings, Router } from '../types';

const API_BASE = '/api';

export const apiService = {
  // Routes
  async getRoutes(): Promise<Router[]> {
    const res = await fetch(`${API_BASE}/routes`);
    if (!res.ok) throw new Error('Failed to fetch routes');
    const data = await res.json();
    return data.routes || [];
  },

  async createRoute(route: Router): Promise<void> {
    const res = await fetch(`${API_BASE}/routes`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(route),
    });
    if (!res.ok) throw new Error('Failed to create route');
  },

  async deleteRoute(path: string): Promise<void> {
    const res = await fetch(`${API_BASE}/routes${path}`, { method: 'DELETE' });
    if (!res.ok) throw new Error('Failed to delete route');
  },

  // Config
  async getConfig(): Promise<Settings> {
    const res = await fetch(`${API_BASE}/config`);
    if (!res.ok) throw new Error('Failed to fetch config');
    return res.json();
  },

  async triggerReload(): Promise<void> {
    const res = await fetch(`${API_BASE}/config/reload`, { method: 'POST' });
    if (!res.ok) throw new Error('Failed to reload config');
  },

  // Health
  async getHealth(): Promise<{ status: string }> {
    const res = await fetch('/health');
    if (!res.ok) throw new Error('Failed to fetch health');
    return res.json();
  },
};
