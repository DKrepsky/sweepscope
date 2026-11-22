import { fireEvent, render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { createRawSnippet } from 'svelte';
import { describe, expect, it, vi } from 'vitest';
import { menuTriggerAttributes } from '../menu/trigger';
import Button from './Button.svelte';

const label = createRawSnippet(() => ({ render: () => '<span>Go</span>' }));

describe('Button', () => {
  it('AC-B1: given an enabled Button, when pointerdown occurs, then data-pressed is set at once', async () => {
    render(Button, { children: label });
    await fireEvent.pointerDown(screen.getByRole('button'));
    expect(screen.getByRole('button')).toHaveAttribute('data-pressed', 'true');
  });

  it('AC-B1: given a pressed Button, when the pointer is released, then data-pressed clears', async () => {
    render(Button, { children: label });
    await fireEvent.pointerDown(screen.getByRole('button'));
    await fireEvent.pointerUp(screen.getByRole('button'));
    expect(screen.getByRole('button')).not.toHaveAttribute('data-pressed');
  });

  it('AC-B1: given an enabled Button, when clicked, then onclick fires once', async () => {
    const onclick = vi.fn();
    render(Button, { children: label, onclick });
    await userEvent.click(screen.getByRole('button'));
    expect(onclick).toHaveBeenCalledOnce();
  });

  it('AC-B1: given Tab focus, when the Button receives focus-visible, then it has the focus-ring class', async () => {
    render(Button, { children: label });
    await userEvent.tab();
    expect(screen.getByRole('button')).toHaveFocus();
    expect(screen.getByRole('button')).toHaveClass('focus-ring');
  });

  it('AC-B1: given a disabled Button with a reason, when clicked, Enter or Space pressed, then onclick is not called', async () => {
    const onclick = vi.fn();
    render(Button, { children: label, onclick, disabled: true, disabledReason: 'No device' });
    const user = userEvent.setup();
    await user.click(screen.getByRole('button'));
    await user.keyboard('{Enter}');
    await user.keyboard(' ');
    expect(onclick).not.toHaveBeenCalled();
  });

  it('AC-B1: given a disabled Button, when focused, then it has aria-disabled and shows the reason', async () => {
    render(Button, { children: label, disabled: true, disabledReason: 'No device' });
    await userEvent.tab();
    expect(screen.getByRole('button')).toHaveAttribute('aria-disabled', 'true');
    expect(screen.getByRole('button')).toHaveFocus();
    expect(screen.getByRole('tooltip')).toHaveTextContent('No device');
  });

  it('given pressed true, when rendered, then aria-pressed is true', () => {
    render(Button, { children: label, pressed: true });
    expect(screen.getByRole('button')).toHaveAttribute('aria-pressed', 'true');
  });

  it('AC-B1: given menu trigger attributes, when spread on Button, then aria-haspopup and aria-expanded reach the button', () => {
    render(Button, { children: label, ...menuTriggerAttributes(true) });
    const button = screen.getByRole('button');
    expect(button).toHaveAttribute('aria-haspopup', 'menu');
    expect(button).toHaveAttribute('aria-expanded', 'true');
  });

  it('AC-B1: given forwarded aria-pressed and data-pressed, when Button has its own values, then its own win', () => {
    const forwarded = { 'aria-pressed': 'mixed', 'data-pressed': 'true' } as const;
    render(Button, { children: label, pressed: false, ...forwarded });
    const button = screen.getByRole('button');
    expect(button).toHaveAttribute('aria-pressed', 'false');
    expect(button).not.toHaveAttribute('data-pressed');
  });

  it('AC-B1: given a forwarded aria-label, when ariaLabel is set, then ariaLabel wins', () => {
    render(Button, { children: label, ariaLabel: 'Own', 'aria-label': 'Forwarded' });
    expect(screen.getByRole('button', { name: 'Own' })).toBeInTheDocument();
  });

  it('AC-B1: given a forwarded data attribute, when rendered, then it reaches the button', () => {
    render(Button, { children: label, 'data-testid': 'trigger' });
    expect(screen.getByTestId('trigger')).toBe(screen.getByRole('button'));
  });
});
