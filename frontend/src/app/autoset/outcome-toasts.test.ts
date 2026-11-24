import { writable } from 'svelte/store';
import { describe, expect, it, vi } from 'vitest';
import type { AcquisitionStatus } from '../../api';
import { acquisitionStatus } from '../instrument/test-data';
import { followAutosetOutcomes } from './outcome-toasts';

type Outcome = NonNullable<AcquisitionStatus['autoset']['outcome']>;

const withOutcome = (outcome: Outcome | null, running = false) =>
  acquisitionStatus({ autoset: { running, noSignal: outcome?.kind === 'noSignal', outcome } });

function setup() {
  const status = writable<AcquisitionStatus | undefined>(acquisitionStatus());
  const notify = vi.fn();
  const retry = vi.fn();
  followAutosetOutcomes({ status, notify, retry });
  return { status, notify, retry };
}

describe('autoset outcome toasts', () => {
  it('given a done outcome, when it arrives, then one info toast shows the backend text', () => {
    const { status, notify } = setup();

    status.set(withOutcome({ id: 1, kind: 'done', text: 'Auto-setup done. CH1 500 mV/div, 200 µs/div.' }));

    expect(notify.mock.calls).toEqual([['info', 'Auto-setup done. CH1 500 mV/div, 200 µs/div.', undefined]]);
  });

  it('given a no-signal outcome, when it arrives, then one info toast shows the backend text', () => {
    const { status, notify } = setup();

    status.set(withOutcome({ id: 1, kind: 'noSignal', text: 'Auto-setup: no signal on any enabled channel.' }));

    expect(notify.mock.calls).toEqual([['info', 'Auto-setup: no signal on any enabled channel.', undefined]]);
  });

  it('given a failed outcome, when it arrives, then a warning toast offers Retry that starts auto-setup again', () => {
    const { status, notify, retry } = setup();

    status.set(withOutcome({ id: 1, kind: 'failed', text: 'Auto-setup could not find a stable setting.' }));

    const [kind, , action] = notify.mock.calls[0]!;
    expect(kind).toBe('warning');
    expect(action.label).toBe('Retry');
    action.run();
    expect(retry).toHaveBeenCalledOnce();
  });

  it('given an outcome already shown, when the slice repeats with the same id, then no second toast shows', () => {
    const { status, notify } = setup();
    const outcome: Outcome = { id: 3, kind: 'done', text: 'Done' };

    status.set(withOutcome(outcome));
    status.set(withOutcome(outcome, true));

    expect(notify).toHaveBeenCalledOnce();
  });

  it('given two outcomes with different ids, when both arrive, then each shows once', () => {
    const { status, notify } = setup();

    status.set(withOutcome({ id: 1, kind: 'done', text: 'First' }));
    status.set(withOutcome({ id: 2, kind: 'done', text: 'Second' }));

    expect(notify.mock.calls.map((call) => call[1])).toEqual(['First', 'Second']);
  });

  it('given no outcome yet, when the status arrives, then nothing shows', () => {
    const { status, notify } = setup();

    status.set(withOutcome(null, true));

    expect(notify).not.toHaveBeenCalled();
  });
});
