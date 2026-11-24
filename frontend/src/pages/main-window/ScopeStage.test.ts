import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { get } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createMockMainWindow, type CursorsModel, type Interpolation } from '../../app/main-window';
import { createRecordingContext } from '../../ui/graph/recording-context';
import ScopeStage from './ScopeStage.svelte';
import { spyActions } from './test-support';

beforeEach(() => {
  window.matchMedia = (() => ({ addEventListener: vi.fn(), removeEventListener: vi.fn() })) as never;
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockImplementation(() => createRecordingContext() as never);
});

const lines: CursorsModel['lines'] = [
  { id: 'x1', axis: 'x', position: 0.3, tag: 'X1 700 us', valueText: 'X1 700 us' },
  { id: 'y2', axis: 'y', position: 0.7, tag: 'Y2 1.50 V', valueText: 'Y2 1.50 V' },
];

function renderStage(patch: { cursors?: Partial<CursorsModel>; interpolation?: Interpolation } = {}) {
  const model = get(createMockMainWindow());
  const actions = spyActions();
  const cursors = { ...model.cursors, enabled: true, available: { enabled: true }, lines, ...patch.cursors };
  render(ScopeStage, {
    props: {
      graph: model.graph,
      channels: model.channels,
      trigger: model.trigger,
      cursors,
      dots: false,
      interpolation: patch.interpolation,
      actions,
      rootFontPx: 16,
    },
  });
  return actions;
}

describe('ScopeStage cursors', () => {
  it('given cursors on, when rendered, then each line is a slider named after its cursor', () => {
    renderStage();
    expect(screen.getByRole('slider', { name: 'Cursor X1' })).toHaveAttribute('aria-valuetext', 'X1 700 us');
    expect(screen.getByRole('slider', { name: 'Cursor Y2' })).toHaveAttribute('aria-orientation', 'vertical');
  });

  it('given a focused cursor, when an arrow key is pressed, then moveCursor gets its id and the new position', async () => {
    const actions = renderStage();
    screen.getByRole('slider', { name: 'Cursor X1' }).focus();
    await userEvent.keyboard('{ArrowRight}');
    expect(actions.moveCursor).toHaveBeenCalledWith('x1', 0.3 + 1 / 250);
  });

  it('given a focused cursor, when Delete is pressed, then removeCursor gets its id', async () => {
    const actions = renderStage();
    screen.getByRole('slider', { name: 'Cursor Y2' }).focus();
    await userEvent.keyboard('{Delete}');
    expect(actions.removeCursor).toHaveBeenCalledWith('y2');
  });

  it('given no waveform, when cursors are shown, then they are disabled with the reason as tooltip', () => {
    renderStage({ cursors: { available: { enabled: false, reason: 'No waveform' } } });
    expect(screen.getByRole('slider', { name: 'Cursor X1' })).toHaveAttribute('title', 'No waveform');
  });

  it('given cursors off, when rendered, then no cursor is shown', () => {
    renderStage({ cursors: { enabled: false, lines: [] } });
    expect(screen.queryByRole('slider', { name: /Cursor/ })).toBeNull();
  });
});

describe('ScopeStage interpolation badge', () => {
  it.each([
    ['sinc', 'sin(x)/x SW'],
    ['linear', 'Linear SW'],
    ['dots', 'Dots'],
  ] as const)('given the mode %s, when rendered, then the badge reads %s', (mode, text) => {
    renderStage({ interpolation: mode });
    expect(screen.getByRole('note')).toHaveTextContent(text);
  });
});
