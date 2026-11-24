import type { Page } from '@playwright/test';
import type { ZeroCalibrationState } from '../../src/api';
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
    ([initial, snapshot, scripted, acknowledged]) => {
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
      const idle = {
        step: 'intro',
        channel: null,
        doneChannels: [false, false, false, false],
        progress: null,
        notShorted: null,
        failure: null,
        table: null,
      };
      const setZero = (patch: Record<string, unknown>) => {
        zero = { ...zero, ...patch, revision: (zero.revision as number) + 1 };
        (snapshot as Record<string, unknown>).zeroCalibration = zero;
        (w.__emit as (e: string, p: unknown) => void)('zero_calibration_changed', zero);
      };
      const show = (next: Record<string, unknown>) => {
        const held = (zero.session as { doneChannels: boolean[] } | null)?.doneChannels ?? idle.doneChannels;
        setZero({ session: { ...idle, doneChannels: held, ...next } });
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
            });
          }
          const doneChannels = held.doneChannels.map((done, index) => done || index === held.channel - 1);
          if (held.channel < 4) return show({ step: 'channel', channel: held.channel + 1, doneChannels });
          const unit = {
            calibrated: true,
            storedAtUtc: '2026-11-24T14:05:00Z',
            tooltip: {
              rows: [{ label: 'CH1', value: '+0.66 mV at 2 mV/div' }],
              notes: ['Zero error removed per channel and range.'],
            },
          };
          setZero({ unit });
          show({ step: 'done', doneChannels, table });
        },
        close_zero_calibration: () => setZero({ session: null }),
      };
      w.__zero = {
        leaveUnshorted: (channels: number[]) => {
          unshorted.clear();
          channels.forEach((channel) => unshorted.add(channel));
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
    [initialSettings, readySnapshot(options.zero ?? {}), options.commands ?? {}, ACKNOWLEDGED_COMMANDS] as const,
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
  leaveUnshorted: (page: Page, channels: number[]): Promise<void> =>
    page.evaluate(
      (list) => (window as unknown as { __zero: { leaveUnshorted(c: number[]): void } }).__zero.leaveUnshorted(list),
      channels,
    ),
  loseDevice: (page: Page): Promise<void> =>
    page.evaluate(() => (window as unknown as { __zero: { loseDevice(): void } }).__zero.loseDevice()),
  showMeasuring: (page: Page, progress: number | null): Promise<void> =>
    page.evaluate(
      (p) => (window as unknown as { __zero: { showMeasuring(p: number | null): void } }).__zero.showMeasuring(p),
      progress,
    ),
};
