import { describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import { createFakeClient } from '../../api';
import type { UiStateEntry } from '../../api';
import { createNoticeStore } from '../notice';
import { chooseTextScale, createTextScaleStore, setTextScale } from './text-scale';

const stored = (json: string, version = 1): UiStateEntry => ({ namespace: 'app.text-scale', version, json });

function setup(loaded: UiStateEntry | null = null) {
  const client = createFakeClient();
  client.respond('load_ui_state', () => loaded);
  client.respond('save_ui_state', () => null);
  const applyRootFontPercent = vi.fn();
  const store = createTextScaleStore({ client, applyRootFontPercent });
  return { client, store, applyRootFontPercent };
}

describe('text scale load', () => {
  it('given no stored entry, when load runs, then the store is 100 and the root font size is 100 percent', async () => {
    const { store, applyRootFontPercent } = setup(null);
    await store.load();
    expect(get(store)).toBe(100);
    expect(applyRootFontPercent).toHaveBeenCalledExactlyOnceWith(100);
  });

  it('given a stored percent of 150, when load runs, then the store is 150 and the root font is applied once', async () => {
    const { store, applyRootFontPercent } = setup(stored('{"percent":150}'));
    await store.load();
    expect(get(store)).toBe(150);
    expect(applyRootFontPercent).toHaveBeenCalledExactlyOnceWith(150);
  });

  it.each(['not json', '{"percent":130}', '{"percent":"150"}', '{}', 'null'])(
    'given stored garbage %s, when load runs, then the store is 100 without throwing',
    async (json) => {
      const { store } = setup(stored(json));
      await store.load();
      expect(get(store)).toBe(100);
    },
  );

  it('given an unknown entry version, when load runs, then the store is 100', async () => {
    const { store } = setup(stored('{"percent":150}', 2));
    await store.load();
    expect(get(store)).toBe(100);
  });

  it('given the namespace app.text-scale, when load runs, then that namespace is requested', async () => {
    const { client, store } = setup(null);
    await store.load();
    expect(client.calls).toEqual([{ name: 'load_ui_state', args: { namespace: 'app.text-scale' } }]);
  });
});

describe('set text scale', () => {
  it('given the store at 100, when setTextScale 125 runs, then the entry is saved and afterwards the store is 125', async () => {
    const { client, store, applyRootFontPercent } = setup();
    await setTextScale(store, 125);
    expect(client.calls).toEqual([{ name: 'save_ui_state', args: { entry: stored('{"percent":125}') } }]);
    expect(get(store)).toBe(125);
    expect(applyRootFontPercent).toHaveBeenCalledExactlyOnceWith(125);
  });

  it('given a save that has not resolved, when setTextScale 125 runs, then the store is still 100', async () => {
    const { client, store } = setup();
    client.respond('save_ui_state', () => new Promise(() => {}));
    void setTextScale(store, 125);
    expect(get(store)).toBe(100);
  });

  it('given the backend rejects the save, when setTextScale 125 runs, then the store stays 100 and the error is thrown', async () => {
    const { client, store, applyRootFontPercent } = setup();
    const failure = { category: 'recoverable', message: 'disk full' };
    client.respond('save_ui_state', () => Promise.reject(failure));
    await expect(setTextScale(store, 125)).rejects.toBe(failure);
    expect(get(store)).toBe(100);
    expect(applyRootFontPercent).not.toHaveBeenCalled();
  });
});

describe('choose text scale', () => {
  it('given the backend rejects the save, when 125 is chosen, then an error notice is shown and the store stays 100', async () => {
    const { client, store } = setup();
    client.respond('save_ui_state', () => Promise.reject({ category: 'recoverable', message: 'disk full' }));
    const notices = createNoticeStore();

    await chooseTextScale(store, notices, 125);

    expect(get(notices)).toEqual({ kind: 'error', text: 'Could not save text size' });
    expect(get(store)).toBe(100);
  });

  it('given an earlier failure notice, when a save then succeeds, then the notice is cleared and the store is 125', async () => {
    const { store } = setup();
    const notices = createNoticeStore();
    notices.show({ kind: 'error', text: 'Could not save text size' });

    await chooseTextScale(store, notices, 125);

    expect(get(notices)).toBeNull();
    expect(get(store)).toBe(125);
  });
});
