import { describe, expect, it, vi } from 'vitest';
import { createFakeClient } from './fake-client';

const eventName = 'any_event' as never;

describe('fake client', () => {
  it('given a registered response, when a command is sent, then the response resolves and the call is recorded', async () => {
    const client = createFakeClient();
    client.respond('load_ui_state', () => null);
    await expect(client.send('load_ui_state', { namespace: 'a' })).resolves.toBeNull();
    expect(client.calls).toEqual([{ name: 'load_ui_state', args: { namespace: 'a' } }]);
  });

  it('given no registered response, when a command is sent, then it rejects naming the command', async () => {
    const client = createFakeClient();
    await expect(client.send('load_ui_state', { namespace: 'a' })).rejects.toThrow('load_ui_state');
  });

  it('given a subscriber, when an event is emitted, then the listener receives the payload', () => {
    const client = createFakeClient();
    const listener = vi.fn();
    client.subscribe(eventName, listener);
    client.emit(eventName, 1 as never);
    expect(listener).toHaveBeenCalledExactlyOnceWith(1);
  });

  it('given the fake client, when subscribe is unsubscribed, then the listener is no longer called', () => {
    const client = createFakeClient();
    const listener = vi.fn();
    const unsubscribe = client.subscribe(eventName, listener);
    unsubscribe();
    client.emit(eventName, 1 as never);
    expect(listener).not.toHaveBeenCalled();
  });

  it('given an open stream, when a message is pushed, then the callback receives it', () => {
    const client = createFakeClient();
    const onMessage = vi.fn();
    client.openStream('frames', onMessage);
    const buffer = new ArrayBuffer(4);
    client.pushStream('frames', buffer);
    expect(onMessage).toHaveBeenCalledExactlyOnceWith(buffer);
  });

  it('given a closed stream, when a message is pushed, then the callback is not called and the close is counted', () => {
    const client = createFakeClient();
    const onMessage = vi.fn();
    client.openStream('frames', onMessage).close();
    client.pushStream('frames', new ArrayBuffer(4));
    expect(onMessage).not.toHaveBeenCalled();
    expect(client.openStreamCount('frames')).toBe(0);
  });
});
