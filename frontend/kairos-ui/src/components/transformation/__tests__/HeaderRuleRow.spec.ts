import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import HeaderRuleRow from '../HeaderRuleRow.vue';
import type { HeaderTransformation } from '../../../types';

const baseRule: HeaderTransformation = {
  action: 'add',
  name: 'X-Test',
  value: '1',
  pattern: null,
  replacement: null,
};

describe('HeaderRuleRow', () => {
  it('renders value field for add action', () => {
    const wrapper = mount(HeaderRuleRow, { props: { modelValue: { ...baseRule } } });
    expect(wrapper.find('.value-input').exists()).toBe(true);
    expect(wrapper.find('.pattern-input').exists()).toBe(false);
  });

  it('renders pattern + replacement for replace action', async () => {
    const wrapper = mount(HeaderRuleRow, {
      props: {
        modelValue: {
          action: 'replace',
          name: 'User-Agent',
          pattern: 'a',
          replacement: 'b',
        },
      },
    });
    expect(wrapper.find('.pattern-input').exists()).toBe(true);
    expect(wrapper.find('.replacement-input').exists()).toBe(true);
    expect(wrapper.find('.value-input').exists()).toBe(false);
  });

  it('hides value field for remove action', () => {
    const wrapper = mount(HeaderRuleRow, {
      props: { modelValue: { action: 'remove', name: 'Authorization' } },
    });
    expect(wrapper.find('.value-input').exists()).toBe(false);
  });

  it('emits update:modelValue on name change', async () => {
    const wrapper = mount(HeaderRuleRow, { props: { modelValue: { ...baseRule } } });
    await wrapper.find('.name-input').setValue('X-New');
    const emitted = wrapper.emitted('update:modelValue');
    expect(emitted).toBeTruthy();
    expect(emitted![0]![0]).toMatchObject({ name: 'X-New' });
  });

  it('emits remove event when trash clicked', async () => {
    const wrapper = mount(HeaderRuleRow, { props: { modelValue: { ...baseRule } } });
    await wrapper.find('.btn-row-remove').trigger('click');
    expect(wrapper.emitted('remove')).toBeTruthy();
  });

  it('marks invalid pattern with .invalid class', () => {
    const wrapper = mount(HeaderRuleRow, {
      props: {
        modelValue: { action: 'replace', name: 'X', pattern: '[unclosed', replacement: 'y' },
      },
    });
    expect(wrapper.find('.pattern-input').classes()).toContain('invalid');
  });

  it('does not render remove button when readonly', () => {
    const wrapper = mount(HeaderRuleRow, {
      props: { modelValue: { ...baseRule }, readonly: true },
    });
    expect(wrapper.find('.btn-row-remove').exists()).toBe(false);
  });
});
