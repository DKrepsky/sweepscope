import { createRawSnippet } from 'svelte';
import { fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import Toast from './Toast.svelte';

const text = createRawSnippet(() => ({ render: () => '<span>Saved</span>' }));

describe('Toast', () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it('AC-BN1: given a Toast with timeoutMs, when the time passes, then ondismiss is called once', () => {
    const ondismiss = vi.fn();
    render(Toast, { kind: 'info', text, timeoutMs: 1000, ondismiss });

    vi.advanceTimersByTime(999);
    expect(ondismiss).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    vi.advanceTimersByTime(5000);

    expect(ondismiss).toHaveBeenCalledOnce();
  });

  it('given no timeoutMs, when 4 seconds pass, then ondismiss is called', () => {
    const ondismiss = vi.fn();
    render(Toast, { kind: 'info', text, ondismiss });

    vi.advanceTimersByTime(4000);

    expect(ondismiss).toHaveBeenCalledOnce();
  });

  it('given a Toast that is removed, when the time passes, then ondismiss is not called', () => {
    const ondismiss = vi.fn();
    const { unmount } = render(Toast, { kind: 'info', text, ondismiss });

    unmount();
    vi.advanceTimersByTime(10000);

    expect(ondismiss).not.toHaveBeenCalled();
  });

  it('AC-BN1: given a Toast, when rendered, then it never takes focus', () => {
    render(Toast, { kind: 'info', text, ondismiss: vi.fn() });

    const toast = screen.getByRole('status');
    expect(toast).not.toHaveAttribute('tabindex');
    expect(toast.querySelector('[autofocus]')).toBeNull();
    expect(document.activeElement).toBe(document.body);
  });

  it('given an error Toast, when a long time passes, then ondismiss is not called', () => {
    const ondismiss = vi.fn();
    render(Toast, { kind: 'error', text, ondismiss });

    vi.advanceTimersByTime(60000);

    expect(ondismiss).not.toHaveBeenCalled();
  });

  it('given an info Toast hovered, when the time passes, then it stays until the pointer leaves', async () => {
    const ondismiss = vi.fn();
    render(Toast, { kind: 'info', text, timeoutMs: 1000, ondismiss });
    const toast = screen.getByRole('status');

    await fireEvent.mouseEnter(toast);
    vi.advanceTimersByTime(5000);
    expect(ondismiss).not.toHaveBeenCalled();
    await fireEvent.mouseLeave(toast);
    vi.advanceTimersByTime(1000);

    expect(ondismiss).toHaveBeenCalledOnce();
  });

  it('given a warning Toast with focus inside, when the time passes, then it stays until focus leaves', async () => {
    const ondismiss = vi.fn();
    render(Toast, { kind: 'warning', text, timeoutMs: 1000, ondismiss });
    const toast = screen.getByRole('status');

    await fireEvent.focusIn(toast);
    vi.advanceTimersByTime(5000);
    expect(ondismiss).not.toHaveBeenCalled();
    await fireEvent.focusOut(toast);
    vi.advanceTimersByTime(1000);

    expect(ondismiss).toHaveBeenCalledOnce();
  });

  it.each([
    ['error', 'alert'],
    ['warning', 'status'],
    ['info', 'status'],
  ] as const)('AC-BN1: given a %s Toast, when rendered, then it has role %s', (kind, role) => {
    render(Toast, { kind, text, ondismiss: vi.fn() });

    expect(screen.getByRole(role)).toBeInTheDocument();
  });
});
