import type { FormattedValue } from './generated/FormattedValue';
import type { OffsetRow } from './generated/OffsetRow';
import type { ZeroCalibrationState } from './generated/ZeroCalibrationState';
import type { ZeroSession } from './generated/ZeroSession';

type Channels = ZeroSession['doneChannels'];

export const DEVICE_LOST_TEXT = 'Device disconnected. Calibration cancelled. Nothing was stored.';
const EMPTY_TOOLTIP = { rows: [], notes: [] };
const NOT_SHORTED_LINE = 'Peak to peak 49.7 mV at 100 mV/div, limit 24.8 mV.';
const RANGES = ['2 mV', '5 mV', '10 mV', '20 mV', '50 mV', '100 mV', '200 mV', '500 mV', '1 V', '2 V', '5 V', '10 V'];
const IDLE: ZeroSession = {
  step: 'intro',
  channel: null,
  doneChannels: [false, false, false, false],
  progress: null,
  notShorted: null,
  failure: null,
  table: null,
};

const quantity = (text: string, unit: string): FormattedValue => ({ value: Number(text) / 1000, text, unit });

const doneTable = (): OffsetRow[] =>
  RANGES.map((range) => {
    const [value = '', unit = ''] = range.split(' ');
    return {
      range: { value: Number(value), text: value, unit: `${unit}/div` },
      offsets: [quantity('0.66', 'mV'), quantity('1.04', 'mV'), quantity('0.83', 'mV'), quantity('0.83', 'mV')],
    };
  });

/** Plays the backend's zero calibration session: what the module answers, in the order it publishes it. */
export interface FakeZeroCalibration {
  state(): ZeroCalibrationState;
  /** Inputs that are not shorted when their turn comes; measuring them ends in the not shorted step. */
  leaveUnshorted(...channels: number[]): void;
  loseDevice(): void;
  open(): void;
  start(): void;
  measure(): void;
  close(): void;
}

class Session implements FakeZeroCalibration {
  private current: ZeroCalibrationState = {
    revision: 1,
    availability: { enabled: true, reason: null },
    unit: { calibrated: false, storedAtUtc: null, tooltip: EMPTY_TOOLTIP },
    session: null,
  };
  private unshorted: number[] = [];

  constructor(private readonly publish: (state: ZeroCalibrationState) => void) {}

  state = () => this.current;

  leaveUnshorted = (...channels: number[]) => void (this.unshorted = channels);

  loseDevice = () => {
    this.change({ availability: { enabled: false, reason: 'No device' } });
    if (this.current.session) this.show({ step: 'failed', failure: DEVICE_LOST_TEXT });
  };

  open = () => {
    if (!this.current.availability.enabled) this.refuse('open');
    if (!this.current.session) this.show({ step: 'intro' });
  };

  start = () => {
    if (this.current.session?.step !== 'intro') this.refuse('start');
    this.show({ step: 'channel', channel: 1 });
  };

  measure = () => {
    const held = this.current.session;
    const channel = held?.channel;
    if (!channel || (held.step !== 'channel' && held.step !== 'notShorted')) return this.refuse('measure');
    this.show({ step: 'measuring', channel, progress: 0.5 });
    if (this.unshorted.includes(channel)) {
      return this.show({ step: 'notShorted', channel, notShorted: [NOT_SHORTED_LINE] });
    }
    this.finishChannel(channel);
  };

  close = () => this.change({ session: null });

  private finishChannel(channel: number) {
    const held = this.current.session?.doneChannels ?? IDLE.doneChannels;
    const doneChannels = held.map((done, index) => done || index === channel - 1) as Channels;
    if (channel < 4) return this.show({ step: 'channel', channel: channel + 1, doneChannels });
    const unit = { calibrated: true, storedAtUtc: '2026-11-24T14:05:00Z', tooltip: EMPTY_TOOLTIP };
    this.change({ unit });
    this.show({ step: 'done', doneChannels, table: doneTable() });
  }

  private show(next: Partial<ZeroSession> & Pick<ZeroSession, 'step'>) {
    const doneChannels = this.current.session?.doneChannels ?? IDLE.doneChannels;
    this.change({ session: { ...IDLE, doneChannels, ...next } });
  }

  private change(patch: Partial<ZeroCalibrationState>) {
    this.current = { ...this.current, ...patch, revision: this.current.revision + 1 };
    this.publish(this.current);
  }

  private refuse(what: string): never {
    throw new Error(`Calibration cannot ${what} now`);
  }
}

export const createFakeZeroCalibration = (publish: (state: ZeroCalibrationState) => void): FakeZeroCalibration =>
  new Session(publish);

type ZeroCommand = 'open' | 'start' | 'measure' | 'close';

/** The four commands answer from the session, like the shell does; `null` is their result. */
export const zeroCalibrationHandlers = (zero: FakeZeroCalibration) =>
  (['open', 'start', 'measure', 'close'] as const).map(
    (action: ZeroCommand) => [`${action}_zero_calibration` as const, () => (zero[action](), null)] as const,
  );
