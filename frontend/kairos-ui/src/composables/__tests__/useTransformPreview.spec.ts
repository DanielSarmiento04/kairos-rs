import { describe, it, expect } from 'vitest';
import { useTransformPreview, type PreviewInput } from '../useTransformPreview';
import type {
  HeaderTransformation,
  QueryTransformation,
  RequestTransformation,
  ResponseTransformation,
} from '../../types';

const baseInput: PreviewInput = {
  method: 'GET',
  path: '/api/v1/users/123',
  query: { debug: '1', region: 'us' },
  headers: { Authorization: 'Bearer xxx', Accept: 'application/json' },
};

describe('useTransformPreview', () => {
  const { preview, applyHeaderRule, applyQueryRule } = useTransformPreview();

  describe('applyHeaderRule', () => {
    it('adds header when absent', () => {
      const out: Record<string, string> = {};
      applyHeaderRule(out, { action: 'add', name: 'X-Test', value: '1' });
      expect(out['X-Test']).toBe('1');
    });

    it('does not override existing header on add', () => {
      const out: Record<string, string> = { 'X-Test': 'existing' };
      applyHeaderRule(out, { action: 'add', name: 'X-Test', value: 'new' });
      expect(out['X-Test']).toBe('existing');
    });

    it('set always overrides', () => {
      const out: Record<string, string> = { 'X-Test': 'old' };
      applyHeaderRule(out, { action: 'set', name: 'X-Test', value: 'new' });
      expect(out['X-Test']).toBe('new');
    });

    it('set creates when absent', () => {
      const out: Record<string, string> = {};
      applyHeaderRule(out, { action: 'set', name: 'X-Test', value: 'new' });
      expect(out['X-Test']).toBe('new');
    });

    it('remove deletes header', () => {
      const out: Record<string, string> = { 'X-Test': '1' };
      applyHeaderRule(out, { action: 'remove', name: 'X-Test' });
      expect(out['X-Test']).toBeUndefined();
    });

    it('replace applies regex', () => {
      const out: Record<string, string> = { 'User-Agent': 'demo/1.0' };
      applyHeaderRule(out, {
        action: 'replace',
        name: 'User-Agent',
        pattern: '(\\d+\\.\\d+)',
        replacement: 'v$1',
      });
      expect(out['User-Agent']).toBe('demo/v1.0');
    });

    it('replace skips on invalid regex', () => {
      const out: Record<string, string> = { 'User-Agent': 'demo/1.0' };
      applyHeaderRule(out, {
        action: 'replace',
        name: 'User-Agent',
        pattern: '[unclosed',
        replacement: 'x',
      });
      expect(out['User-Agent']).toBe('demo/1.0');
    });

    it('replace skips when header absent', () => {
      const out: Record<string, string> = {};
      applyHeaderRule(out, {
        action: 'replace',
        name: 'Missing',
        pattern: 'a',
        replacement: 'b',
      });
      expect(out['Missing']).toBeUndefined();
    });
  });

  describe('applyQueryRule', () => {
    it('adds query param when absent', () => {
      const out: Record<string, string> = {};
      applyQueryRule(out, { action: 'add', name: 'api_key', value: 'secret' });
      expect(out.api_key).toBe('secret');
    });

    it('set overrides', () => {
      const out: Record<string, string> = { api_key: 'old' };
      applyQueryRule(out, { action: 'set', name: 'api_key', value: 'new' });
      expect(out.api_key).toBe('new');
    });

    it('remove deletes', () => {
      const out: Record<string, string> = { debug: '1' };
      applyQueryRule(out, { action: 'remove', name: 'debug' });
      expect(out.debug).toBeUndefined();
    });
  });

  describe('preview (full flow)', () => {
    it('returns unchanged result for null transformations', () => {
      const result = preview(baseInput, null, null);
      expect(result.request.path).toBe(baseInput.path);
      expect(result.request.headers).toEqual(baseInput.headers);
      expect(result.request.query).toEqual(baseInput.query);
      expect(result.response.status).toBe(200);
      expect(result.changes).toEqual([]);
    });

    it('applies request headers + query + path rewrite', () => {
      const reqT: RequestTransformation = {
        headers: [
          { action: 'remove', name: 'Authorization' },
          { action: 'add', name: 'X-Forwarded-By', value: 'kairos' },
        ],
        path: { pattern: '^/api/v1/(.+)$', replacement: '/$1' },
        query_params: [{ action: 'remove', name: 'debug' }],
      };
      const result = preview(baseInput, reqT, null);
      expect(result.request.path).toBe('/users/123');
      expect(result.request.headers).not.toHaveProperty('Authorization');
      expect(result.request.headers['X-Forwarded-By']).toBe('kairos');
      expect(result.request.query).not.toHaveProperty('debug');
      expect(result.changes.length).toBeGreaterThan(0);
      expect(result.changes.some((c) => c.field === 'path')).toBe(true);
      expect(result.changes.some((c) => c.field === 'header:Authorization')).toBe(true);
    });

    it('applies response status code mapping', () => {
      const resT: ResponseTransformation = {
        status_code_mappings: [{ from: 404, to: 200 }],
        headers: [],
      };
      const result = preview(baseInput, null, resT, 404);
      expect(result.response.status).toBe(200);
      const statusChange = result.changes.find((c) => c.field === 'response:status');
      expect(statusChange).toBeDefined();
      expect(statusChange?.before).toBe('404');
      expect(statusChange?.after).toBe('200');
    });

    it('skips invalid path regex without crashing', () => {
      const reqT: RequestTransformation = {
        headers: [],
        path: { pattern: '[unclosed', replacement: '/' },
        query_params: [],
      };
      const result = preview(baseInput, reqT, null);
      expect(result.request.path).toBe(baseInput.path);
    });

    it('handles undefined transformation configs', () => {
      const result = preview(baseInput, undefined, undefined);
      expect(result.request.path).toBe(baseInput.path);
    });

    it('diff includes removed query params with (removed) marker', () => {
      const reqT: RequestTransformation = {
        headers: [],
        path: null,
        query_params: [{ action: 'remove', name: 'debug' }],
      };
      const result = preview(baseInput, reqT, null);
      const debugChange = result.changes.find((c) => c.field === 'query:debug');
      expect(debugChange?.after).toBe('(removed)');
    });
  });
});
