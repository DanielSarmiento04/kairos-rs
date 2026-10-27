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

export interface AiProviderConfig {
  provider: string;
  model: string;
  api_key?: string | null;
}

export interface AiPolicy {
  enabled: boolean;
  strategy: AiRoutingStrategy;
  provider?: string | null;
  fallback_backend_index?: number | null;
  fallback_providers?: AiProviderConfig[];
  streaming?: boolean | null;
  semantic_cache?: SemanticCacheConfig | null;
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

export interface SemanticCacheConfig {
  enabled: boolean;
  similarity_threshold: number;
  ttl_seconds: number;
  embedding_provider: string;
  embedding_model: string;
  max_entries: number;
}

export interface SemanticCacheStats {
  hits: number;
  misses: number;
  hit_ratio: number;
  current_entries: number;
  max_entries: number;
  evictions: number;
}

export interface AiSettings {
  provider: string;
  model: string;
  api_key?: string | null;
  fallback_providers?: AiProviderConfig[];
  semantic_cache?: SemanticCacheConfig | null;
}

export interface RedisConfig {
  enabled: boolean;
  url: string;
  pool_size: number;
  timeout_ms: number;
  fallback_to_memory: boolean;
}

export interface RedisHealthResponse {
  success: boolean;
  connected: boolean;
  latency_ms?: number;
  url?: string;
  pool_size?: number;
  fallback_to_memory?: boolean;
  message?: string;
}

export interface AdminAuthConfig {
  enabled: boolean;
  api_keys: string[];
  header_name: string;
}

export interface AdminAuthVerifyResponse {
  success: boolean;
  auth_required: boolean;
  authenticated: boolean;
}

export interface Settings {
  version: number;
  jwt?: JwtSettings | null;
  rate_limit?: RateLimitConfig | null;
  cors?: CorsConfig | null;
  metrics?: MetricsConfig | null;
  server?: ServerConfig | null;
  ai?: AiSettings | null;
  admin_auth?: AdminAuthConfig | null;
  redis?: RedisConfig | null;
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

export interface MetricValueRaw {
  type: 'Counter' | 'Gauge' | 'Histogram';
  value: number;
}

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

export interface CacheStats {
  hits: number;
  misses: number;
  hit_ratio: number;
  evictions: number;
  current_entries: number;
  max_entries: number;
}

export interface CacheStatsResponse {
  success: boolean;
  data?: CacheStats;
  semantic?: SemanticCacheStats;
  message?: string;
}

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
  start: string;
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
  metricName?: string;
  percentiles?: number[];
  timeRange: TimeRangePreset;
  aggregation: AggregationInterval | 'raw';
  gridX: number;
  gridY: number;
  gridW: number;
  gridH: number;
  title?: string;
  routeFilter?: string[];
}

export interface DashboardLayout {
  id: string;
  name: string;
  charts: ChartConfig[];
  createdAt: string;
  updatedAt: string;
}
