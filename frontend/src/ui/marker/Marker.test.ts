import { fireEvent, render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { createRawSnippet } from 'svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import Marker from './Marker.svelte';

const track = { top: 100, left: 0, height: 400, width: 400, bottom: 500, right: 400, x: 0, y: 100 } as DOMRect;

function setup(props: Record<string, unknown> = {}) {
  vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockReturnValue(track);
  const handlers = { ondrag: vi.fn(), oncommit: vi.fn(), ondelete: vi.fn() };
  render(Marker, { kind: 'ground', position: 0.5, label: '1', name: 'CH1 ground', ...handlers, ...props });
  const marker = screen.getByRole('slider', { name: 'CH1 ground' });
  Object.assign(marker, { setPointerCapture: vi.fn(), releasePointerCapture: vi.fn() });
  return { marker, ...handlers };
}

const move = (el: Element, clientY: number) => fireEvent(el, new MouseEvent('pointermove', { clientY, bubbles: true }));
const down = (el: Element, clientY: number) => fireEvent(el, new MouseEvent('pointerdown', { clientY, bubbles: true }));
const up = (el: Element) => fireEvent(el, new MouseEvent('pointerup', { bubbles: true }));

afterEach(() => vi.restoreAllMocks());

describe('Marker drag', () => {
  it('AC-MK1: given a draggable marker, when the pointer moves while captured, then ondrag gets each clamped position and release commits the last once', async () => {
    const { marker, ondrag, oncommit } = setup();
    await down(marker, 300);
    await move(marker, 200);
    await move(marker, 0);
    await move(marker, 900);
    await up(marker);
    expect(ondrag.mock.calls.map(([p]) => p)).toEqual([0.5, 0.75, 1, 0]);
    expect(oncommit).toHaveBeenCalledExactlyOnceWith(0);
  });

  it('given no drag in progress, when the pointer moves, then nothing is reported', async () => {
    const { marker, ondrag, oncommit } = setup();
    await move(marker, 200);
    await up(marker);
    expect(ondrag).not.toHaveBeenCalled();
    expect(oncommit).not.toHaveBeenCalled();
  });

  it('AC-MK1: given disabled with a reason, when pointerdown occurs, then no drag starts and the tooltip is the reason', async () => {
    const { marker, ondrag, oncommit } = setup({ disabled: true, disabledReason: 'Trigger not available: offline' });
    await down(marker, 300);
    await move(marker, 200);
    await up(marker);
    expect(ondrag).not.toHaveBeenCalled();
    expect(oncommit).not.toHaveBeenCalled();
    expect(marker).toHaveAttribute('title', 'Trigger not available: offline');
    expect(marker).toHaveAttribute('aria-disabled', 'true');
  });

  it('given a horizontal trigger position marker, when dragged, then the position follows the pointer x', async () => {
    const { marker, ondrag } = setup({ kind: 'trigger-position', name: 'CH1 ground' });
    await fireEvent(marker, new MouseEvent('pointerdown', { clientX: 100, bubbles: true }));
    expect(ondrag).toHaveBeenCalledWith(0.25);
  });
});

describe('Marker keyboard', () => {
  it('given a focused marker, when Alt+ArrowUp is pressed, then ondrag and oncommit get a step toward 1', async () => {
    const { marker, ondrag, oncommit } = setup({ step: 0.1 });
    marker.focus();
    await userEvent.keyboard('{Alt>}{ArrowUp}{/Alt}');
    expect(ondrag).toHaveBeenCalledWith(0.6);
    expect(oncommit).toHaveBeenCalledWith(0.6);
  });

  it('given a focused marker, when Alt+ArrowDown is pressed, then the step is toward 0', async () => {
    const { marker, oncommit } = setup({ step: 0.1 });
    marker.focus();
    await userEvent.keyboard('{Alt>}{ArrowDown}{/Alt}');
    expect(oncommit).toHaveBeenCalledWith(0.4);
  });

  it('given a focused marker, when ArrowUp is pressed without Alt, then nothing happens', async () => {
    const { marker, oncommit } = setup();
    marker.focus();
    await userEvent.keyboard('{ArrowUp}');
    expect(oncommit).not.toHaveBeenCalled();
  });

  it('given a focused marker, when Delete is pressed, then ondelete is called', async () => {
    const { marker, ondelete } = setup();
    marker.focus();
    await userEvent.keyboard('{Delete}');
    expect(ondelete).toHaveBeenCalledOnce();
  });

  it('given a marker at 1, when Alt+ArrowUp is pressed, then the position stays clamped at 1', async () => {
    const { marker, oncommit } = setup({ position: 1 });
    marker.focus();
    await userEvent.keyboard('{Alt>}{ArrowUp}{/Alt}');
    expect(oncommit).toHaveBeenCalledWith(1);
  });

  it('given a disabled marker, when it is focused and Delete is pressed, then ondelete is not called', async () => {
    const { marker, ondelete } = setup({ disabled: true, disabledReason: 'why' });
    marker.focus();
    await userEvent.keyboard('{Delete}');
    expect(marker).toHaveFocus();
    expect(ondelete).not.toHaveBeenCalled();
  });
});

describe('Marker looks', () => {
  it('given pending, when rendered, then the tag shows ... and the dashed outline class is present', () => {
    const { marker } = setup({ pending: true });
    expect(screen.getByText('...')).toBeInTheDocument();
    expect(marker).toHaveClass('pending');
  });

  it('given a tag snippet and not pending, when rendered, then the tag shows the snippet', () => {
    const tag = createRawSnippet(() => ({ render: () => '<span>0.50 V</span>' }));
    setup({ tag });
    expect(screen.getByText('0.50 V')).toBeInTheDocument();
    expect(screen.queryByText('...')).not.toBeInTheDocument();
  });

  it('given a position of 0.25, when rendered, then a vertical marker sits at 75 percent from the top', () => {
    const { marker } = setup({ position: 0.25 });
    expect(marker.style.top).toBe('75%');
  });

  it('given a double click, when not disabled, then onreset is called', async () => {
    const onreset = vi.fn();
    const { marker } = setup({ onreset });
    await userEvent.dblClick(marker);
    expect(onreset).toHaveBeenCalledOnce();
  });
});
