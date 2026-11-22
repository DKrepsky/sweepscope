import type { Page } from '@playwright/test';

export interface IpcCall {
  command: string;
  args: unknown;
}

/**
 * Replaces the Tauri IPC bridge in the page. The two settings commands work against `sessionStorage`
 * (so a reload behaves like a restart); any other command is refused like the real capability does.
 */
export async function stubTauri(page: Page, initialSettings?: Record<string, unknown>): Promise<void> {
  await page.addInitScript((initial) => {
    const KEY = '__ss_settings';
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
        throw new Error(`Command ${command} not allowed by capability`);
      },
    };
  }, initialSettings);
}

export const ipcLog = (page: Page): Promise<IpcCall[]> =>
  page.evaluate(() => (window as unknown as { __ipcLog: IpcCall[] }).__ipcLog);
