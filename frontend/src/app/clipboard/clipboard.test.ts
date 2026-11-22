import { describe, expect, it, vi } from 'vitest';
import { createCopyText } from './clipboard';

async function settle(): Promise<void> {
  await Promise.resolve();
  await Promise.resolve();
}

describe('copy text', () => {
  it('given the clipboard accepts the text, when copying, then an info notification confirms it', async () => {
    const notify = vi.fn();
    const write = vi.fn().mockResolvedValue(undefined);
    createCopyText({ write, notify })('sudo cp rule');
    await settle();
    expect(write).toHaveBeenCalledWith('sudo cp rule');
    expect(notify).toHaveBeenCalledExactlyOnceWith({ kind: 'info', text: 'Command copied' });
  });

  it('given the clipboard refuses the text, when copying, then an error notification says so and nothing is confirmed', async () => {
    const notify = vi.fn();
    const write = vi.fn().mockRejectedValue(new Error('denied'));
    createCopyText({ write, notify })('x');
    await settle();
    expect(notify).toHaveBeenCalledExactlyOnceWith({ kind: 'error', text: 'Could not copy to the clipboard' });
  });
});
