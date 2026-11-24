# ADR 0020: Auto-setup is a pure decision lib driven step by step by `acquisition`, without publishing intermediate frames

Status: proposed
Date: 2026-11-24
Issue: #22

## Context
Auto-setup must choose range and time base from short captures of an unknown signal from 10 Hz to 10 MHz (#22 AC1), say `No signal` for a flat input (AC2), fail without changing the setup (AC3) and never open a dialog
(AC4). Every configuration costs a full `configure` (over 100 ms) and a capture (up to 16.4 divisions of time). The capture loop belongs to the `device` worker, the setup to `acquisition`; a lib may not depend on another
crate of ours (`LIB_ALLOWLIST` is empty). Bus handlers must not block on hardware (ADR 0010 item 5).

## Decision
1. **`libs/autoset`** (new, plain data in and out, no dependency): a state machine `Autoset::new(tables, current) -> Autoset` and `next(&mut self, observation: Option<&Observation>) -> Step`, with
   `Observation { channels: [Option<ChannelObservation { min_code, max_code, clipped, period_s: Option<f64> }>; 4], sample_interval_s }` and `Step::{ Probe(Plan), Done(Plan), NoSignal(Plan), Failed(Reason) }`. A `Plan` is
   range index and position per enabled channel, time base index and trigger (source, level volts). The lib knows tables (ranges, time bases, counts per division, record length) as numbers handed in, not as kernel types.
2. **Algorithm** (rules the tests fix). Vertical and time base depend on each other (a record shorter than a period under-reads the amplitude), so they interleave:
   - Rungs, fast to slow, at 1 us/div, 100 us/div and 20 ms/div (records of 16.4 us, 1.64 ms and 328 ms). At each rung: settle the range of every enabled channel (at most 3 probes: clipped, code 0 or 255, goes up two table steps; peak to peak
     under 6 counts at a range above the most sensitive goes down by the ratio that would make it about 5 divisions), then look for a period on the reference channel (the lowest enabled channel with a measurable period). The first rung with a period gives `T`.
   - Time base: the table step in `[0.2 T, 0.6 T]` per division (2 to 6 periods in 10 divisions; the 1-2-5 table always has one, step ratio at most 2.5 < 3). Roll time bases are never chosen.
   - Final vertical: probe at the chosen time base, then per channel the smallest range whose peak to peak is at most 7 divisions without clipping (prefer a verified range when two fit); position is minus the mid level in divisions, clamped to the range's reach.
     One verification probe: a period outside 1.5 to 7 periods allows one correction, then `Failed`.
   - No signal: after the last rung every enabled channel has under 3 counts of peak to peak at the most sensitive range tried (10 mV/div or the nearest table value below). Plan: all channels 1 V/div, position 0, 1 ms/div (a flat line at zero).
   - Trigger: edge, rising, source reference channel, level at mid amplitude, Auto sweep; coupling, probe, invert and enable are untouched.
   - Budget: 15 s of wall time or 14 probes, then `Failed` and the previous setup is restored.
3. **Driver in `acquisition`** (adapter `module/autoset_run.rs`): `Autoset` request sets an `AutosetRun`; the module sends `ApplySetup` for each `Plan::Probe`, takes the next `CaptureCompleted` of that revision, builds the `Observation` with
   `libs/measurements` (period, peak to peak) and `is_clipped`, and calls `next`. Intermediate captures are not published as `FrameCaptured` (the screen keeps the last frame and shows the `Settling` chip), so nothing flickers; on `Done`
   the final plan is applied as a normal setup change and run intent becomes Run. `Failed`, a disconnect or the budget restores the saved model and publishes the outcome.
4. **State and outcome** in the `status` slice: `autoset: { running, no_signal, outcome: Option<{ id, kind: Done | NoSignal | Failed, text }> }`. `no_signal` is cleared by the next Auto-setup or when a later accepted frame shows a peak-to-peak
   of at least 3 counts on an enabled channel. Run, Single, Force and the time base steppers are unavailable with the reason `Auto-setup is running`; the request returns at once.
5. **Not available** in roll (reason published), with no device, offline, or no enabled channel.

## Alternatives considered
- **One-shot estimate from a single capture.** Cannot cover six decades of frequency with 4096 samples.
- **Lib depends on `libs/measurements`.** Needs an allowlist entry for one function the adapter can call; plain-data observations keep the rule intact and make the lib trivially testable with synthetic observations.
- **Run auto-setup in the `device` worker.** Needs `acquisition` knowledge (ranges, position limits, probes, table); inverts the dependency.
- **Publish intermediate frames.** Shows the search, but flashes five scales in two seconds and breaks the "previous setup stays on failure" feel.
- **Make it cancellable.** The budget is short and the capture is not cancellable (driver); a cancel would still wait for the capture.

## Consequences
- AC1 holds for periods always and for amplitude where the range table allows (the 2 to 5 steps leave a 7 % amplitude band per decade where the nearest range gives 2.9 or 7.25 divisions); recorded as a limit, reported to the PM.
- Simulator signals need frequency, amplitude and offset (work package WP5) to test the whole range.
- The lib is the place to tune the algorithm after hardware runs without touching module code.
