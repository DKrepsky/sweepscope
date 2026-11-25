import type { Page } from '@playwright/test';
import type { TooltipText, ZeroCalibrationState } from '../../src/api';
import type { ChannelTooltip } from '../../src/api/generated/ChannelTooltip';
import { instrumentState, stateSnapshot } from '../../src/app/instrument/test-data';

export interface IpcCall {
  command: string;
  args: unknown;
}

/** A connected simulator with channels 1 to 3 on and 4 off, so the dock shows both row states. */
const readySnapshot = (zero: Partial<ZeroCalibrationState>) => {
  const base = instrumentState();
  const snapshot = stateSnapshot({
    instrument: {
      ...base,
      channels: base.channels.map((channel) => ({ ...channel, enabled: channel.number < 4 })),
    },
  });
  return { ...snapshot, zeroCalibration: { ...snapshot.zeroCalibration, ...zero } };
};

const RANGE_MV = [2, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000, 10000];
const rangeLabel = (mv: number) => (mv >= 1000 ? `${mv / 1000} V/div` : `${mv} mV/div`);
/** Offset text in the backend's form: signed millivolts, U+2212 for minus, `0 V` for zero. */
const offsetLabel = (mv: number) => {
  if (mv === 0) return '0 V';
  const digits = Math.abs(mv) < 10 ? 2 : Math.abs(mv) < 100 ? 1 : 0;
  return `${mv < 0 ? '\u2212' : '+'}${Number(Math.abs(mv).toFixed(digits))} mV`;
};
/** Stored offsets of the stub unit, millivolts per channel (CH3 negative, CH4 exactly zero); ranges from 50 mV/div up are zero. */
const STUB_MV = [0.66, 1.04, -0.83, 0];

/** The backend's tooltip texts in miniature: 12 ranges by 3 probes for each channel. Typical values when not calibrated. */
export function tooltipFor(calibrated: boolean): TooltipText {
  const cells = (mv: number): ChannelTooltip => ({
    cells: RANGE_MV.map((range, index) => {
      const value = calibrated ? (index < 4 ? mv : 0) : [0.66, 1.04, 0.83, 0.83][Math.min(index, 3)]!;
      const text = (factor: number) =>
        `${offsetLabel(Number((value * factor).toFixed(4)))} at ${rangeLabel(range * factor)}`;
      return [text(1), text(10), text(100)];
    }),
  });
  const [one, two, three, four] = STUB_MV.map(cells) as [
    ChannelTooltip,
    ChannelTooltip,
    ChannelTooltip,
    ChannelTooltip,
  ];
  return {
    heading: calibrated ? 'Zero error removed per channel and range.' : 'Typical zero error of one measured unit:',
    channels: [one, two, three, four],
  };
}

export interface StubOptions {
  /** Per command: the value it returns, or an error message it rejects with (the M2 file commands and `set_interpolation`). */
  commands?: Record<string, { result?: unknown; error?: string }>;
  /** Initial zero calibration slice (for example a calibrated unit). The four calibration commands then play the session. */
  zero?: Partial<ZeroCalibrationState>;
}

/** Commands the stub acknowledges without a script; each must be one the main window's capability grants. */
export const ACKNOWLEDGED_COMMANDS: string[] = [
  'plugin:event|unlisten',
  'open_frame_stream',
  'close_frame_stream',
  'open_roll_stream',
  'close_roll_stream',
  'open_spectrum_stream',
  'close_spectrum_stream',
  'set_running',
  'force_trigger',
  'set_channel',
  'step_setting',
  'reset_setting',
  'set_trigger_position',
  'set_trigger_level',
  'set_trigger',
  'use_simulator',
  'retry_connection',
  'set_interpolation',
];

/**
 * Replaces the Tauri IPC bridge in the page: a fixed ready snapshot, event listening and the frame stream accepted
 * (no frames arrive), and every other control command logged and acknowledged. The two settings commands work against
 * `sessionStorage` (so a reload behaves like a restart); an unknown command is refused like the real capability does.
 */
