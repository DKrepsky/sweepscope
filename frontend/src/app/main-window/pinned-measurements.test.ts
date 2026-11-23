import { get } from 'svelte/store';
import { describe, expect, it } from 'vitest';
import { createFakeClient } from '../../api';
import { createPinnedMeasurements, DEFAULT_PINNED } from './pinned-measurements';

const entry = (json: string, version = 1) => ({ namespace: 'measurements', version, json });

function setup(saved: ReturnType<typeof entry> | null = null) {
  const client = createFakeClient();
  client.respond('load_ui_state', () => saved);
  client.respond('save_ui_state', () => null);
  return { client, pinned: createPinnedMeasurements({ client }) };
}

const saves = (client: ReturnType<typeof setup>['client']) => client.calls.filter((c) => c.name === 'save_ui_state');

describe('pinned measurements', () => {
  it('given nothing saved, when loaded, then the defaults are pinned', async () => {
    const { pinned } = setup();
    await pinned.load();
    expect(get(pinned)).toEqual(DEFAULT_PINNED);
  });

  it('given a saved choice, when loaded, then it replaces the defaults', async () => {
    const { pinned } = setup(entry('{"pinned":["vmax","vmin"]}'));
    await pinned.load();
    expect(get(pinned)).toEqual(['vmax', 'vmin']);
  });

  it('given an unreadable blob, when loaded, then the defaults stay', async () => {
    const { pinned } = setup(entry('not json'));
    await pinned.load();
    expect(get(pinned)).toEqual(DEFAULT_PINNED);
  });

  it('given a blob of another version, when loaded, then the defaults stay', async () => {
    const { pinned } = setup(entry('{"pinned":["vmax"]}', 2));
    await pinned.load();
    expect(get(pinned)).toEqual(DEFAULT_PINNED);
  });

  it('given a row, when pinned, then it is added once and the choice is saved', async () => {
    const { client, pinned } = setup();
    await pinned.set('vmax', true);
    await pinned.set('vmax', true);
    expect(get(pinned).filter((id) => id === 'vmax')).toHaveLength(1);
    expect(saves(client)).toHaveLength(2);
  });

  it('given a pinned row, when unpinned, then it is gone and the saved blob omits it', async () => {
    const { client, pinned } = setup();
    await pinned.set('frequency', false);
    expect(get(pinned)).not.toContain('frequency');
    expect(JSON.stringify(saves(client)[0]?.args)).not.toContain('frequency');
  });

  it('given a save that fails, when a row is pinned, then the choice still holds for the session', async () => {
    const { client, pinned } = setup();
    client.respond('save_ui_state', () => Promise.reject(new Error('disk')));
    await pinned.set('vmax', true);
    expect(get(pinned)).toContain('vmax');
  });
});
