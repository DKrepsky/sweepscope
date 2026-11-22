import { get } from 'svelte/store';
import { describe, expect, it } from 'vitest';
import { createNotifications } from './notifications';

describe('notifications', () => {
  it('given a new store, when read, then there is nothing to show', () => {
    expect(get(createNotifications())).toEqual([]);
  });

  it('given two notifications, when read, then the newest is first', () => {
    const store = createNotifications();
    store.notify({ kind: 'info', text: 'a' });
    store.notify({ kind: 'warning', text: 'b' });
    expect(get(store).map((n) => n.text)).toEqual(['b', 'a']);
  });

  it('given three notifications, when read, then only the two newest show', () => {
    const store = createNotifications();
    for (const text of ['a', 'b', 'c']) store.notify({ kind: 'info', text });
    expect(get(store).map((n) => n.text)).toEqual(['c', 'b']);
  });

  it('given three notifications and the newest dismissed, when read, then the older one shows again', () => {
    const store = createNotifications();
    for (const text of ['a', 'b', 'c']) store.notify({ kind: 'info', text });
    store.dismiss(get(store)[0]!.id);
    expect(get(store).map((n) => n.text)).toEqual(['b', 'a']);
  });

  it('given an id that does not exist, when dismissed, then nothing changes', () => {
    const store = createNotifications();
    store.notify({ kind: 'info', text: 'a' });
    store.dismiss(99);
    expect(get(store)).toHaveLength(1);
  });
});
