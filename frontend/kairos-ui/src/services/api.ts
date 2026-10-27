import { authService } from './auth';
import type {
  Settings,
  Router,
  RouteResponse,
  ValidateRouteResponse,
  JwtSettings,
  RateLimitConfig,
  CorsConfig,
  MetricsConfig,
  ServerConfig,
  AiSettings,
  AdminAuthConfig,
  AdminAuthVerifyResponse,
  HealthStatus,
  HistoricalMetricsQuery,
  HistoricalMetricPoint,
  AggregatedMetricPoint,
  RawHistoricalMetricPoint,
  PlaygroundRequest,
  PlaygroundResponse,
  CacheStatsResponse,
  LatencyPercentileQuery,
  PercentilePoint,
} from '../types';

const API_BASE = '/api';

async function apiFetch(input: string, init: RequestInit = {}): Promise<Response> {
  const headers = authService.getAuthHeaders(init.headers);
  const response = await fetch(input, { ...init, headers });
  if (response.status === 401) {
    authService.handleUnauthorized();
  }
  return response;
}

export const apiService = {
  // Admin Auth
  async verifyAdminAuth(key?: string): Promise<AdminAuthVerifyResponse> {
    return authService.verify(key);
  },

  async updateAdminAuthConfig(config: AdminAuthConfig): Promise<{ success: boolean; message: string }> {
    const res = await apiFetch(`${API_BASE}/config/admin-auth`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(config),
    });
    return res.json();
  },

  // Routes CRUD & Validation
  async getRoutes(): Promise<Router[]> {
    const res = await apiFetch(`${API_BASE}/routes`);
    if (!res.ok) {
      const err = await res.json().catch(() => ({ message: 'Failed to fetch routes' }));
      throw new Error(err.message || 'Failed to fetch routes');
    }
    const data: RouteResponse = await res.json();
    return data.routes || [];
  },

  async getRoute(path: string): Promise<Router> {
    const encodedPath = encodeURIComponent(path.startsWith('/') ? path.slice(1) : path);
    const res = await apiFetch(`${API_BASE}/routes/${encodedPath}`);
    if (!res.ok) {
      const err = await res.json().catch(() => ({ message: 'Failed to fetch route' }));
      throw new Error(err.message || 'Failed to fetch route');
    }
    const data: RouteResponse = await res.json();
    if (!data.route) throw new Error('Route not found');
    return data.route;
  },

  async createRoute(route: Router): Promise<RouteResponse> {
    const res = await apiFetch(`${API_BASE}/routes`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(route),
    });
    const data: RouteResponse = await res.json().catch(() => ({
      success: res.ok,
      message: res.ok ? 'Created' : 'Failed to create route',
    }));
    if (!res.ok) {
      throw new Error(data.message || 'Failed to create route');
    }
    return data;
  },

  async updateRoute(path: string, route: Router): Promise<RouteResponse> {
    const encodedPath = encodeURIComponent(path.startsWith('/') ? path.slice(1) : path);
    const res = await apiFetch(`${API_BASE}/routes/${encodedPath}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(route),
    });
    const data: RouteResponse = await res.json().catch(() => ({
      success: res.ok,
      message: res.ok ? 'Updated' : 'Failed to update route',
    }));
    if (!res.ok) {
      throw new Error(data.message || 'Failed to update route');
    }
    return data;
  },

  async deleteRoute(path: string): Promise<RouteResponse> {
    const encodedPath = encodeURIComponent(path.startsWith('/') ? path.slice(1) : path);
    const res = await apiFetch(`${API_BASE}/routes/${encodedPath}`, {
      method: 'DELETE',
    });
    const data: RouteResponse = await res.json().catch(() => ({
      success: res.ok,
      message: res.ok ? 'Deleted' : 'Failed to delete route',
    }));
    if (!res.ok) {
      throw new Error(data.message || 'Failed to delete route');
    }
    return data;
  },

  async validateRoute(route: Router): Promise<ValidateRouteResponse> {
    const res = await apiFetch(`${API_BASE}/routes/validate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ route }),
    });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ message: 'Validation request failed' }));
      return { valid: false, error: err.message || 'Validation request failed' };
    }
    return res.json();
  },

  // Configuration Management
  async getConfig(): Promise<Settings> {
    const res = await apiFetch(`${API_BASE}/config`);
    if (!res.ok) throw new Error('Failed to fetch configuration');
    return res.json();
  },

  async updateAiConfig(config: AiSettings): Promise<{ success: boolean; message: string }> {
    const res = await apiFetch(`${API_BASE}/config/ai`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(config),
    });
    return res.json();
  },

  async updateJwtConfig(
    config: JwtSettings
  ): Promise<{ success: boolean; message: string }> {
    const res = await apiFetch(`${API_BASE}/config/jwt`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(config),
    });
    return res.json();
  },

  async updateRateLimitConfig(
    config: RateLimitConfig
  ): Promise<{ success: boolean; message: string }> {
    const res = await apiFetch(`${API_BASE}/config/rate-limit`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(config),
    });
    return res.json();
  },

  async updateCorsConfig(
    config: CorsConfig
  ): Promise<{ success: boolean; message: string }> {
    const res = await apiFetch(`${API_BASE}/config/cors`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(config),
    });
    return res.json();
  },

  async updateMetricsConfig(
    config: MetricsConfig
  ): Promise<{ success: boolean; message: string }> {
    const res = await apiFetch(`${API_BASE}/config/metrics`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(config),
    });
    return res.json();
  },

  async updateServerConfig(
    config: ServerConfig
  ): Promise<{ success: boolean; message: string }> {
    const res = await apiFetch(`${API_BASE}/config/server`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(config),
    });
    return res.json();
  },

  async reloadConfig(): Promise<{ success: boolean; message: string }> {
    const res = await apiFetch(`${API_BASE}/config/reload`, { method: 'POST' });
    return res.json();
  },

  // Cache Management
  async getCacheStats(): Promise<CacheStatsResponse> {
    const res = await apiFetch(`${API_BASE}/cache/stats`);
    if (!res.ok) throw new Error('Failed to fetch cache stats');
    return res.json();
  },

  async clearCache(): Promise<{ success: boolean; message: string }> {
    const res = await apiFetch(`${API_BASE}/cache/clear`, { method: 'POST' });
    return res.json();
  },

  async invalidateCache(pattern: string, is_regex = false): Promise<{ success: boolean; message: string; evicted_count?: number }> {
    const res = await apiFetch(`${API_BASE}/cache/invalidate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ pattern, is_regex }),
    });
    return res.json();
  },

  // Historical Metrics
  async getMetricsList(): Promise<{ metrics: string[] }> {
    const res = await apiFetch(`${API_BASE}/metrics/list`);
    if (!res.ok) return { metrics: [] };
    const data = await res.json();
    return { metrics: data.metrics || [] };
  },

  async queryHistoricalMetrics(query: HistoricalMetricsQuery): Promise<{
    metric_name: string;
    points: HistoricalMetricPoint[];
  }> {
    const params = new URLSearchParams({
      name: query.metric_name,
      start: query.start_time,
      end: query.end_time,
    });
    if (query.interval) params.set('interval', query.interval);

    const res = await apiFetch(`${API_BASE}/metrics/history?${params.toString()}`);
    if (!res.ok) throw new Error('Failed to fetch metrics history');
    const data = await res.json();

    const points = (data.points || []).map(
      (p: AggregatedMetricPoint | RawHistoricalMetricPoint) => {
        let val = 0;
        if ('avg' in p) {
          val = p.avg;
        } else if ('value' in p) {
          val = typeof p.value === 'number' ? p.value : p.value.value;
        }
        return {
          timestamp: p.timestamp,
          value: val,
        };
      }
    );

    return {
      metric_name: data.metric_name || query.metric_name,
      points,
    };
  },

  async getRawPrometheus(): Promise<string> {
    const res = await apiFetch('/metrics');
    if (!res.ok) throw new Error('Failed to fetch raw metrics');
    return res.text();
  },

  async queryPercentiles(query: LatencyPercentileQuery): Promise<{
    metric_name: string;
    points: PercentilePoint[];
  }> {
    const params = new URLSearchParams({
      name: query.metric_name || 'http_request_duration_ms',
      start: query.start_time,
      end: query.end_time,
      percentiles: (query.percentiles || [50, 95, 99]).join(','),
      interval: query.interval || '1m',
    });

    const res = await apiFetch(`${API_BASE}/metrics/percentiles?${params.toString()}`);
    if (!res.ok) {
      return { metric_name: query.metric_name, points: [] };
    }
    const data = await res.json();
    return {
      metric_name: data.metric_name || query.metric_name,
      points: data.points || [],
    };
  },

  // Health Checks
  async getHealth(): Promise<HealthStatus> {
    const res = await apiFetch('/health');
    if (!res.ok) throw new Error('Failed to fetch health');
    return res.json();
  },

  async getReadiness(): Promise<{ status: string; timestamp?: string }> {
    const res = await apiFetch('/ready');
    if (!res.ok) throw new Error('Failed to fetch readiness');
    return res.json();
  },

  async getLiveness(): Promise<{ status: string; timestamp?: string }> {
    const res = await apiFetch('/live');
    if (!res.ok) throw new Error('Failed to fetch liveness');
    return res.json();
  },

  // Interactive Testing / Playground
  async sendPlaygroundRequest(req: PlaygroundRequest): Promise<PlaygroundResponse> {
    const startTime = performance.now();
    try {
      const res = await fetch(req.path, {
        method: req.method,
        headers: req.headers,
        body: req.method !== 'GET' && req.method !== 'HEAD' ? req.body : undefined,
      });
      const endTime = performance.now();
      const latency_ms = Math.round(endTime - startTime);

      const headers: Record<string, string> = {};
      res.headers.forEach((val, key) => {
        headers[key] = val;
      });

      const body = await res.text();
      return {
        status: res.status,
        statusText: res.statusText,
        latency_ms,
        headers,
        body,
      };
    } catch (err: unknown) {
      const endTime = performance.now();
      const errorMessage = err instanceof Error ? err.message : String(err);
      return {
        status: 0,
        statusText: 'Network Error',
        latency_ms: Math.round(endTime - startTime),
        headers: {},
        body: errorMessage,
      };
    }
  },
};
