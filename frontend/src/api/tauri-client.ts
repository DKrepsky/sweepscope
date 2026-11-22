import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { BackendClient } from './backend-client';
import type { CommandName } from './generated/commands';

/** Rust command name for every contract command. Exhaustive, so a new command cannot be forgotten. */
export const tauriCommandNames: Record<CommandName, string> = {
  load_ui_state: 'load_ui_state',
  save_ui_state: 'save_ui_state',
};

export interface TauriTransport {
  invoke(command: string, args: unknown): Promise<unknown>;
  listen(event: string, handler: (payload: unknown) => void): Promise<() => void>;
}

const tauriTransport: TauriTransport = {
  invoke: (command, args) => invoke(command, args as Record<string, unknown>),
  listen: (event, handler) => listen(event, (message) => handler(message.payload)),
};

function unsubscribeWhenReady(pending: Promise<() => void>): () => void {
  return () => {
    void pending.then((unlisten) => unlisten());
  };
}

export function createTauriClient(transport: TauriTransport = tauriTransport): BackendClient {
  return {
    send: (name, args) => transport.invoke(tauriCommandNames[name], args) as Promise<never>,
    subscribe: (name, listener) =>
      unsubscribeWhenReady(transport.listen(name, (payload) => listener(payload as never))),
    openStream: () => {
      throw new Error('Streams are not available yet');
    },
  };
}