export async function stubTauri(
  page: Page,
  initialSettings?: Record<string, unknown>,
  options: StubOptions = {},
): Promise<void> {
  await page.addInitScript(
    ([initial, snapshot, scripted, acknowledged, tooltips]) => {
      const KEY = '__ss_settings';
      const ACKNOWLEDGED = new Set(acknowledged);
      const commands = scripted as Record<string, { result?: unknown; error?: string }>;
      const log: { command: string; args: unknown }[] = [];
      (window as unknown as Record<string, unknown>).__ipcLog = log;
      // Live driving: callbacks registered by the Tauri API, event listeners by name, and the open frame channel id.
      const callbacks = new Map<number, (message: unknown) => void>();
      const listeners = new Map<string, number[]>();
      let nextCallback = 1;
      let nextMessage = 0;
      let frameChannel: number | undefined;
      const w = window as unknown as Record<string, unknown>;
      w.__emit = (event: string, payload: unknown) => {
        for (const id of listeners.get(event) ?? []) callbacks.get(id)?.({ event, id, payload });
      };
      w.__sendFrame = (bytes: number[]) => {
        if (frameChannel === undefined) throw new Error('No frame channel is open');
        callbacks.get(frameChannel)?.({ index: nextMessage++, message: new Uint8Array(bytes).buffer });
      };
      // Zero calibration: the backend session in miniature (QA stub, mirrors the shell commands and the slice).
      const RANGES = [
        '2 mV',
        '5 mV',
        '10 mV',
        '20 mV',
        '50 mV',
        '100 mV',
        '200 mV',
        '500 mV',
        '1 V',
        '2 V',
        '5 V',
        '10 V',
      ];
      const stubTooltip = (calibrated: boolean) => (calibrated ? tooltips.calibrated : tooltips.typical);
      const quantity = (text: string, unit: string) => ({ value: Number(text) / 1000, text, unit });
      const table = RANGES.map((range) => {
        const [number = '', unit = ''] = range.split(' ');
        return {
          range: { value: Number(number), text: number, unit: unit + '/div' },
          offsets: [quantity('0.66', 'mV'), quantity('1.04', 'mV'), quantity('-0.83', 'mV'), quantity('0.83', 'mV')],
        };
      });
      let zero = (snapshot as { zeroCalibration: Record<string, unknown> }).zeroCalibration;
      const unshorted = new Set<number>();
      const clipped = new Set<number>();
      const idle = {
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
      const setZero = (patch: Record<string, unknown>) => {
        zero = { ...zero, ...patch, revision: (zero.revision as number) + 1 };
        (snapshot as Record<string, unknown>).zeroCalibration = zero;
        (w.__emit as (e: string, p: unknown) => void)('zero_calibration_changed', zero);
      };
      const show = (next: Record<string, unknown>) => {
        const held = zero.session as { doneChannels: boolean[]; overridden: boolean[] } | null;
        setZero({
          session: {
            ...idle,
            doneChannels: held?.doneChannels ?? idle.doneChannels,
            overridden: held?.overridden ?? idle.overridden,
            ...next,
          },
        });
      };
      const next = (held: { channel: number; doneChannels?: boolean[] }) => {
        const session = zero.session as { doneChannels: boolean[] };
        const doneChannels = session.doneChannels.map((done, index) => done || index === held.channel - 1);
        if (held.channel < 4) return show({ step: 'channel', channel: held.channel + 1, doneChannels });
        setZero({ unit: { calibrated: true, storedAtUtc: '2026-11-24T14:05:00Z', tooltip: stubTooltip(true) } });
        show({ step: 'done', doneChannels, table });
      };
      const zeroCommands: Record<string, () => void> = {
        open_zero_calibration: () => {
          if (!(zero.availability as { enabled: boolean }).enabled) throw new Error('Calibration cannot open now');
          if (!zero.session) show({ step: 'intro' });
        },
        start_zero_calibration: () => show({ step: 'channel', channel: 1 }),
        measure_zero_calibration: () => {
          const held = zero.session as { channel: number; doneChannels: boolean[] };
          if (unshorted.has(held.channel)) {
            return show({
              step: 'notShorted',
              channel: held.channel,
              notShorted: ['Peak to peak 49.7 mV at 100 mV/div, limit 24.8 mV.'],
              canContinue: !clipped.has(held.channel),
            });
          }
          next(held);
        },
        // Accepts the rejected reading: the channel is stored as overridden and the sweep carries on without checks.
        continue_zero_calibration: () => {
          const held = zero.session as { step: string; channel: number; canContinue: boolean; overridden: boolean[] };
          if (held?.step !== 'notShorted' || !held.canContinue)
            throw new Error('That is not possible at this step of the zero calibration');
          const overridden = held.overridden.map((was, index) => was || index === held.channel - 1);
          show({ overridden });
          next(held);
        },
        close_zero_calibration: () => setZero({ session: null }),
      };
      w.__zero = {
        leaveUnshorted: (channels: number[], clippedChannels: number[]) => {
          unshorted.clear();
          clipped.clear();
          channels.forEach((channel) => unshorted.add(channel));
          clippedChannels.forEach((channel) => clipped.add(channel));
        },
        loseDevice: () => {
          setZero({ availability: { enabled: false, reason: 'No device' } });
          if (zero.session)
            show({ step: 'failed', failure: 'Device disconnected. Calibration cancelled. Nothing was stored.' });
        },
        showMeasuring: (progress: number | null) => show({ step: 'measuring', channel: 1, progress }),
      };
      if (initial && !sessionStorage.getItem(KEY)) sessionStorage.setItem(KEY, JSON.stringify(initial));
      const read = (): Record<string, unknown> => JSON.parse(sessionStorage.getItem(KEY) ?? '{}');
      (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {
        transformCallback: (callback: (message: unknown) => void) => {
          const id = nextCallback++;
          callbacks.set(id, callback);
          return id;
        },
        unregisterCallback: (id: number) => callbacks.delete(id),
        invoke: async (command: string, args: Record<string, unknown>) => {
          log.push({ command, args });
          if (command === 'plugin:event|listen') {
            const { event, handler } = args as { event: string; handler: number };
            listeners.set(event, [...(listeners.get(event) ?? []), handler]);
            return handler;
          }
          if (command === 'open_frame_stream') frameChannel = (args as { onFrame: { id: number } }).onFrame.id;
          if (command === 'load_ui_state') return read()[(args as { namespace: string }).namespace] ?? null;
          if (command === 'save_ui_state') {
            const entry = (args as { entry: { namespace: string } }).entry;
            sessionStorage.setItem(KEY, JSON.stringify({ ...read(), [entry.namespace]: entry }));
            return null;
          }
          if (command === 'get_snapshot') return snapshot;
          const zeroCommand = zeroCommands[command];
          if (zeroCommand && !commands[command]) return (zeroCommand(), null);
          const scriptedCommand = commands[command];
          if (scriptedCommand?.error !== undefined) throw new Error(scriptedCommand.error);
          if (scriptedCommand) return scriptedCommand.result ?? null;
          if (ACKNOWLEDGED.has(command)) return null;
          throw new Error(`Command ${command} not allowed by capability`);
        },
      };
    },
    [
      initialSettings,
      readySnapshot(options.zero ?? {}),
      options.commands ?? {},
      ACKNOWLEDGED_COMMANDS,
      { calibrated: tooltipFor(true), typical: tooltipFor(false) },
    ] as const,
  );
}

/** Pushes a backend event (`connection_changed`, `instrument_changed`, `status_changed`, `facts_changed`) into the page. */
export const emit = (page: Page, event: string, payload: unknown): Promise<void> =>
  page.evaluate(
    ([name, body]) => (window as unknown as { __emit: (e: string, p: unknown) => void }).__emit(name as string, body),
    [event, payload] as const,
  );

/** Pushes one wire frame (ADR 0011) through the open frame channel. */
export const sendFrame = (page: Page, bytes: number[]): Promise<void> =>
  page.evaluate((data) => (window as unknown as { __sendFrame: (b: number[]) => void }).__sendFrame(data), bytes);

export const ipcLog = (page: Page): Promise<IpcCall[]> =>
  page.evaluate(() => (window as unknown as { __ipcLog: IpcCall[] }).__ipcLog);

/** Drives the stub's zero calibration session: inputs that fail the shorted check, a lost device, a measuring step. */
export const zeroStub = {
  /** Channels whose input fails the check; those in `clipped` also refuse Continue (the reading clipped). */
  leaveUnshorted: (page: Page, channels: number[], clipped: number[] = []): Promise<void> =>
    page.evaluate(
      ([list, clip]) =>
        (window as unknown as { __zero: { leaveUnshorted(c: number[], k: number[]): void } }).__zero.leaveUnshorted(
          list as number[],
          clip as number[],
        ),
      [channels, clipped] as const,
    ),
  loseDevice: (page: Page): Promise<void> =>
    page.evaluate(() => (window as unknown as { __zero: { loseDevice(): void } }).__zero.loseDevice()),
  showMeasuring: (page: Page, progress: number | null): Promise<void> =>
    page.evaluate(
      (p) => (window as unknown as { __zero: { showMeasuring(p: number | null): void } }).__zero.showMeasuring(p),
      progress,
    ),
};
