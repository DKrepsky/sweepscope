import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { get, writable } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { LayoutTier } from '../../app/layout';
import { createMockMainWindow } from '../../app/main-window';
import type { Notification } from '../../app/notifications';
import type { TextScalePercent } from '../../app/text-scale';
import { createRecordingContext } from '../../ui/graph/recording-context';
import MainWindow from './MainWindow.svelte';
import { calledNames, spyActions } from './test-support';

const tier: LayoutTier = { width: 'w1', height: 'h1', rootFontPx: 16 };

beforeEach(() => {
  window.matchMedia = (() => ({ addEventListener: vi.fn(), removeEventListener: vi.fn() })) as never;
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockImplementation(() => createRecordingContext() as never);
});

function setup() {
  const actions = spyActions();
  render(MainWindow, {
    props: {
      model: writable(get(createMockMainWindow())),
      actions,
      tier: writable(tier),
      textScale: writable<TextScalePercent>(100),
      notifications: writable<Notification[]>([]),
      onnotificationdismiss: () => {},
      ontextscale: () => {},
    },
  });
  return { actions, user: userEvent.setup() };
}

describe('window shortcuts', () => {
  it.each([
    ['c', 'toggleCursors'],
    ['a', 'startAutoset'],
    ['{Control>}s{/Control}', 'saveSetup'],
    ['{Control>}e{/Control}', 'exportCsv'],
    ['{Control>}p{/Control}', 'saveScreenshot'],
  ])('given the window has focus, when %s is pressed, then only %s is called', async (keys, name) => {
    const { actions, user } = setup();
    await user.keyboard(keys);
    expect(calledNames(actions)).toEqual([name]);
  });

  it('given a plain s, when pressed, then nothing is called', async () => {
    const { actions, user } = setup();
    await user.keyboard('s');
    expect(calledNames(actions)).toEqual([]);
  });

  it('given a text field has focus, when c is typed, then no action is called', async () => {
    const { actions, user } = setup();
    const field = document.createElement('input');
    document.body.append(field);
    field.focus();
    await user.keyboard('c');
    field.remove();
    expect(calledNames(actions)).toEqual([]);
  });

  it('given a text field has focus, when Ctrl+S is pressed, then no action is called', async () => {
    const { actions, user } = setup();
    const field = document.createElement('input');
    document.body.append(field);
    field.focus();
    await user.keyboard('{Control>}s{/Control}');
    field.remove();
    expect(calledNames(actions)).toEqual([]);
  });

  it('given Ctrl+S, when pressed, then the browser default is prevented', async () => {
    setup();
    const event = new KeyboardEvent('keydown', { key: 's', ctrlKey: true, bubbles: true, cancelable: true });
    window.dispatchEvent(event);
    expect(event.defaultPrevented).toBe(true);
  });

  it('given the sidebar, when Clear is clicked, then clearCursors is called', async () => {
    const { actions, user } = setup();
    await user.click(screen.getByRole('button', { name: 'Clear' }));
    expect(calledNames(actions)).toEqual(['clearCursors']);
  });
});
