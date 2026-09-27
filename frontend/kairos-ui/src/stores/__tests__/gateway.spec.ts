import { describe, it, expect, beforeEach } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useGatewayStore } from '../gateway';

describe('Gateway Pinia Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it('initializes with default metrics snapshot', () => {
    const store = useGatewayStore();
    expect(store.metrics.requests_total).toBe(0);
    expect(store.metrics.success_rate).toBe(100.0);
    expect(store.wsConnected).toBe(false);
  });

  it('manages telemetry logs buffer using shallowRef', () => {
    const store = useGatewayStore();
    expect(store.telemetryLogs).toEqual([]);

    store.addTelemetryLog({
      id: 'log-1',
      timestamp: '12:00:00',
      method: 'GET',
      path: '/api/test',
      status: 200,
      latency_ms: 12,
      protocol: 'http',
    });

    expect(store.telemetryLogs.length).toBe(1);
    expect(store.telemetryLogs[0]?.path).toBe('/api/test');

    store.clearTelemetryLogs();
    expect(store.telemetryLogs.length).toBe(0);
  });
});
