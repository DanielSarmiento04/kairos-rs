import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import TimeRangePicker from '../TimeRangePicker.vue';

describe('TimeRangePicker', () => {
  it('renders all preset buttons', () => {
    const wrapper = mount(TimeRangePicker, { props: { modelValue: '1h' } });
    const buttons = wrapper.findAll('.preset-btn');
    // 6 presets + 1 "Custom" button
    expect(buttons.length).toBe(7);
    expect(buttons.map((b) => b.text().trim())).toEqual([
      '5m',
      '15m',
      '1h',
      '6h',
      '24h',
      '7d',
      'Custom',
    ]);
  });

  it('marks the active preset', () => {
    const wrapper = mount(TimeRangePicker, { props: { modelValue: '6h' } });
    const active = wrapper.findAll('.preset-btn').filter((b) => b.classes('active'));
    expect(active).toHaveLength(1);
    expect(active[0]!.text().trim()).toBe('6h');
  });

  it('emits update:modelValue when a preset is clicked', async () => {
    const wrapper = mount(TimeRangePicker, { props: { modelValue: '1h' } });
    // 6 presets + 1 custom; index 1 = "15m"
    await wrapper.findAll('.preset-btn')[1]!.trigger('click');
    const emitted = wrapper.emitted('update:modelValue');
    expect(emitted).toBeTruthy();
    expect(emitted![0]![0]).toBe('15m');
  });

  it('shows custom range inputs when modelValue is custom', async () => {
    const wrapper = mount(TimeRangePicker, {
      props: {
        modelValue: 'custom',
        customRange: { start: '2025-01-01T00:00:00Z', end: '2025-01-01T01:00:00Z' },
      },
    });
    const inputs = wrapper.findAll('input[type="datetime-local"]');
    expect(inputs.length).toBe(2);
  });

  it('hides custom inputs for preset values', () => {
    const wrapper = mount(TimeRangePicker, { props: { modelValue: '1h' } });
    const inputs = wrapper.findAll('input[type="datetime-local"]');
    expect(inputs.length).toBe(0);
  });

  it('clicking Custom emits update:modelValue=custom', async () => {
    const wrapper = mount(TimeRangePicker, {
      props: { modelValue: '1h', customRange: { start: '', end: '' } },
    });
    const customBtn = wrapper.findAll('.preset-btn').find((b) =>
      b.text().includes('Custom'),
    )!;
    await customBtn.trigger('click');
    const emitted = wrapper.emitted('update:modelValue');
    expect(emitted![0]![0]).toBe('custom');
  });
});