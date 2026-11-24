import { render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import type { PeakModel } from '../../app/main-window';
import PeakPanel from './PeakPanel.svelte';

const peak = (patch: Partial<PeakModel> = {}): PeakModel => ({
  mode: 'auto',
  channel: 2,
  bin: 82,
  frequency: '1.001 kHz',
  level: '−6.9 dBV',
  available: { enabled: true },
  note: 'Frequency resolution is one bin (12.21 Hz).',
  bins: 2049,
  marker: { position: 0.04, level: 0.8, step: 0.0005, valueText: '1.001 kHz −6.9 dBV' },
  ...patch,
});

function setup(model: PeakModel) {
  const onmode = vi.fn();
  render(PeakPanel, { props: { peak: model, onmode } });
  return { onmode, user: userEvent.setup() };
}

describe('PeakPanel', () => {
  it('given a marker, when rendered, then channel, bin, frequency and level are shown', () => {
    setup(peak());
    expect(screen.getByText('CH2')).toBeInTheDocument();
    expect(screen.getByText('82')).toBeInTheDocument();
    expect(screen.getByText('1.001 kHz')).toBeInTheDocument();
    expect(screen.getByText('−6.9 dBV')).toBeInTheDocument();
  });

  it('given the model, when rendered, then the footnote states the bin resolution', () => {
    setup(peak());
    expect(screen.getByText('Frequency resolution is one bin (12.21 Hz).')).toBeInTheDocument();
  });

  it('given Off, when rendered, then the bin shows a dash like the other values', () => {
    setup(peak({ mode: 'off', marker: undefined, frequency: '-', level: '-' }));
    expect(screen.getAllByText('-')).toHaveLength(3);
  });

  it('given Auto, when Manual is clicked, then the mode changes to manual', async () => {
    const { onmode, user } = setup(peak());
    await user.click(screen.getByRole('button', { name: 'Manual' }));
    expect(onmode).toHaveBeenCalledExactlyOnceWith('manual');
  });

  it('given Auto, when Off is clicked, then the mode changes to off', async () => {
    const { onmode, user } = setup(peak());
    await user.click(screen.getByRole('button', { name: 'Off' }));
    expect(onmode).toHaveBeenCalledExactlyOnceWith('off');
  });

  it('given the mode is already Auto, when Auto is clicked, then nothing changes', async () => {
    const { onmode, user } = setup(peak());
    await user.click(screen.getByRole('button', { name: 'Auto' }));
    expect(onmode).not.toHaveBeenCalled();
  });

  it('given no spectrum, when rendered, then the mode control is disabled and a click changes nothing', async () => {
    const { onmode, user } = setup(
      peak({ available: { enabled: false, reason: 'No spectrum yet' }, marker: undefined }),
    );
    expect(screen.getByRole('button', { name: 'Manual' })).toHaveAttribute('aria-disabled', 'true');
    await user.click(screen.getByRole('button', { name: 'Manual' }));
    expect(onmode).not.toHaveBeenCalled();
  });
});
