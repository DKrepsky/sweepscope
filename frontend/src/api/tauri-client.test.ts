import { describe, expect, it, vi } from 'vitest';
import { COMMANDS } from './generated/commands';
import { createTauriClient, tauriCommandNames, type TauriTransport } from './tauri-client';

function fakeTransport(overrides: Partial<TauriTransport> = {}): TauriTransport {
  return {
    invoke: vi.fn().mockResolvedValue(null),
    listen: vi.fn().mockResolvedValue(() => {}),
    ...overrides,
  };
}

describe('tauri client contract', () => {
  it('given the generated command list, when compared with the keys the Tauri client maps, then they are equal', () => {
    expect(Object.keys(tauriCommandNames).sort()).toEqual([...COMMANDS].sort());
  });

  it('given a command, when sent, then the transport receives the mapped name and the arguments untouched', async () => {
    const transport = fakeTransport({ invoke: vi.fn().mockResolvedValue({ namespace: 'a', version: 1, json: '{}' }) });
    const result = await createTauriClient(transport).send('load_ui_state', { namespace: 'a' });
    expect(transport.invoke).toHaveBeenCalledExactlyOnceWith('load_ui_state', { namespace: 'a' });
    expect(result).toEqual({ namespace: 'a', version: 1, json: '{}' });
  });

  it('given the transport rejects with a kernel error, when sent, then the same error reaches the caller', async () => {
    const failure = { category: 'fatal', message: 'boom' };
    const transport = fakeTransport({ invoke: vi.fn().mockRejectedValue(failure) });
    await expect(createTauriClient(transport).send('load_ui_state', { namespace: 'a' })).rejects.toBe(failure);
  });

  it('given a subscription, when unsubscribed after the transport resolved, then the transport listener is removed', async () => {
    const unlisten = vi.fn();
    const transport = fakeTransport({ listen: vi.fn().mockResolvedValue(unlisten) });
    const unsubscribe = createTauriClient(transport).subscribe('x' as never, () => {});
    await Promise.resolve();
    unsubscribe();
    await Promise.resolve();
    expect(unlisten).toHaveBeenCalledOnce();
  });

  it('given a subscription, when the transport delivers a payload, then the listener receives it', () => {
    let deliver: (payload: unknown) => void = () => {};
    const transport = fakeTransport({
      listen: vi.fn().mockImplementation((_event: string, handler: (payload: unknown) => void) => {
        deliver = handler;
        return Promise.resolve(() => {});
      }),
    });
    const listener = vi.fn();
    createTauriClient(transport).subscribe('x' as never, listener);
    deliver(42);
    expect(listener).toHaveBeenCalledExactlyOnceWith(42);
  });

  it('given the reserved stream interface, when openStream is called, then it throws', () => {
    expect(() => createTauriClient(fakeTransport()).openStream(undefined as never)).toThrow();
  });
});
