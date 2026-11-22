import type { BackendClient, UiStateEntry } from '../../api';

export function loadUiState(client: BackendClient, namespace: string): Promise<UiStateEntry | null> {
  return client.send('load_ui_state', { namespace });
}

export async function saveUiState(client: BackendClient, entry: UiStateEntry): Promise<void> {
  await client.send('save_ui_state', { entry });
}
