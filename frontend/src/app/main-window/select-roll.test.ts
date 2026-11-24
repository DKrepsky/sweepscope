import { describe, expect, it } from 'vitest';
import type { InstrumentState } from '../../api';
import { instrumentState } from '../instrument/test-data';
import { selectRoll } from './select-roll';
import { modelInputs } from './test-inputs';

/** 1 s/div: the window spans 10 s over 2500 samples, 4 ms each. */
const rolling = (): InstrumentState => {
  const base = instrumentState();
  return {
    ...base,
    horizontal: {
      ...base.horizontal,
      roll: true,
      windowText: '10 s',
      timeBase: { ...base.horizontal.timeBase, label: { value: 1, text: '1', unit: 's/div' } },
    },
  };
};

const gap = (missed: number, from = 0.2, to = 0.4) => ({ from, to, missed, approximate: true as const });

describe('selectRoll', () => {
  it('given a triggered time base, when read, then roll is inactive and nothing is shown', () => {
    expect(selectRoll(modelInputs())).toEqual({ active: false, window: '', gaps: [], gapCount: 0, announcement: '' });
  });

  it('given roll mode, when read, then the window text is the backend one', () => {
    expect(selectRoll(modelInputs({ instrument: rolling() })).window).toBe('10 s');
  });

  it('given a gap of 300 missed samples at 4 ms each, when read, then the band says GAP 1.20 s and keeps its place', () => {
    const roll = { channels: [], gaps: [gap(300, 0.1, 0.22)], gapCount: 1 };
    const model = selectRoll(modelInputs({ instrument: rolling(), roll }));
    expect(model.gaps).toEqual([{ from: 0.1, to: 0.22, label: 'GAP 1.20 s' }]);
    expect(model.gapCount).toBe(1);
  });

  it('given a gap, when read, then the announcement names its duration', () => {
    const roll = { channels: [], gaps: [gap(300)], gapCount: 1 };
    expect(selectRoll(modelInputs({ instrument: rolling(), roll })).announcement).toBe('Data gap of 1.20 s');
  });

  it('given two gaps, when read, then the announcement is the newest', () => {
    const roll = { channels: [], gaps: [gap(300), gap(25)], gapCount: 2 };
    expect(selectRoll(modelInputs({ instrument: rolling(), roll })).announcement).toBe('Data gap of 100 ms');
  });

  it('given a gap that scrolled out, when read, then the count still includes it', () => {
    const roll = { channels: [], gaps: [], gapCount: 3 };
    expect(selectRoll(modelInputs({ instrument: rolling(), roll })).gapCount).toBe(3);
  });

  it('given no time base value, when a gap is read, then it is labelled without a duration', () => {
    const base = rolling();
    const instrument = {
      ...base,
      horizontal: {
        ...base.horizontal,
        timeBase: { ...base.horizontal.timeBase, label: { value: 0, text: '-', unit: '' } },
      },
    };
    const roll = { channels: [], gaps: [gap(300)], gapCount: 1 };
    const model = selectRoll(modelInputs({ instrument, roll }));
    expect(model.gaps[0]?.label).toBe('GAP');
    expect(model.announcement).toBe('Data gap');
  });

  it('given a stale window while not rolling, when read, then no gaps are shown', () => {
    const roll = { channels: [], gaps: [gap(300)], gapCount: 1 };
    expect(selectRoll(modelInputs({ roll })).gaps).toEqual([]);
  });
});
