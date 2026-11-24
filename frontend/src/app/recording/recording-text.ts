import type { RecordingState } from '../../api/generated/RecordingState';

type Failure = NonNullable<RecordingState['failure']>;

const SECONDS_PER_HOUR = 3600;
const SECONDS_PER_MINUTE = 60;
const BYTE_UNITS = ['B', 'kB', 'MB', 'GB', 'TB'];
const BYTES_PER_STEP = 1000;

const twoDigits = (value: number) => String(value).padStart(2, '0');

export function formatElapsed(seconds: number): string {
  const whole = Math.max(0, Math.floor(seconds));
  const hours = Math.floor(whole / SECONDS_PER_HOUR);
  const minutes = Math.floor((whole % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE);
  return `${twoDigits(hours)}:${twoDigits(minutes)}:${twoDigits(whole % SECONDS_PER_MINUTE)}`;
}

/** Three significant digits, decimal prefixes (`12.4 MB`), like the backend's engineering format. */
export function formatBytes(bytes: number): string {
  let value = Math.max(0, bytes);
  let unit = 0;
  while (value >= BYTES_PER_STEP && unit < BYTE_UNITS.length - 1) {
    value /= BYTES_PER_STEP;
    unit += 1;
  }
  const digits = unit === 0 ? 0 : value < 10 ? 2 : value < 100 ? 1 : 0;
  return `${value.toFixed(digits)} ${BYTE_UNITS[unit]}`;
}

/** The readout chip while recording; the elapsed time and size come from the slice, once per second. */
export function recordingChip(state: RecordingState | undefined): string | undefined {
  if (!state?.active) return undefined;
  const written = `${state.bytesWritten.text} ${state.bytesWritten.unit}`;
  return `Rec ${formatElapsed(state.duration.value)} · ${written}`;
}

/** The banner text of a recording that stopped by itself; `text` is the backend's reason (`the disk is full`). */
export function failureText({ text, bytes, fileName }: Failure): string {
  const reason = text.replace(/\.$/, '');
  return `Recording stopped: ${reason}. ${formatBytes(bytes)} written to ${fileName}. The display is still running.`;
}
