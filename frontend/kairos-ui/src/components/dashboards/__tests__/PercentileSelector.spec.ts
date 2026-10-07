import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import PercentileSelector from '../PercentileSelector.vue';

describe('PercentileSelector', () => {
  it('renders default options', () => {
    const wrapper = mount(PercentileSelector, { props: { modelValue: [95] } });
    const labels = wrapper.findAll('label');
    expect(labels).toHaveLength(6); // 50, 75, 90, 95, 99, 99.9
    expect(labels[3]!.text()).toContain('p95');
  });

  it('renders custom options when provided', () => {
    const wrapper = mount(PercentileSelector, {
      props: { modelValue: [], options: [25, 50, 75] },
    });
    expect(wrapper.findAll('label')).toHaveLength(3);
  });

  it('marks active percentiles', () => {
    const wrapper = mount(PercentileSelector, {
      props: { modelValue: [50, 99], options: [50, 99] },
    });
    const labels = wrapper.findAll('label');
    expect(labels[0]!.classes()).toContain('active');
    expect(labels[1]!.classes()).toContain('active');
  });

  it('emits sorted unique values when toggling on', async () => {
    const wrapper = mount(PercentileSelector, { props: { modelValue: [95] } });
    // Set checkbox value rather than clicking label (happy-dom doesn't
    // bubble click events from <label> → <input type="checkbox"> reliably).
    const inputs = wrapper.findAll('input[type="checkbox"]');
    await inputs[0]!.setValue(true); // p50
    const emitted = wrapper.emitted('update:modelValue');
    expect(emitted).toBeTruthy();
    expect(emitted![0]![0]).toEqual([50, 95]);
  });

  it('emits sorted values without the toggled-off item', async () => {
    const wrapper = mount(PercentileSelector, { props: { modelValue: [50, 95] } });
    const inputs = wrapper.findAll('input[type="checkbox"]');
    // p95 sits at index 3 in the default [50, 75, 90, 95, 99, 99.9].
    await inputs[3]!.setValue(false);
    const emitted = wrapper.emitted('update:modelValue');
    expect(emitted).toBeTruthy();
    expect(emitted![0]![0]).toEqual([50]);
  });

  it('emits nothing until user interaction', () => {
    const wrapper = mount(PercentileSelector, { props: { modelValue: [99, 50] } });
    expect(wrapper.emitted('update:modelValue')).toBeFalsy();
  });
});