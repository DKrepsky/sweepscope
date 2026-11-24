# Implementation spec: M4 Polish (#33, #34, #37, #38, #50)

Status: draft, NEEDS_HUMAN_APPROVAL (G3)
Links: [design spec rev 4](design-spec.md); ADRs [0021](../../adr/0021-zero-calibration-as-its-own-module.md), [0022](../../adr/0022-headless-capture-cli-as-a-second-composition-root.md), [0023](../../adr/0023-remove-the-verification-markers-from-the-product.md), [0024](../../adr/0024-packaging-and-application-identifier.md).
Detail for developers (packages, order, tests, numbers, risks): [work-plan.md](work-plan.md), not part of the review.

## 1. What changes
| Story | Result |
|---|---|
| #33 | A guided zero calibration per device: modal dialog, bottom-bar status tag, offsets applied to everything shown or exported |
| #34 | Accuracy page in the docs; the status tag tooltip carries the accuracy summary |
| #37 | `.deb` and AppImage, udev rule in the `.deb`, final identifier and icon |
| #38 | Headless capture command with distinct exit codes |
| #50 | No UNVERIFIED or PARTIALLY VERIFIED marker anywhere; connection status restyled |

No acceptance criterion is removed or weakened.

## 2. Modules and libs
| Module | Change and responsibility |
|---|---|
| `calibration` (new feature) | Owns the whole capability: session (intro, channel 1 to 4, measuring, not shorted, done, failed), the sweep over all twelve ranges per channel, the shorted-input rules and statistics, the offsets and their file (several units side by side), the status slice and the accuracy texts. Depends on the kernel and `libs/engineering-format` only. It does not touch the device: it borrows the instrument through the kernel messages below |
| `acquisition` | Minimal touchpoints: can be held (paused, commands refused, state restored on release), captures probe setups on request, applies published offsets (corrects `zero_code` of every frame and roll item, compensates the trigger level). Loses the verification-marker fields |
| `device` | Reports a unit identity (vendor serial-like field with family and PCB); the simulator gains a shorted input, a realistic zero error and noise per range |
| `measurement`, `export`, `display`, `spectrum` | Marker and old accuracy-note removal; export writes a `calibrated` flag instead of `range_verified` and gets a pure `encode_frame` function; raw format version 1 stays, old files open |
| `cli` (new crate) | Second composition root without Tauri (ADR 0022) |
| `kernel` | Four new messages and two fields (section 3) |
| Shell | Wires `calibration` (snapshot member, event, four commands), identifier and bundle settings, one simulator environment variable |
| Frontend | Marker removal in 14 places, connection item, calibration tag and tooltip, calibration dialog, toolbar chip `Paused (calibration)`, a calibration store and use cases in the app layer |
| Libs | No new lib. `engineering-format` may gain one quantity if volts do not reach microvolts |

