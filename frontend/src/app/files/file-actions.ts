import type { SavedFile } from '../../api/generated/SavedFile';
import type { FileActions, FileActionsDeps } from './types';

const FALLBACK_MESSAGE = 'The file action failed';

function messageOf(failure: unknown): string {
  const message = (failure as { message?: unknown } | undefined)?.message;
  return typeof message === 'string' && message ? message : FALLBACK_MESSAGE;
}

/** Each action asks the backend, which shows the file dialog: a null reply means the user cancelled. */
export function createFileActions({ client, notify, composeScreenshot }: FileActionsDeps): FileActions {
  const reportSaved = (saved: SavedFile | null) => {
    if (saved) notify('info', `Saved ${saved.fileName}`);
  };

  const guarded = async (action: () => Promise<void>) => {
    try {
      await action();
    } catch (failure) {
      notify('error', messageOf(failure));
    }
  };

  const exportAs = (format: 'csv' | 'raw') =>
    guarded(async () => reportSaved(await client.send('export_frame', { format })));

  return {
    saveSetup: () => guarded(async () => reportSaved(await client.send('save_setup', {}))),
    loadSetup: () =>
      guarded(async () => {
        const loaded = await client.send('load_setup', {});
        if (!loaded) return;
        if (loaded.warnings.length > 0)
          notify('warning', `Loaded ${loaded.fileName} with changes: ${loaded.warnings.join('; ')}`);
        else notify('info', `Loaded ${loaded.fileName}`);
      }),
    exportCsv: () => exportAs('csv'),
    exportRaw: () => exportAs('raw'),
    saveScreenshot: () =>
      guarded(async () => {
        const png = await composeScreenshot();
        reportSaved(await client.sendBytes<SavedFile | null>('save_screenshot', png));
      }),
  };
}
