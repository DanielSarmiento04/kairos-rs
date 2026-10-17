import { describe, it, expect } from 'vitest';
import { mount } from '@vue/test-utils';
import PresetTemplates from '../PresetTemplates.vue';
import { TRANSFORMATION_PRESETS, type TransformationPreset } from '../presets';

describe('PresetTemplates', () => {
  it('renders only request-scope presets for request scope', () => {
    const wrapper = mount(PresetTemplates, { props: { scope: 'request' } });
    const options = wrapper.findAll('option');
    const visibleIds = options.map((o) => o.attributes('value')).filter(Boolean);
    const reqPresetIds = TRANSFORMATION_PRESETS.filter(
      (p) => p.scope === 'request' || p.scope === 'both',
    ).map((p) => p.id);
    reqPresetIds.forEach((id) => expect(visibleIds).toContain(id));
    const resOnly = TRANSFORMATION_PRESETS.filter((p) => p.scope === 'response').map((p) => p.id);
    resOnly.forEach((id) => expect(visibleIds).not.toContain(id));
  });

  it('renders only response-scope presets for response scope', () => {
    const wrapper = mount(PresetTemplates, { props: { scope: 'response' } });
    const options = wrapper.findAll('option');
    const visibleIds = options.map((o) => o.attributes('value')).filter(Boolean);
    const resPresetIds = TRANSFORMATION_PRESETS.filter(
      (p) => p.scope === 'response' || p.scope === 'both',
    ).map((p) => p.id);
    resPresetIds.forEach((id) => expect(visibleIds).toContain(id));
  });

  it('emits apply event with full preset data when apply clicked', async () => {
    const wrapper = mount(PresetTemplates, { props: { scope: 'request' } });
    const select = wrapper.find('select');
    await select.setValue('strip-auth');
    await wrapper.find('.btn-sub-action').trigger('click');
    const emitted = wrapper.emitted('apply');
    expect(emitted).toBeTruthy();
    const payload = emitted![0]![0] as TransformationPreset;
    expect(payload.id).toBe('strip-auth');
    expect(payload.request).toBeDefined();
  });

  it('does not emit apply when nothing selected', async () => {
    const wrapper = mount(PresetTemplates, { props: { scope: 'request' } });
    const applyBtn = wrapper.find('.btn-sub-action');
    expect(applyBtn.attributes('disabled')).toBeDefined();
  });
});
