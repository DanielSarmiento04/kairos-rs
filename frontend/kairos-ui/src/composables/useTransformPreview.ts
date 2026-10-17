import type {
  HeaderTransformation,
  QueryTransformation,
  RequestTransformation,
  ResponseTransformation,
} from '@/types';

export interface PreviewInput {
  method: string;
  path: string;
  query: Record<string, string>;
  headers: Record<string, string>;
  body?: string;
}

export interface PreviewChange {
  field: string;
  before: string;
  after: string;
}

export interface PreviewResult {
  request: {
    path: string;
    query: Record<string, string>;
    headers: Record<string, string>;
  };
  response: {
    status: number;
    headers: Record<string, string>;
  };
  changes: PreviewChange[];
}

/**
 * Client-side transformation preview.
 *
 * Mirrors `crates/kairos-rs/src/middleware/transform.rs` behavior so users can
 * see what will happen to a sample request/response before saving a route.
 *
 * All operations are pure (no network) and side-effect free.
 */
export function useTransformPreview() {
  /**
   * Apply a single header rule to a header map in place.
   * Returns a list of changes the rule introduced (for UI diff highlighting).
   */
  function applyHeaderRule(
    out: Record<string, string>,
    rule: HeaderTransformation,
  ): PreviewChange[] {
    const changes: PreviewChange[] = [];
    switch (rule.action) {
      case 'add': {
        if (rule.value != null && out[rule.name] === undefined) {
          out[rule.name] = rule.value;
          changes.push({ field: `header:${rule.name}`, before: '(absent)', after: rule.value });
        }
        break;
      }
      case 'set': {
        if (rule.value != null) {
          const before = out[rule.name] ?? '(absent)';
          out[rule.name] = rule.value;
          if (before !== rule.value) {
            changes.push({ field: `header:${rule.name}`, before, after: rule.value });
          }
        }
        break;
      }
      case 'remove': {
        if (rule.name in out) {
          const before = out[rule.name] ?? '';
          delete out[rule.name];
          changes.push({ field: `header:${rule.name}`, before, after: '(removed)' });
        }
        break;
      }
      case 'replace': {
        if (rule.pattern && rule.replacement != null && rule.name in out) {
          try {
            const re = new RegExp(rule.pattern);
            const before = out[rule.name] ?? '';
            const after = before.replace(re, rule.replacement);
            if (before !== after) {
              out[rule.name] = after;
              changes.push({ field: `header:${rule.name}`, before, after });
            }
          } catch {
            /* invalid regex — silently skip, UI will surface the error */
          }
        }
        break;
      }
    }
    return changes;
  }

  /** Apply a single query rule in place. Same semantics as headers minus `replace`. */
  function applyQueryRule(
    out: Record<string, string>,
    rule: QueryTransformation,
  ): PreviewChange[] {
    const changes: PreviewChange[] = [];
    switch (rule.action) {
      case 'add': {
        if (rule.value != null && out[rule.name] === undefined) {
          out[rule.name] = rule.value;
          changes.push({ field: `query:${rule.name}`, before: '(absent)', after: rule.value });
        }
        break;
      }
      case 'set': {
        if (rule.value != null) {
          const before = out[rule.name] ?? '(absent)';
          out[rule.name] = rule.value;
          if (before !== rule.value) {
            changes.push({ field: `query:${rule.name}`, before, after: rule.value });
          }
        }
        break;
      }
      case 'remove': {
        if (rule.name in out) {
          const before = out[rule.name] ?? '';
          delete out[rule.name];
          changes.push({ field: `query:${rule.name}`, before, after: '(removed)' });
        }
        break;
      }
    }
    return changes;
  }

  /**
   * Run a full preview against the given input + request/response transformation configs.
   *
   * @param simulatedStatus  Status code to assume for the simulated backend response
   *                         (so status_code_mappings have something to map against).
   */
  function preview(
    input: PreviewInput,
    reqT: RequestTransformation | null | undefined,
    resT: ResponseTransformation | null | undefined,
    simulatedStatus: number = 200,
  ): PreviewResult {
    const changes: PreviewChange[] = [];
    const headers: Record<string, string> = { ...input.headers };
    const query: Record<string, string> = { ...input.query };
    let path = input.path;

    // request transformations
    reqT?.headers?.forEach((r) => applyHeaderRule(headers, r).forEach((c) => changes.push(c)));
    reqT?.query_params?.forEach((r) => applyQueryRule(query, r).forEach((c) => changes.push(c)));

    if (reqT?.path) {
      try {
        const re = new RegExp(reqT.path.pattern);
        const after = path.replace(re, reqT.path.replacement);
        if (path !== after) {
          changes.push({ field: 'path', before: path, after });
        }
        path = after;
      } catch {
        /* invalid path regex — skip */
      }
    }

    // response transformations
    const resHeaders: Record<string, string> = { 'content-type': 'application/json' };
    let status = simulatedStatus;
    resT?.headers?.forEach((r) => applyHeaderRule(resHeaders, r));
    resT?.status_code_mappings?.forEach((m) => {
      if (status === m.from) {
        changes.push({ field: 'response:status', before: String(status), after: String(m.to) });
        status = m.to;
      }
    });

    return {
      request: { path, query, headers },
      response: { status, headers: resHeaders },
      changes,
    };
  }

  return { preview, applyHeaderRule, applyQueryRule };
}
