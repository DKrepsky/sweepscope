# Addendum: hardware feedback on M4 (zero calibration)

Status: draft, for human review. Extends [implementation-spec.md](implementation-spec.md); detail for developers in [work-plan.md](work-plan.md) section 9. No ADR: both changes stay inside the `calibration` module and its slice; no kernel message is added.

## 1. What the hardware run showed
1. A real short on CH4 was refused: level 1.45 mV from zero at 2 mV/div, limit 994 uV. The limit was 12 codes at every non-gate range, which is 0.99 mV at 2 mV/div. The protocol (A.3) lists 0.66 mV on one unit's first channel; another channel of the same unit reads 1.45 mV, so the zero error is a channel-dependent analogue offset of the front end, not noise, and a code-based limit is too tight where one code is only 83 uV.
2. The status tag tooltip lists offsets for ranges the channels are not on, and carries accuracy notes the human does not want.

## 2. Shorted-input rules (item A)
The gate range (100 mV/div) still decides "signal present": peak to peak above 6 codes (25 mV), s.d. above 1.5 codes, a clipped sample, or a level above 8 codes (33 mV). Noise limits at the other ranges stay (3 x the A.3 reference s.d.). Only the level limit at the other ranges changes:

| Range | Before | After |
|---|---|---|
| level limit, other ranges | 12 codes (0.99 mV at 2 mV/div, 2.5 mV at 5 mV/div, 5 mV at 10 mV/div) | the larger of 12 codes and **2.5 mV at the input** (so 2 mV/div: 30 codes = 1.25 div; 5 mV/div and above: unchanged) |

Why 2.5 mV: the largest real zero error seen is 1.45 mV (1.7 x margin); a front-end offset budget of a few mV is plausible for an 8-bit analogue front end whose vendor trim table has no separate row for the two most sensitive ranges (protocol A.3: "residual amplifier offset"). It stays small enough to catch what the gate misses: a DC level between 2.5 mV and 33 mV passes the gate but is rejected at 2 mV/div and 5 mV/div; a DC level below 2.5 mV cannot be told from an offset by any test. Upper bound from the data: probing is at position 0 where the zero code is about 31 to 39, so a negative error beyond about 30 codes clips; 2.5 mV is inside that.

## 3. `Continue` (item A, workflow)
When the check fails the dialog offers `Cancel`, `Try again` and `Continue` (design by the Designer). `Continue` accepts the rejected measurement as the offset of that range and carries on with the channel.
- **New request** `ContinueZeroChannel` (module public surface, returns at once, shell command `continue_zero_calibration`). Valid only at step `NotShorted`; no new step.
- **Sweep resumes, nothing is re-measured.** The failed range is judged at the gate first (or later), so the sweep keeps the readings accepted so far and the rejected one; `Continue` stores the rejected reading's offset and measures the remaining ranges. From then on checks are off for this channel, except that a clipped reading is never storable (the offset would be wrong by an unknown amount). `Try again` still starts the channel over from the gate.
- **Clipped or unmeasurable readings do not offer `Continue`**: the slice says so with `canContinue = false`.
- **Slice**: `ZeroSession` gains `canContinue` and `overridden` (channels stored despite a failed check), so the Done step can say so. The commit stays atomic after channel 4; Cancel before it still stores nothing.
- The unit is calibrated afterwards like any other; the stored values are the measured ones.

## 4. Status tag tooltip (item B)
- **The frontend picks the row, the backend supplies every text.** The calibration module does not learn the channel ranges. It would need either a new kernel message or a dependency on `acquisition` (not allowed), and it would republish its slice on every range knob turn. Instead the slice carries preformatted texts for all twelve ranges and three probe factors per channel; the tag combines them with the instrument slice (enabled channels, selected range index, probe), which the frontend already holds. Numbers are still never formatted in the frontend.
- **Interface change**: `TooltipText { heading, channels: [ChannelTooltip; 4] }`, `ChannelTooltip { cells: Vec<[String; 3]> }`, one cell per range index, one string per probe 1x, 10x, 100x (for example `+0.66 mV at 2 mV/div`, `+6.6 mV at 20 mV/div` with a 10x probe, so the text matches the range shown on the channel). `rows` and `notes` are removed; ts-rs files change.
- **Not calibrated**: the typical zero error of that range (A.3: +0.66, +1.04, +0.83, +0.83 mV on the four most sensitive ranges, `up to 4.1 mV (2 counts)` style above). **Calibrated**: the stored offset of that range, `-` when none is stored.
- Shown per enabled channel only; the heading and the stored date line stay; the Gain, extreme-position and Resolution notes are dropped.

## 5. Critical risks
- 2.5 mV is tuned on two readings of one unit; a unit with a larger offset is refused until `Continue` is used, which is exactly why `Continue` exists. Check CH1 to CH4 on hardware.
- `Continue` can store a wrong offset if the user continues with a real signal attached; the Done step and the tooltip heading say that a channel was stored with a failed check.
- Probe-scaled tooltip text assumes the instrument slice range index equals the capability range index; a test pins it.
