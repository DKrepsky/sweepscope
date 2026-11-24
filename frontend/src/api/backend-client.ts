import type {
  CommandArgs,
  CommandName,
  CommandResult,
  EventName,
  EventPayload,
  StreamKind,
} from './generated/commands';

export interface StreamHandle {
  close(): void;
}

export interface BackendClient {
  send<C extends CommandName>(name: C, args: CommandArgs[C]): Promise<CommandResult[C]>;
  /** Sends a raw binary body instead of JSON arguments. */
  sendBytes<R>(command: string, bytes: Uint8Array): Promise<R>;
  /** Returns the unsubscribe function. */
  subscribe<E extends EventName>(name: E, listener: (payload: EventPayload[E]) => void): () => void;
  /** Binary messages of a backend stream; `close` stops the backend producer. */
  openStream(kind: StreamKind, onMessage: (data: ArrayBuffer) => void): StreamHandle;
}
