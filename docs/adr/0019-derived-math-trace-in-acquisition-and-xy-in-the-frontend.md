# ADR 0019: The math channel is instrument state computed once in `acquisition` and carried with the frame; XY is a presentation transform

Status: proposed
Date: 2026-11-24
Issue: #30, #31 (also touches #29)

## Context
A math channel (A+B, A-B, AxB, invert A) must behave like a channel: its own trace, scale and position (#30 AC1), measurements (AC2), a reason when a source is off (AC3) and an explicit scale and unit (AC4). Four consumers need
its samples: `display` (trace), `measurement` (M column), `spectrum` (FFT of M) and later `export`. Features may not import each other (P2). Per-channel `invert` and `probe` already live in `acquisition` and travel in `FrameContext`
(ADR 0011, 0014); the math setting is the same kind of thing (instrument state that downstream modules apply). XY draws two channels of one frame against each other.

## Decision
1. **State.** `acquisition` owns `MathState` in the instrument slice: `{ enabled, operation: Add | Subtract | Multiply | InvertA, source_a, source_b, scale (1-2-5 table, V/div or V2/div), position_divs, unit, status: Ok | Mixed | Unavailable(reason) }`,
   changed by `set_math` (patch) and `step_setting` / `reset_setting` with targets `MathScale` and `MathPosition`. One math channel in v1 (design D8). Defaults when first enabled: `Subtract`, A = CH1, B = CH2 (or the first two enabled channels),
   scale = nearest table value at or above the larger source scale (Multiply: the product). Not stored in setup files in v1. `Mixed` means the two sources have different volts per division for Add and Subtract; the
   scale and unit are always published as text.
2. **Computation once.** A new pure `libs/math-channel`: `apply(operation, a_volts, b_volts) -> Vec<f64>` over probe-scaled volts. `acquisition` runs it per accepted frame and publishes
   `FrameCaptured { frame, context, derived: Option<Arc<DerivedTrace>> }` where `DerivedTrace { volts: Vec<f64>, unit, volts_per_div, position_divs }` (record length, probe-scaled; invert of sources is ignored, the physical signal is used).
   `display` scales it to divisions as channel 5 (mask bit 4 of ADR 0011, no layout change), `measurement` measures it as column `M` (channel 5, clipping never applies), `spectrum` transforms it. A missing source (off) or roll mode gives `derived = None` and `Unavailable` with the reason.
3. **XY is frontend-only.** Both axes are already in the frame as divisions from the top edge; XY maps `x = 0.5 + (4 - X[i]) / 10`, `y = Y[i] / 8` in the graph (a renderer concern, coordinates only). X and Y selection is presentation state (design 4.1).
   The effective interpolation of the frame is whatever `display` sent (upsampled at fast time bases); XY always draws straight segments between the points it receives. A channel position moves the figure on its axis.
4. **No backend XY state, no XY command.** The backend's only XY duty is to keep delivering frames.

## Alternatives considered
- **A `math` feature crate publishing its own trace.** Clean ownership, but the trace must reach `display` and `measurement` in step with the frame it came from; two latest-wins events cannot be matched without a frame id, which adds a new contract and races.
- **Each consumer computes math from the frame and the setting.** Three copies of the same step and three places that read the math setting.
- **Compute math in the browser.** Breaks P1b and measurements still need it in Rust.
- **XY computed in the backend as a point list.** Heavy, duplicates the frame, and the view choice is presentation state.

## Consequences
- `FrameCaptured` gains one optional field (kernel contract change, one file); consumers that ignore math are unchanged.
- `acquisition` grows by one small module; if it outgrows the crate it is split (ADR 0010 item 1).
- Measurements, FFT and display of M all use the same samples, so they cannot disagree.
- Math is only for frame captures (not roll).
