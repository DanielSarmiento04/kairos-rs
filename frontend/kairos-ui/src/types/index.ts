export interface JwtSettings {
  secret: string;
  issuer?: string | null;
  audience?: string | null;
  required_claims: string[];
}

export interface RateLimitConfig {
  enabled: boolean;
  requests_per_second: number;
  burst_size: number;
  strategy: string;
}

export interface CorsConfig {
  allowed_origins: string[];
  allowed_methods: string[];
  allowed_headers: string[];
  max_age: number;
}

export interface MetricsConfig {
  enabled: boolean;
  port: number;
  path: string;
}

export interface ServerConfig {
  host: string;
  port: number;
  workers: number;
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
}

export interface RouteBackend {
  host: string;
  port: number;
  weight?: number;
}

export interface AiPolicy {
  enabled: boolean;
  strategy: string;
}

export interface RetryConfig {
  max_retries: number;
  initial_backoff_ms: number;
  max_backoff_ms: number;
}

export interface Router {
  protocol?: string | null;
  host?: string | null;
  port?: number | null;
  backends?: RouteBackend[] | null;
  external_path: string;
  internal_path: string;
  methods: string[];
  auth_required: boolean;
  ai_policy?: AiPolicy | null;
  retry?: RetryConfig | null;
}

export interface TelemetryMetric {
  latency: number;
  tokens?: number;
  cost?: number;
  method: string;
  status: number;
  path: string;
  timestamp: string;
}
