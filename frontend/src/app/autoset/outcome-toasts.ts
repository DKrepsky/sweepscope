import type { Readable, Unsubscriber } from 'svelte/store';
import type { AcquisitionStatus } from '../../api';
import type { NotificationAction } from '../notifications';

type OutcomeKind = NonNullable<AcquisitionStatus['autoset']['outcome']>['kind'];

export interface OutcomeToastDeps {
  status: Pick<Readable<AcquisitionStatus | undefined>, 'subscribe'>;
  notify: (kind: 'info' | 'warning', text: string, action?: NotificationAction) => void;
  retry: () => void;
}

const TOAST_KIND: Record<OutcomeKind, 'info' | 'warning'> = { done: 'info', noSignal: 'info', failed: 'warning' };

/** One toast per outcome id, so a repeat of the same slice or a reconnect stays silent; a failure offers `Retry`. */
export function followAutosetOutcomes({ status, notify, retry }: OutcomeToastDeps): Unsubscriber {
  let lastShownId: number | undefined;
  return status.subscribe((value) => {
    const outcome = value?.autoset.outcome;
    if (!outcome || outcome.id === lastShownId) return;
    lastShownId = outcome.id;
    const action = outcome.kind === 'failed' ? { label: 'Retry', run: retry } : undefined;
    notify(TOAST_KIND[outcome.kind], outcome.text, action);
  });
}
