import { fireEvent, render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import Cursor from './Cursor.svelte';

const track = { top: 100, left: 50, height: 400, width: 500, bottom: 500, right: 550, x: 50, y: 100 } as DOMRect;

function setup(props: Record<string, unknown> = {}) {
  vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockReturnValue(track);
  const handlers = { onmove: vi.fn(), onremove: vi.fn() };
  render(Cursor, { axis: 'x', position: 0.5, label: 'X1', valueText: '700 us', ...handlers, ...props });
  const cursor = screen.getByRole('slider', { name: 'X1' });
  Object.assign(cursor, { setPointerCapture: vi.fn() });
  return { cursor, ...handlers };
}

const pointer = (el: Element, type: string, init: { clientX?: number; clientY?: number } = {}) =>
  fireEvent(el, new MouseEvent(type, { ...init, bubbles: true }));

afterEach(() => vi.restoreAllMocks());

describe('Cursor semantics', () => {
  it('given an x cursor, when rendered, then it is a horizontal slider whose value is the position and text is the readout', () => {
    const { cursor } = setup({ position: 0.35 });
    expect(cursor).toHaveAttribute('aria-orientation', 'horizontal');
    expect(cursor).toHaveAttribute('aria-valuenow', '0.35');
    expect(cursor).toHaveAttribute('aria-valuetext', '700 us');
  });

  it('given a y cursor, when rendered, then it is a vertical slider', () => {
    const { cursor } = setup({ axis: 'y' });
    expect(cursor).toHaveAttribute('aria-orientation', 'vertical');
  });

  it('given a disabled cursor with a reason, when rendered, then the reason is its tooltip and a drag does nothing', async () => {
    const { cursor, onmove } = setup({ disabled: true, disabledReason: 'No waveform' });
    expect(cursor).toHaveAttribute('title', 'No waveform');
    await pointer(cursor, 'pointerdown', { clientX: 300 });
    expect(onmove).not.toHaveBeenCalled();
  });
});

describe('Cursor drag', () => {
  it('given an x cursor, when the pointer is pressed and moved, then onmove fires per event with the position from the left and aria-valuenow follows', async () => {
    const { cursor, onmove } = setup();
    await pointer(cursor, 'pointerdown', { clientX: 300 });
    await pointer(cursor, 'pointermove', { clientX: 175 });
    expect(onmove.mock.calls).toEqual([[0.5], [0.25]]);
    expect(cursor).toHaveAttribute('aria-valuenow', '0.25');
  });

  it('given a y cursor, when dragged below the screen, then the position clamps to 1 from the top', async () => {
    const { cursor, onmove } = setup({ axis: 'y' });
    await pointer(cursor, 'pointerdown', { clientY: 300 });
    await pointer(cursor, 'pointermove', { clientY: 900 });
    expect(onmove).toHaveBeenLastCalledWith(1);
  });

  it('given a drag was released, when the pointer moves again, then nothing moves', async () => {
    const { cursor, onmove } = setup();
    await pointer(cursor, 'pointerdown', { clientX: 300 });
    await pointer(cursor, 'pointerup');
    onmove.mockClear();
    await pointer(cursor, 'pointermove', { clientX: 100 });
    expect(onmove).not.toHaveBeenCalled();
  });

  it('given a drag in progress, when Escape is pressed, then the cursor returns to where it started and the drag ends', async () => {
    const { cursor, onmove } = setup({ position: 0.4 });
    await pointer(cursor, 'pointerdown', { clientX: 300 });
    await fireEvent.keyDown(cursor, { key: 'Escape' });
    expect(onmove).toHaveBeenLastCalledWith(0.4);
    onmove.mockClear();
    await pointer(cursor, 'pointermove', { clientX: 100 });
    expect(onmove).not.toHaveBeenCalled();
  });
});

describe('Cursor keyboard', () => {
  it('given a focused x cursor, when ArrowRight is pressed, then it moves one 1/250 step', async () => {
    const { cursor, onmove } = setup({ position: 0.5 });
    cursor.focus();
    await userEvent.keyboard('{ArrowRight}');
    expect(onmove).toHaveBeenCalledWith(0.5 + 1 / 250);
  });

  it('given a focused x cursor, when Shift+ArrowLeft is pressed, then it moves ten steps left', async () => {
    const { cursor, onmove } = setup({ position: 0.5 });
    cursor.focus();
    await userEvent.keyboard('{Shift>}{ArrowLeft}{/Shift}');
    expect(onmove).toHaveBeenCalledWith(0.5 - 10 / 250);
  });

  it('given a focused y cursor, when ArrowUp is pressed, then it moves one 1/200 step up the screen', async () => {
    const { cursor, onmove } = setup({ axis: 'y', position: 0.5 });
    cursor.focus();
    await userEvent.keyboard('{ArrowUp}');
    expect(onmove).toHaveBeenCalledWith(0.5 - 1 / 200);
  });

  it('given a cursor at the edge, when an arrow pushes it outside, then it clamps', async () => {
    const { cursor, onmove } = setup({ position: 1 });
    cursor.focus();
    await userEvent.keyboard('{ArrowRight}');
    expect(onmove).toHaveBeenCalledWith(1);
  });

  it('given a focused cursor, when Delete is pressed, then onremove is called', async () => {
    const { cursor, onremove } = setup();
    cursor.focus();
    await userEvent.keyboard('{Delete}');
    expect(onremove).toHaveBeenCalledOnce();
  });

  it('given a cursor, when it is double-clicked, then onremove is called', async () => {
    const { cursor, onremove } = setup();
    await userEvent.dblClick(cursor);
    expect(onremove).toHaveBeenCalledOnce();
  });
});
