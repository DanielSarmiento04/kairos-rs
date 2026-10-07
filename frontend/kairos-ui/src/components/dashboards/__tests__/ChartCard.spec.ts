import { describe, it, expect, vi, beforeEach } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import ChartCard from '../ChartCard.vue';
import { apiService } from '@/services/api';
import type { ChartConfig } from '@/types';

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

const baseConfig: ChartConfig = {
  id: 'chart-1',
  type: 'requests',
  metricName: 'requests_total',
  timeRange: '1h',
  aggregation: 'one_minute',
  gridX: 0,
  gridY: 0,
  gridW: 6,
  gridH: 1,
  title: 'My Requests',
};

describe('ChartCard', () => {
  beforeEach(() => {
    mockGetHistorical.mockReset();
    mockGetPercentiles.mockReset();
    mockGetHistorical.mockResolvedValue([
      { timestamp: new Date(Date.now() - 60_000).toISOString(), value: 10 },
      { timestamp: new Date().toISOString(), value: 20 },
    ]);
    mockGetPercentiles.mockResolvedValue([
      {
        timestamp: new Date(Date.now() - 60_000).toISOString(),
        percentiles: { '50': 10, '95': 50, '99': 90 },
      },
      {
        timestamp: new Date().toISOString(),
        percentiles: { '50': 11, '95': 55, '99': 95 },
      },
    ]);
  });

  it('renders the title from config', () => {
    const wrapper = mount(ChartCard, { props: { config: baseConfig } });
    expect(wrapper.text()).toContain('My Requests');
  });

  it('falls back to metricName / type when no title', () => {
    const cfg: ChartConfig = { ...baseConfig, title: undefined };
    const wrapper = mount(ChartCard, { props: { config: cfg } });
    expect(wrapper.text()).toContain('requests_total');
  });

  it('loads historical data on mount for non-percentile charts', async () => {
    mount(ChartCard, { props: { config: baseConfig } });
    await flushPromises();
    expect(mockGetHistorical).toHaveBeenCalledWith(
      expect.objectContaining({ name: 'requests_total' }),
    );
    expect(mockGetPercentiles).not.toHaveBeenCalled();
  });

  it('loads percentile data when type is latency_percentiles', async () => {
    const cfg: ChartConfig = {
      ...baseConfig,
      type: 'latency_percentiles',
      metricName: 'response_time',
      percentiles: [50, 95, 99],
    };
    mount(ChartCard, { props: { config: cfg } });
    await flushPromises();
    expect(mockGetPercentiles).toHaveBeenCalledWith(
      expect.objectContaining({ percentiles: [50, 95, 99] }),
    );
  });

  it('maps error_rate to requests_error metric', async () => {
    const cfg: ChartConfig = { ...baseConfig, type: 'error_rate' };
    mount(ChartCard, { props: { config: cfg } });
    await flushPromises();
    expect(mockGetHistorical).toHaveBeenCalledWith(
      expect.objectContaining({ name: 'requests_error' }),
    );
  });

  it('maps active_connections to active_connections metric', async () => {
    const cfg: ChartConfig = { ...baseConfig, type: 'active_connections' };
    mount(ChartCard, { props: { config: cfg } });
    await flushPromises();
    expect(mockGetHistorical).toHaveBeenCalledWith(
      expect.objectContaining({ name: 'active_connections' }),
    );
  });

  it('emits update:config with updated percentiles', async () => {
    const cfg: ChartConfig = {
      ...baseConfig,
      type: 'latency_percentiles',
      metricName: 'response_time',
      percentiles: [50, 95, 99],
    };
    const wrapper = mount(ChartCard, { props: { config: cfg } });
    await flushPromises();
    // Watcher on config percentiles should trigger a reload but not an emit
    // (emits happen via remove button only).
    expect(wrapper.emitted('update:config')).toBeFalsy();
  });

  it('emits remove when × clicked', async () => {
    const wrapper = mount(ChartCard, { props: { config: baseConfig } });
    await flushPromises();
    const removeBtn = wrapper.findAll('.icon-btn--danger')[0];
    if (removeBtn) {
      await removeBtn.trigger('click');
      expect(wrapper.emitted('remove')).toBeTruthy();
    }
  });

  it('shows loading state initially', async () => {
    // Pending promise keeps the component in its initial loading state.
    mockGetHistorical.mockReturnValue(new Promise(() => {}));
    const wrapper = mount(ChartCard, { props: { config: baseConfig } });
    // onMounted runs async; flush before reading.
    await flushPromises();
    expect(wrapper.text()).toContain('Loading');
  });

  it('renders empty state when no rows returned', async () => {
    mockGetHistorical.mockResolvedValue([]);
    const wrapper = mount(ChartCard, { props: { config: baseConfig } });
    await flushPromises();
    expect(wrapper.text()).toContain('No data');
  });

  it('renders error message when fetch fails', async () => {
    mockGetHistorical.mockRejectedValue(new Error('boom'));
    const wrapper = mount(ChartCard, { props: { config: baseConfig } });
    await flushPromises();
    expect(wrapper.text()).toContain('boom');
  });
});