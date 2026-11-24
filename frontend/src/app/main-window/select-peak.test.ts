import { describe, expect, it } from 'vitest';
import { spectrumState } from '../instrument/test-data';
import type { SpectrumFrameInput } from '../views';
import { selectPeak } from './select-peak';
import { modelInputs } from './test-inputs';

/** Eleven bins of 100 Hz: bin 3 is the backend's peak at -6.9 dBV, level divisions 1.5 from the top. */
const spectrumFrame = (peakBin: number | undefined = 3): SpectrumFrameInput => {
  const levels = new Float32Array(11).fill(-60);
  const divisions = new Float32Array(11).fill(6.5);
  levels[3] = -6.9;
  divisions[3] = 1.5;
  levels[5] = -30;
  divisions[5] = 4;
  return { bins: 11, binHz: 100, unitsPerDiv: 10, channels: [{ channel: 1, peakBin, levels, divisions }] };
};

describe('selectPeak', () => {
  it('given Auto, when read, then the marker sits on the backend peak with its level and texts', () => {
    const peak = selectPeak(modelInputs({ spectrumFrame: spectrumFrame() }));
    expect(peak.bin).toBe(3);
    expect(peak.frequency).toBe('300.0 Hz');
    expect(peak.level).toBe('−6.9 dBV');
    expect(peak.marker).toEqual({ position: 0.3, level: 1 - 1.5 / 8, step: 0.1, valueText: '300.0 Hz −6.9 dBV' });
  });

  it('given Auto, when a new spectrum has another peak, then the marker follows it', () => {
    const peak = selectPeak(modelInputs({ spectrumFrame: spectrumFrame(5) }));
    expect(peak.bin).toBe(5);
    expect(peak.marker?.valueText).toBe('500.0 Hz −30.0 dBV');
  });

  it('given Manual at bin 5, when the backend peak is elsewhere, then the marker stays on bin 5', () => {
    const peak = selectPeak(
      modelInputs({ spectrumFrame: spectrumFrame(3), peak: { mode: 'manual', channel: 1, bin: 5 } }),
    );
    expect(peak.bin).toBe(5);
    expect(peak.marker?.position).toBe(0.5);
  });

  it('given Off, when read, then there is no marker and every value is a dash', () => {
    const peak = selectPeak(modelInputs({ spectrumFrame: spectrumFrame(), peak: { mode: 'off', channel: 1, bin: 0 } }));
    expect(peak.marker).toBeUndefined();
    expect(peak.frequency).toBe('-');
    expect(peak.level).toBe('-');
  });

  it('given the selected channel has no spectrum, when read, then there is no marker', () => {
    const peak = selectPeak(
      modelInputs({ spectrumFrame: spectrumFrame(), peak: { mode: 'auto', channel: 2, bin: 0 } }),
    );
    expect(peak.marker).toBeUndefined();
  });

  it('given the marker, when read, then the panel and the marker text are the same numbers', () => {
    const peak = selectPeak(modelInputs({ spectrumFrame: spectrumFrame() }));
    expect(peak.marker?.valueText).toBe(`${peak.frequency} ${peak.level}`);
  });

  it('given the linear scale, when read, then the level is in volts', () => {
    const frame = spectrumFrame();
    (frame.channels[0]!.levels as Float32Array)[3] = 0.5;
    const peak = selectPeak(modelInputs({ spectrumFrame: frame, spectrum: spectrumState({ scale: 'linear' }) }));
    expect(peak.level).toBe('500 mV');
  });

  it('given the spectrum slice, when read, then the note gives the bin spacing', () => {
    expect(selectPeak(modelInputs({ spectrumFrame: spectrumFrame() })).note).toBe(
      'Frequency resolution is one bin (12.2 Hz).',
    );
  });

  it('given no spectrum yet, when read, then the mode control is unavailable with a reason', () => {
    expect(selectPeak(modelInputs()).available).toEqual({ enabled: false, reason: 'No spectrum yet' });
  });

  it('given a spectrum, when read, then the mode control is available and the bin count is known', () => {
    const peak = selectPeak(modelInputs({ spectrumFrame: spectrumFrame() }));
    expect(peak.available.enabled).toBe(true);
    expect(peak.bins).toBe(11);
  });
});
