# ADR 0018: A `spectrum` feature over a pure `libs/fft` with its own radix-2 transform; spectra travel on their own stream

Status: proposed
Date: 2026-11-24
Issue: #29

## Context
Architecture P1b puts the FFT in Rust (`libs/fft`, feature `spectrum`, architecture sections 3 and 4.4). The record is always 4096 samples per channel (a power of two), so bin spacing is `sample rate / 4096` (#29 AC3).
The frame wire (ADR 0011) carries volts-scaled divisions of a time-domain window; a spectrum has different axes, tick labels and a peak. The peak marker is frontend presentation state but must read frequency and level
of any bin without a round trip (same argument as cursors, architecture Q-E5).

## Decision
1. **`libs/fft`** (new, no dependency): `spectrum(volts: &[f64], window: Window, sample_rate_hz: f64) -> Spectrum { bin_hz, rms: Vec<f64> }` for lengths that are a power of two (otherwise an error value), `Window { Rectangular, Hann, Hamming, Blackman }`,
   `fn peak_bin(&Spectrum, Window) -> Option<usize>`. Internals (iterative radix-2, window tables, scaling) stay private. **No new dependency**: a 4096-point radix-2 transform is about 60 lines and fully testable; `rustfft` (MIT or Apache,
   large) is not justified for one fixed power-of-two size (P15).
2. **Conventions** (answers to design Q-M3-2, Q-M3-3): single-sided, `N/2 + 1` bins (DC to Nyquist). Window gain is corrected so a sine centred on a bin reads its true amplitude whatever the window (coherent gain of the window).
   Level is **volts RMS** per bin (the DC bin is `|mean|`, no doubling); **dBV = 20 log10(V rms / 1 V)**, floor clamp -200 dBV. The whole 4096 record is used (not only the visible 10 divisions), with probe-scaled volts. Auto peak is the
   highest bin after skipping the DC bin and the window's main-lobe bins (1, 2, 2, 3 for Rectangular, Hann, Hamming, Blackman); its frequency is the **bin centre** (no interpolation; the UI states "resolution is one bin").
3. **`features/spectrum`** (new crate `sweepscope-spectrum`): owns the slice `SpectrumState { revision, active, window, scale: Dbv | Linear, bin_spacing, span, sample_rate (FormattedValue), top_label, ticks_x[], ticks_y[], availability }`,
   subscribes latest-wins to `FrameCaptured` while `active`, computes one spectrum per present channel plus the math trace (channel 5, ADR 0019), and publishes the encoded spectrum frame on `SpectrumFrameReady`.
   Requests `SetSpectrum { active?, window?, scale? }` (patch-shaped, ADR 0013) and `GetSpectrum`. `active` is true only while the FFT view is open (the one presentation fact promoted to a command, rule 4.5.1, because it removes
   up to four transforms per frame otherwise). Axis: dBV fixed top +10 dBV at 10 dB/div; Linear automatic 1-2-5 top (8 divisions) that changes only when the largest peak exceeds the top or falls below a quarter of it.
   Tick texts are backend text (`at` in 0 to 1, `text`).
4. **Spectrum frame `SSS1`**, own stream (`open_spectrum_stream`, `close_spectrum_stream`, latest-wins, shell forwarder like `FrameStreams`). Little endian:
   header 56 bytes: `0 u32 magic 0x31535353 | 4 u32 sequence | 8 f64 produced µs | 16 f64 sent µs (stamped by the shell) | 24 u8 mask (bit 4 = M) | 25 u8 flags (bit 0 linear) | 26 u8 window (0 to 3) | 27 u8 0 | 28 u32 bins | 32 f64 bin_hz | 40 f32 top | 44 f32 units per division | 48 u32 setup revision | 52 u32 0`,
   then `u32 peak_bin` per present channel (u32::MAX = none), then per present channel `bins` f32 levels (dBV or V rms) followed by `bins` f32 divisions from the top edge (so the graph draws without computing and the marker reads a bin by lookup).
5. **Frontend arithmetic is lookup only.** The marker readout formats `bin x bin_hz` and `levels[bin]` with the TypeScript formatting port, extended with `Frequency` (four digits) and a dBV rule and covered by the shared golden vectors.

## Alternatives considered
- **`rustfft` or `realfft`.** Faster for arbitrary lengths; unneeded here, adds a dependency and its transitive crates.
- **Send levels only and scale to divisions in the frontend.** Smaller (half) but puts the axis mapping in TypeScript; the frame is 66 KB for five spectra, which the spike showed is no problem.
- **Reuse the frame stream (`SSF1`) with a flag.** One stream, but the module would have to merge two producers into one latest-wins event or the shell forwarder would need a two-event wait the bus does not offer.
- **Compute the spectrum always.** Wastes up to five transforms per frame when the view is not open.
- **Interpolated peak frequency.** Looks more precise than 12 Hz bins justify (PRD 5.2: do not claim more).

## Consequences
- New crates: `libs/fft` and `features/spectrum`; two new stream commands; `StateSnapshot` gains the spectrum slice.
- FFT is only meaningful for a frame capture: unavailable in roll with the backend reason.
- Dynamic range is bounded by the 8-bit ADC (about 48 dB plus 33 dB processing gain); the UI makes no claim below -75 dBV.
