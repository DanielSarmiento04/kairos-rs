export type Protocol = 'http' | 'websocket' | 'ftp' | 'dns';

export type LoadBalancingStrategy =
  | 'round_robin'
  | 'least_connections'
  | 'random'
  | 'weighted'
  | 'ip_hash';

export interface RouteBackend {
  host: string;
  port: number;
  weight: number;
  health_check_path?: string | null;
}

export interface RetryConfig {
  max_retries: number;
  initial_backoff_ms: number;
  max_backoff_ms: number;
  backoff_multiplier: number;
  retry_on_status_codes: number[];
  retry_on_connection_error: boolean;
}

export type AiRoutingStrategy =
  | { content_analysis: { model?: string | null } }
  | { latency_prediction: Record<string, never> }
  | { anomaly_detection: Record<string, never> }
  | string;

export interface AiPolicy {
  enabled: boolean;
  strategy: AiRoutingStrategy;
  provider?: string | null;
  fallback_backend_index?: number | null;
}

// ─── Transformation (mirrors crates/kairos-rs/src/middleware/transform.rs) ───

export type TransformAction = 'add' | 'set' | 'remove' | 'replace';

export interface HeaderTransformation {
  action: TransformAction;
  name: string;
  /** Required for action=add / set */
  value?: string | null;
  /** Only used for action=replace */
  pattern?: string | null;
  /** Only used for action=replace */
  replacement?: string | null;
}

export interface PathTransformation {
  /** Regex pattern, e.g. ^/api/v1/(.+)$ */
  pattern: string;
  /** Replacement template with capture groups, e.g. /$1 */
  replacement: string;
}

export interface QueryTransformation {
  action: TransformAction; // 'add' | 'set' | 'remove'
  name: string;
  value?: string | null;
}

export interface RequestTransformation {
  headers?: HeaderTransformation[] | null;
  path?: PathTransformation | null;
  query_params?: QueryTransformation[] | null;
}

export interface StatusCodeMapping {
  from: number; // HTTP status code (u16 in Rust, JSON number)
  to: number;
  /** Optional condition expression (future); backend currently ignores */
  condition?: string | null;
}

export interface ResponseTransformation {
  headers?: HeaderTransformation[] | null;
  status_code_mappings?: StatusCodeMapping[] | null;
}

export interface Router {
  external_path: string;
  internal_path: string;
  methods: string[];
  auth_required: boolean;
  protocol?: Protocol | null;
  host?: string | null;
  port?: number | null;
  backends?: RouteBackend[] | null;
  load_balancing_strategy?: LoadBalancingStrategy;
  retry?: RetryConfig | null;
  request_transformation?: RequestTransformation | null;
  response_transformation?: ResponseTransformation | null;
  ai_policy?: AiPolicy | null;
}

export interface RouteResponse {
  success: boolean;
  message: string;
  route?: Router | null;
  routes?: Router[] | null;
}

export interface ValidateRouteRequest {
  route: Router;
}

export interface ValidateRouteResponse {
  valid: boolean;
  error?: string | null;
  warnings?: string[] | null;
}

export interface JwtSettings {
  secret: string;
  issuer?: string | null;
  audience?: string | null;
  required_claims: string[];
}

export interface RateLimitConfig {
  enabled?: boolean;
  strategy: 'FixedWindow' | 'SlidingWindow' | 'TokenBucket' | string;
  window_type?: 'Second' | 'Minute' | 'Hour' | string;
  max_requests: number;
  requests_per_second?: number;
  burst_size?: number;
  redis_url?: string | null;
}

export interface CorsConfig {
  allowed_origins: string[];
  allowed_methods: string[];
  allowed_headers: string[];
  allow_credentials?: boolean;
  max_age?: number;
}

export interface MetricsConfig {
  endpoint: string;
  enable_per_route_metrics: boolean;
  enabled?: boolean;
  port?: number;
  path?: string;
}

export interface ServerConfig {
  host: string;
  port: number;
  workers: number;
  keep_alive?: number;
}

export interface AiSettings {
  provider: string;
  model: string;
  api_key?: string | null;
}

export interface Settings {
  version: number;
  jwt?: JwtSettings | null;
  rate_limit?: RateLimitConfig | null;
  cors?: CorsConfig | null;
  metrics?: MetricsConfig | null;
  server?: ServerConfig | null;
  ai?: AiSettings | null;
  routers: Router[];
}

