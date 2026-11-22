import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import RunButton from './RunButton.svelte';

describe('RunButton', () => {
  it('AC-B2: given running, when rendered, then the label is Stop with the stop class and glyph', () => {
    const { container } = render(RunButton, { running: true, onclick: vi.fn() });
    expect(screen.getByRole('button', { name: 'Stop' })).toHaveClass('stop');
    expect(container.querySelector('svg')).toBeInTheDocument();
  });

  it('AC-B2: given not running, when rendered, then the label is Run with the run class', () => {
    render(RunButton, { running: false, onclick: vi.fn() });
    expect(screen.getByRole('button', { name: 'Run' })).toHaveClass('run');
  });

  it('AC-B2: given a click, when it happens, then only onclick fires and the label does not change', async () => {
    const onclick = vi.fn();
    render(RunButton, { running: false, onclick });
    await userEvent.click(screen.getByRole('button'));
    expect(onclick).toHaveBeenCalledOnce();
    expect(screen.getByRole('button')).toHaveTextContent('Run');
  });

  it('AC-B2: given focus outside any text field, when Space is pressed, then it activates once', async () => {
    const onclick = vi.fn();
    render(RunButton, { running: false, onclick });
    await userEvent.tab();
    await userEvent.keyboard(' ');
    expect(onclick).toHaveBeenCalledOnce();
  });
});
