import { describe, expect, it } from 'vitest';
import type { Phase } from '../../api';
import { connectionChip, graphState, isAcquiring, statusChips } from './selectors';
import { acquisitionStatus, connectionSnapshot, instrumentState } from './test-data';

const ready = connectionSnapshot({ state: 'ready' });
const disconnected = connectionSnapshot({ state: 'disconnected', identity: null, source: null });

describe('isAcquiring', () => {
  it('given a ready connection and a running instrument, when selected, then it is acquiring', () => {
    expect(isAcquiring(ready, instrumentState({ running: true }))).toBe(true);
  });

  it('given a ready connection and a stopped instrument, when selected, then it is not acquiring', () => {
    expect(isAcquiring(ready, instrumentState({ running: false }))).toBe(false);
  });

  it('given a lost connection and a stale running flag, when selected, then it is not acquiring', () => {
    expect(isAcquiring(disconnected, instrumentState({ running: true }))).toBe(false);
  });

  it('given no slices yet, when selected, then it is not acquiring', () => {
    expect(isAcquiring(undefined, undefined)).toBe(false);
  });
});

describe('graphState', () => {
  it('given acquiring with a frame, when selected, then the graph is live', () => {
    expect(graphState(ready, instrumentState({ running: true }), true)).toBe('live');
  });

  it('given a disconnected scope with a last frame, when selected, then the graph is frozen', () => {
    expect(graphState(disconnected, instrumentState({ running: false }), true)).toBe('frozen');
  });

  it('given a stopped scope with a last frame, when selected, then the graph is frozen', () => {
    expect(graphState(ready, instrumentState({ running: false }), true)).toBe('frozen');
  });

  it('given no frame ever, when selected, then the graph is empty', () => {
    expect(graphState(ready, instrumentState({ running: true }), false)).toBe('empty');
  });
});

describe('statusChips', () => {
  const chips = (phase: Phase, running = true, connection = ready) =>
    statusChips(connection, instrumentState({ running }), acquisitionStatus({ phase }));

  it('given the waiting phase while running, when selected, then the chips are Run and Waiting for trigger', () => {
    expect(chips('waitingForTrigger').map((c) => c.text)).toEqual(['Run', 'Waiting for trigger']);
  });

  it('given the triggered phase, when selected, then the second chip is the filled trigger chip', () => {
    expect(chips('triggered')[1]).toMatchObject({ kind: 'triggered', filled: true });
  });

  it('given the auto phase, when selected, then the second chip is dim with an explanation', () => {
    expect(chips('auto')[1]).toMatchObject({ kind: 'auto', text: 'Auto', dim: true });
    expect(chips('auto')[1]?.tooltip).toBeTruthy();
  });

  it('given the armed phase, when selected, then the second chip says Armed', () => {
    expect(chips('armed')[1]).toMatchObject({ kind: 'armed', text: 'Armed' });
  });

  it('given a stopped instrument, when selected, then the only chip is Stop', () => {
    expect(chips('stopped', false)).toEqual([{ id: 'state', kind: 'stop', text: 'Stop' }]);
  });

  it('given a finished single capture, when selected, then the chips are Stop and Done', () => {
    expect(chips('done', false).map((c) => c.text)).toEqual(['Stop', 'Done']);
  });

  it('given a stopped instrument in a phase other than done, when selected, then the phase chip is not shown', () => {
    expect(chips('armed', false).map((c) => c.text)).toEqual(['Stop']);
  });

  it('given a disconnected scope, when selected, then the only chip is Offline', () => {
    expect(chips('armed', true, disconnected)).toEqual([{ id: 'state', kind: 'offline', text: 'Offline' }]);
  });

  it('given a connection error, when selected, then the only chip is an error chip', () => {
    const failed = connectionSnapshot({ state: 'error' });
    expect(chips('armed', true, failed).map((c) => c.kind)).toEqual(['error']);
  });

  it('given a connecting scope, when selected, then the chip says Connecting', () => {
    const connecting = connectionSnapshot({ state: 'connecting' });
    expect(chips('stopped', false, connecting)[0]?.text).toBe('Connecting');
  });

  it('given no slices yet, when selected, then the chip says Connecting', () => {
    expect(statusChips(undefined, undefined, undefined)[0]?.text).toBe('Connecting');
  });
});

describe('connectionChip', () => {
  it('given a ready simulator, when selected, then state is Connected, detail the model, and simulated', () => {
    expect(connectionChip(ready)).toMatchObject({ state: 'Connected', detail: 'Hantek 6254BC', simulated: true });
  });

  it('given a ready scope with firmware, when selected, then the tooltip names the firmware', () => {
    const identity = { ...ready.identity!, firmware: '1.2.3', simulated: false };
    const chip = connectionChip(connectionSnapshot({ identity, simulatorActive: false }));
    expect(chip.tooltip).toContain('1.2.3');
    expect(chip.simulated).toBe(false);
  });

  it('given a disconnected scope, when selected, then state is Disconnected', () => {
    expect(connectionChip(disconnected).state).toBe('Disconnected');
  });

  it('given a connection error with an issue, when selected, then the detail is the issue message', () => {
    const issue = { kind: 'communication', message: 'USB transfer failed', help: null } as const;
    expect(connectionChip(connectionSnapshot({ state: 'error', issue }))).toMatchObject({
      state: 'Error',
      detail: 'USB transfer failed',
    });
  });

  it('given no connection slice yet, when selected, then state is Connecting', () => {
    expect(connectionChip(undefined).state).toBe('Connecting');
  });
});
