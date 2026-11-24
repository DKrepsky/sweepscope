import type { OffsetRow, TooltipText, ZeroSession, ZeroStep } from '../../api';

export const CALIBRATION_STEPS = 6;

export type CalibrationTagModel =
  | { state: 'none'; tooltip: TooltipText }
  | { state: 'calibrated'; tooltip: TooltipText; storedAt: string | undefined }
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
  failure: string | null;
  table: OffsetRow[] | null;
  /** The unit already holds offsets; the intro step then shows when they were stored. */
  calibrated: boolean;
  storedAt: string | undefined;
}
