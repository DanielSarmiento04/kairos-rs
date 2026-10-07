import { describe, it, expect } from 'vitest';
import { mount, flushPromises } from '@vue/test-utils';
import TransformationEditor from '../TransformationEditor.vue';
import type { RequestTransformation, ResponseTransformation } from '../../../types';

describe('TransformationEditor', () => {
  it('shows empty state when both transformations are null', () => {
    const wrapper = mount(TransformationEditor, {
      props: { requestTransformation: null, responseTransformation: null },
    });
    expect(wrapper.text()).toContain('No transformations configured');
  });

  it('renders sections when at least one transformation provided', () => {
    const req: RequestTransformation = { headers: [], path: null, query_params: [] };
    const wrapper = mount(TransformationEditor, {
      props: { requestTransformation: req, responseTransformation: null },
    });
    expect(wrapper.text()).toContain('Request Transformation');
    expect(wrapper.text()).toContain('Response Transformation');
  });

  it('emits initial request + response transformations when Enable clicked', async () => {
    const wrapper = mount(TransformationEditor, {
      props: { requestTransformation: null, responseTransformation: null },
    });
    await wrapper.find('.empty-state .btn-primary').trigger('click');
    const reqEmits = wrapper.emitted('update:requestTransformation');
    const resEmits = wrapper.emitted('update:responseTransformation');
    expect(reqEmits).toBeTruthy();
    expect(resEmits).toBeTruthy();
    expect(reqEmits![0]![0]).toMatchObject({ headers: [], query_params: [] });
    expect(resEmits![0]![0]).toMatchObject({ headers: [], status_code_mappings: [] });
  });

  it('emits update:requestTransformation when add header rule clicked', async () => {
    const req: RequestTransformation = { headers: [], path: null, query_params: [] };
    const wrapper = mount(TransformationEditor, {
      props: { requestTransformation: req, responseTransformation: null },
    });
    const addBtn = wrapper.findAll('.btn-add')[0]!;
    await addBtn.trigger('click');
    const emitted = wrapper.emitted('update:requestTransformation');
    expect(emitted).toBeTruthy();
    const reqPayload = emitted![emitted!.length - 1]![0] as RequestTransformation;
    expect(reqPayload.headers!.length).toBe(1);
  });

  it('emits update:responseTransformation when add status mapping clicked', async () => {
    const res: ResponseTransformation = { headers: [], status_code_mappings: [] };
    const wrapper = mount(TransformationEditor, {
      props: { requestTransformation: null, responseTransformation: res },
    });
    const addBtns = wrapper.findAll('.btn-add');
    // Find the status mapping add button by looking for the one inside status section
    const statusBtn = addBtns[addBtns.length - 1]!;
    await statusBtn.trigger('click');
    const emitted = wrapper.emitted('update:responseTransformation');
    expect(emitted).toBeTruthy();
    const resPayload = emitted![emitted!.length - 1]![0] as ResponseTransformation;
    expect(resPayload.status_code_mappings!.length).toBe(1);
  });

  it('renders preview panel', () => {
    const req: RequestTransformation = { headers: [], path: null, query_params: [] };
    const wrapper = mount(TransformationEditor, {
      props: { requestTransformation: req, responseTransformation: null },
    });
    expect(wrapper.find('.preview-panel').exists()).toBe(true);
  });

  it('passes through readonly flag', async () => {
    const req: RequestTransformation = { headers: [], path: null, query_params: [] };
    const wrapper = mount(TransformationEditor, {
      props: { requestTransformation: req, responseTransformation: null, readonly: true },
    });
    // readonly mode still renders sections, just hides add buttons
    const addBtns = wrapper.findAll('.btn-add');
    expect(addBtns.length).toBe(0);
    // wait a tick for any async render
    await flushPromises();
  });
});
