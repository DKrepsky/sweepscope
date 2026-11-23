import type { FileActions, FileActionsDeps } from './types';

const nothing = () => Promise.resolve();

/** STUB: the use cases (commands, toasts, warnings) arrive with WP15; until then every action does nothing. */
export const createFileActions: (deps: FileActionsDeps) => FileActions = () => ({
  saveSetup: nothing,
  loadSetup: nothing,
  exportCsv: nothing,
  exportRaw: nothing,
  saveScreenshot: nothing,
});
