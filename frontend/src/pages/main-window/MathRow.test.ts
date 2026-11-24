import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { get } from 'svelte/store';
import { describe, expect, it } from 'vitest';
import { createMockMainWindow, type MathModel } from '../../app/main-window';
import MathRow from './MathRow.svelte';
import { spyActions } from './test-support';

function setup(change: Partial<MathModel['row']> = {}) {
  const { math, channels } = get(createMockMainWindow());
  const actions = spyActions();
  render(MathRow, { props: { math: { ...math, row: { ...math.row, ...change } }, channels, actions } });
  return { actions, user: userEvent.setup() };
}

describe('MathRow', () => {
  it('given math off, when the M toggle is pressed, then math is enabled', async () => {
    const { actions, user } = setup();

    await user.click(screen.getByRole('button', { name: 'M, math channel enabled' }));

    expect(actions.setMathEnabled).toHaveBeenCalledExactlyOnceWith(true);
  });

  it('given math on, when the M toggle is pressed, then math is disabled', async () => {
    const { actions, user } = setup({ enabled: true });

    await user.click(screen.getByRole('button', { name: 'M, math channel enabled' }));

    expect(actions.setMathEnabled).toHaveBeenCalledExactlyOnceWith(false);
  });

  it('given the operation menu, when Add is chosen, then the operation is add', async () => {
    const { actions, user } = setup();

    await user.click(screen.getByRole('button', { name: /math operation/ }));
    await user.click(screen.getByRole('menuitem', { name: 'A+B' }));

    expect(actions.setMathOperation).toHaveBeenCalledExactlyOnceWith('add');
  });

  it('given the source A menu, when CH3 is chosen, then source A is channel 3', async () => {
    const { actions, user } = setup();

    await user.click(screen.getByRole('button', { name: /math source A/ }));
    await user.click(screen.getByRole('menuitem', { name: 'CH3' }));

    expect(actions.setMathSource).toHaveBeenCalledExactlyOnceWith('a', 3);
  });

  it('given the invert operation, when rendered, then source B is disabled and shows a dash', () => {
    setup({ operation: 'invertA', sourceB: undefined });

    const sourceB = screen.getByRole('button', { name: /math source B/ });
    expect(sourceB).toHaveAttribute('aria-disabled', 'true');
    expect(sourceB).toHaveTextContent('B -');
  });

  it('given a mixed status, when rendered, then the Mixed badge carries the backend detail', () => {
    setup({ status: 'mixed', statusDetail: 'CH1 500 mV/div, CH2 20 mV/div.' });

    expect(screen.getByText('Mixed')).toBeInTheDocument();
    expect(screen.getByText('CH1 500 mV/div, CH2 20 mV/div.')).toBeInTheDocument();
  });

  it('given an unavailable status, when rendered, then the Unavailable badge shows', () => {
    setup({ status: 'unavailable', statusDetail: 'CH2 is off' });

    expect(screen.getByText('Unavailable')).toBeInTheDocument();
  });

  it('given a position typed into the field, when committed, then the position is sent in divisions', async () => {
    const { actions, user } = setup({ enabled: true });
    const field = screen.getByRole('textbox', { name: 'Math vertical position in divisions' });

    await user.clear(field);
    await user.type(field, '−1,5{Enter}');

    expect(actions.setMathPosition).toHaveBeenCalledExactlyOnceWith(-1.5);
  });

  it('given a backend that blocks the vertical group, when rendered, then the toggle is disabled', () => {
    setup({ availability: { enabled: false, reason: 'Math is not available in roll mode.' } });

    expect(screen.getByRole('button', { name: 'M, math channel enabled' })).toHaveAttribute('aria-disabled', 'true');
  });
});
