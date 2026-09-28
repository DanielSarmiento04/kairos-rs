import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { apiService } from '../api';
import type { Router } from '../../types';

describe('apiService', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('fetches routes successfully', async () => {
    const mockRoutes: Router[] = [
      {
        external_path: '/api/test',
        internal_path: '/test',
        methods: ['GET'],
        auth_required: false,
      },
    ];

    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, message: 'Found 1 routes', routes: mockRoutes }),
    });

    const routes = await apiService.getRoutes();
    expect(routes).toEqual(mockRoutes);
    expect(global.fetch).toHaveBeenCalledWith('/api/routes');
  });

  it('handles route creation and errors properly', async () => {
    const newRoute: Router = {
      external_path: '/api/new',
      internal_path: '/new',
      methods: ['POST'],
      auth_required: true,
    };

    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, message: 'Route created successfully' }),
    });

    const res = await apiService.createRoute(newRoute);
    expect(res.success).toBe(true);

    global.fetch = vi.fn().mockResolvedValue({
      ok: false,
      json: async () => ({ success: false, message: 'Route already exists' }),
    });

    await expect(apiService.createRoute(newRoute)).rejects.toThrow('Route already exists');
  });

  it('validates a route using the backend endpoint', async () => {
    const routeToValidate: Router = {
      external_path: '/api/v1/valid',
      internal_path: '/valid',
      methods: ['GET'],
      auth_required: false,
    };

    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ valid: true, error: null, warnings: null }),
    });

    const val = await apiService.validateRoute(routeToValidate);
    expect(val.valid).toBe(true);
    expect(val.error).toBeNull();
  });

  it('fetches health check endpoint', async () => {
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ status: 'healthy', version: '0.3.2', uptime: 100 }),
    });

    const health = await apiService.getHealth();
    expect(health.status).toBe('healthy');
    expect(health.version).toBe('0.3.2');
  });
});
