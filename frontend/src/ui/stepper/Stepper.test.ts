import { fireEvent, render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import Stepper from './Stepper.svelte';

const setup = (extra = {}) => {
  const onstep = vi.fn();
  const onreset = vi.fn();
  render(Stepper, {
    value: '500',
    unit: 'mV/div',
    valueText: '500 millivolts per division',
    ariaLabel: 'Volts per division',
    onstep,
    onreset,
    ...extra,
  });
  return { onstep, onreset, value: () => screen.getByRole('spinbutton') };
};

describe('Stepper', () => {
  it('AC-ST1: given a Stepper, when the wheel turns one notch up, then onstep(1) fires once', async () => {
    const { onstep, value } = setup();
    await fireEvent.wheel(value(), { deltaY: -100 });
    expect(onstep).toHaveBeenCalledExactlyOnceWith(1);
  });

  it('AC-ST1: given a Stepper, when the wheel turns one notch down, then onstep(-1) fires once', async () => {
    const { onstep, value } = setup();
    await fireEvent.wheel(value(), { deltaY: 100 });
    expect(onstep).toHaveBeenCalledExactlyOnceWith(-1);
  });

  it('AC-ST1: given focus, when ArrowUp and ArrowDown are pressed, then onstep fires per key', async () => {
    const { onstep } = setup();
    await userEvent.tab();
    await userEvent.keyboard('{ArrowUp}{ArrowDown}');
    expect(onstep.mock.calls).toEqual([[1], [-1]]);
  });

  it('AC-ST1: given focus, when PageUp and PageDown are pressed, then onstep fires coarse', async () => {
    const { onstep } = setup();
    await userEvent.tab();
    await userEvent.keyboard('{PageUp}{PageDown}');
    expect(onstep.mock.calls).toEqual([
      [1, true],
      [-1, true],
    ]);
  });

  it('AC-ST1: given atMax, when the wheel turns up, then onstep is not called and the increase button is aria-disabled', async () => {
    const { onstep, value } = setup({ atMax: true });
    await fireEvent.wheel(value(), { deltaY: -100 });
    expect(onstep).not.toHaveBeenCalled();
    expect(screen.getByRole('button', { name: 'Increase' })).toHaveAttribute('aria-disabled', 'true');
  });

  it('AC-ST1: given atMax, when the increase button is clicked, then onstep is not called', async () => {
    const { onstep } = setup({ atMax: true });
    await userEvent.click(screen.getByRole('button', { name: 'Increase' }));
    expect(onstep).not.toHaveBeenCalled();
  });

  it('AC-ST1: given a double click on the value, when it happens, then onreset fires', async () => {
    const { onreset, value } = setup();
    await userEvent.dblClick(value());
    expect(onreset).toHaveBeenCalledOnce();
  });

  it('AC-ST1: given a valueText, when rendered, then aria-valuetext equals it', () => {
    const { value } = setup();
    expect(value()).toHaveAttribute('aria-valuetext', '500 millivolts per division');
  });

  it('given the decrease button, when clicked, then onstep(-1) fires', async () => {
    const { onstep } = setup();
    await userEvent.click(screen.getByRole('button', { name: 'Decrease' }));
    expect(onstep).toHaveBeenCalledExactlyOnceWith(-1);
  });

  it('AC-I8: given an off stepper, when the value is double-clicked or stepped, then nothing fires', async () => {
    const { onstep, onreset, value } = setup({ off: true });
    await userEvent.dblClick(value());
    await fireEvent.wheel(value(), { deltaY: -100 });
    expect(onreset).not.toHaveBeenCalled();
    expect(onstep).not.toHaveBeenCalled();
  });
});
