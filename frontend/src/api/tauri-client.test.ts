import { describe, expect, it, vi } from 'vitest';
import { COMMANDS, STREAMS } from './generated/commands';
import { createTauriClient, tauriCommandNames, tauriStreamCommands, type TauriTransport } from './tauri-client';

function fakeTransport(overrides: Partial<TauriTransport> = {}): TauriTransport {
  return {
    invoke: vi.fn().mockResolvedValue(null),
    listen: vi.fn().mockResolvedValue(() => {}),
    openChannel: vi.fn().mockResolvedValue(undefined),
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

  it('given bytes, when sent, then the transport receives the command and the same bytes', async () => {
    const transport = fakeTransport({ invoke: vi.fn().mockResolvedValue({ fileName: 'a.png' }) });
    const bytes = new Uint8Array([137, 80, 78, 71]);
    const result = await createTauriClient(transport).sendBytes('save_screenshot', bytes);
    expect(transport.invoke).toHaveBeenCalledExactlyOnceWith('save_screenshot', bytes);
    expect(result).toEqual({ fileName: 'a.png' });
  });

  it('given the transport rejects, when bytes are sent, then the same error reaches the caller', async () => {
    const failure = { category: 'fatal', message: 'boom' };
    const transport = fakeTransport({ invoke: vi.fn().mockRejectedValue(failure) });
    await expect(createTauriClient(transport).sendBytes('save_screenshot', new Uint8Array())).rejects.toBe(failure);
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

  it('given the generated stream list, when compared with the keys the Tauri client maps, then they are equal', () => {
    expect(Object.keys(tauriStreamCommands).sort()).toEqual([...STREAMS].sort());
  });

  function openedFrames(overrides: Partial<TauriTransport> = {}) {
    let deliver: (message: unknown) => void = () => {};
    const transport = fakeTransport({
      openChannel: vi.fn().mockImplementation((_command: string, handler: (message: unknown) => void) => {
        deliver = handler;
        return Promise.resolve();
      }),
      ...overrides,
    });
    const received: ArrayBuffer[] = [];
    const handle = createTauriClient(transport).openStream('frames', (data) => received.push(data));
    return { transport, handle, received, deliver: (message: unknown) => deliver(message) };
  }

  it('given a stream kind, when opened, then the transport opens a channel on the mapped open command', () => {
    const { transport } = openedFrames();
    expect(transport.openChannel).toHaveBeenCalledExactlyOnceWith('open_frame_stream', expect.any(Function));
  });

  it('given an open stream, when the channel delivers an ArrayBuffer, then the callback receives the same buffer', () => {
    const { received, deliver } = openedFrames();
    const buffer = new Uint8Array([1, 2, 3]).buffer;
    deliver(buffer);
    expect(received).toEqual([buffer]);
  });

  it('given an open stream, when the channel delivers a byte array, then the callback receives those bytes as a buffer', () => {
    const { received, deliver } = openedFrames();
    deliver([1, 2, 3]);
    expect([...new Uint8Array(received[0] as ArrayBuffer)]).toEqual([1, 2, 3]);
  });

  it('given an open stream, when closed, then the close command is invoked and later messages are ignored', () => {
    const { transport, handle, received, deliver } = openedFrames();
    handle.close();
    deliver(new ArrayBuffer(1));
    expect(transport.invoke).toHaveBeenCalledExactlyOnceWith('close_frame_stream', {});
    expect(received).toEqual([]);
  });

  it('given an open stream, when closed twice, then the close command is invoked once', () => {
    const { transport, handle } = openedFrames();
    handle.close();
    handle.close();
    expect(transport.invoke).toHaveBeenCalledOnce();
  });

  it('given the transport cannot listen, when subscribed, then the failure is reported and unsubscribing is harmless', async () => {
    const failure = new Error('no listener');
    const report = vi.fn();
    const transport = fakeTransport({ listen: vi.fn().mockRejectedValue(failure) });

    const unsubscribe = createTauriClient(transport, report).subscribe('x' as never, () => {});
    await Promise.resolve();
    await Promise.resolve();

    expect(report).toHaveBeenCalledExactlyOnceWith('listen x', failure);
    expect(() => unsubscribe()).not.toThrow();
  });

  it('given the transport cannot open the channel, when a stream is opened, then the failure is reported', async () => {
    const failure = new Error('no channel');
    const report = vi.fn();
    const transport = fakeTransport({ openChannel: vi.fn().mockRejectedValue(failure) });

    createTauriClient(transport, report).openStream('frames', () => {});
    await Promise.resolve();
    await Promise.resolve();

    expect(report).toHaveBeenCalledExactlyOnceWith('open_frame_stream', failure);
  });

  it('given the close command fails, when a stream is closed, then the failure is reported', async () => {
    const failure = new Error('no close');
    const report = vi.fn();
    const transport = fakeTransport({ invoke: vi.fn().mockRejectedValue(failure) });

    createTauriClient(transport, report)
      .openStream('frames', () => {})
      .close();
    await Promise.resolve();
    await Promise.resolve();

    expect(report).toHaveBeenCalledExactlyOnceWith('close_frame_stream', failure);
  });
});
