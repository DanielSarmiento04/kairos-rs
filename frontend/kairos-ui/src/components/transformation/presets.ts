import type {
  HeaderTransformation,
  PathTransformation,
  QueryTransformation,
  RequestTransformation,
  ResponseTransformation,
} from '@/types';

export interface TransformationPreset {
  id: string;
  name: string;
  description: string;
  scope: 'request' | 'response' | 'both';
  request?: Partial<RequestTransformation>;
  response?: Partial<ResponseTransformation>;
}

/**
 * Common transformation templates users can apply with one click.
 * Mirror of backend capabilities in crates/kairos-rs/src/middleware/transform.rs.
 */
export const TRANSFORMATION_PRESETS: TransformationPreset[] = [
  {
    id: 'strip-auth',
    name: 'Strip Authorization header',
    description: 'Remove Authorization header before forwarding to the backend.',
    scope: 'request',
    request: {
      headers: [
        {
          action: 'remove',
          name: 'Authorization',
          value: null,
          pattern: null,
          replacement: null,
        },
      ],
    },
  },
  {
    id: 'add-gateway-tag',
    name: 'Add X-Forwarded-By tag',
    description: 'Tag every forwarded request with X-Forwarded-By: kairos-gateway.',
    scope: 'request',
    request: {
      headers: [
        {
          action: 'add',
          name: 'X-Forwarded-By',
          value: 'kairos-gateway',
          pattern: null,
          replacement: null,
        },
      ],
    },
  },
  {
    id: 'api-version-strip',
    name: 'Rewrite /api/v1/* → /*',
    description: 'Strip the /api/v1 prefix from incoming paths.',
    scope: 'request',
    request: {
      path: { pattern: '^/api/v1/(.+)$', replacement: '/$1' },
    },
  },
  {
    id: 'add-api-key',
    name: 'Inject api_key query parameter',
    description: 'Append api_key=*** to every request.',
    scope: 'request',
    request: {
      query_params: [
        { action: 'add', name: 'api_key', value: '${API_KEY}' },
      ],
    },
  },
  {
    id: 'mask-server-header',
    name: 'Remove Server header (response)',
    description: 'Hide backend identity from clients.',
    scope: 'response',
    response: {
      headers: [
        {
          action: 'remove',
          name: 'Server',
          value: null,
          pattern: null,
          replacement: null,
        },
      ],
    },
  },
  {
    id: 'health-404-to-200',
    name: 'Map 404 → 200 for health checks',
    description: 'For paths matching /health, treat 404 as success.',
    scope: 'response',
    response: {
      status_code_mappings: [
        { from: 404, to: 200, condition: "path == '/health'" },
      ],
    },
  },
];

/** Filter presets that match a given scope (or `both`). */
export function presetsForScope(scope: 'request' | 'response'): TransformationPreset[] {
  return TRANSFORMATION_PRESETS.filter((p) => p.scope === scope || p.scope === 'both');
}