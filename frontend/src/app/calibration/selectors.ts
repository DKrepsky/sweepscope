import type { ConnectionSnapshot, RecordingState, ZeroCalibrationState } from '../../api';
import { formatStoredDate } from './format-date';
import { CALIBRATION_STEPS, type CalibrationDialogModel, type CalibrationTagModel } from './types';

export const STOP_RECORDING_FIRST = 'Stop recording first';
export const NO_DEVICE = 'No device';

const NO_TOOLTIP = { rows: [], notes: [] };

/** Why calibration cannot be opened now, or null. A device problem comes before the recording, which only needs stopping. */
function blockedReason(
  zero: ZeroCalibrationState | undefined,
  connection: ConnectionSnapshot | undefined,
  recording: RecordingState | undefined,
): string | null {
  if (connection?.state !== 'ready' || !zero?.availability.enabled) return zero?.availability.reason ?? NO_DEVICE;
  return recording?.active ? STOP_RECORDING_FIRST : null;
}

export function selectCalibrationTag(
  zero: ZeroCalibrationState | undefined,
  connection: ConnectionSnapshot | undefined,
  recording: RecordingState | undefined,
): CalibrationTagModel {
  const reason = blockedReason(zero, connection, recording);
  if (reason !== null || !zero) return { state: 'unavailable', reason: reason ?? NO_DEVICE };
  const tooltip = zero.unit?.tooltip ?? NO_TOOLTIP;
  if (!zero.unit?.calibrated) return { state: 'none', tooltip };
  return { state: 'calibrated', tooltip, storedAt: formatStoredDate(zero.unit.storedAtUtc) };
}

/** Measuring, not shorted and failed show the number of the step they belong to; Done is always the last. */
function stepNumber(session: NonNullable<ZeroCalibrationState['session']>): number {
  switch (session.step) {
    case 'intro':
      return 1;
    case 'done':
      return CALIBRATION_STEPS;
    default:
      return session.channel === null ? 1 : session.channel + 1;
  }
}

export function selectCalibrationDialog(zero: ZeroCalibrationState | undefined): CalibrationDialogModel | undefined {
  const session = zero?.session;
  if (!zero || !session) return undefined;
  return {
    step: session.step,
    stepNumber: stepNumber(session),
    channel: session.channel,
    doneChannels: session.doneChannels,
    progress: session.progress,
    notShorted: session.notShorted ?? [],
    failure: session.failure,
    table: session.table,
    calibrated: zero.unit?.calibrated ?? false,
    storedAt: formatStoredDate(zero.unit?.storedAtUtc),
  };
}

/** Acquisition is held for as long as a session exists, whatever step it is in. */
export function selectPausedForCalibration(zero: ZeroCalibrationState | undefined): boolean {
  return zero?.session != null;
}
