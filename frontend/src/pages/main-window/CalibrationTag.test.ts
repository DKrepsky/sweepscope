import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { CalibrationTagModel } from '../../app/calibration';
import CalibrationTag from './CalibrationTag.svelte';

const tooltip = {
  heading: 'Zero error removed per channel and range.',
  lines: ['CH1 +0.66 mV at 2 mV/div', 'CH3 0 V at 1 V/div'],
};

const none: CalibrationTagModel = { state: 'none', tooltip };
const calibrated: CalibrationTagModel = { state: 'calibrated', tooltip, storedAt: '2026-11-24 14:05' };
const unavailable: CalibrationTagModel = { state: 'unavailable', reason: 'No device' };

afterEach(() => vi.useRealTimers());

function setup(tag: CalibrationTagModel) {
  const onopen = vi.fn();
  render(CalibrationTag, { tag, onopen });
  return { onopen, user: userEvent.setup() };
}

describe('CalibrationTag', () => {
  it('given no calibration, when rendered, then the button says so and offers the dialog', () => {
    setup(none);

    expect(screen.getByRole('button', { name: 'No calibration. Open calibration dialog' })).toHaveTextContent(
      'No calibration',
    );
  });

  it('given a calibrated unit, when rendered, then the button says Calibrated', () => {
    setup(calibrated);

    expect(screen.getByRole('button', { name: 'Calibrated. Open calibration dialog' })).toHaveTextContent('Calibrated');
  });

  it('given the tag is clicked, when it is available, then the dialog is asked to open', async () => {
    const { onopen, user } = setup(none);

    await user.click(screen.getByRole('button'));

    expect(onopen).toHaveBeenCalledTimes(1);
  });

  it('given the tag has focus, when Enter is pressed, then the dialog is asked to open', async () => {
    const { onopen, user } = setup(calibrated);
    await user.tab();

    await user.keyboard('{Enter}');

    expect(onopen).toHaveBeenCalledTimes(1);
  });

  it('given an unavailable tag, when rendered, then it is a disabled button whose title is the reason', async () => {
    const { onopen, user } = setup(unavailable);
    const button = screen.getByRole('button');

    await user.click(button);

    expect(button).toBeDisabled();
    expect(button).toHaveTextContent('Calibration -');
    expect(button).toHaveAttribute('title', 'No device');
    expect(onopen).not.toHaveBeenCalled();
  });

  it('given an unavailable tag for a recording, when rendered, then the title says to stop recording', () => {
    setup({ state: 'unavailable', reason: 'Stop recording first' });

    expect(screen.getByRole('button')).toHaveAttribute('title', 'Stop recording first');
  });

  it('given keyboard focus on the tag, when it arrives, then the tooltip shows the heading and the lines at once', async () => {
    const { user } = setup(calibrated);

    await user.tab();

    const tip = screen.getByRole('tooltip');
    expect(tip).toHaveTextContent('Calibrated 2026-11-24 14:05');
    expect(tip).toHaveTextContent('Zero error removed per channel and range.');
    expect(tip).toHaveTextContent('CH1 +0.66 mV at 2 mV/div');
    expect(tip).toHaveTextContent('CH3 0 V at 1 V/div');
    expect(tip).toHaveTextContent('Click to calibrate again.');
    expect(screen.getByRole('button')).toHaveAttribute('aria-describedby', tip.id);
  });

  it('given no calibration, when the tooltip shows, then it has the plain hint', async () => {
    const { user } = setup(none);

    await user.tab();

    const tip = screen.getByRole('tooltip');
    expect(tip).toHaveTextContent('No calibration');
    expect(tip).toHaveTextContent('Click to calibrate.');
    expect(tip).not.toHaveTextContent('again');
  });

  it('given a hover, when the delay has not passed, then no tooltip shows and after it the tooltip appears', async () => {
    vi.useFakeTimers();
    const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
    render(CalibrationTag, { tag: none, onopen: () => {} });

    await user.hover(screen.getByRole('button'));
    vi.advanceTimersByTime(499);
    expect(screen.queryByRole('tooltip')).toBeNull();
    vi.advanceTimersByTime(1);

    expect(await screen.findByRole('tooltip')).toBeInTheDocument();
  });

  it('given the tooltip is open, when Escape is pressed, then it closes', async () => {
    const { user } = setup(none);
    await user.tab();

    await user.keyboard('{Escape}');

    expect(screen.queryByRole('tooltip')).toBeNull();
  });

  it('given a calibrated tag without a date, when the tooltip shows, then the heading is plain', async () => {
    const { user } = setup({ state: 'calibrated', tooltip, storedAt: undefined });

    await user.tab();

    expect(screen.getByRole('tooltip').querySelector('b')).toHaveTextContent(/^Calibrated$/);
  });

  it('given lines for CH1 and CH3, when the tooltip shows, then each channel name takes its channel colour', async () => {
    const { user } = setup(calibrated);

    await user.tab();

    const names = screen.getByRole('tooltip').querySelectorAll<HTMLElement>('.ch');
    expect([...names].map((name) => name.textContent)).toEqual(['CH1', 'CH3']);
    expect([...names].map((name) => name.parentElement?.style.getPropertyValue('--chc'))).toEqual([
      'var(--ss-color-ch1)',
      'var(--ss-color-ch3)',
    ]);
  });

  it('given no channel enabled, when the tooltip shows, then a muted line says so and the hint stays', async () => {
    const { user } = setup({
      state: 'calibrated',
      tooltip: { heading: 'Zero error removed.', lines: [] },
      storedAt: undefined,
    });

    await user.tab();

    const tip = screen.getByRole('tooltip');
    expect(tip).toHaveTextContent('No channel enabled');
    expect(tip.querySelector('li')).toBeNull();
    expect(tip).toHaveTextContent('Click to calibrate again.');
  });
});
