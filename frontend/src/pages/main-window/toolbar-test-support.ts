import { render } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { get } from 'svelte/store';
import { vi } from 'vitest';
import { createMockMainWindow, type MainWindowModel, type ToolbarModel } from '../../app/main-window';
import Toolbar from './Toolbar.svelte';

/** Every callback of the toolbar is a spy; the model is the mock one, changed by the test where it needs to. */
export function renderToolbar(
  change: (toolbar: ToolbarModel) => ToolbarModel = (toolbar) => toolbar,
  changeModel: (model: MainWindowModel) => MainWindowModel = (model) => model,
) {
  const calls = {
    onfile: vi.fn(),
    ontogglecursors: vi.fn(),
    ontogglerun: vi.fn(),
    onarmsingle: vi.fn(),
    onautoset: vi.fn(),
    onsetview: vi.fn(),
    onsetpersistence: vi.fn(),
    onclearpersistence: vi.fn(),
    ontogglemath: vi.fn(),
    onstartrecording: vi.fn(),
    onstoprecording: vi.fn(),
  };
  const model = changeModel(get(createMockMainWindow()));
  render(Toolbar, {
    props: {
      toolbar: change(model.toolbar),
      views: model.views,
      persistence: model.persistence,
      autoset: model.autoset,
      recording: model.recording,
      toolLabels: true,
      runLabels: true,
      drawerButton: false,
      drawerOpen: false,
      textScale: 100,
      onforce: () => {},
      ontoggledrawer: () => {},
      ontextscale: () => {},
      ...calls,
    },
  });
  return { calls, user: userEvent.setup() };
}
