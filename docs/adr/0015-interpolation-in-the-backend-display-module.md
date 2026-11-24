# ADR 0015: Interpolation is computed in `display`; the mode is domain state with an automatic default

Status: proposed
Date: 2026-11-23
Issue: #13

## Context
M1 left `linear` and `dots` as a frontend presentation store and said interpolation "becomes domain state when sin(x)/x (#13) needs the backend"
(spec M1 2.2). sin(x)/x is a computation beyond trivial arithmetic (P1b), the wire frame already holds the samples the screen shows (ADR 0011),
and time-base steps from 2 ns to about 500 ns have fewer than 250 samples per division (PRD 5.2): at 2 ns per division the window holds 6 points.
The graph can only draw straight lines between the points it receives, so a smooth edge needs more points.

## Decision
1. **New `libs/interpolation`**: `upsample_window(record: &[f32], start: usize, len: usize, factor: usize) -> Vec<f32>` returns
   `(len - 1) * factor + 1` points on the original grid refined by `factor`; the original samples are reproduced exactly (`out[k * factor] == record[start + k]`).
   Inside: windowed sinc (Lanczos, 8 taps each side) reading the record outside the window so the window edges are correct; the record is edge-replicated beyond its ends.
2. **`display` owns the mode.** State slice `DisplayState { revision, interpolation: Sinc | Linear | Dots, automatic: bool }`, event `DisplayChanged`, requests
   `GetDisplay` and `SetInterpolation { mode }`. `automatic` is true until the user picks a mode (each app start begins automatic; the choice is not
   persisted and not part of a setup file). Automatic means sin(x)/x when the screen has fewer than 250 samples per division (`(window_len - 1) / 10 < 250`),
   otherwise linear. `interpolation` is always the effective mode.
3. **What is sent per mode.** Sinc: the window upsampled by `factor = clamp(ceil(250 / spd), 1, 512)`, so the frame carries about 2501 points per channel
   whatever the time base; header `N` and the sample interval (interval divided by `factor`) and the trigger index (times `factor`) follow. Linear and Dots: the original
   window; the graph draws lines or 3 px squares (design 8.2). Dots therefore show only real samples. The layout of ADR 0011 does not change.
4. **Changing the mode** re-encodes the last captured frame and publishes it again, so the change shows on the next redraw also while stopped or frozen,
   without restarting acquisition.

## Alternatives considered
- **Interpolate in the browser.** Simple to ship, breaks P1b, and the 4 channel x 2501 point work belongs in tested Rust.
- **Always send the maximum and let the graph decimate.** Sends 40 KB at every time base for a need that exists only at fast ones.
- **Mode as frontend state and send both versions.** Doubles the payload.
- **Persist the choice.** Not asked; adds a settings key and a migration.

## Consequences
- The M1 frontend `interpolation` store is replaced by a `display` store; `Graph`'s `dots` prop stays.
- Cursor and trigger readouts use screen fractions and the time base, so extra points change nothing for them.
- Sinc rings at very sharp edges (Gibbs); that is what the mode is called and is documented in the tooltip.