export interface AdminMetricsSnapshot {
  requests_total: number;
  active_connections: number;
  requests_error: number;
  success_rate: number;
  uptime: number;
  peak_connections: number;
}

export interface HistoricalMetricPoint {
  timestamp: string;
  value: number;
}

/** Raw metric value wrapper returned by the backend's serde tagged enum.
 *  Mirrors `MetricValue` in `crates/kairos-rs/src/services/metrics_store.rs`. */
export interface MetricValueRaw {
  type: 'Counter' | 'Gauge' | 'Histogram';
  value: number;
}

/** Raw point as returned by `/api/metrics/history` in raw (non-aggregated) mode.
 *  `value` may be a plain number (older configs) or a `MetricValueRaw`
 *  wrapper. Normalize via `apiService.getHistoricalMetrics` before passing
 *  to chart components. */
export interface RawHistoricalMetricPoint {
  timestamp: string;
  value: number | MetricValueRaw;
}

export interface AggregatedMetricPoint {
  timestamp: string;
  min: number;
  max: number;
  avg: number;
  count: number;
}

export type AggregationInterval = 'one_minute' | 'five_minutes' | 'one_hour' | 'one_day';

export interface HistoricalMetricsQuery {
  name: string;
  start: string;
  end: string;
  interval?: AggregationInterval;
}

export interface HealthStatus {
  status: string;
  version?: string;
  timestamp?: string;
  uptime?: number;
}

export interface TelemetryLog {
  id: string;
  timestamp: string;
  method: string;
  path: string;
  status: number;
  latency_ms: number;
  protocol: Protocol;
  tokens?: number;
  cost?: number;
  backend?: string;
  ai_routed?: boolean;
  provider?: string;
}

export interface PlaygroundRequest {
  method: string;
  path: string;
  headers: Record<string, string>;
  body?: string;
}

export interface PlaygroundResponse {
  status: number;
  statusText: string;
  latency_ms: number;
  headers: Record<string, string>;
  body: string;
}

// ─── Dashboards & Time-series Charts ──────────────────────────────────────────
// Mirrors `crates/kairos-rs/src/services/percentile.rs` and the new
// `/api/metrics/latency/percentiles` endpoint.

/** One time-windowed percentile sample. `percentiles` is keyed by the
 *  percentile value as a string with trailing zeros stripped
 *  (e.g. `"50"`, `"95"`, `"99.9"`). */
export interface PercentilePoint {
  timestamp: string;
  percentiles: Record<string, number>;
}

export interface LatencyPercentileQuery {
  name: string;
  start: string;
  end: string;
  interval: AggregationInterval;
  percentiles?: number[];
}

export type TimeRangePreset =
  | '5m'
  | '15m'
  | '1h'
  | '6h'
  | '24h'
  | '7d'
  | 'custom';

export interface CustomTimeRange {
  /** ISO 8601 timestamp. */
  start: string;
  /** ISO 8601 timestamp. */
  end: string;
}

export type ChartType =
  | 'requests'
  | 'error_rate'
  | 'latency_percentiles'
  | 'active_connections'
  | 'custom';

export interface ChartConfig {
  id: string;
  type: ChartType;
  /** Metric name for `requests` / `active_connections` / `custom`. */
  metricName?: string;
  /** Required when `type === 'latency_percentiles'`. */
  percentiles?: number[];
  timeRange: TimeRangePreset;
  /** Use `'raw'` for raw points or any `AggregationInterval` for grouped. */
  aggregation: AggregationInterval | 'raw';
  /** 12-column CSS grid: 0-11 horizontal position. */
  gridX: number;
  /** Row index (auto-stacks by default). */
  gridY: number;
  /** Width in grid columns (1-12). */
  gridW: number;
  /** Height in grid rows. */
  gridH: number;
  /** Optional human-readable title; falls back to metricName / type. */
  title?: string;
  /** Optional route filter for per-route breakdown charts. */
  routeFilter?: string[];
}

export interface DashboardLayout {
  id: string;
  name: string;
  charts: ChartConfig[];
  /** ISO 8601. */
  createdAt: string;
  /** ISO 8601. */
  updatedAt: string;
}
