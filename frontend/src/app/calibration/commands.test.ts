import { describe, expect, it, vi } from 'vitest';
import { createFakeClient } from '../../api';
import { createCalibrationCommands } from './commands';

function setup() {
  const client = createFakeClient();
  const notify = vi.fn();
  return { client, notify, commands: createCalibrationCommands({ client, notify }) };
}

describe('calibration commands', () => {
  it('given the use cases, when each is called, then each sends its backend command with no arguments', () => {
    const { client, commands } = setup();

    commands.openCalibration();
    commands.startCalibration();
    commands.calibrateChannel();
    commands.continueChannel();
    commands.closeCalibration();

    expect(client.calls).toEqual([
      { name: 'open_zero_calibration', args: {} },
      { name: 'start_zero_calibration', args: {} },
      { name: 'measure_zero_calibration', args: {} },
      { name: 'continue_zero_calibration', args: {} },
      { name: 'close_zero_calibration', args: {} },
    ]);
  });

  it('given the backend refuses, when a command is sent, then the error text becomes an error toast', async () => {
    const { client, notify, commands } = setup();
    client.respond('open_zero_calibration', () => Promise.reject(new Error('Auto-setup is running')));

    commands.openCalibration();

    await vi.waitFor(() => expect(notify).toHaveBeenCalledWith('error', 'Auto-setup is running'));
  });

  it('given a refusal without text, when a command is sent, then a fallback error toast shows', async () => {
    const { client, notify, commands } = setup();
    client.respond('close_zero_calibration', () => Promise.reject(undefined));

    commands.closeCalibration();

    await vi.waitFor(() => expect(notify).toHaveBeenCalledWith('error', 'The calibration command failed'));
  });
});
