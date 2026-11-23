import type { NotificationRequest } from '../notifications';

export interface ClipboardDeps {
  write: (text: string) => Promise<void>;
  notify: (request: NotificationRequest) => void;
}

export function createCopyText({ write, notify }: ClipboardDeps): (text: string) => void {
  return (text) => {
    void write(text).then(
      () => notify({ kind: 'info', text: 'Command copied' }),
      () => notify({ kind: 'error', text: 'Could not copy to the clipboard' }),
    );
  };
}
