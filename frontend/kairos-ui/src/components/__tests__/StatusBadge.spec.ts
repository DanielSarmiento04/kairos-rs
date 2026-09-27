import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import StatusBadge from '../StatusBadge.vue';

describe('StatusBadge', () => {
  it('renders method badges with correct classes', () => {
    const wrapper = mount(StatusBadge, {
      props: { type: 'method', value: 'GET' },
    });
    expect(wrapper.text()).toBe('GET');
    expect(wrapper.classes()).toContain('method-get');
  });

  it('renders status code badges with correct classes', () => {
    const wrapper200 = mount(StatusBadge, {
      props: { type: 'status', value: 200 },
    });
    expect(wrapper200.classes()).toContain('status-2xx');

    const wrapper502 = mount(StatusBadge, {
      props: { type: 'status', value: 502 },
    });
    expect(wrapper502.classes()).toContain('status-5xx');
  });

  it('renders protocol badges properly', () => {
    const wrapper = mount(StatusBadge, {
      props: { type: 'protocol', value: 'websocket' },
    });
    expect(wrapper.text()).toBe('WEBSOCKET');
    expect(wrapper.classes()).toContain('protocol-websocket');
  });

  it('renders auth badges properly', () => {
    const wrapperAuth = mount(StatusBadge, {
      props: { type: 'auth', value: true },
    });
    expect(wrapperAuth.text()).toBe('🔒 JWT');
    expect(wrapperAuth.classes()).toContain('auth-required');

    const wrapperPublic = mount(StatusBadge, {
      props: { type: 'auth', value: false },
    });
    expect(wrapperPublic.text()).toBe('🔓 Public');
    expect(wrapperPublic.classes()).toContain('auth-public');
  });
});
