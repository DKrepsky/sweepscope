import { describe, expect, it, vi } from 'vitest';
import { createFakeClient } from '../../api';
import { createInstrumentCommands, type InstrumentCommands } from './commands';

function setup() {
  const client = createFakeClient();
  const notify = vi.fn();
  return { client, notify, commands: createInstrumentCommands({ client, notify }) };
}

type Case = [string, (c: InstrumentCommands) => void, string, unknown];

const cases: Case[] = [
  ['setRunning', (c) => c.setRunning(false), 'set_running', { running: false }],
  ['forceTrigger', (c) => c.forceTrigger(), 'force_trigger', {}],
  ['setChannel', (c) => c.setChannel(2, { invert: true }), 'set_channel', { channel: 2, patch: { invert: true } }],
  [
    'step',
    (c) => c.step({ kind: 'range', channel: 1 }, 1),
    'step_setting',
    { target: { kind: 'range', channel: 1 }, direction: 1 },
  ],
  ['reset', (c) => c.reset({ kind: 'timeBase' }), 'reset_setting', { target: { kind: 'timeBase' } }],
  ['setTriggerPosition', (c) => c.setTriggerPosition(0.25), 'set_trigger_position', { fraction: 0.25 }],
  ['setTriggerLevel', (c) => c.setTriggerLevel(-1.5), 'set_trigger_level', { divisions: -1.5 }],
  ['setTrigger', (c) => c.setTrigger({ slope: 'falling' }), 'set_trigger', { patch: { slope: 'falling' } }],
  ['useSimulator', (c) => c.useSimulator(true), 'use_simulator', { enabled: true }],
  ['retryConnection', (c) => c.retryConnection(), 'retry_connection', {}],
  ['setInterpolation', (c) => c.setInterpolation('dots'), 'set_interpolation', { mode: 'dots' }],
  ['armSingle', (c) => c.armSingle(), 'arm_single', {}],
  ['startAutoset', (c) => c.startAutoset(), 'start_autoset', {}],
  ['setMath', (c) => c.setMath({ enabled: true }), 'set_math', { patch: { enabled: true } }],
  ['setPersistence', (c) => c.setPersistence('1s'), 'set_persistence', { value: '1s' }],
  ['setSpectrum', (c) => c.setSpectrum({ active: true }), 'set_spectrum', { patch: { active: true } }],
  ['startRecording', (c) => c.startRecording(), 'start_recording', {}],
  ['stopRecording', (c) => c.stopRecording(), 'stop_recording', {}],
  [
    'step of the math scale',
    (c) => c.step({ kind: 'mathScale' }, -1),
    'step_setting',
    { target: { kind: 'mathScale' }, direction: -1 },
  ],
];

describe('instrument commands', () => {
  it.each(cases)('given %s, when called, then exactly one matching command is sent', (_name, call, command, args) => {
    const { client, commands } = setup();
    client.respond(command as never, () => null as never);
    call(commands);
    expect(client.calls).toEqual([{ name: command, args }]);
  });

  it('given a rejected command, when it fails, then the message goes to notify as an error', async () => {
    const { client, notify, commands } = setup();
    client.respond('set_running', () => Promise.reject({ category: 'recoverable', message: 'No scope connected' }));
    commands.setRunning(true);
    await vi.waitFor(() => expect(notify).toHaveBeenCalledExactlyOnceWith('error', 'No scope connected'));
  });

  it('given a failure that is not a kernel error, when it fails, then its message is notified', async () => {
    const { client, notify, commands } = setup();
    client.respond('force_trigger', () => Promise.reject(new Error('socket closed')));
    commands.forceTrigger();
    await vi.waitFor(() => expect(notify).toHaveBeenCalledExactlyOnceWith('error', 'socket closed'));
  });

  it('given a failure that carries no message, when it fails, then a generic error is notified', async () => {
    const { client, notify, commands } = setup();
    client.respond('force_trigger', () => Promise.reject(undefined));
    commands.forceTrigger();
    await vi.waitFor(() => expect(notify).toHaveBeenCalledExactlyOnceWith('error', 'The command failed'));
  });

  it('given a command without a registered response, when called, then it does not throw', () => {
    const { commands } = setup();
    expect(() => commands.forceTrigger()).not.toThrow();
  });
});
