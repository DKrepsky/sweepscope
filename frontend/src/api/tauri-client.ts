import { Channel, invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { BackendClient } from './backend-client';
import type { CommandName, StreamKind } from './generated/commands';

/** Rust command name for every contract command. Exhaustive, so a new command cannot be forgotten. */
export const tauriCommandNames: Record<CommandName, string> = {
  load_ui_state: 'load_ui_state',
  save_ui_state: 'save_ui_state',
  get_snapshot: 'get_snapshot',
  set_running: 'set_running',
  force_trigger: 'force_trigger',
  set_channel: 'set_channel',
  step_setting: 'step_setting',
  reset_setting: 'reset_setting',
  set_trigger_position: 'set_trigger_position',
  set_trigger_level: 'set_trigger_level',
  set_trigger: 'set_trigger',
  use_simulator: 'use_simulator',
  retry_connection: 'retry_connection',
  set_interpolation: 'set_interpolation',
  export_frame: 'export_frame',
  save_screenshot: 'save_screenshot',
  save_setup: 'save_setup',
  load_setup: 'load_setup',
  arm_single: 'arm_single',
  start_autoset: 'start_autoset',
  set_math: 'set_math',
  set_persistence: 'set_persistence',
  set_spectrum: 'set_spectrum',
  start_recording: 'start_recording',
  stop_recording: 'stop_recording',
  open_zero_calibration: 'open_zero_calibration',
  start_zero_calibration: 'start_zero_calibration',
  measure_zero_calibration: 'measure_zero_calibration',
  close_zero_calibration: 'close_zero_calibration',
};

/** Rust commands that open and close each backend stream. Exhaustive, like the command names. */
export const tauriStreamCommands: Record<StreamKind, { open: string; close: string }> = {
  frames: { open: 'open_frame_stream', close: 'close_frame_stream' },
  roll: { open: 'open_roll_stream', close: 'close_roll_stream' },
  spectrum: { open: 'open_spectrum_stream', close: 'close_spectrum_stream' },
};

export interface TauriTransport {
  invoke(command: string, args: unknown): Promise<unknown>;
  listen(event: string, handler: (payload: unknown) => void): Promise<() => void>;
  /** Calls `command` with a channel as its `onFrame` argument; every channel message goes to `onMessage`. */
  openChannel(command: string, onMessage: (message: unknown) => void): Promise<void>;
}

const tauriTransport: TauriTransport = {
  invoke: (command, args) => invoke(command, args as Record<string, unknown>),
  listen: (event, handler) => listen(event, (message) => handler(message.payload)),
  openChannel: (command, onMessage) => {
    const onFrame = new Channel<unknown>();
    onFrame.onmessage = onMessage;
    return invoke(command, { onFrame });
  },
};

function toArrayBuffer(message: unknown): ArrayBuffer {
  if (message instanceof ArrayBuffer) return message;
  return new Uint8Array(message as ArrayLike<number>).buffer;
}

/** Receives failures of background transport calls that have no caller to reject to. */
export type ErrorReporter = (what: string, error: unknown) => void;

const reportToConsole: ErrorReporter = (what, error) => {
  console.error(`${what} failed`, error);
};

export function createTauriClient(
  transport: TauriTransport = tauriTransport,
  report: ErrorReporter = reportToConsole,
): BackendClient {
  const unsubscribeWhenReady = (event: string, pending: Promise<() => void>): (() => void) => {
    const unlisten = pending.catch((error: unknown) => {
      report(`listen ${event}`, error);
      return () => {};
    });
    return () => {
      void unlisten.then((stop) => stop());
    };
  };

  return {
    send: (name, args) => transport.invoke(tauriCommandNames[name] ?? name, args) as Promise<never>,
    sendBytes: <R>(command: string, bytes: Uint8Array) => transport.invoke(command, bytes) as Promise<R>,
    subscribe: (name, listener) =>
      unsubscribeWhenReady(
        name,
        transport.listen(name, (payload) => listener(payload as never)),
      ),
    openStream: (kind, onMessage) => {
      const commands = tauriStreamCommands[kind];
      let open = true;
      transport
        .openChannel(commands.open, (message) => open && onMessage(toArrayBuffer(message)))
        .catch((error: unknown) => report(commands.open, error));
      return {
        close: () => {
          if (!open) return;
          open = false;
          transport.invoke(commands.close, {}).catch((error: unknown) => report(commands.close, error));
        },
      };
    },
  };
}
