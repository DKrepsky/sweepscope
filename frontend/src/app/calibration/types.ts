import type { OffsetRow, ZeroSession, ZeroStep } from '../../api';

export const CALIBRATION_STEPS = 6;

/** The heading is the backend's; each line is one enabled channel, `CH1 +0.66 mV at 2 mV/div`. */
export interface CalibrationTooltip {
  heading: string;
  lines: string[];
}

export type CalibrationTagModel =
  | { state: 'none'; tooltip: CalibrationTooltip }
  | { state: 'calibrated'; tooltip: CalibrationTooltip; storedAt: string | undefined }
  | { state: 'unavailable'; reason: string };

export interface CalibrationDialogModel {
  step: ZeroStep;
  /** One based, of `CALIBRATION_STEPS`. */
  stepNumber: number;
  /** 1 to 4 while a channel is in turn. */
  channel: number | null;
  doneChannels: ZeroSession['doneChannels'];
  /** 0 to 1, or null when the backend cannot say how far a measurement is (indeterminate bar). */
  progress: number | null;
  notShorted: string[];
  /** `Continue` may accept the rejected reading. */
  canContinue: boolean;
  /** Channels stored although the shorted-input check failed. */
  overridden: ZeroSession['overridden'];
  failure: string | null;
  table: OffsetRow[] | null;
  /** The unit already holds offsets; the intro step then shows when they were stored. */
  calibrated: boolean;
  storedAt: string | undefined;
}
