# Work plan: M4 Polish (internal, for developers, QA and reviewers; the human reviews the implementation spec only)

Read with [implementation-spec.md](implementation-spec.md), [design-spec.md](design-spec.md) and ADRs 0021 to 0024. Branch `feat/m4-polish`; per package `feat/m4-polish-<wp>-<slug>` from it after its dependencies merged; a package never edits a file owned by another package (exception: a later package may fix a test of an earlier merged package when the edit follows directly from its own change, and says so in its handoff); commits stay local until G4; generated-types check clean after every merge.

## 1. Interfaces in detail

### 1.1 Kernel (WP1)
```rust
DeviceIdentity { .., #[serde(skip)] #[ts(skip)] unit_id: Option<String> }
FrameContext   { .., calibrated: [bool; 4] }                    // replaces range_verified
pub struct ZeroOffsets { pub unit_id: String, pub volts: [Vec<f64>; 4] }   // per channel, index = range index, input-referred volts; empty vecs = none
pub struct HoldInstrument { pub on: bool, pub reason: String }  // Request<Response = ()>; Err(Recoverable) when refused
pub struct CaptureProbe { pub id: u64, pub setup: AcquisitionSetup, pub discard: u8, pub keep: u8 }   // Request<()>, returns at once
pub struct ProbeCaptured { pub id: u64, pub frames: Vec<Arc<Frame>> }  // Event; empty frames = failed or timed out
pub struct ZeroOffsetsChanged { pub offsets: Option<ZeroOffsets> }     // Event; None = no offsets for the connected unit
```
### 1.2 `calibration` crate (WP2 skeleton, domain; WP10 module)
Slice and requests as in the spec section 3:
```rust
ZeroCalibrationState { revision, availability: Availability, unit: Option<UnitCalibration>, session: Option<ZeroSession> }
UnitCalibration { calibrated, stored_at_utc: Option<String>, tooltip: TooltipText { rows: Vec<TooltipRow{label,value}>, notes: Vec<String> } }
ZeroStep { Intro, Channel, Measuring, NotShorted, Done, Failed }
ZeroSession { step, channel: Option<u8>, done_channels: [bool;4], progress: Option<f64>, not_shorted: Option<Vec<String>>, failure: Option<String>, table: Option<Vec<OffsetRow{ range: FormattedValue, offsets: [Option<FormattedValue>;4] }>> }
OpenZeroCalibration, StartZeroCalibration, MeasureZeroChannel, CloseZeroCalibration (Request<()>), GetZeroCalibration, ZeroCalibrationChanged (Event)
```
`availability.reason`: `No device`, `Zero calibration needs a ready scope`, `This scope reports no serial number`, `Auto-setup is running` (from a refused hold). Commands registered: `open_zero_calibration`, `start_zero_calibration`, `measure_zero_calibration`, `close_zero_calibration` (args `{}`, result `null`).
Domain layout (pure, unit tested): sweep state machine (`ZeroSweep::next(Option<&Reading>) -> Step{Measure(plan), Done(offsets), NotShorted(rejection)}`), statistics, offsets and file (`sweepscope-zero-offsets` version 1, units map, `.corrupt` copy once), session machine, slice view builder, accuracy texts. Module: handlers, own thread, subscriptions to `ConnectionChanged` and `ProbeCaptured`, `Storage` access, probe setup builder (all channels enabled, DC, bandwidth limit off, position 0, one range, the 2 ms/div time base found by seconds in the capabilities, edge trigger on the target channel level 0 at 50 %, auto sweep; discard 1, keep 3), timeouts (3 s per probe, 40 s per channel).

