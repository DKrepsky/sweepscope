import { writable, type Readable } from 'svelte/store';

export interface FailureDismissal extends Pick<Readable<number>, 'subscribe'> {
  dismiss(id: number): void;
}

/** The id of the newest recording failure the user has dismissed; a failure with a higher id is new and shows again. */
export function createFailureDismissal(): FailureDismissal {
  const dismissed = writable(0);
  return {
    subscribe: dismissed.subscribe,
    dismiss: (id) => dismissed.update((current) => Math.max(current, id)),
  };
}
