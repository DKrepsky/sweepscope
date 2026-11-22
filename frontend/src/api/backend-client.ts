import type { CommandArgs, CommandName, CommandResult, EventName, EventPayload } from './generated/commands';

export interface BackendClient {
  send<C extends CommandName>(name: C, args: CommandArgs[C]): Promise<CommandResult[C]>;
  /** Returns the unsubscribe function. */
  subscribe<E extends EventName>(name: E, listener: (payload: EventPayload[E]) => void): () => void;
  /** Reserved so the interface does not change when binary streams arrive. */
  openStream(kind: never): never;
}
