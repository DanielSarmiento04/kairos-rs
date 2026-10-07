import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import PathRuleEditor from '../PathRuleEditor.vue';

describe('PathRuleEditor', () => {
  it('shows empty state when modelValue is null', () => {
    const wrapper = mount(PathRuleEditor, { props: { modelValue: null } });
    expect(wrapper.text()).toContain('No path rewriting configured');
    expect(wrapper.find('.path-fields').exists()).toBe(false);
  });

  it('emits new PathTransformation when Enable clicked', async () => {
    const wrapper = mount(PathRuleEditor, { props: { modelValue: null } });
    await wrapper.find('.btn-sub-action').trigger('click');
    const emitted = wrapper.emitted('update:modelValue');
    expect(emitted).toBeTruthy();
    expect(emitted![0]![0]).toEqual({ pattern: '', replacement: '' });
  });

  it('renders pattern + replacement inputs when enabled', () => {
    const wrapper = mount(PathRuleEditor, {
      props: { modelValue: { pattern: '^/api/v1/(.+)$', replacement: '/$1' } },
    });
    expect(wrapper.findAll('input').length).toBe(2);
    expect(wrapper.text()).not.toContain('No path rewriting');
  });

  it('emits update:modelValue on pattern change', async () => {
    const wrapper = mount(PathRuleEditor, {
      props: { modelValue: { pattern: '', replacement: '' } },
    });
    const inputs = wrapper.findAll('input');
    await inputs[0]!.setValue('^/v2/(.+)$');
    const emits = wrapper.emitted('update:modelValue');
    expect(emits).toBeTruthy();
    expect(emits![0]![0]).toMatchObject({ pattern: '^/v2/(.+)$' });
  });

  it('emits null when Disable clicked', async () => {
    const wrapper = mount(PathRuleEditor, {
      props: { modelValue: { pattern: '^/api/v1/(.+)$', replacement: '/$1' } },
    });
    await wrapper.find('.btn-row-remove').trigger('click');
    const emitted = wrapper.emitted('update:modelValue');
    expect(emitted).toBeTruthy();
    expect(emitted![emitted!.length - 1]![0]).toBeNull();
  });

  it('marks invalid pattern with .invalid class', () => {
    const wrapper = mount(PathRuleEditor, {
      props: { modelValue: { pattern: '[unclosed', replacement: 'x' } },
    });
    expect(wrapper.find('input').classes()).toContain('invalid');
  });
});
