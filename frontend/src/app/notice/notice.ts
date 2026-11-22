import { writable, type Readable } from 'svelte/store';

export interface Notice {
  kind: 'info' | 'warning' | 'error';
  text: string;
}

export interface NoticeStore extends Pick<Readable<Notice | null>, 'subscribe'> {
  show(notice: Notice): void;
  dismiss(): void;
}

/** Holds the single transient message the window shows as a toast. */
export function createNoticeStore(): NoticeStore {
  const current = writable<Notice | null>(null);
  return {
    subscribe: current.subscribe,
    show: (notice) => current.set(notice),
    dismiss: () => current.set(null),
  };
}
