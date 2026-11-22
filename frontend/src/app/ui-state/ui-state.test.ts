import { describe, expect, it } from 'vitest';
import { createFakeClient } from '../../api';
import { loadUiState, saveUiState } from './ui-state';

describe('ui state services', () => {
  it('given a saved entry, when loaded through the client, then the stored entry is returned', async () => {
    const entry = { namespace: 'n', version: 1, json: '{}' };
    const client = createFakeClient();
    client.respond('save_ui_state', () => null);
    client.respond('load_ui_state', () => entry);
    await saveUiState(client, entry);
    await expect(loadUiState(client, 'n')).resolves.toEqual(entry);
    expect(client.calls.map((call) => call.name)).toEqual(['save_ui_state', 'load_ui_state']);
  });
});
