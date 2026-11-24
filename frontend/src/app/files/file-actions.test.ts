import { describe, expect, it, vi } from 'vitest';
import { createFakeClient } from '../../api';
import { createFileActions } from './file-actions';

const PNG = new Uint8Array([0x89, 0x50, 0x4e, 0x47]);

function setup() {
  const client = createFakeClient();
  const notify = vi.fn();
  const composeScreenshot = vi.fn(() => Promise.resolve(PNG));
  return { client, notify, composeScreenshot, actions: createFileActions({ client, notify, composeScreenshot }) };
}

describe('file actions', () => {
  it('given a saved file, when CSV is exported, then format csv is sent and an info toast names the file', async () => {
    const { client, notify, actions } = setup();
    client.respond('export_frame', () => ({ fileName: 'a.csv' }));
    await actions.exportCsv();
    expect(client.calls).toEqual([{ name: 'export_frame', args: { format: 'csv' } }]);
    expect(notify).toHaveBeenCalledExactlyOnceWith('info', 'Saved a.csv');
  });

  it('given a saved file, when raw is exported, then format raw is sent', async () => {
    const { client, notify, actions } = setup();
    client.respond('export_frame', () => ({ fileName: 'a.bin' }));
    await actions.exportRaw();
    expect(client.calls).toEqual([{ name: 'export_frame', args: { format: 'raw' } }]);
    expect(notify).toHaveBeenCalledExactlyOnceWith('info', 'Saved a.bin');
  });

  it('given the user cancels the dialog, when a file is exported, then there is no toast', async () => {
    const { client, notify, actions } = setup();
    client.respond('export_frame', () => null);
    await actions.exportCsv();
    expect(notify).not.toHaveBeenCalled();
  });

  it('given a rejected command, when a file is exported, then an error toast carries the backend text', async () => {
    const { client, notify, actions } = setup();
    client.respond('export_frame', () => Promise.reject({ category: 'recoverable', message: 'permission denied' }));
    await actions.exportCsv();
    expect(notify).toHaveBeenCalledExactlyOnceWith('error', 'permission denied');
  });

  it('given a saved setup, when it is saved, then an info toast names the file', async () => {
    const { client, notify, actions } = setup();
    client.respond('save_setup', () => ({ fileName: 'bench.json' }));
    await actions.saveSetup();
    expect(notify).toHaveBeenCalledExactlyOnceWith('info', 'Saved bench.json');
  });

  it('given a setup without warnings, when it is loaded, then one info toast names the file', async () => {
    const { client, notify, actions } = setup();
    client.respond('load_setup', () => ({ fileName: 'bench.json', warnings: [] }));
    await actions.loadSetup();
    expect(notify).toHaveBeenCalledExactlyOnceWith('info', 'Loaded bench.json');
  });

  it('given a setup with warnings, when it is loaded, then one warning toast lists them all', async () => {
    const { client, notify, actions } = setup();
    client.respond('load_setup', () => ({
      fileName: 'bench.json',
      warnings: ['CH1 range not available', 'bad probe'],
    }));
    await actions.loadSetup();
    expect(notify).toHaveBeenCalledOnce();
    expect(notify).toHaveBeenCalledWith('warning', expect.stringContaining('CH1 range not available'));
    expect(notify).toHaveBeenCalledWith('warning', expect.stringContaining('bad probe'));
  });

  it('given the user cancels, when a setup is loaded, then there is no toast', async () => {
    const { client, notify, actions } = setup();
    client.respond('load_setup', () => null);
    await actions.loadSetup();
    expect(notify).not.toHaveBeenCalled();
  });

  it('given a failing load, when it is loaded, then an error toast carries the backend text', async () => {
    const { client, notify, actions } = setup();
    client.respond('load_setup', () => Promise.reject({ category: 'recoverable', message: 'Not a SweepScope setup' }));
    await actions.loadSetup();
    expect(notify).toHaveBeenCalledExactlyOnceWith('error', 'Not a SweepScope setup');
  });

  it('given a composed picture, when the screenshot is saved, then the PNG bytes are sent and the file is named', async () => {
    const { client, notify, actions } = setup();
    client.respond('save_screenshot', () => ({ fileName: 's.png' }));
    await actions.saveScreenshot();
    expect(client.calls).toEqual([{ name: 'save_screenshot', args: PNG }]);
    expect(notify).toHaveBeenCalledExactlyOnceWith('info', 'Saved s.png');
  });

  it('given a composer that fails, when the screenshot is saved, then nothing is sent and an error toast shows', async () => {
    const { client, notify, composeScreenshot, actions } = setup();
    composeScreenshot.mockRejectedValue(new Error('no canvas'));
    await actions.saveScreenshot();
    expect(client.calls).toEqual([]);
    expect(notify).toHaveBeenCalledExactlyOnceWith('error', 'no canvas');
  });
});
