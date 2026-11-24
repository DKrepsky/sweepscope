import type { CommandName } from './generated/commands';
import type { FormattedValue } from './generated/FormattedValue';
import type { OffsetRow } from './generated/OffsetRow';
import type { TooltipText } from './generated/TooltipText';
import type { ZeroCalibrationState } from './generated/ZeroCalibrationState';
import type { ZeroSession } from './generated/ZeroSession';

type Channels = ZeroSession['doneChannels'];

export const DEVICE_LOST_TEXT = 'Device disconnected. Calibration cancelled. Nothing was stored.';
const EMPTY_TOOLTIP: TooltipText = {
  heading: '',
  channels: [0, 1, 2, 3].map(() => ({ cells: [] })) as TooltipText['channels'],
};
const NOT_SHORTED_LINE = 'Peak to peak 49.7 mV at 100 mV/div, limit 24.8 mV.';
const RANGES = ['2 mV', '5 mV', '10 mV', '20 mV', '50 mV', '100 mV', '200 mV', '500 mV', '1 V', '2 V', '5 V', '10 V'];
const IDLE: ZeroSession = {
  step: 'intro',
  channel: null,
  doneChannels: [false, false, false, false],
  progress: null,
  notShorted: null,
  canContinue: false,
  overridden: [false, false, false, false],
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
  /** Like `leaveUnshorted`, but the reading clips: `Continue` is not offered for it. */
  leaveClipped(...channels: number[]): void;
  loseDevice(): void;
  open(): void;
  start(): void;
  measure(): void;
  /** Accepts the rejected reading and resumes the sweep; the channel is stored as overridden. */
  continue(): void;
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
  private clipped: number[] = [];

  constructor(private readonly publish: (state: ZeroCalibrationState) => void) {}

  state = () => this.current;

  leaveUnshorted = (...channels: number[]) => void (this.unshorted = channels);

  leaveClipped = (...channels: number[]) => {
    this.clipped = channels;
    this.unshorted = channels;
  };

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
      const canContinue = !this.clipped.includes(channel);
      return this.show({ step: 'notShorted', channel, notShorted: [NOT_SHORTED_LINE], canContinue });
    }
    this.finishChannel(channel);
  };

  continue = () => {
    const held = this.current.session;
    if (!held?.channel || held.step !== 'notShorted' || !held.canContinue) return this.refuse('continue');
    const overridden = held.overridden.map((was, index) => was || index === held.channel! - 1) as Channels;
    this.show({ step: 'measuring', channel: held.channel, progress: 0.5, overridden });
    this.finishChannel(held.channel);
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
    const { doneChannels, overridden } = this.current.session ?? IDLE;
    this.change({ session: { ...IDLE, doneChannels, overridden, ...next } });
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

type ZeroCommand = 'open' | 'start' | 'measure' | 'continue' | 'close';

/** The commands answer from the session, like the shell does; `null` is their result. The names are typed loosely: `continue` is not in the generated contract yet. */
export const zeroCalibrationHandlers = (zero: FakeZeroCalibration) =>
  (['open', 'start', 'measure', 'continue', 'close'] as const).map(
    (action: ZeroCommand) => [`${action}_zero_calibration` as CommandName, () => (zero[action](), null)] as const,
  );
