import { derived, writable, type Readable } from 'svelte/store';

/** A button the toast offers, such as `Retry`. */
export interface NotificationAction {
  label: string;
  run: () => void;
}

export interface NotificationRequest {
  kind: 'info' | 'warning' | 'error';
  text: string;
  action?: NotificationAction;
}

export interface Notification extends NotificationRequest {
  id: number;
}

export interface Notifications extends Pick<Readable<Notification[]>, 'subscribe'> {
  notify(request: NotificationRequest): void;
  dismiss(id: number): void;
}

const MAX_SHOWN = 2;

/** The transient messages the window shows as toasts: the two newest, newest first (design 10.2). */
export function createNotifications(): Notifications {
  const all = writable<Notification[]>([]);
  let nextId = 1;
  return {
    subscribe: derived(all, (list) => list.slice(-MAX_SHOWN).reverse()).subscribe,
    notify: (request) => all.update((list) => [...list, { ...request, id: nextId++ }]),
    dismiss: (id) => all.update((list) => list.filter((entry) => entry.id !== id)),
  };
}
