import type { BackendClient } from './backend-client';
import type { CommandArgs, CommandName, CommandResult, EventName, EventPayload } from './generated/commands';

export type Call = { name: CommandName; args: CommandArgs[CommandName] };
type Handler<C extends CommandName> = (args: CommandArgs[C]) => CommandResult[C] | Promise<CommandResult[C]>;
type Listener = (payload: never) => void;

export type FakeClient = BackendClient & {
  respond<C extends CommandName>(name: C, handler: Handler<C>): void;
  emit<E extends EventName>(name: E, payload: EventPayload[E]): void;
  readonly calls: Call[];
};

export function createFakeClient(): FakeClient {
  const handlers = new Map<CommandName, Handler<CommandName>>();
  const listeners = new Map<EventName, Set<Listener>>();
  const calls: Call[] = [];

  return {
    calls,
    respond: (name, handler) => void handlers.set(name, handler as unknown as Handler<CommandName>),
    send: async (name, args) => {
      calls.push({ name, args });
      const handler = handlers.get(name);
      if (!handler) throw new Error(`No fake response registered for ${name}`);
      return (await handler(args)) as CommandResult[typeof name];
    },
    subscribe: (name, listener) => {
      const set = listeners.get(name) ?? new Set<Listener>();
      set.add(listener as Listener);
      listeners.set(name, set);
      return () => void set.delete(listener as Listener);
    },
    emit: (name, payload) => listeners.get(name)?.forEach((listener) => listener(payload as never)),
    openStream: () => {
      throw new Error('Streams are not available yet');
    },
  };
}
