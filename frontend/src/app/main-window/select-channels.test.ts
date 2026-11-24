import { describe, expect, it } from 'vitest';
import type { InstrumentState, MeasurementsState } from '../../api';
import { selectChannels } from './select-channels';

const channel = (number: number) =>
  ({
    number,
    enabled: true,
    range: { label: { text: '500', unit: 'mV' }, faster: { enabled: true }, slower: { enabled: true } },
    position: { text: '0.00', unit: 'div' },
    positionDivs: 0,
    coupling: 'dc',
    bandwidthLimit: false,
    probeText: '1x',
    invert: false,
    unverified: null,
    enable: { enabled: true },
  }) as unknown as InstrumentState['channels'][number];

const instrument = {
  controls: { vertical: { enabled: true } },
  channels: [channel(1), channel(2)],
} as unknown as InstrumentState;

const slice = (clippedChannel: number): MeasurementsState => ({
  revision: 1,
  accuracyNote: '',
  warnings: [],
  channels: [1, 2].map((number) => ({
    channel: number,
    clipped: number === clippedChannel,
    unverified: null,
    measurements: [],
  })),
});

describe('selectChannels clipping', () => {
  it('given channel 2 is clipped in the slice, when selected, then only channel 2 is flagged clipped', () => {
    const channels = selectChannels(undefined, instrument, 1, slice(2));

    expect(channels.map((model) => model.clipped ?? false)).toEqual([false, true]);
  });

  it('given no measurements slice, when selected, then no channel is flagged clipped', () => {
    const channels = selectChannels(undefined, instrument, 1);

    expect(channels.some((model) => model.clipped)).toBe(false);
  });
});
