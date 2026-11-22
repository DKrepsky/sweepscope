import type { Page } from '@playwright/test';
import { instrumentState, stateSnapshot } from '../../src/app/instrument/test-data';

export interface IpcCall {
  command: string;
  args: unknown;
}

/** A connected simulator with channels 1 to 3 on and 4 off, so the dock shows both row states. */
const readySnapshot = () => {
  const base = instrumentState();
  return stateSnapshot({
    instrument: { ...base, channels: base.channels.map((channel) => ({ ...channel, enabled: channel.number < 4 })) },
  });
};

/**
 * Replaces the Tauri IPC bridge in the page: a fixed ready snapshot, event listening and the frame stream accepted
 * (no frames arrive), and every other control command logged and acknowledged. The two settings commands work against
 * `sessionStorage` (so a reload behaves like a restart); an unknown command is refused like the real capability does.
 */
export async function stubTauri(page: Page, initialSettings?: Record<string, unknown>): Promise<void> {
  await page.addInitScript(
    ([initial, snapshot]) => {
      const KEY = '__ss_settings';
      const ACKNOWLEDGED = new Set([
        'plugin:event|listen',
        'open_frame_stream',
        'close_frame_stream',
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
      ]);
      const log: { command: string; args: unknown }[] = [];
      (window as unknown as Record<string, unknown>).__ipcLog = log;
      if (initial && !sessionStorage.getItem(KEY)) sessionStorage.setItem(KEY, JSON.stringify(initial));
      const read = (): Record<string, unknown> => JSON.parse(sessionStorage.getItem(KEY) ?? '{}');
      (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {
        transformCallback: () => 0,
        invoke: async (command: string, args: Record<string, unknown>) => {
          log.push({ command, args });
          if (command === 'load_ui_state') return read()[(args as { namespace: string }).namespace] ?? null;
          if (command === 'save_ui_state') {
            const entry = (args as { entry: { namespace: string } }).entry;
            sessionStorage.setItem(KEY, JSON.stringify({ ...read(), [entry.namespace]: entry }));
            return null;
          }
          if (command === 'get_snapshot') return snapshot;
          if (ACKNOWLEDGED.has(command)) return null;
          throw new Error(`Command ${command} not allowed by capability`);
        },
      };
    },
    [initialSettings, readySnapshot()] as const,
  );
}

export const ipcLog = (page: Page): Promise<IpcCall[]> =>
  page.evaluate(() => (window as unknown as { __ipcLog: IpcCall[] }).__ipcLog);
