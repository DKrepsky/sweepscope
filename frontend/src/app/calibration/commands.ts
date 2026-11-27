import type { BackendClient, CommandName } from '../../api';

export interface CalibrationCommands {
  openCalibration(): void;
  startCalibration(): void;
  /** Measures the channel in turn; the user has shorted its input. */
  calibrateChannel(): void;
  /** Accepts the rejected reading of the channel and carries on; only offered at the not shorted step. */
  continueChannel(): void;
  /** Cancels at any step before Done, closes at Done. */
  closeCalibration(): void;
}

interface Deps {
  client: BackendClient;
  notify: (kind: 'error', text: string) => void;
}

const FALLBACK_MESSAGE = 'The calibration command failed';

function messageOf(failure: unknown): string {
  const message = (failure as { message?: unknown } | undefined)?.message;
  return typeof message === 'string' && message ? message : FALLBACK_MESSAGE;
}

/** Fire and forget like the instrument commands: the session arrives as an event, a refusal becomes an error toast. */
export function createCalibrationCommands({ client, notify }: Deps): CalibrationCommands {
  const run = (name: CommandName) => () => {
    client.send(name, {}).catch((failure: unknown) => notify('error', messageOf(failure)));
  };
  return {
    openCalibration: run('open_zero_calibration'),
    startCalibration: run('start_zero_calibration'),
    calibrateChannel: run('measure_zero_calibration'),
    continueChannel: run('continue_zero_calibration'),
    closeCalibration: run('close_zero_calibration'),
  };
}
