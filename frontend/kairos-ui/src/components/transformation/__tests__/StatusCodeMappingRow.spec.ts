import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import StatusCodeMappingRow from '../StatusCodeMappingRow.vue';
import type { StatusCodeMapping } from '../../../types';

const baseMapping: StatusCodeMapping = { from: 404, to: 200, condition: null };

describe('StatusCodeMappingRow', () => {
  it('renders from and to inputs', () => {
    const wrapper = mount(StatusCodeMappingRow, { props: { modelValue: { ...baseMapping } } });
    const numberInputs = wrapper.findAll('input[type="number"]');
    expect(numberInputs.length).toBe(2);
  });

  it('emits update:modelValue on from change', async () => {
    const wrapper = mount(StatusCodeMappingRow, { props: { modelValue: { ...baseMapping } } });
    const fromInput = wrapper.findAll('input[type="number"]')[0]!;
    await fromInput.setValue('503');
    const emitted = wrapper.emitted('update:modelValue');
    expect(emitted).toBeTruthy();
    expect(emitted![0]![0]).toMatchObject({ from: 503 });
  });

  it('emits update:modelValue on condition change', async () => {
    const wrapper = mount(StatusCodeMappingRow, { props: { modelValue: { ...baseMapping } } });
    const condInput = wrapper.find('input[type="text"]');
    await condInput.setValue("path == '/health'");
    const emitted = wrapper.emitted('update:modelValue');
    expect(emitted).toBeTruthy();
    expect(emitted![0]![0]).toMatchObject({ condition: "path == '/health'" });
  });

  it('emits remove when trash clicked', async () => {
    const wrapper = mount(StatusCodeMappingRow, { props: { modelValue: { ...baseMapping } } });
    await wrapper.find('.btn-row-remove').trigger('click');
    expect(wrapper.emitted('remove')).toBeTruthy();
  });

  it('marks invalid status codes out of range', () => {
    const wrapper = mount(StatusCodeMappingRow, {
      props: { modelValue: { from: 99, to: 200, condition: null } },
    });
    const fromInput = wrapper.findAll('input[type="number"]')[0]!;
    expect(fromInput.classes()).toContain('invalid');
  });
});
