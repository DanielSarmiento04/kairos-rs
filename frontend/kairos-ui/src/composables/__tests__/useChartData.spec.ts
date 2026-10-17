import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useChartData, resolveTimeRange } from '../useChartData';
import { apiService } from '@/services/api';
import type {
  AggregatedMetricPoint,
  HistoricalMetricPoint,
  PercentilePoint,
} from '@/types';

vi.mock('@/services/api', () => ({
  apiService: {
    getHistoricalMetrics: vi.fn(),
    getLatencyPercentiles: vi.fn(),
  },
}));

const mockGetHistorical = apiService.getHistoricalMetrics as unknown as ReturnType<
  typeof vi.fn
>;
const mockGetPercentiles = apiService.getLatencyPercentiles as unknown as ReturnType<
  typeof vi.fn
>;

describe('resolveTimeRange', () => {
  it('returns ISO start/end for a preset range', () => {
    const { start, end } = resolveTimeRange('1h');
    const diff = new Date(end).getTime() - new Date(start).getTime();
    expect(diff).toBe(60 * 60 * 1000);
  });

  it('uses custom range when provided', () => {
    const custom = { start: '2025-01-01T00:00:00Z', end: '2025-01-01T01:00:00Z' };
    const r = resolveTimeRange('custom', custom);
    expect(r.start).toBe(custom.start);
    expect(r.end).toBe(custom.end);
  });

  it('throws if custom range is missing', () => {
    expect(() => resolveTimeRange('custom')).toThrow();
  });

  it('handles 7d range', () => {
    const { start, end } = resolveTimeRange('7d');
    const diff = new Date(end).getTime() - new Date(start).getTime();
    expect(diff).toBe(7 * 24 * 60 * 60 * 1000);
  });
});

describe('useChartData', () => {
  beforeEach(() => {
    mockGetHistorical.mockReset();
    mockGetPercentiles.mockReset();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('fetchHistorical delegates to apiService with raw aggregation', async () => {
    mockGetHistorical.mockResolvedValue([]);
    const { fetchHistorical } = useChartData();
    await fetchHistorical('requests_total', '1h', 'raw');
    expect(mockGetHistorical).toHaveBeenCalledTimes(1);
    const arg = mockGetHistorical.mock.calls[0]![0];
    expect(arg.name).toBe('requests_total');
    expect(arg.start).toBeDefined();
    expect(arg.end).toBeDefined();
    // Raw mode → no `interval` key in the payload.
    expect('interval' in arg).toBe(false);
  });

  it('fetchHistorical sends interval when not raw', async () => {
    mockGetHistorical.mockResolvedValue([]);
    const { fetchHistorical } = useChartData();
    await fetchHistorical('requests_total', '1h', 'one_minute');
    expect(mockGetHistorical).toHaveBeenCalledWith(
      expect.objectContaining({ interval: 'one_minute' }),
    );
  });

  it('fetchPercentiles delegates to apiService with percentiles array', async () => {
    mockGetPercentiles.mockResolvedValue([]);
    const { fetchPercentiles } = useChartData();
    await fetchPercentiles('response_time', '1h', 'one_minute', [50, 95, 99]);
    expect(mockGetPercentiles).toHaveBeenCalledWith(
      expect.objectContaining({ percentiles: [50, 95, 99] }),
    );
  });

  it('fetchPercentiles omits percentiles when array is empty', async () => {
    mockGetPercentiles.mockResolvedValue([]);
    const { fetchPercentiles } = useChartData();
    await fetchPercentiles('response_time', '1h', 'one_minute', []);
    const arg = mockGetPercentiles.mock.calls[0]![0];
    expect(arg.percentiles).toBeUndefined();
  });

  it('buildHistoricalQuery omits interval for raw', () => {
    const { buildHistoricalQuery } = useChartData();
    const q = buildHistoricalQuery('m', '1h', 'raw');
    expect(q.interval).toBeUndefined();
  });

  it('buildHistoricalQuery includes interval otherwise', () => {
    const { buildHistoricalQuery } = useChartData();
    const q = buildHistoricalQuery('m', '1h', 'five_minutes');
    expect(q.interval).toBe('five_minutes');
  });

  it('buildPercentileQuery sets name/start/end/interval', () => {
    const { buildPercentileQuery } = useChartData();
    const q = buildPercentileQuery('rt', '6h', 'one_hour', [95]);
    expect(q.name).toBe('rt');
    expect(q.interval).toBe('one_hour');
    expect(q.percentiles).toEqual([95]);
  });

  it('toCsvHistorical serializes raw + aggregated rows', () => {
    const rows: (HistoricalMetricPoint | AggregatedMetricPoint)[] = [
      { timestamp: '2025-01-01T00:00:00Z', value: 42 },
      { timestamp: '2025-01-01T00:01:00Z', min: 1, max: 5, avg: 3, count: 4 },
    ];
    // Mock URL/document to capture blob content.
    const created: { content?: string; name?: string } = {};
    const originalCreate = URL.createObjectURL;
    URL.createObjectURL = vi.fn((blob: Blob) => {
      blob.text().then((t) => (created.content = t));
      return 'blob:fake';
    }) as typeof URL.createObjectURL;
    const originalRevoke = URL.revokeObjectURL;
    URL.revokeObjectURL = vi.fn();

    const { toCsvHistorical } = useChartData();
    toCsvHistorical(rows, 'test.csv');

    URL.createObjectURL = originalCreate;
    URL.revokeObjectURL = originalRevoke;

    return Promise.resolve().then(() => {
      expect(created.content).toBeDefined();
      expect(created.content!).toContain('timestamp,value,min,max,avg,count');
      expect(created.content!).toContain('2025-01-01T00:00:00Z,42,,,,');
    });
  });

  it('toCsvPercentiles serializes one row per percentile', () => {
    const rows: PercentilePoint[] = [
      { timestamp: '2025-01-01T00:00:00Z', percentiles: { '50': 10, '95': 50 } },
      { timestamp: '2025-01-01T00:01:00Z', percentiles: { '50': 12, '95': 55 } },
    ];
    const created: { content?: string } = {};
    const originalCreate = URL.createObjectURL;
    URL.createObjectURL = vi.fn((blob: Blob) => {
      blob.text().then((t) => (created.content = t));
      return 'blob:fake';
    }) as typeof URL.createObjectURL;
    const originalRevoke = URL.revokeObjectURL;
    URL.revokeObjectURL = vi.fn();

    const { toCsvPercentiles } = useChartData();
    toCsvPercentiles(rows, 'percentiles.csv');

    URL.createObjectURL = originalCreate;
    URL.revokeObjectURL = originalRevoke;

    return Promise.resolve().then(() => {
      expect(created.content).toBeDefined();
      expect(created.content!).toContain('timestamp,50,95');
      expect(created.content!).toContain('2025-01-01T00:00:00Z,10,50');
    });
  });

  it('toCsvPercentiles handles empty rows gracefully', () => {
    const { toCsvPercentiles } = useChartData();
    // Should not throw, just produce header-only CSV.
    expect(() => toCsvPercentiles([], 'empty.csv')).not.toThrow();
  });
});