import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import SparklineChart from '../SparklineChart.vue';

describe('SparklineChart', () => {
  it('renders SVG polyline with data points', () => {
    const wrapper = mount(SparklineChart, {
      props: {
        data: [10, 20, 30, 40],
        color: '#2563eb',
        unit: ' req/s',
      },
    });

    expect(wrapper.find('polyline').exists()).toBe(true);
    expect(wrapper.text()).toContain('40 req/s');
  });

  it('handles empty or single-point data gracefully', () => {
    const wrapper = mount(SparklineChart, {
      props: {
        data: [],
      },
    });
    expect(wrapper.find('polyline').exists()).toBe(false);
    expect(wrapper.text()).toContain('0');
  });
});
