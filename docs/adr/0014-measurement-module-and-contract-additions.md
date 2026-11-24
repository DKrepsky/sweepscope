# ADR 0014: A `measurement` feature over a pure `libs/measurements`, fed by `FrameCaptured`; three small kernel additions

Status: proposed
Date: 2026-11-23
Issue: #15 (also #14, #18); answers architecture Q-E8

## Context
Story #15 needs twelve automatic measurements per channel with validity, clipping detection and an accuracy note. Architecture P1 and P1b put
measurement maths in Rust, features may not import each other (P2), and `FrameCaptured` already carries the whole 4096-sample record with the
data needed to turn codes into volts. Three things are missing for a feature that is not `acquisition`: the probe factor, whether the range is
verified, and `FormattedValue` (defined in `acquisition`, which `measurement` may not import).

## Decision
1. **`libs/measurements`** (new, pure, no dependency): `measure(samples: &[f64], sample_interval_s: f64, volts_per_count: f64) -> Measured`.
   Each of the twelve quantities is `Result<f64, Invalid>`; `Invalid` is a code enum (`NoSignal`, `FewerThanOnePeriod`, `NoCompleteEdge`,
   `EdgeTooFast`, `NoCompletePulse`) and the lib never formats text. `fn is_clipped(codes: &[u8]) -> bool`.
2. **`features/measurement`** (new crate `sweepscope-measurement`): subscribes latest-wins to `FrameCaptured`, converts every present channel
   to probe-scaled volts, calls the lib, adds `Clipped` and the unverified marker, formats with `libs/engineering-format`, and publishes the slice
   `MeasurementsState` (revisioned, at most 4 per second, only when a text changed) as `MeasurementsChanged`. Request `GetMeasurements`.
   Measurements are computed for **all twelve quantities of every enabled channel on every frame**; which rows are shown (pinned) is presentation
   state in the frontend (nothing in the backend depends on it, P1b). One record is at most 16 KiB, so this costs well under a millisecond.
3. **Basis of the numbers.** The whole captured record (16.4 divisions), not only the 10 visible divisions; invert is ignored (the physical signal
   is measured); `Vmean` and `Vrms` use the whole periods between the first and the last rising mid-crossing when there are at least two, otherwise
   the whole record.
4. **Q-E8 ruling (clipping and invalid reasons).** A channel is clipped when any sample code of the record is 0 or 255 (the converter rails).
   Clipping invalidates the amplitude-dependent quantities (Vmax, Vmin, Vpp, Vmean, Vrms, rise time, fall time) with reason `Clipped`; frequency,
   period, widths and duty cycle stay valid (they depend on crossings of the mid level, which clipping leaves in place). `Invalid` carries a code and
   a backend text; the frontend shows `-` and the text as a numbered footnote. The slice carries `clipped` per channel for the `Clip` badge of design 8.4.
5. **Kernel additions** (reviewed under ADR 0010 item 7): `FormattedValue` moves from `acquisition` to `kernel::contracts` (same shape, same
   generated TypeScript file, re-exported by `acquisition`); `FrameContext` gains `probe: [f64; 4]`, `range_verified: [bool; 4]` and
   `seconds_per_div: f64` (in-process only, the wire frame of ADR 0011 does not change); and `kernel::files` with `FilePicker` (ADR 0016).
6. **Accuracy note.** One backend constant in `measurement` is published in the slice (`accuracy_note`); the UI shows it in the measurement panel
   and the docs quote the same sentence. Wording is a PM decision (spec section 1.3).

## Alternatives considered
- **Measure inside `acquisition`.** No new crate, but acquisition would own device state, setup, status and a second computation, and every frame
  would pass through it twice. Rejected: separate reasons to change (P12) and the architecture lists `measurement` as its own feature.
- **Measure on the display window only.** Matches what the user sees but loses up to 6.4 divisions of record, which decides whether one period fits at
  slow time bases; clipping off screen would be missed.
- **Compute only pinned measurements.** Would make pinning domain state and add a command and a slice change for a saving that is not needed.
- **Frontend computes from the wire frame.** Breaks P1b and the wire frame holds divisions, not volts.

## Consequences
- Two new crates and one new lib; `check-architecture` already accepts them (feature depends on kernel and libs only).
- `FrameContext` growth touches `acquisition` once (WP1) and nothing else.
- Frequency accuracy is limited by sample quantisation of the crossings (see the spec, risk R2); a later story can average over frames (statistics story).
