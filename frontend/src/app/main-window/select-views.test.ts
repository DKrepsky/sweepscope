import { describe, expect, it } from 'vitest';
import { instrumentFacts, instrumentState, spectrumState } from '../instrument/test-data';
import { selectViews } from './select-views';
import { modelInputs } from './test-inputs';

const withChannelsOn = (...on: number[]) =>
  instrumentState({ channels: instrumentState().channels.map((c) => ({ ...c, enabled: on.includes(c.number) })) });

describe('selectViews XY', () => {
  it('given CH1 and CH3 on and Y = CH3, when read, then there is no problem', () => {
    const { xy } = selectViews(modelInputs({ instrument: withChannelsOn(1, 3), xyAxes: { x: 1, y: 3 } }));
    expect(xy.problem).toBeUndefined();
  });

  it('given Y is CH2 and CH2 is off, when read, then the note names the channel and offers to enable it', () => {
    const { xy } = selectViews(modelInputs({ instrument: withChannelsOn(1, 3), xyAxes: { x: 1, y: 2 } }));
    expect(xy.problem).toEqual({ title: 'XY cannot draw', text: 'CH2 (Y) is off.', enable: 2 });
  });

  it('given X is CH1 and CH1 is off, when read, then the note names the X axis', () => {
    const { xy } = selectViews(modelInputs({ instrument: withChannelsOn(2, 3), xyAxes: { x: 1, y: 2 } }));
    expect(xy.problem).toEqual({ title: 'XY cannot draw', text: 'CH1 (X) is off.', enable: 1 });
  });

  it('given only CH1 on, when read, then the note asks for a second channel and has no action', () => {
    const { xy } = selectViews(modelInputs({ instrument: withChannelsOn(1) }));
    expect(xy.problem).toEqual({
      title: 'XY needs two enabled channels',
      text: 'Only CH1 is on. Enable another channel.',
    });
  });

  it('given channel 1 is the X axis, when read, then the Y selector refuses it with the reason', () => {
    const { xy } = selectViews(modelInputs({ instrument: withChannelsOn(1, 2, 3), xyAxes: { x: 1, y: 2 } }));
    expect(xy.yOptions[0]?.availability).toEqual({ enabled: false, reason: 'Already used as X' });
    expect(xy.xOptions[1]?.availability).toEqual({ enabled: false, reason: 'Already used as Y' });
  });

  it('given channel 4 is off, when read, then both selectors disable it as off', () => {
    const { xy } = selectViews(modelInputs({ instrument: withChannelsOn(1, 2) }));
    expect(xy.xOptions[3]?.availability).toEqual({ enabled: false, reason: 'Channel off' });
    expect(xy.yOptions[3]?.availability).toEqual({ enabled: false, reason: 'Channel off' });
  });

  it('given the sample rate, when read, then the readline says the segments are straight', () => {
    const facts = instrumentFacts({ sampleRate: { value: 1.25e6, text: '1.25', unit: 'MS/s' } });
    expect(selectViews(modelInputs({ facts })).xy.readline).toBe('Fs 1.25 MS/s · straight segments');
  });
});

describe('selectViews FFT', () => {
  it('given a spectrum slice, when read, then the readout texts come from the backend', () => {
    const spectrum = spectrumState({
      span: { value: 25000, text: '25', unit: 'kHz' },
      binSpacing: { value: 12.21, text: '12.21', unit: 'Hz' },
      topLabel: '+10 dBV',
      window: 'blackman',
    });
    const { fft } = selectViews(modelInputs({ spectrum }));
    expect(fft.readout).toMatchObject({ span: '0 to 25 kHz', binSpacing: '12.21 Hz', topLabel: '+10 dBV' });
    expect(fft.windowLabel).toBe('Blackman');
  });

  it('given the rectangular window, when read, then the menu button text is short', () => {
    const { fft } = selectViews(modelInputs({ spectrum: spectrumState({ window: 'rectangular' }) }));
    expect(fft.windowShort).toBe('Rect');
  });

  it('given a spectrum frame, when read, then the scale step is shown per division', () => {
    const spectrumFrame = { bins: 2, binHz: 1, unitsPerDiv: 10, channels: [] };
    expect(selectViews(modelInputs({ spectrumFrame })).fft.readout.scale).toBe('10 dB/div');
  });

  it('given the sample rate and the bin spacing, when read, then the readline gives the formula', () => {
    const facts = instrumentFacts({ sampleRate: { value: 50e3, text: '50', unit: 'kS/s' } });
    const spectrum = spectrumState({ binSpacing: { value: 12.21, text: '12.21', unit: 'Hz' } });
    expect(selectViews(modelInputs({ facts, spectrum })).fft.readline).toBe('Fs 50 kS/s · bin 12.21 Hz (Fs / N)');
  });

  it('given no channel enabled, when read, then the note asks to enable a channel', () => {
    const { fft } = selectViews(modelInputs({ instrument: withChannelsOn() }));
    expect(fft.problem).toEqual({ title: 'No spectrum', text: 'Enable a channel to see its spectrum.' });
  });

  it('given roll mode, when read, then the FFT tab is disabled with the backend reason', () => {
    const base = instrumentState();
    const reason = 'FFT is not available in roll mode: it needs a full record.';
    const instrument = { ...base, controls: { ...base.controls, frameViews: { enabled: false, reason } } };
    const views = selectViews(modelInputs({ instrument }));
    expect(views.tabAvailability.fft).toEqual({ enabled: false, reason });
    expect(views.fft.available).toEqual({ enabled: false, reason });
  });
});
