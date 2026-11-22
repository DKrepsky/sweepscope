import { get } from 'svelte/store';
import { describe, expect, it } from 'vitest';
import { createNoticeStore } from './notice';

describe('notice store', () => {
  it('given a new store, when read, then there is no notice', () => {
    expect(get(createNoticeStore())).toBeNull();
  });

  it('given a shown notice, when it is dismissed, then there is no notice', () => {
    const store = createNoticeStore();
    store.show({ kind: 'error', text: 'x' });
    store.dismiss();
    expect(get(store)).toBeNull();
  });

  it('given a shown notice, when another is shown, then the latest replaces it', () => {
    const store = createNoticeStore();
    store.show({ kind: 'info', text: 'a' });
    store.show({ kind: 'error', text: 'b' });
    expect(get(store)).toEqual({ kind: 'error', text: 'b' });
  });
});