### 1.3 Sweep rules (constants in one file, tune after hardware)
Offset codes = mean over kept codes minus `zero_code`; volts = codes x range / counts per division (24.15 in the four-channel mode). Gate range 100 mV/div measured first: not shorted if peak to peak above 6 codes or s.d. above 1.5 codes (`SignalPresent`), a clipped sample (code 0 or 255), or |mean - zero| above 8 codes (`LevelOffZero`). Other ranges: `TooNoisy` if s.d. above `max(2, 3 x reference)`, `LevelOffZero` beyond 12 codes. Reference s.d. per range from protocol A.3: `[24, 8.6, 4.1, 2.1, 0.9, 0.5, 0.5, 0.05, 0, 0.4, 0, 0]` codes (standard error of the mean about 0.2 code at 2 mV with 12 288 samples). Rejection lines (backend text, volts): `Peak to peak 49.7 mV at 100 mV/div, limit 24.8 mV.`, `Level 33.1 mV from zero at 100 mV/div, limit 33.1 mV.`, `Noise 12.0 mV rms at 2 mV/div, limit 3.0 mV.`
Progress = ranges done / 12. Estimated about 5 s per channel (12 x (0.15 s configure + 4 x 0.07 s)); budget 40 s.

### 1.4 Offset application in `acquisition`
`zero_code += volts / range_volts_per_div * frame.counts_per_div` per channel with data, range matched from the frame's `volts_per_div` against the capabilities; applied to `FrameCaptured`, roll items and the re-derived held frame (raw frame stays held); `context.calibrated[ch]` true when offsets exist for the unit; trigger level sent to the device shifted by the offset of the source channel and range. While held: setup and run commands fail with the hold reason, `ControlsAvailability` carries it; release re-sends the model with a new revision and resumes capturing if it was running; Single in progress is cancelled. `CaptureProbe` while not held is refused; probes are applied with fresh revisions, frames of other revisions are dropped; probe frames never published.

### 1.5 Accuracy texts (backend constants in `calibration`, golden tested; "about 0.1 division" approved by the PM)
Not calibrated: heading note `Typical zero error of one measured unit:`; rows `2 mV/div +0.66 mV (8 counts)`, `5 mV/div +1.04 mV (5 counts)`, `10 mV/div +0.83 mV (2 counts)`, `20 mV/div +0.83 mV (1 count)`, `50 mV/div and above up to 2 counts between channels`. Calibrated: one row per channel, `CH1 +0.66 mV at 2 mV/div` (largest by |offset| / range), `-` when none. Notes: (calibrated only) `Zero error removed per channel and range, measured with the input shorted (about 12 000 samples averaged).`; `Gain: about 2 % from 50 mV to 1 V/div (one unit measured), not verified outside that range.`; `At extreme vertical positions readings can still be off by 2 to 3 counts (about 0.1 division).`; `Resolution is one count, 1/24 of a division.` Derivation: A.2 counts per division 24.1 vs 24.15, 23.8 at 100 mV/div, volts 0.294 to 0.301 for 0.298 V, 0.99 to 1.00 for 0.994 V (worst 1.5 %); A.3 mean codes at position 128 (136, 131 to 133, 129 to 130, 128.9, then 128.0 to 128.3) times range / 24.15; protocol 9 for channel skew (2 counts).

### 1.6 `device`, `export`, shell, CLI
- Hantek `unit_id = "{family}/{pcb}/{serial}"` when the serial-like field is non-empty after trimming padding, else `None`. Simulator `unit_id = "simulator"`, `SignalKind::Shorted`, `SimulatorControl::set_zero_error_volts(channel, range, v)`, defaults from A.3 (0.66, 1.04, 0.83, 0.83 mV on the four most sensitive ranges, 0 above, small fixed per-channel skew), noise s.d. per range from 1.3, zero error added to every signal. Shell reads `SWEEPSCOPE_SIMULATOR_SIGNAL=shorted|open` (default open: the sine) and sets all four channels.
- `export`: `encode_frame(&FrameCaptured, &str, ExportFormat) -> Result<Vec<u8>, KernelError>`, `file_extension`, `raw_layout_text`, `iso_8601_utc`; `calibrated` replaces `range_verified` (CSV `chN_zero_offset_applied`); decoder treats a missing `calibrated` as false and ignores `range_verified`.
- Shell: snapshot member `zeroCalibration` (nine slices), event `zero_calibration_changed`, the four commands, composition of `calibration`, env variable, a test that the CLI's config folder name ends with the identifier in `tauri.conf.json`.
- CLI: ADR 0022; files `capture-NNNN.csv|bin` (raw also writes the layout document once); refuses to overwrite; waits for Ready up to `--device-wait` (default 8 s, includes the 2 s settle), `--timeout` default 10 s per frame; run sequence: wait Ready, `LoadSetup`, `SetRunning(true)`, collect frames of the loaded setup, stop, drop modules; config folder `$XDG_CONFIG_HOME/io.github.dkrepsky.sweepscope` or `~/.config/...`, `--config-dir` override; read-only storage; `cli` is a shell crate in `tools/check-architecture.mjs` with rule `cli-without-tauri`.

