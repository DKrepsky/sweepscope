import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { createRawSnippet } from 'svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import Tooltip from './Tooltip.svelte';

const child = createRawSnippet(() => ({ render: () => '<button>target</button>' }));
const text = 'Volts per division, 2 mV to 5 V';

describe('Tooltip', () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it('AC-TT1: given hover for 500 ms, when time passes, then the tooltip shows with unit and range', async () => {
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
    render(Tooltip, { text, children: child });
    await user.hover(screen.getByRole('button'));
    vi.advanceTimersByTime(499);
    expect(screen.queryByRole('tooltip')).toBeNull();
    vi.advanceTimersByTime(1);
    expect(await screen.findByRole('tooltip')).toHaveTextContent(text);
  });

  it('AC-TT1: given keyboard focus, when the target is focused, then the tooltip shows at once', async () => {
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
    render(Tooltip, { text, children: child });
    await user.tab();
    expect(screen.getByRole('tooltip')).toHaveTextContent(text);
  });

  it('AC-TT1: given a shown tooltip, when Escape is pressed, then it hides', async () => {
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
    render(Tooltip, { text, children: child });
    await user.tab();
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('tooltip')).toBeNull();
  });

  it('given a pointer that leaves before the delay, when time passes, then the tooltip never shows', async () => {
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
    render(Tooltip, { text, children: child });
    await user.hover(screen.getByRole('button'));
    await user.unhover(screen.getByRole('button'));
    vi.advanceTimersByTime(1000);
    expect(screen.queryByRole('tooltip')).toBeNull();
  });

  it('given a shown tooltip, when the control is inspected, then aria-describedby points at the tooltip', async () => {
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
    render(Tooltip, { text, children: child });
    await user.tab();
    const tipId = screen.getByRole('tooltip').id;
    expect(screen.getByRole('button')).toHaveAttribute('aria-describedby', tipId);
  });

  it('given a hidden tooltip, when the control is inspected, then it has no aria-describedby', async () => {
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
    render(Tooltip, { text, children: child });
    await user.tab();
    await user.keyboard('{Escape}');
    expect(screen.getByRole('button')).not.toHaveAttribute('aria-describedby');
  });
});
