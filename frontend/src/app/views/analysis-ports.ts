import type { Readable } from 'svelte/store';
import type { MathPatch } from '../../api/generated/MathPatch';
import type { Persistence } from '../../api/generated/Persistence';
import type { RecordingState } from '../../api/generated/RecordingState';
import type { SpectrumPatch } from '../../api/generated/SpectrumPatch';
import type { SpectrumState } from '../../api/generated/SpectrumState';

/** The commands the analysis features need beyond the instrument ones; the instrument commands module provides them. */
export interface AnalyseCommands {
  armSingle(): void;
  startAutoset(): void;
  setMath(patch: MathPatch): void;
  setPersistence(value: Persistence): void;
  setSpectrum(patch: SpectrumPatch): void;
  startRecording(): void;
  stopRecording(): void;
}

/** What the model reads of the roll window; `RollView` of the waveform module fits it. */
export interface RollInput {
  channels: { channel: number; samples: ArrayLike<number> }[];
  gaps: { from: number; to: number; missed: number; approximate: true }[];
  gapCount: number;
}

/** What the model reads of a decoded spectrum; `DecodedSpectrum` of the waveform module fits it. */
export interface SpectrumFrameInput {
  bins: number;
  binHz: number;
  /** Size of one division on the level axis, dBV or volts rms. */
  unitsPerDiv: number;
  channels: {
    channel: number;
    peakBin?: number | undefined;
    levels: ArrayLike<number>;
    divisions: ArrayLike<number>;
  }[];
}

export interface AnalysisSources {
  spectrum: Readable<SpectrumState | undefined>;
  recording: Readable<RecordingState | undefined>;
  roll: Readable<RollInput | undefined>;
  spectrumFrame: Readable<SpectrumFrameInput | undefined>;
}
