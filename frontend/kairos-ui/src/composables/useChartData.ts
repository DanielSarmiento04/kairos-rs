/**
 * `useChartData` — fetch time-series and percentile data, build query
 * parameters, and serialize results to CSV.
 *
 * Designed to be used both by the new `/dashboards` view and by ad-hoc
 * tooling (e.g. a future CLI/REPL). Pure functions are easy to unit-test.
 */
import type {
  AggregatedMetricPoint,
  AggregationInterval,
  CustomTimeRange,
  HistoricalMetricPoint,
  HistoricalMetricsQuery,
  LatencyPercentileQuery,
  PercentilePoint,
  TimeRangePreset,
} from '@/types';
import { apiService } from '@/services/api';

const RANGE_TO_MS: Record<Exclude<TimeRangePreset, 'custom'>, number> = {
  '5m': 5 * 60 * 1000,
  '15m': 15 * 60 * 1000,
  '1h': 60 * 60 * 1000,
  '6h': 6 * 60 * 60 * 1000,
  '24h': 24 * 60 * 60 * 1000,
  '7d': 7 * 24 * 60 * 60 * 1000,
};

/** Compute the `[start, end]` ISO timestamps for a preset or custom range. */
export function resolveTimeRange(
  range: TimeRangePreset,
  custom?: CustomTimeRange,
  now: Date = new Date(),
): { start: string; end: string } {
  if (range === 'custom') {
    if (!custom) throw new Error('Custom time range requires { start, end }');
    return { start: custom.start, end: custom.end };
  }
  const ms = RANGE_TO_MS[range];
  const end = now;
  const start = new Date(end.getTime() - ms);
  return { start: start.toISOString(), end: end.toISOString() };
}

export interface UseChartDataApi {
  resolveTimeRange: typeof resolveTimeRange;
  fetchHistorical: (
    metricName: string,
    range: TimeRangePreset,
    aggregation: AggregationInterval | 'raw',
    custom?: CustomTimeRange,
  ) => Promise<(HistoricalMetricPoint | AggregatedMetricPoint)[]>;
  fetchPercentiles: (
    metricName: string,
    range: TimeRangePreset,
    aggregation: AggregationInterval,
    percentiles: number[],
    custom?: CustomTimeRange,
  ) => Promise<PercentilePoint[]>;
  toCsvHistorical: (
    rows: (HistoricalMetricPoint | AggregatedMetricPoint)[],
    filename: string,
  ) => void;
  toCsvPercentiles: (rows: PercentilePoint[], filename: string) => void;
  /** Build a `HistoricalMetricsQuery` object without firing the request. */
  buildHistoricalQuery: (
    metricName: string,
    range: TimeRangePreset,
    aggregation: AggregationInterval | 'raw',
    custom?: CustomTimeRange,
  ) => HistoricalMetricsQuery;
  /** Build a `LatencyPercentileQuery` object without firing the request. */
  buildPercentileQuery: (
    metricName: string,
    range: TimeRangePreset,
    aggregation: AggregationInterval,
    percentiles: number[],
    custom?: CustomTimeRange,
  ) => LatencyPercentileQuery;
}

export function useChartData(): UseChartDataApi {
  function buildHistoricalQuery(
    metricName: string,
    range: TimeRangePreset,
    aggregation: AggregationInterval | 'raw',
    custom?: CustomTimeRange,
  ): HistoricalMetricsQuery {
    const { start, end } = resolveTimeRange(range, custom);
    const query: HistoricalMetricsQuery = {
      name: metricName,
      start,
      end,
    };
    if (aggregation !== 'raw') query.interval = aggregation;
    return query;
  }

  function buildPercentileQuery(
    metricName: string,
    range: TimeRangePreset,
    aggregation: AggregationInterval,
    percentiles: number[],
    custom?: CustomTimeRange,
  ): LatencyPercentileQuery {
    const { start, end } = resolveTimeRange(range, custom);
    return {
      name: metricName,
      start,
      end,
      interval: aggregation,
      percentiles: percentiles.length > 0 ? percentiles : undefined,
    };
  }

  function fetchHistorical(
    metricName: string,
    range: TimeRangePreset,
    aggregation: AggregationInterval | 'raw',
    custom?: CustomTimeRange,
  ) {
    const query = buildHistoricalQuery(metricName, range, aggregation, custom);
    return apiService.getHistoricalMetrics(query);
  }

  function fetchPercentiles(
    metricName: string,
    range: TimeRangePreset,
    aggregation: AggregationInterval,
    percentiles: number[],
    custom?: CustomTimeRange,
  ) {
    const query = buildPercentileQuery(metricName, range, aggregation, percentiles, custom);
    return apiService.getLatencyPercentiles(query);
  }

  function toCsvHistorical(
    rows: (HistoricalMetricPoint | AggregatedMetricPoint)[],
    filename: string,
  ): void {
    const header = 'timestamp,value,min,max,avg,count';
    const body = rows
      .map((r) => {
        if ('avg' in r) {
          return [r.timestamp, '', r.min, r.max, r.avg, r.count].join(',');
        }
        return [r.timestamp, r.value, '', '', '', ''].join(',');
      })
      .join('\n');
    download(`${header}\n${body}`, filename);
  }

  function toCsvPercentiles(rows: PercentilePoint[], filename: string): void {
    if (rows.length === 0) {
      download('timestamp,percentile,latency_ms', filename);
      return;
    }
    // Determine column order from the first row to keep it stable.
    const keys = Object.keys(rows[0]!.percentiles);
    const header = ['timestamp', ...keys].join(',');
    const body = rows
      .map((r) =>
        [r.timestamp, ...keys.map((k) => r.percentiles[k] ?? '')].join(','),
      )
      .join('\n');
    download(`${header}\n${body}`, filename);
  }

  function download(content: string, filename: string): void {
    if (typeof document === 'undefined') return;
    const blob = new Blob([content], { type: 'text/csv;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  return {
    resolveTimeRange,
    fetchHistorical,
    fetchPercentiles,
    toCsvHistorical,
    toCsvPercentiles,
    buildHistoricalQuery,
    buildPercentileQuery,
  };
}