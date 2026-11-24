import { describe, expect, it } from 'vitest';
import { acquisitionStatus, instrumentState } from '../instrument/test-data';
import { selectToolbar } from './select-toolbar';
import { modelInputs } from './test-inputs';

const extras = { hasWaveform: true, cursorsOn: false };
const withTrigger = (sweep: 'auto' | 'normal') => ({
  ...instrumentState().trigger,
  sweep,
});

describe('single', () => {
  it('given the waiting phase while running with sweep auto, when selected, then the Single is armed', () => {
    const inputs = modelInputs({ status: acquisitionStatus({ phase: 'waitingForTrigger' }) });

    expect(selectToolbar(inputs, extras).single?.armed).toBe(true);
  });

  it('given the done phase, when selected, then the Single is not armed', () => {
    const inputs = modelInputs({
      status: acquisitionStatus({ phase: 'done' }),
      instrument: instrumentState({ running: false }),
    });

    expect(selectToolbar(inputs, extras).single?.armed).toBe(false);
  });

  it('given a stopped scope still in the waiting phase, when selected, then the Single is not armed', () => {
    const inputs = modelInputs({
      status: acquisitionStatus({ phase: 'waitingForTrigger' }),
      instrument: instrumentState({ running: false }),
    });

    expect(selectToolbar(inputs, extras).single?.armed).toBe(false);
  });

  it('given a normal sweep waiting for its trigger, when selected, then the Single is not shown as armed', () => {
    const inputs = modelInputs({
      status: acquisitionStatus({ phase: 'waitingForTrigger' }),
      instrument: instrumentState({ trigger: withTrigger('normal') }),
    });

    expect(selectToolbar(inputs, extras).single?.armed).toBe(false);
  });

  it('given the backend refuses Single with a reason, when selected, then the reason is carried', () => {
    const base = instrumentState();
    const controls = { ...base.controls, single: { enabled: false, reason: 'Auto-setup is running' } };

    const toolbar = selectToolbar(modelInputs({ instrument: instrumentState({ controls }) }), extras);

    expect(toolbar.single?.availability).toEqual({ enabled: false, reason: 'Auto-setup is running' });
  });
});

describe('chips', () => {
  it('given a single that finished, when selected, then the chips read Stop and Done', () => {
    const inputs = modelInputs({
      status: acquisitionStatus({ phase: 'done' }),
      instrument: instrumentState({ running: false }),
    });

    expect(selectToolbar(inputs, extras).chips.map((chip) => chip.text)).toEqual(['Stop', 'Done']);
  });

  it('given an armed single, when selected, then the chips read Run and Waiting for trigger', () => {
    const inputs = modelInputs({ status: acquisitionStatus({ phase: 'waitingForTrigger' }) });

    expect(selectToolbar(inputs, extras).chips.map((chip) => chip.text)).toEqual(['Run', 'Waiting for trigger']);
  });
});

describe('math tool', () => {
  it('given math is on in the backend, when selected, then the tool is on', () => {
    const base = instrumentState();
    const instrument = instrumentState({ math: { ...base.math, enabled: true } });

    expect(selectToolbar(modelInputs({ instrument }), extras).tools.math).toBe(true);
  });

  it('given roll mode, when selected, then the Math tool carries the backend reason', () => {
    const base = instrumentState();
    const reason = 'XY, FFT, persistence, math and cursors need a captured record; not available in roll mode.';
    const controls = { ...base.controls, frameViews: { enabled: false, reason } };

    const toolbar = selectToolbar(modelInputs({ instrument: instrumentState({ controls }) }), extras);

    expect(toolbar.mathAvailability).toEqual({ enabled: false, reason });
  });
});

describe('cursors tool', () => {
  const rollWithReason = () => {
    const base = instrumentState();
    const reason = 'cursors need a captured record; not available in roll mode.';
    const controls = { ...base.controls, frameViews: { enabled: false, reason } };
    return { reason, instrument: { ...base, horizontal: { ...base.horizontal, roll: true }, controls } };
  };

  it('given roll mode with a waveform drawn, when selected, then Cursors is disabled with the backend reason', () => {
    const { reason, instrument } = rollWithReason();

    const toolbar = selectToolbar(modelInputs({ instrument }), extras);

    expect(toolbar.cursorsAvailability).toEqual({ enabled: false, reason });
  });

  it('given roll mode with cursors already on, when selected, then Cursors stays enabled to close them', () => {
    const { instrument } = rollWithReason();

    const toolbar = selectToolbar(modelInputs({ instrument }), { hasWaveform: true, cursorsOn: true });

    expect(toolbar.cursorsAvailability).toEqual({ enabled: true });
  });
});

describe('run controls', () => {
  it('given auto-setup is running, when selected, then Run and Force carry the backend reason', () => {
    const base = instrumentState();
    const busy = { enabled: false, reason: 'Auto-setup is running' };
    const controls = { ...base.controls, run: busy, force: busy };

    const toolbar = selectToolbar(modelInputs({ instrument: instrumentState({ controls }) }), extras);

    expect([toolbar.availability, toolbar.force]).toEqual([busy, busy]);
  });

  it('given no instrument yet, when selected, then Single is unavailable with the connection reason', () => {
    const toolbar = selectToolbar(modelInputs({ instrument: undefined, connection: undefined }), extras);

    expect(toolbar.single?.availability).toEqual({ enabled: false, reason: 'Connecting' });
  });
});
