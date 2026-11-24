import type {
  BackendClient,
  ChannelPatch,
  InterpolationMode,
  MathPatch,
  Persistence,
  SettingTarget,
  SpectrumPatch,
  TriggerPatch,
} from '../../api';
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
  setInterpolation(mode: InterpolationMode): void;
  armSingle(): void;
  startAutoset(): void;
  setMath(patch: MathPatch): void;
  setPersistence(value: Persistence): void;
  setSpectrum(patch: SpectrumPatch): void;
  /** The backend runs the file picker; a cancelled picker is not an error. */
  startRecording(): void;
  stopRecording(): void;
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
    setInterpolation: (mode) => guard(() => send('set_interpolation', { mode })),
    armSingle: () => guard(() => send('arm_single', {})),
    startAutoset: () => guard(() => send('start_autoset', {})),
    setMath: (patch) => guard(() => send('set_math', { patch })),
    setPersistence: (value) => guard(() => send('set_persistence', { value })),
    setSpectrum: (patch) => guard(() => send('set_spectrum', { patch })),
    startRecording: () => guard(() => send('start_recording', {})),
    stopRecording: () => guard(() => send('stop_recording', {})),
  };
}
