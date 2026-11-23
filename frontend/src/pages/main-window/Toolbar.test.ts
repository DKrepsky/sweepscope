import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { get } from 'svelte/store';
import { describe, expect, it, vi } from 'vitest';
import { createMockMainWindow, type ToolbarModel } from '../../app/main-window';
import Toolbar from './Toolbar.svelte';

function setup(change: (toolbar: ToolbarModel) => ToolbarModel = (toolbar) => toolbar) {
  const onfile = vi.fn();
  const toolbar = change(get(createMockMainWindow()).toolbar);
  render(Toolbar, {
    props: {
      toolbar,
      toolLabels: true,
      runLabels: true,
      drawerButton: false,
      drawerOpen: false,
      textScale: 100,
      ontogglerun: () => {},
      onforce: () => {},
      ontoggledrawer: () => {},
      ontextscale: () => {},
      onfile,
    },
  });
  return { onfile, user: userEvent.setup() };
}

const noWaveform = (toolbar: ToolbarModel): ToolbarModel => ({
  ...toolbar,
  files: {
    ...toolbar.files,
    exportCsv: { enabled: false, reason: 'No waveform' },
    exportRaw: { enabled: false, reason: 'No waveform' },
    screenshot: { enabled: false, reason: 'No waveform' },
  },
});

describe('file menu', () => {
  it('given the File button, when it is opened, then the five items are listed', async () => {
    const { user } = setup();
    await user.click(screen.getByRole('button', { name: /File/ }));
    const names = screen.getAllByRole('menuitem').map((item) => item.textContent?.trim());
    expect(names).toEqual([
      'Save setup...',
      'Load setup...',
      'Export CSV...',
      'Export raw capture...',
      'Save screenshot...',
    ]);
  });

  it.each([
    ['Save setup...', 'saveSetup'],
    ['Load setup...', 'loadSetup'],
    ['Export CSV...', 'exportCsv'],
    ['Export raw capture...', 'exportRaw'],
    ['Save screenshot...', 'saveScreenshot'],
  ])('given the menu is open, when %s is chosen, then only %s is requested', async (label, action) => {
    const { onfile, user } = setup();
    await user.click(screen.getByRole('button', { name: /File/ }));
    await user.click(screen.getByRole('menuitem', { name: label }));
    expect(onfile.mock.calls).toEqual([[action]]);
  });

  it('given no waveform, when the menu is opened, then exports and screenshot are disabled and setups stay enabled', async () => {
    const { user } = setup(noWaveform);
    await user.click(screen.getByRole('button', { name: /File/ }));
    const disabled = (name: string) => screen.getByRole('menuitem', { name }).getAttribute('aria-disabled');
    expect(disabled('Export CSV...')).toBe('true');
    expect(disabled('Export raw capture...')).toBe('true');
    expect(disabled('Save screenshot...')).toBe('true');
    expect(disabled('Save setup...')).not.toBe('true');
  });

  it('given no waveform, when the screenshot button is clicked, then nothing is requested', async () => {
    const { onfile, user } = setup(noWaveform);
    await user.click(screen.getByRole('button', { name: 'Screenshot' }));
    expect(onfile).not.toHaveBeenCalled();
  });

  it('given a waveform, when the screenshot button is clicked, then saveScreenshot is requested', async () => {
    const { onfile, user } = setup();
    await user.click(screen.getByRole('button', { name: 'Screenshot' }));
    expect(onfile.mock.calls).toEqual([['saveScreenshot']]);
  });
});
