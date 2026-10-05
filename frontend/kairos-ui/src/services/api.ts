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
  HealthStatus,
  HistoricalMetricsQuery,
  HistoricalMetricPoint,
  AggregatedMetricPoint,
  RawHistoricalMetricPoint,
  PlaygroundRequest,
  PlaygroundResponse,
} from '../types';

const API_BASE = '/api';

export const apiService = {
  // Routes CRUD & Validation
  async getRoutes(): Promise<Router[]> {
    const res = await fetch(`${API_BASE}/routes`);
    if (!res.ok) {
      const err = await res.json().catch(() => ({ message: 'Failed to fetch routes' }));
      throw new Error(err.message || 'Failed to fetch routes');
    }
    const data: RouteResponse = await res.json();
    return data.routes || [];
  },

  async getRoute(path: string): Promise<Router> {
    const encodedPath = encodeURIComponent(path.startsWith('/') ? path.slice(1) : path);
    const res = await fetch(`${API_BASE}/routes/${encodedPath}`);
    if (!res.ok) {
      const err = await res.json().catch(() => ({ message: 'Failed to fetch route' }));
      throw new Error(err.message || 'Failed to fetch route');
    }
    const data: RouteResponse = await res.json();
    if (!data.route) throw new Error('Route not found');
    return data.route;
  },

  async createRoute(route: Router): Promise<RouteResponse> {
    const res = await fetch(`${API_BASE}/routes`, {
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
    const res = await fetch(`${API_BASE}/routes/${encodedPath}`, {
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
    const res = await fetch(`${API_BASE}/routes/${encodedPath}`, {
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
    const res = await fetch(`${API_BASE}/routes/validate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ route }),
    });
    if (!res.ok) {
      return { valid: false, error: `Validation request failed with status ${res.status}` };
    }
    return res.json();
  },

  // Configuration Management
  async getConfig(): Promise<Settings> {
    const res = await fetch(`${API_BASE}/config`);
    if (!res.ok) throw new Error('Failed to fetch config');
    return res.json();
  },

  async updateAiConfig(ai: AiSettings): Promise<{ success: boolean; message: string }> {
    const res = await fetch(`${API_BASE}/config/ai`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(ai),
    });
    const data = await res.json();
    if (!res.ok) throw new Error(data.message || 'Failed to update AI configuration');
    return data;
  },

  async updateJwtConfig(jwt: JwtSettings): Promise<{ success: boolean; message: string }> {
    const res = await fetch(`${API_BASE}/config/jwt`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(jwt),
    });
    const data = await res.json();
    if (!res.ok) throw new Error(data.message || 'Failed to update JWT configuration');
    return data;
  },

  async updateRateLimitConfig(
    rateLimit: RateLimitConfig,
  ): Promise<{ success: boolean; message: string }> {
    const res = await fetch(`${API_BASE}/config/rate-limit`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(rateLimit),
    });
    const data = await res.json();
    if (!res.ok) throw new Error(data.message || 'Failed to update Rate Limit configuration');
    return data;
  },

  async updateCorsConfig(cors: CorsConfig): Promise<{ success: boolean; message: string }> {
    const res = await fetch(`${API_BASE}/config/cors`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(cors),
    });
    const data = await res.json();
    if (!res.ok) throw new Error(data.message || 'Failed to update CORS configuration');
    return data;
  },

  async updateMetricsConfig(
    metrics: MetricsConfig,
  ): Promise<{ success: boolean; message: string }> {
    const res = await fetch(`${API_BASE}/config/metrics`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(metrics),
    });
    const data = await res.json();
    if (!res.ok) throw new Error(data.message || 'Failed to update Metrics configuration');
    return data;
  },

  async updateServerConfig(server: ServerConfig): Promise<{ success: boolean; message: string }> {
    const res = await fetch(`${API_BASE}/config/server`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(server),
    });
    const data = await res.json();
    if (!res.ok) throw new Error(data.message || 'Failed to update Server configuration');
    return data;
  },

  async triggerReload(): Promise<{ success: boolean; message: string }> {
    const res = await fetch(`${API_BASE}/config/reload`, { method: 'POST' });
    const data = await res.json().catch(() => ({
      success: res.ok,
      message: res.ok ? 'Reloaded' : 'Failed to reload',
    }));
    if (!res.ok) throw new Error(data.message || 'Failed to reload config');
    return data;
  },

  // Observability & Historical Metrics
  async getMetricsList(): Promise<string[]> {
    const res = await fetch(`${API_BASE}/metrics/list`);
    if (!res.ok) throw new Error('Failed to fetch metrics list');
    return res.json();
  },

  async getHistoricalMetrics(
    query: HistoricalMetricsQuery,
  ): Promise<(HistoricalMetricPoint | AggregatedMetricPoint)[]> {
    const params = new URLSearchParams({
      name: query.name,
      start: query.start,
      end: query.end,
    });
    if (query.interval) {
      params.append('interval', query.interval);
    }
    const res = await fetch(`${API_BASE}/metrics/history?${params.toString()}`);
    if (!res.ok) throw new Error('Failed to fetch historical metrics');
    // Backend wraps scalar values in `MetricValue` (serde tagged enum).
    // Normalize to a plain `number` so chart components can render
    // directly without branching on the wire format.
    const raw: Array<RawHistoricalMetricPoint | AggregatedMetricPoint> = await res.json();
    return raw.map((p): HistoricalMetricPoint | AggregatedMetricPoint => {
      if ('avg' in p) {
        return p;
      }
      const v = p.value;
      if (typeof v === 'object' && v !== null) {
        return { timestamp: p.timestamp, value: v.value };
      }
      return { timestamp: p.timestamp, value: v };
    });
  },

  async getRawPrometheus(): Promise<string> {
    const res = await fetch('/metrics');
    if (!res.ok) throw new Error('Failed to fetch Prometheus metrics');
    return res.text();
  },

  // Health Checks
  async getHealth(): Promise<HealthStatus> {
    const res = await fetch('/health');
    if (!res.ok) throw new Error('Failed to fetch health');
    return res.json();
  },

  async getReadiness(): Promise<{ status: string; timestamp?: string }> {
    const res = await fetch('/ready');
    if (!res.ok) throw new Error('Failed to fetch readiness');
    return res.json();
  },

  async getLiveness(): Promise<{ status: string; timestamp?: string }> {
    const res = await fetch('/live');
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