## 3. Interfaces (public contracts)
New kernel messages (reviewed addition to ADR 0010's list; total thirteen):
- `HoldInstrument { on, reason }` request: `calibration` to `acquisition`. Stops capturing, refuses setup and run commands with the reason, restores model and run state on release. Refused while auto-setup runs.
- `CaptureProbe { id, setup, discard, keep }` request and `ProbeCaptured { id, frames }` event: `acquisition` applies the setup and returns frames; none reach the display.
- `ZeroOffsetsChanged { offsets }` event: `calibration` to `acquisition`, published when a unit becomes ready, after a commit, and for an unknown unit (empty).

Kernel data: `DeviceIdentity.unit_id` (not sent to the frontend); `FrameContext.calibrated` replaces `range_verified`.

`calibration` public surface: requests `OpenZeroCalibration`, `StartZeroCalibration`, `MeasureZeroChannel`, `CloseZeroCalibration` (all return at once), request `GetZeroCalibration`, event `ZeroCalibrationChanged`. The slice `ZeroCalibrationState` has a revision, an availability with reason, the unit state (calibrated, stored date, tooltip rows and notes) and the session (step, channel, finished channels, progress, not-shorted lines, failure text, the done table of twelve ranges by four channels in volts). Shell commands: `open_zero_calibration`, `start_zero_calibration`, `measure_zero_calibration`, `close_zero_calibration`; event `zero_calibration_changed`; snapshot member `zeroCalibration`.

`export`: `encode_frame(frame, timestamp, format)`, file extension and layout text helpers.
CLI: `sweepscope-cli capture --setup F [--count N] [--out DIR] [--format csv|raw] [--timeout S] [--device-wait S] [--simulator]`; exit codes 0 ok, 1 failure, 2 usage, 10 no device, 11 timeout, 12 permission denied.

## 4. Workflows
- **Calibration.** Open pauses acquisition (the hold). Per channel the user shorts the input and presses Calibrate; for each of the twelve ranges `calibration` asks for a probe capture (four channels, 2 ms/div, DC), drops the first frame, averages the next three, and checks the input is really shorted (flat and close to zero). A bad input stops the channel with measured numbers and stores nothing. After channel 4 all offsets are written at once and published; Cancel or Esc at any earlier point stores nothing. Closing releases the hold: the previous setup and run state return (a Single is cancelled, not re-armed). Offsets are in volts at the input, per channel and range, stored per unit. A different unit shows NO CALIBRATION.
- **Applying.** Every frame and roll item is corrected once in `acquisition`, so trace, cursors, measurements, FFT, math, exports and recordings use calibrated volts; saved files carry the corrected zero code and the `calibrated` flag.
- **Tooltip.** The status tag shows, when not calibrated, typical zero errors of one measured unit; when calibrated, the largest stored offset per channel (ranked relative to the range, shown in volts with the range) plus the accuracy notes.
- **CLI.** Waits for the scope, loads the setup, runs, writes N frames, stops. Stored offsets apply; the app's files are never modified.
- **Packaging.** `tauri build` makes the `.deb` (rule, icons, dependencies) and the AppImage; a manual CI job builds, nothing is published.
- **Markers.** Removed from data and UI; ranges behave as before.

## 5. Dependencies
No new dependencies (CLI arguments are parsed by hand; the bundler tooling already exists).

## 6. Decisions for the human
| ID | Decision | Default |
|---|---|---|
| E1 | Calibration is its own module `calibration`, no new lib (human ruling, applied; ADR 0021) | Settled |
| E2 | Offsets in volts at the input, measured with four channels at 2 ms/div and position 0; the key is channel and range only | Approve |
| E3 | Tooltip "largest offset" ranks by offset divided by range, shown in volts with its range (`CH1 +0.66 mV at 2 mV/div`); plain volts would always pick 10 V/div | Approve; Designer adds the range suffix |
| E4 | CLI in M4 with exit codes 0, 1, 2, 10, 11, 12 | Schedule |
| E5 | Q-E3 and Q-E4 (marker flag in scale options, old accuracy note) are void | Approve |
| E6 | The new identifier orphans `~/.config/dev.sweepscope.app`; no migration (nothing released) | Approve |
| E7 | Export files keep raw format version 1; `range_verified` becomes `calibrated`; old files still open | Approve |
| E8 | Only the `.deb` installs the udev rule; the AppImage cannot, the install guide covers it | Approve |
| E9 | No serial-like field on the scope: the tag is disabled with the reason | Approve |
| E10 | Accuracy wording with "about 0.1 division" | Settled (PM approved) |
| E11 | Keep the `Exp` badge on the Video trigger, reword its tooltip | Keep |

PRD, design and architecture amendments: PM amends the PRD (goal 3, 5.2, VER-3, US-14, APP-5, roadmap, risks; note #50 AC3 names VER-1 and VER-2, which are channel settings in the PRD); Designer amends the design document; the architecture document is updated by me after approval.

## 7. Critical risks and hardware limits
- **Unit identity** is a vendor-defined field (driver protocol 3.1, unverified): two units could share offsets. Mitigated by family and PCB in the key and by disabling calibration when the field is empty.
- **Shorted-input detection** is tuned on one unit (PRD 5.2). A quiet open input may pass; a good short may be refused. Limits are constants checked on hardware; the dialog shows the measured numbers.
- **Offset assumptions** (independent of mode, position, coupling, bandwidth limit) are unverified; position dependent errors of 2 to 3 counts remain and the tooltip says so. Checked on hardware before release.
- **Kernel growth**: four messages; acquisition's touchpoints are only those.
- **Identifier** change cannot be repeated later without orphaning user data.
- Driver limits unchanged (PRD 5.2): `capture()` is not cancellable, so Cancel waits for the current probe (under a second).
