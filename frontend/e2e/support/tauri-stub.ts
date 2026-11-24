import type { Page } from '@playwright/test';
import { instrumentState, stateSnapshot } from '../../src/app/instrument/test-data';

export interface IpcCall {
  command: string;
  args: unknown;
}

/** A connected simulator with channels 1 to 3 on and 4 off, so the dock shows both row states. */
const readySnapshot = (unverifiedTimeBase: boolean) => {
  const base = instrumentState();
  const timeBase = { ...base.horizontal.timeBase, verified: !unverifiedTimeBase };
  return stateSnapshot({
    instrument: {
      ...base,
      channels: base.channels.map((channel) => ({ ...channel, enabled: channel.number < 4 })),
      horizontal: { ...base.horizontal, timeBase },
    },
  });
};

export interface StubOptions {
  /** The selected time base step is one the hardware has not been measured at. */
  unverifiedTimeBase?: boolean;
  /** Per command: the value it returns, or an error message it rejects with (the M2 file commands and `set_interpolation`). */
  commands?: Record<string, { result?: unknown; error?: string }>;
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
      readySnapshot(options.unverifiedTimeBase ?? false),
      options.commands ?? {},
      ACKNOWLEDGED_COMMANDS,
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
