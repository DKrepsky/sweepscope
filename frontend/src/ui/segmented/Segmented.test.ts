import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import Segmented from './Segmented.svelte';
import source from './Segmented.svelte?raw';

const options = [
  { id: 'edge', label: 'Edge' },
  { id: 'pulse', label: 'Pulse' },
  { id: 'video', label: 'Video', disabled: true, disabledReason: 'Experimental' },
];

const setup = (extra = {}) => {
  const onchange = vi.fn();
  render(Segmented, { value: 'edge', options, onchange, ariaLabel: 'Trigger type', ...extra });
  return onchange;
};

describe('Segmented', () => {
  it('AC-S1: given 3 options, when ArrowRight is pressed, then focus moves to the next enabled option', async () => {
    setup();
    screen.getByRole('button', { name: 'Edge' }).focus();
    await userEvent.keyboard('{ArrowRight}');
    expect(screen.getByRole('button', { name: 'Pulse' })).toHaveFocus();
    await userEvent.keyboard('{ArrowRight}');
    expect(screen.getByRole('button', { name: 'Pulse' })).toHaveFocus();
  });

  it('AC-S1: given focus on an option, when Enter is pressed, then onchange fires with its id', async () => {
    const onchange = setup();
    screen.getByRole('button', { name: 'Pulse' }).focus();
    await userEvent.keyboard('{Enter}');
    expect(onchange).toHaveBeenCalledExactlyOnceWith('pulse');
  });

  it('AC-S1: given a value, when rendered, then exactly one item has aria-pressed true', () => {
    setup({ value: 'pulse' });
    const pressed = screen.getAllByRole('button').filter((b) => b.getAttribute('aria-pressed') === 'true');
    expect(pressed).toHaveLength(1);
    expect(pressed[0]).toHaveTextContent('Pulse');
  });

  it('AC-S1: given a disabled option, when selected by Enter, then onchange is not called', async () => {
    const onchange = setup();
    screen.getByRole('button', { name: 'Video' }).focus();
    await userEvent.keyboard('{Enter}');
    expect(onchange).not.toHaveBeenCalled();
  });

  it('AC-S1: given a disabled option, when focused, then its reason shows', async () => {
    setup();
    screen.getByRole('button', { name: 'Video' }).focus();
    expect(await screen.findByRole('tooltip')).toHaveTextContent('Experimental');
  });

  it('given the option button style, when read, then white-space is nowrap (jsdom has no component CSS, e2e checks the layout)', () => {
    const rule = /\n {2}button \{([^}]*)\}/.exec(source)?.[1] ?? '';
    expect(rule).toMatch(/white-space:\s*nowrap/);
  });
});
