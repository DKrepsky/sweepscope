# ADR 0021: Zero calibration is its own feature module; the instrument is lent to it through four kernel messages

Status: Accepted
Date: 2026-11-24
Issue: #33, #34

## Context
#33 needs a guided session that measures the zero offset of every voltage range of each input, stores it per unit and applies it to every number shown or exported. Rules from the design: nothing is stored until all four channels are done, acquisition is paused while the dialog is open, a channel that is off still gets a step, a not-shorted input stays on its channel.
Facts: a frame carries `zero_code` per channel and every consumer converts with it, so correcting it once corrects everything; hardware offsets are per channel and range and look like an input-referred analogue offset (protocol 9, A.3); `acquisition` owns the setup and run intent, `device` owns the thread; bus handlers must not block on hardware (ADR 0010).

## Decision
1. **New feature crate `calibration`** (architecture 4.2: one capability end to end). It owns the session, the sweep rules and statistics, the offsets, their file, the slice `zero_calibration` and the accuracy texts. It depends only on the kernel and `libs/engineering-format`. The sweep logic is feature domain logic and lives in the module's domain, not in a lib.
2. **Four kernel messages** (reviewed addition, ADR 0010 item 7; total thirteen): requests `HoldInstrument { on, reason }` and `CaptureProbe { id, setup, discard, keep }`, events `ProbeCaptured { id, frames }` and `ZeroOffsetsChanged { offsets }`.
   - `HoldInstrument(on)`: `acquisition` stops capturing, remembers whether it was running, rejects setup and run commands with the reason, and on release re-sends its model and resumes if it was running. Refused while auto-setup runs.
   - `CaptureProbe`: `acquisition` applies the given setup, discards `discard` frames, returns the next `keep` frames in `ProbeCaptured`; none are published as `FrameCaptured`. Only valid while held. Same mechanism as the auto-setup probes (ADR 0020).
   - `ZeroOffsetsChanged`: published by `calibration` when a unit becomes ready, after a commit, and for an unknown unit (empty offsets). `acquisition` subscribes, corrects `zero_code` of every published frame and roll item, re-derives the held frame, and shifts the trigger level by the offset. `FrameContext.range_verified` becomes `calibrated`.
3. **The module builds the probe setup itself** from the capabilities in `ConnectionChanged`: all channels enabled, DC, bandwidth limit off, position 0, one range on all channels, 2 ms per division, auto sweep. Four channels give the four-channel ADC mode of the reference data. The first frame per range is discarded, the next three are kept.
4. **Offsets are stored in volts at the input**, per channel and range, applied with the counts per division of the frame (`zero_code += volts / range * counts`). Coupling, probe, bandwidth limit and inversion are not in the key. Offsets are measured at position 0 and applied everywhere. Assumptions to check on hardware.
5. **Commit is atomic**: pending results of finished channels live in the session; one file write after channel 4, publish after the write. A failed write ends the session with an error.
6. **Unit identity**: `DeviceIdentity.unit_id` (`family/pcb/serial` from the driver's info string; `simulator` for the simulator; none when the serial field is empty, which disables calibration). Not serialised to the frontend.
7. The module publishes the slice with a revision (ADR 0013), takes four commands (`open`, `start`, `measure`, `close`) and returns at once; the sweep runs on the module's own thread.

## Alternatives considered
- **Part of `acquisition`** (first draft): no new messages, but a second capability inside a crate that already holds setup, math and auto-setup; rejected by the human.
- **Sweep logic in `libs/zero-offset`**: it is domain logic with hardware rules, not generic code (architecture 4.4).
- **Calibration talks to `device` directly**: two writers of the device setup and a revision clash; the hold keeps `acquisition` the only writer.
- **Offsets in codes**: counts per division differ by mode (21.1 to 24.15), up to 14 % error.
- **Apply offsets in each consumer or in `device`**: five places to keep equal, or hidden from the state.

## Consequences
- Four new kernel messages and one new feature crate; acquisition's touchpoints are exactly the hold, the probe and the offsets event.
- A saved capture carries the corrected `zero_code` and a `calibrated` flag.
- The CLI (ADR 0022) loads the same offsets file, so headless captures are calibrated.
- Risk: serial-like field is vendor data; shorted-input limits are tuned on one unit.
