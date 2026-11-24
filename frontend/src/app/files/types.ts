import type { BackendClient } from '../../api';

export interface FileActions {
  saveSetup(): Promise<void>;
  loadSetup(): Promise<void>;
  exportCsv(): Promise<void>;
  exportRaw(): Promise<void>;
  saveScreenshot(): Promise<void>;
}

export interface FileActionsDeps {
  client: BackendClient;
  notify: (kind: 'info' | 'warning' | 'error', text: string) => void;
  /** PNG bytes of the window as it looks now; supplied by the page layer. */
  composeScreenshot: () => Promise<Uint8Array>;
}
