import type { BackendClient, ChannelPatch, SettingTarget, TriggerPatch } from '../../api';
import type { ChannelNumber } from '../main-window/types';

export interface InstrumentCommands {
  setRunning(on: boolean): void;
  forceTrigger(): void;
  setChannel(channel: ChannelNumber, patch: ChannelPatch): void;
  step(target: SettingTarget, direction: -1 | 1): void;
  reset(target: SettingTarget): void;
  setTriggerPosition(fraction: number): void;
  setTriggerLevel(divisions: number): void;
  setTrigger(patch: TriggerPatch): void;
  useSimulator(enabled: boolean): void;
  retryConnection(): void;
}

interface Deps {
  client: BackendClient;
  notify: (kind: 'error', text: string) => void;
}

const FALLBACK_MESSAGE = 'The command failed';

function messageOf(failure: unknown): string {
  const message = (failure as { message?: unknown } | undefined)?.message;
  return typeof message === 'string' && message ? message : FALLBACK_MESSAGE;
}

/** Commands are fire and forget: the new state arrives as an event, a failure becomes an error toast. */
export function createInstrumentCommands({ client, notify }: Deps): InstrumentCommands {
  const send: BackendClient['send'] = (name, args) => client.send(name, args);
  const guard = (start: () => Promise<unknown>) => {
    start().catch((failure: unknown) => notify('error', messageOf(failure)));
  };

  return {
    setRunning: (running) => guard(() => send('set_running', { running })),
    forceTrigger: () => guard(() => send('force_trigger', {})),
    setChannel: (channel, patch) => guard(() => send('set_channel', { channel, patch })),
    step: (target, direction) => guard(() => send('step_setting', { target, direction })),
    reset: (target) => guard(() => send('reset_setting', { target })),
    setTriggerPosition: (fraction) => guard(() => send('set_trigger_position', { fraction })),
    setTriggerLevel: (divisions) => guard(() => send('set_trigger_level', { divisions })),
    setTrigger: (patch) => guard(() => send('set_trigger', { patch })),
    useSimulator: (enabled) => guard(() => send('use_simulator', { enabled })),
    retryConnection: () => guard(() => send('retry_connection', {})),
  };
}