### 1.7 Frontend contracts
`app/calibration`: store with revision guard, use cases `openCalibration`, `startCalibration`, `calibrateChannel`, `closeCalibration`, selectors `selectCalibrationTag(zero, connection, recording)` (`none`, `calibrated`, `unavailable` with reason, recording active gives `Stop recording first`), `selectCalibrationDialog(zero)` (step n of 6: Intro 1, Channel k is k + 1, Measuring and NotShorted keep the channel's number, Done 6, Failed keeps its number), `selectPausedForCalibration`. `ui/step-chips`, `ui/connection-item` presentational; `CalibrationTag`, `CalibrationDialog` in `pages/main-window`; token `color-scrim`. Dates formatted in the frontend from the ISO text; numbers never.

## 2. Work packages
| WP | Owner | Files and crates (disjoint within a wave) | Depends on | Done when |
|---|---|---|---|---|
| WP1 Kernel | developer-1 | `backend/kernel/**` | none | section 1.1 compiled, tests green |
| WP2 Calibration domain | developer-2 | `backend/features/calibration/**` new crate: manifest, `lib.rs`, messages and slice types, `domain/**`, `tests/zero_domain*`; module files as empty stubs | WP1 | sweep, statistics, offsets and file, session, view, texts pass with synthetic frames; public surface of 1.2 compiles; may add a quantity to `backend/libs/engineering-format/**` (owner for this purpose) |
| WP3 Device | developer-3 | `backend/features/device/src/**`, `tests/**` except `accuracy_page.rs` | WP1 | unit id, shorted input, zero error, noise |
| WP4 Measurement and fixtures | developer-4 | `backend/features/measurement/**`, fixtures in `display/tests`, `spectrum/tests`, `recording/tests` | WP1 | markers and note gone, tests green |
| WP5 Export | developer-5 | `backend/features/export/**`, `docs/formats/frame-raw-v1.md` | WP1 | section 1.6 |
| WP6 Acquisition | developer-6 | `backend/features/acquisition/**` | WP1 | markers removed; hold, probe, offsets application and trigger compensation per 1.4 (tests with a fake calibrator publishing offsets and requesting probes) |
| WP7 Packaging | developer-7 | `src-tauri/tauri.conf.json`, `packaging/**`, `.github/workflows/ci.yml`, `docs/install.md`, `docs/usb-permissions.md` | none | both packages build; `dpkg -c` shows rule, icons (16 to 512), desktop file; `dpkg -I` shows webkit 4.1, gtk3, libusb |
| WP8 Accuracy page | developer-8 | `docs/accuracy.md`, `README.md`, `backend/features/device/tests/accuracy_page.rs` (new) | none | PM reviewed; test fails when page and capability table differ |
| WP10 Calibration module | developer-2 | `backend/features/calibration/src/module/**`, `tests/zero_module*` | WP2, WP6 for the integration test | with the simulator through the real acquisition: full session, commit, offsets applied, cancel, lost device, store failure, other unit, hold refused during auto-setup |
| WP11 Shell | developer-10 | `src-tauri/src/**`, `src-tauri/permissions/**`, `src-tauri/capabilities/**`, generated `commands.ts`, `StateSnapshot.ts` | WP2 | section 1.6 shell; snapshot and manifest tests |
| WP12 Frontend removal, app side | developer-11 | `app/main-window/{types.ts,mock-model.ts,mock-main-window.ts,select-channels*,select-horizontal*,select-trigger*,live-main-window*,live-app*,select-cursors.test.ts,select-alias-and-math.test.ts}`, `app/instrument/{test-data.ts,stores.test.ts}` | WP4, WP6 | no marker in data; tests updated |
| WP13 Frontend removal, view side | developer-12 | `ui/measurement-matrix/**`, `ui/screenshot/**`, `pages/main-window/{ChannelRow,ScaleStrip,HorizontalGroup,HorizontalReadout,TriggerGroup}*`, `{compose-screenshot*,MeasurementsPanel*,MainWindow.test.ts}`, `app/main-window/select-measurements*`, `pages/gallery/**`, `e2e/marker-a11y.spec.ts` | WP4, WP6, WP12 | design 4.1 rows 1 to 9 and 12 to 14; `Exp` tooltip reworded; test that no component reads `verified` |
| WP14 CLI | developer-5 | `cli/**`, root `Cargo.toml`, `tools/check-architecture.mjs` and test, `docs/cli.md` | WP2, WP3, WP5, WP6 | CLI tests below |
| WP15 Frontend api and app layer | developer-11 | `api/{fake-client,backend-client,tauri-client,index}.ts` and tests, `app/calibration/**`, `app/instrument/{stores.ts,index.ts}`, `app/main-window/{live-actions.ts,actions.ts,select-toolbar*,index.ts}`, `app/dialog-guard/**` | WP11, WP12 | store, selectors, use cases, fake client emulating the session |
| WP16 Frontend status bar | developer-13 | `ui/status-bar/**`, `ui/status-chip/**`, `ui/connection-item/**`, `pages/main-window/{MainStatusBar.svelte,CalibrationTag.svelte}`, `app/main-window/select-status.ts` | WP15 | design 2 and 2.1; `Accuracy` item and `Simulator` chip gone |
| WP17 Frontend dialog | developer-12 | `ui/step-chips/**`, `pages/main-window/{CalibrationDialog.svelte,calibration-steps/**,MainWindow.svelte,Toolbar.svelte}`, `design-system/tokens*`, `ui/dialog/**`, `e2e/m4-polish.spec.ts` (QA creates) | WP15 | design 3 and 8, focus rules, 200 % text scale |

Notes: developer-2 does WP2 then WP10; developer-5 WP5 then WP14; developer-11 WP12 then WP15; developer-12 WP13 then WP17; at most eight concurrent. WP6 is the gate for acquisition consumers; the calibration crate is developed against a fake acquisition (a test responder for hold, probe and offsets) so WP2 and WP6 run in parallel. ts-rs files are committed by the package that changes the type; `commands.ts` and `StateSnapshot.ts` only by WP11.

### Waves
| Wave | Packages | Starts when |
|---|---|---|
| A | WP1, WP7, WP8 | approval |
| B | WP2, WP3, WP4, WP5, WP6 | WP1 merged |
| C | WP10, WP11, WP12, WP14 | WP2 and WP6 merged (WP12: WP4 and WP6) |
| D | WP13, WP15 | WP12 and WP11 merged |
| E | WP16, WP17 | WP15 merged |
| F | QA, review | all merged |

## 3. Test strategy (given / when / then)
- **Sweep and rules** (WP2, synthetic codes): shorted codes centred on zero at the gate continue; 12 codes peak to peak gives `SignalPresent` with volts and limit; mean 9 codes above zero gives `LevelOffZero`; a clipped sample gives `SignalPresent`; all twelve accepted gives one offset per range with `(mean - zero) x range / counts` (8 codes at 2 mV and 24.15 is 0.662 mV); order is gate first then ascending, progress monotonic to 1; a noisy later range gives `TooNoisy` and no offsets.
- **Domain**: file with two units applies only the connected one; unknown unit none; corrupt file copied once to `.corrupt`; session refuses Calibrate at Intro, moves to Measuring at Channel 1, commits only after the fourth accepted channel, cancel after three commits nothing, rejection keeps channel and earlier results; largest per channel by |offset| / range; table 12 rows ascending with `None`; texts golden.
- **Module** (fake acquisition responder, then real simulator): full session reaches Done and stores the unit; open inputs give NotShorted and a retry after shorting succeeds; cancel at channel 3 writes nothing and releases the hold; device lost gives Failed; storage write failure gives Failed and no offsets published; other unit gives none; hold refused gives the reason in availability; probe timeout fails within the budget.
- **Acquisition** (WP6): while held setup and run commands fail with the reason; release restores the model with a new revision and resumes only if it was running; Single is cancelled; offsets event corrects `zero_code` of frames and roll items and the held frame is re-derived; counts per division of the frame are used (k = 1 case); trigger level shifted; probes are not published; `calibrated` in the context; marker fields gone.
- **Device / export / measurement**: shorted simulated channel centres on `zero_code` plus zero error with the table noise; Hantek unit id format and empty serial; old raw file with `range_verified` decodes, new header has `calibrated` only, CSV `chN_zero_offset_applied`; measurement slice has no marker; accuracy page guard.
- **CLI**: three files and exit 0 with `--count 3 --simulator`; exit 10 with no device within `--device-wait`; exit 11 for a normal sweep on a flat input with `--timeout 1`, written frames kept; existing target gives 1 and stays untouched; bad option gives 2; permission error gives 12; stored offsets give `zero_offset_applied: true` and the config folder is unmodified; folder constant matches the identifier; the architecture check fails when `cli` lists `tauri`.
- **Frontend**: tag states and click (none, calibrated, unavailable with reasons including recording); step numbers and chip states; progress bar value and indeterminate case; focus to the step heading never Calibrate, back to the tag on close; Esc closes at Done and cancels while Measuring; Failed note with one Close; toolbar chip `Paused (calibration)`; status bar has no Accuracy item or Simulator chip and the connection item shows the five states; no marker text, label or class anywhere; done table scrolls with sticky header at 200 % text scale (Playwright). E2E with `SWEEPSCOPE_SIMULATOR_SIGNAL=shorted`: wizard to Done, tag `CALIBRATED`, reopen shows the date, restart keeps it (isolated config); default signal shows the not-shorted note.
- **Packaging**: `dpkg -I` and `dpkg -c` checks; install in a clean Ubuntu 24.04 container resolves dependencies; Xvfb smoke start; AppImage runs with the right name and icon.
- **Hardware** (QA extends the checklist): H-M4-1 real short, Done table recorded, under 40 s per channel; H-M4-2 open input beside the calibrator is rejected, a quiet open input recorded either way; H-M4-3 shorted input reads 0 V within one code at 2, 5, 10 mV/div and the calibrator still reads about 1.9 V p-p at 1 V/div; H-M4-4 offsets hold with one channel at the fastest time base (k = 1), AC coupling and bandwidth limit; H-M4-5 replug and restart keep calibration and the serial-like field is stable; H-M4-6 CLI exit codes 10, 11, 12 on the real scope; H-M4-7 `.deb` on a clean machine activates the rule.

## 4. Risks (non-critical ones kept here)
Sweep slower than estimated (configure over 100 ms, relays settle; budget and determinate progress); `capture()` not cancellable (Cancel is immediate in the state, the late probe is ignored by id); the probe setup changes ranges on all four inputs while the dialog is open (inputs are passive, readings flicker; the dialog is modal and tells the user to disconnect probes); simulator values are model assumptions taken from A.3; AppImage tooling is downloaded at build time; glibc 2.39 floor; old SSR1 readers fail on new files (only this reader exists); `Quantity::Volts` may not reach microvolts (WP2 checks first); WP6 gates acquisition consumers (kept focused); trigger position uncertainty of about 6 samples is stated in the CLI docs only.

## 5. Dependencies
None added.

## 6. After approval
Done after approval: staff engineer updated `docs/architecture.md` (tree: `calibration`, `cli/`; 8.2 CLI rule; section 9 rows for ADR 0021 to 0024; section 11: Q-E3 and Q-E4 void; kernel messages thirteen), sends handoffs to developers and copies QA; QA writes the test plan and extends the hardware checklist before wave C ends. Accuracy page outline (WP8): how to read it; the one-unit statement and statement policy (measurements on one 6254BC, never tolerances, wording "about"); PRD 5.1 capability table with Verified, Partly verified, Not verified from protocol 10; twelve ranges (50 mV to 1 V/div verified against a DC source, others zero level measured, gain not checked) with A.3 zero errors; 36 time bases (0 to 22, 24, 25, 27, 30 measured); trigger position about 6 samples; what calibration does and does not remove; how to report another unit.

## 7. Review and QA rulings
(empty; appended after review and QA)

## 8. PRD amendments still open for the PM
Already amended by the PM: goal 3, 5.2 volts row, VER-3, US-14, M4 roadmap row. Left: APP-5, the risk table, DEV-4, TRG-3, "5 % of a division" becomes "about 0.1 division", and the README quote of the old accuracy sentence.

## 9. Hardware feedback round (human run on a 6254BC; see [addendum-hardware-feedback.md](addendum-hardware-feedback.md))

### 9.1 Rules and constants (`calibration/src/domain/rules.rs`, `sweep.rs`)
- New `OTHER_MIN_LEVEL_VOLTS = 2.5e-3`. `judge_other` level limit in codes = `max(OTHER_MAX_LEVEL_CODES, OTHER_MIN_LEVEL_VOLTS / range * counts_per_div)` (at 2 mV/div about 30.2 codes). The rejection line prints that limit in volts (`Level 1.45 mV from zero at 2 mV/div, limit 2.50 mV.`). Gate rules unchanged. Constants stay in one file.
- `Rejection` gains `continuable: bool` (false only for a clipped gate reading; true for every other rejection, since clipping is routine at 2 mV/div).
- `ZeroSweep` keeps the rejected reading: new state `pending: Option<Reading>`, `forced: bool`. `next(Some(reading))` on a rejection stores `pending` and returns `NotShorted` (the sweep object is kept, `order`, `cursor` and accepted offsets untouched). New `ZeroSweep::accept_rejected(&mut self) -> Result<Step, NothingPending>`: stores the offset of `pending` (same formula as an accepted reading), sets `forced`, advances the cursor and returns the next `Measure` or `Done`. While `forced`, `judge` returns no rejection: every check, clipping included, is skipped. `progress()` unchanged.

### 9.2 Session and module (`domain/session.rs`, `domain/view.rs`, `module/core.rs`, `module/mod.rs`, `messages.rs`)
- `SessionMachine::resume()`: `NotShorted` to `Measuring`, progress kept, `not_shorted` cleared, channel marked overridden; wrong step gives `WrongStep`. `ZeroSession` gains `can_continue: bool` (true only at `NotShorted` when the rejection is continuable) and `overridden: [bool; 4]`.
- `Core::apply(Step::NotShorted)` no longer drops the sweep: new `Active::pause_measuring()` clears `probe` and `channel_deadline` but keeps `sweep`. `Core::continue_channel()`: require `NotShorted` and a sweep with a pending reading, call `accept_rejected`, `machine.resume()`, set a fresh `CHANNEL_TIMEOUT` deadline, then `apply(step, channel)` (so `Done` goes through `accept_channel` and the commit). `Core::measure()` (Try again) drops the sweep as before.
- Request `ContinueZeroChannel` (`Request<Response = ()>`), registered next to the other four; returns at once; wrong step gives the existing `WRONG_STEP` recoverable error.
- Developer: developer-2 (owner of the crate). Files: `backend/features/calibration/src/**`, `tests/zero_domain_sweep.rs`, `zero_domain_session.rs`, `zero_module_session.rs`, `zero_module_support/**`.

### 9.3 Tooltip texts (`domain/texts.rs`, `domain/view.rs`, `messages.rs`)
- Types (ts-rs): `TooltipText { heading: String, channels: [ChannelTooltip; 4] }`, `ChannelTooltip { cells: Vec<[String; 3]> }`; `TooltipRow` removed. Cell k, probe slot p (factors 1, 10, 100): `"{sign}{mV} at {range x factor}"` for calibrated, with the offset multiplied by the factor and formatted with the existing `offset_text`; `-` when the stored vector has no entry. Not calibrated: sensitive ranges (A.3 constants, volts multiplied by the factor) as `+0.66 mV at 2 mV/div`; ranges from 50 mV/div: `up to {2 counts in volts} at {range}` with `2 x range / 24.15`, formatted with `engineering-format`. Both need the capability ranges: `unit_calibration(stored, ranges)` already takes them; the typical table is now a function of `ranges`.
- Heading: not calibrated `Typical zero error:`, calibrated `Zero error removed per channel and range, measured with the input shorted (about 12 000 samples averaged).` The three other notes and their constants are deleted. The stored date line stays (`stored_at_utc`).
- Golden test over all twelve ranges and three probes, both states.

### 9.4 Shell (developer-10, `src-tauri/src/**`, `permissions/**`, `capabilities/**`, generated `commands.ts`)
Command `continue_zero_calibration` (args `{}`, result `null`) with manifest entry, permission file, capability and integration test; `commands.ts` regenerated here only.

### 9.5 Frontend
- developer-11 (api and app): client method `continueZeroCalibration` in `api/{backend-client,tauri-client,fake-client,fake-zero-calibration}.ts` (fake emulates a rejection with `canContinue`, resume and `overridden`), `app/calibration/commands.ts` use case `continueChannel`, `selectors.ts`: `selectCalibrationTag(zero, connection, recording, instrument)` builds the tooltip lines: for every enabled channel, `channels[n].cells[range.index][probeSlot(channel.probe)]` (slot -1 hides the line), prefixed `CHn `; no enabled channel gives the heading only. `selectCalibrationDialog` exposes `canContinue` and `overridden`. Tests: given channels 1 at index 0 and 2 at index 11, when calibrated or not, then two lines with those ranges; a range knob turn changes the line without a backend event; probe 10x shows the x10 text; disabled channel hidden.
- developer-12 (view; after the Designer's revision): `NotShortedStep.svelte` gets the `Continue` button (behaviour per the Designer), `DoneStep.svelte` states overridden channels, `CalibrationTag.svelte` and its tests render heading plus per-channel lines (no table of ranges, no notes), `CalibrationDialog.svelte` wiring. e2e: QA extends `frontend/e2e/m4-polish.spec.ts` and `tauri-stub.ts`.

### 9.6 Tests (given / when / then)
- Given a shorted CH4 at 1.45 mV at 2 mV/div (simulator `set_zero_error_volts`), when measured, then accepted and stored. Given 3.0 mV at 2 mV/div, then `NotShorted` with limit 2.50 mV and `canContinue`. Given 6 mV at 5 mV/div, then rejected (12 codes). Given a 20 mV DC at the gate (level 8 codes = 33 mV not exceeded) then rejected at 2 mV/div, not at the gate.
- Given a rejection at 2 mV/div after the gate, when `ContinueZeroChannel`, then no probe is repeated for the gate, the 2 mV/div reading is stored, the remaining ranges are measured, the channel is accepted and `overridden[ch]` is true; given four channels, a single atomic write.
- Given a clipped gate reading, then `canContinue` is false and the request fails with `WRONG_STEP`. Given a forced channel with clipped low samples at 2 mV/div (noise 24 codes, zero code 35), then it completes; given a clipped reading off zero at 2 mV/div, then `canContinue` is true. Given `Try again` after a rejection, then the sweep restarts at the gate. Given Cancel after `Continue`, then nothing is stored. Given the channel timeout, then the deadline restarts at `Continue`.
- Request outside `NotShorted`: recoverable error, state unchanged.

### 9.7 Order
developer-2 (9.1 to 9.3, backend) and developer-11 (9.5 app, against the fake client and the new generated types: types first from 9.3, ts-rs files committed by developer-2) in parallel; developer-10 (9.4) after 9.2 compiles; developer-12 after the Designer's revision and developer-11. QA retests items 1 and 2 on hardware (`hardware-checklist.md`: add CH1 to CH4 shorted at 2 mV/div, and tooltip at mixed ranges).

### 9.8 Risks
At position 0 the zero code at 2 mV/div is about 31 to 39 with noise s.d. around 20 codes, so low samples clip and bias the mean by about 0.6 code (0.05 mV); acceptable, verify on hardware. If a unit shows more than 2.5 mV, change the constant only.
