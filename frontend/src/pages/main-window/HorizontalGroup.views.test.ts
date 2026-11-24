import { render, screen, within } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { get } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createMockMainWindow, type FftModel, type ViewsModel, type XyModel } from '../../app/main-window';
import HorizontalGroup from './HorizontalGroup.svelte';
import { spyActions } from './test-support';

beforeEach(() => {
  window.matchMedia = (() => ({ addEventListener: vi.fn(), removeEventListener: vi.fn() })) as never;
});

function setup(tab: ViewsModel['tab'], patch: { xy?: Partial<XyModel>; fft?: Partial<FftModel> } = {}) {
  const model = get(createMockMainWindow());
  const actions = spyActions();
  const views = {
    ...model.views,
    tab,
    xy: { ...model.views.xy, ...patch.xy },
    fft: { ...model.views.fft, ...patch.fft },
  };
  render(HorizontalGroup, { props: { horizontal: model.horizontal, views, actions } });
  return { actions, user: userEvent.setup() };
}

describe('HorizontalGroup in XY', () => {
  it('given XY, when rendered, then Trig pos and Interp are replaced by the axis selectors and Time/div stays', () => {
    setup('xy');
    expect(screen.queryByText('Trig pos')).toBeNull();
    expect(screen.queryByText('Interp')).toBeNull();
    expect(screen.getByText('Time/div')).toBeInTheDocument();
    expect(screen.getByRole('group', { name: 'X axis channel' })).toBeInTheDocument();
    expect(screen.getByRole('group', { name: 'Y axis channel' })).toBeInTheDocument();
  });

  it('given XY, when rendered, then the readline says the segments are straight', () => {
    setup('xy');
    expect(screen.getByText('Fs 1.25 MS/s · straight segments')).toBeInTheDocument();
  });

  it('given X is CH1, when CH3 is chosen on the X selector, then setXyAxis gets x and 3', async () => {
    const { actions, user } = setup('xy');
    const x = within(screen.getByRole('group', { name: 'X axis channel' }));
    await user.click(x.getByRole('button', { name: '3' }));
    expect(actions.setXyAxis).toHaveBeenCalledExactlyOnceWith('x', 3);
  });

  it('given a channel is off, when its segment is clicked, then nothing is chosen', async () => {
    const { actions, user } = setup('xy');
    const y = within(screen.getByRole('group', { name: 'Y axis channel' }));
    expect(y.getByRole('button', { name: '4' })).toHaveAttribute('aria-disabled', 'true');
    await user.click(y.getByRole('button', { name: '4' }));
    expect(actions.setXyAxis).not.toHaveBeenCalled();
  });

  it('given the other axis uses a channel, when rendered, then its segment is disabled', () => {
    const xOptions = get(createMockMainWindow()).views.xy.xOptions.map((option) =>
      option.channel === 2 ? { ...option, availability: { enabled: false, reason: 'Already used as Y' } } : option,
    );
    setup('xy', { xy: { xOptions } });
    const x = within(screen.getByRole('group', { name: 'X axis channel' }));
    expect(x.getByRole('button', { name: '2' })).toHaveAttribute('aria-disabled', 'true');
  });
});

describe('HorizontalGroup in FFT', () => {
  it('given FFT, when rendered, then Scale and Window replace Trig pos and Interp in place', () => {
    setup('fft');
    expect(screen.queryByText('Trig pos')).toBeNull();
    expect(screen.queryByText('Interp')).toBeNull();
    expect(screen.getByRole('group', { name: 'Spectrum scale' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Window: Hann' })).toBeInTheDocument();
    expect(screen.getByText('Fs 50 kS/s · bin 12.21 Hz (Fs / N)')).toBeInTheDocument();
  });

  it('given dBV, when Linear is clicked, then setSpectrumScale gets linear', async () => {
    const { actions, user } = setup('fft');
    await user.click(screen.getByRole('button', { name: 'Linear' }));
    expect(actions.setSpectrumScale).toHaveBeenCalledExactlyOnceWith('linear');
  });

  it('given FFT, when the window button is opened, then the four windows are offered and the current one is checked', async () => {
    const { user } = setup('fft');
    await user.click(screen.getByRole('button', { name: 'Window: Hann' }));
    const items = screen.getAllByRole('menuitemcheckbox').map((item) => item.textContent?.replace('✓', '').trim());
    expect(items).toEqual(['Rectangular', 'Hann', 'Hamming', 'Blackman']);
    expect(screen.getByRole('menuitemcheckbox', { name: /Hann/ })).toHaveAttribute('aria-checked', 'true');
  });

  it('given the window menu is open, when Blackman is chosen, then setSpectrumWindow gets blackman and the menu closes', async () => {
    const { actions, user } = setup('fft');
    await user.click(screen.getByRole('button', { name: 'Window: Hann' }));
    await user.click(screen.getByRole('menuitemcheckbox', { name: /Blackman/ }));
    expect(actions.setSpectrumWindow).toHaveBeenCalledExactlyOnceWith('blackman');
    expect(screen.queryByRole('menuitemcheckbox')).toBeNull();
  });

  it('given roll mode, when FFT is shown, then the scale and window controls are disabled with the reason', () => {
    const available = { enabled: false, reason: 'FFT is not available in roll mode: it needs a full record.' };
    setup('fft', { fft: { available } });
    expect(screen.getByRole('button', { name: 'Window: Hann' })).toHaveAttribute('aria-disabled', 'true');
    expect(screen.getByRole('button', { name: 'dBV' })).toHaveAttribute('aria-disabled', 'true');
  });

  it('given FFT, when the Time/div label is read, then its tooltip says it sets the sample rate', () => {
    setup('fft');
    expect(screen.getByText('Time/div')).toHaveAttribute('title', expect.stringContaining('sample rate'));
  });
});

describe('HorizontalGroup in Yt', () => {
  it('given Yt, when rendered, then the time domain rows are shown', () => {
    setup('yt');
    expect(screen.getByText('Trig pos')).toBeInTheDocument();
    expect(screen.getByText('Interp')).toBeInTheDocument();
  });
});
