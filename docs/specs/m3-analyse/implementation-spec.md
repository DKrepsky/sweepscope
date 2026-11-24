# Implementation spec: M3 Analyse (#22 to #31)

Status: draft, waiting for G3
Links: PRD 4.2, 5.1, 5.2, 6, 8.3 (US-02, 05, 07, 08, 10, 11, 15, 16); [architecture](../../architecture.md); [design](../../design.md); [design spec](design-spec.md) (approved G2, D1 to D11) and its [prototypes](prototypes/);
M2 spec [m2-measure](../m2-measure/implementation-spec.md); ADRs [0009](../../adr/0009-roll-stream-gap-item.md), [0011](../../adr/0011-frame-wire-format-and-window-crop.md), [0013](../../adr/0013-state-slices-revisions-and-command-style.md),
new [0017](../../adr/0017-roll-run-mode-items-and-recording-feature.md), [0018](../../adr/0018-spectrum-feature-and-fft-lib.md), [0019](../../adr/0019-derived-math-trace-in-acquisition-and-xy-in-the-frontend.md), [0020](../../adr/0020-auto-setup-as-a-decision-lib-driven-by-acquisition.md);
driver `docs/protocol.md` (6.1, 6.4, 7.4, 8.4, 10, A.4, A.5, A.7); [hardware checklist](hardware-checklist.md); spike report [4-roll-streaming](../../spikes/m0/4-roll-streaming.md).
Branch `feat/m3-analyse`; each developer works in a worktree branch off it (section 4.3).
Exit criterion (PRD 9, M3): US-02, 05, 07, 08, 10, 11, 15, 16 pass; roll, single shot, pulse trigger, auto-setup on the CAL signal, the alias threshold and the FFT of the CAL square pass on hardware; everything else on the simulator in automated tests.
Out of scope: zoom and pan (#32), reference traces, user formulas, statistics, offline captures, a second math channel, persistence in XY, FFT or roll, FFT peak interpolation, sharing math in setup files.

## 1. Scope

### 1.1 What is delivered
| Story | Deliverable |
|---|---|
| #22 Auto-setup | `libs/autoset`, driver in `acquisition`, status slice `autoset`, Toasts and the `No signal` tag (ADR 0020) |
| #23 Aliasing | `measurement` publishes `warnings` (alias per channel with hysteresis); badges in four places; status bar `Fs` highlight on channel toggle (already exists, test only) |
| #24 Single | `ArmSingle`, phase `Done`, sweep override, held full-intensity frame, chips |
| #25 Pulse trigger | `TriggerSetup.kind`, Hantek and simulator mapping, `set_trigger` fields, text parsing in `libs/engineering-format`, Pulse rows, error state |
| #26 Persistence | `DisplayState.persistence`, age-bucket accumulation in `ui/graph`, menu, tags, Clear |
| #27 Roll | roll run mode, worker roll loop, roll items, `RollWindow`, gap overlay, availability reasons (ADR 0017) |
| #28 Recording | feature `recording`, `.ssroll` v1 and its document, chip, error Banner |
| #29 FFT | `libs/fft`, feature `spectrum`, `SSS1` stream, FFT view, window menu, scale, peak panel (ADR 0018) |
| #30 Math | `libs/math-channel`, math state and derived trace in `acquisition`, row M, chip, M column, M in FFT (ADR 0019) |
| #31 XY | frontend transform in `ui/graph`, axis selectors, `ScreenNote` for cannot-draw cases (ADR 0019) |
| Shared | new tokens, `ScreenNote`, `Marker` kinds, `StatusChip` kind, `Button` busy, icons; dock height tiers re-measured (section 7) |

### 1.2 Decisions the human must make (G3)
| ID | Decision | Recommendation |
|---|---|---|
| E1 | Roll uses ADR 0009's accepted item stream (second Channel, `SSL1` items), not "send the window as a frame at 20 Hz" (simpler, 400 times the byte rate of the spike) | Approve the stream (ADR 0017) |
| E2 | New feature crates `spectrum` and `recording`, new libs `fft`, `math-channel`, `autoset`; no external dependency | Approve |
| E3 | `FrameCaptured` gains `derived` (math trace) and the kernel gains `RollPolled` and `RollCaptured` (messages eight and nine) | Approve (ADR 0017, 0019) |
| E4 | Dock height thresholds rise by 0.5 to 1.5 rem (section 7); the floor stays 1024 x 720 | Approve |
| E5 | Persistence ghosts in 8 age buckets at CSS resolution (section 3.9); grading quantised, overlap brightening between buckets only | Approve; Designer asked |
| E6 | The items of the PM message (roll limits, recording lifecycle, alias hysteresis, Single semantics, Auto-setup behaviour, FFT axis): defaults in section 9 | Approve defaults |

### 1.3 Rulings that touch acceptance criteria (reported, none decided here)
| Story, AC | Finding | Proposal | Needs |
|---|---|---|---|
| #22 AC1 (3 to 7 divisions) | The 1-2-5 range table has gaps at the 2 -> 5 steps: amplitudes of 14 to 15 units of the lower step (a 7 % band per decade) give 7.25 or 2.9 divisions on the two nearest ranges. 2 to 6 periods is always reachable (step ratio at most 2.5 < 3) | Read as "3 to 7 divisions where the range table allows; otherwise the nearest range, never clipping" and say so in the PRD | PM and human (G1 wording) |
| #26 AC1 (grading) | Quantised in 8 steps, see E5 | As E5 | Designer, PM |
| #24 AC2 (8 samples) | The driver measured +6 to -6 samples depending on channel count and time base (A.4); the reported trigger index is only accurate to about 20 bytes, so the check measures the edge in the data, not the index. Two samples of margin, other time bases untested | Verify on hardware at 1, 2 and 4 channels (checklist H-05); if one fails the AC needs the measured figure | QA, human on hardware |
| #27 AC4 (memory) | The spike's Rust RSS rule stays a condition: one-hour soak on the product with a real display, plus a blank-page run (checklist H-03) | As the spike report section 6 | QA, human |
| #23 AC2 (above a fifth of Fs) | Measured frequency above Fs/2 is itself an alias; no valid frequency means no warning (section 9, Q-M3-10) | Document in the tooltip help and the manual | PM |

## 2. Design of the change

### 2.1 Modules and crates touched
| Crate or folder | Change |
|---|---|
| `backend/kernel` | `contracts`: `TriggerKind`, `PulseCondition`, `DerivedTrace`, `DerivedUnit`, `RollContext`, `RollPolled`, `RollPolledItem`, `RollCaptured`, `RollItem`, `RollCloseReason`; `FrameCaptured.derived`; `TriggerSetup.kind` |
| `backend/libs/fft` (new) | `spectrum`, `peak_bin`, `Window` |
| `backend/libs/math-channel` (new) | `apply` |
| `backend/libs/autoset` (new) | decision state machine over plain observations |
| `backend/libs/engineering-format` | `parse_seconds`, quantities `Decibel` and `VoltsSquaredPerDiv`; golden vectors extended |
| `backend/features/device` | simulator `SignalSpec` (frequency, amplitude, offset, pulse train), pulse trigger in simulator and `hantek/convert.rs`; worker roll loop with indices, gaps, `RollPolled`, session lifecycle |
| `backend/features/acquisition` | single, roll mode, availability, pulse state and `set_trigger` fields, math state and derive, autoset adapter, `Phase::Done`, setup file keys for pulse |
| `backend/features/display` | persistence setting, math as channel 5, roll items |
| `backend/features/measurement` | `M` column (channel 5), `warnings` (alias) |
| `backend/features/spectrum` (new) | module, slice, `SSS1` encoder |
| `backend/features/recording` (new) | module, `.ssroll` writer, slice |
| `src-tauri` | composition (order below), commands, events, snapshot (eight slices), two more streams, manifest and permissions |
| `frontend/src/api` | stream kinds `roll`, `spectrum`; generated types |
| `frontend/src/app` | `instrument` (stores `spectrum`, `recording`; commands), `waveform` (decoders, `RollWindow`, feeds), `views` (new: view tab, XY axes, peak marker, persistence epoch), `formatting` (frequency four digits, dBV), `autoset` and `recording` (new use cases), `main-window` (model, selectors, actions), `layout` (thresholds) |
| `frontend/src/ui` | `graph` (ticks, NaN breaks, XY, persistence layer), `marker` (kinds), `screen-note` (new), `status-chip`, `button`, `measurement-matrix`, `icon` |
| `frontend/src/pages/main-window` | toolbar, stage, horizontal group, peak panel, dock (row M, Pulse), scale strip, status bar, measurement panel |
| `docs/formats/roll-record-v1.md` (new), `docs/architecture.md`, `docs/design-system/**` | format document; architecture updates after approval (section 10); tokens |

Construction order in `composition.rs` (consumers before producers, as M2): settings, recording, spectrum, measurement, export, acquisition, device, display. `display` last because it publishes on subscribe.

### 2.2 State ownership
| State | Owner | Notes |
|---|---|---|
| Roll mode, roll session, availability reasons | backend `acquisition` | derived from the time base; published in `InstrumentState.horizontal.roll` and `controls` |
| Single armed, phase `Done` | backend `acquisition` | `Phase` in the status slice |
| Pulse trigger (kind, condition, width), polarity (reuses slope) | backend `acquisition`, in `TriggerSetup` and the setup file | domain: it reaches the hardware |
| Math (operation, sources, scale, position, enabled) | backend `acquisition` (ADR 0019) | not in setup files in v1 |
| Auto-setup run, `no_signal`, last outcome | backend `acquisition`, status slice | |
| Persistence setting | backend `display` | domain by architecture 4.5; the accumulation itself is rendering (5.7) |
| Persistence ghosts, clear action | frontend `ui/graph` and `app/views` epoch | presentation |
| Spectrum window, scale, active; spectra | backend `spectrum` | `active` is the one promoted presentation fact (ADR 0018) |
| Alias warnings | backend `measurement` | `warnings` list in its slice (Q-E2) |
| Recording (active, file, bytes, failure) | backend `recording` | |
| View tab (`yt`, `xy`, `fft`), XY axis choice, peak marker mode and bin, selected channel | frontend `app/views` | presentation |
| Roll window (2500 samples per channel, gaps) | frontend `RollWindow` | bounded; the backend has no history |
| Banner dismissal, Toast shown flags | frontend, in memory | per id |

### 2.3 Data flow
```
device worker --RollPolled (index, gap)--> acquisition --RollCaptured (+Opened/Closed, context)--> display --DisplayRollItem--> shell --roll Channel--> RollWindow --> Graph
                                                                                          \--> recording --> .ssroll file, RecordingChanged (1/s)
device worker --CaptureCompleted--> acquisition --FrameCaptured (+derived math)--+--> display --DisplayFrameReady--> frame Channel --> Graph (Yt, XY, persistence)
                                                                                 +--> measurement --MeasurementsChanged (M column, warnings)
                                                                                 +--> spectrum (only while active) --SpectrumFrameReady--> spectrum Channel --> Graph (FFT)
                                                                                 +--> export (unchanged)
acquisition <--CaptureCompleted-- device  (auto-setup probes: not published as FrameCaptured)
frontend --arm_single / start_autoset / set_math / set_persistence / set_spectrum / start_recording / set_trigger (pulse)--> modules
```

## 3. Interfaces (fixed before work starts; WP1 implements all Rust types and stubs)

### 3.1 Kernel (`kernel::contracts`)
```rust
// setup.rs
pub struct TriggerSetup { /* existing fields */ pub kind: TriggerKind }        // Edge by default
pub enum TriggerKind { Edge, Pulse { width_s: f64, condition: PulseCondition } }   // slope = polarity for Pulse
#[derive(Serialize, Deserialize, TS)] #[serde(rename_all = "camelCase")] #[ts(export)]
pub enum PulseCondition { Equal, NotEqual, Greater, Less }

// messages.rs
pub enum DerivedUnit { Volts, VoltsSquared }
pub struct DerivedTrace { pub volts: Vec<f64>, pub unit: DerivedUnit, pub volts_per_div: f64, pub position_divs: f64 }   // record length, probe-scaled
pub struct FrameCaptured { pub frame: Arc<Frame>, pub context: FrameContext, pub derived: Option<Arc<DerivedTrace>> }

pub struct RollContext { pub session: u32, pub invert: [bool; 4], pub probe: [f64; 4], pub seconds_per_div: f64 }
pub enum RollPolledItem { Samples { first_index: u64, frame: Arc<Frame> }, Gap { after: u64, missed: u64 } }
pub struct RollPolled { pub setup_revision: u64, pub item: RollPolledItem }            // Event, device -> acquisition
pub enum RollCloseReason { Stopped, LeftRoll, Disconnected, SetupChanged }
pub enum RollItem { Opened, Samples { first_index: u64, frame: Arc<Frame> }, Gap { after: u64, missed: u64 }, Closed { reason: RollCloseReason } }
pub struct RollCaptured { pub session: u32, pub item: RollItem, pub context: RollContext }   // Event, acquisition -> display, recording
```
Rule for consumers of `RollCaptured` (display, recording): keep `expected_index`; on `Samples` with `first_index > expected_index` insert `Gap { after: expected_index - 1, missed: first_index - expected_index }` before the samples; on `Gap` set `expected_index = after + 1 + missed`.
The device worker counts indices from 0 at the start of a session (`RollPolled` has no session; `acquisition` assigns it).

### 3.2 Libs
```rust
// libs/fft
pub enum Window { Rectangular, Hann, Hamming, Blackman }
pub struct Spectrum { pub bin_hz: f64, pub rms: Vec<f64> }                 // N/2 + 1 bins, rms volts per bin, window gain corrected
pub fn spectrum(volts: &[f64], window: Window, sample_rate_hz: f64) -> Result<Spectrum, FftError>   // length must be a power of two, at least 8
pub fn peak_bin(spectrum: &Spectrum, window: Window) -> Option<usize>      // skips DC and the window's main lobe: 1, 2, 2, 3 bins

// libs/math-channel
pub enum Operation { Add, Subtract, Multiply, InvertA }
pub fn apply(operation: Operation, a: &[f64], b: Option<&[f64]>) -> Option<Vec<f64>>   // None when b is needed and missing or lengths differ

// libs/autoset  (plain data, no dependency)
pub struct Tables { pub range_volts_per_div: Vec<f64>, pub range_verified: Vec<bool>, pub max_position_divs: Vec<f64>, pub time_base_s_per_div: Vec<f64>,
                    pub roll_from_index: usize, pub counts_per_div: f64, pub record_len: usize, pub samples_per_div: f64 }
pub struct ChannelObservation { pub min_code: u8, pub max_code: u8, pub clipped: bool, pub period_s: Option<f64> }
pub struct Observation { pub channels: [Option<ChannelObservation>; 4], pub sample_interval_s: f64 }
pub struct Plan { pub range_index: [u8; 4], pub position_divs: [f64; 4], pub time_base_index: u8, pub trigger_source: u8, pub trigger_level_volts: f64 }
pub enum Step { Probe(Plan), Done(Plan), NoSignal(Plan), Failed(FailReason) }
pub struct Autoset { /* private */ }
impl Autoset { pub fn new(tables: Tables, enabled: [bool; 4], start: Plan) -> Self;  pub fn first(&mut self) -> Step;  pub fn next(&mut self, observation: &Observation) -> Step; }

// libs/engineering-format
pub fn parse_seconds(text: &str) -> Result<f64, ParseError>      // "500 us", "500us", "500 µs", "0.5 ms", "2e-3", "1.5 s"; a bare number is seconds; rejects empty, negative, NaN, unknown unit
// Quantity gains: Decibel (one decimal, "-6.9 dB"), VoltsSquaredPerDiv ("V²/div", 1-2-5 exact like VoltsPerDiv)
```

### 3.3 `acquisition` messages (WP1 adds them to `messages.rs` with inert values)
```rust
pub enum Phase { Stopped, Armed, WaitingForTrigger, Triggered, Auto, Done }          // Done: Single finished
pub struct AutosetStatus { pub running: bool, pub no_signal: bool, pub outcome: Option<AutosetOutcome> }
pub struct AutosetOutcome { pub id: u64, pub kind: AutosetOutcomeKind /* Done, NoSignal, Failed */, pub text: String }
pub struct AcquisitionStatus { revision, phase, pub autoset: AutosetStatus }
pub struct ControlsAvailability { /* existing */ pub single: Availability, pub autoset: Availability, pub record: Availability, pub frame_views: Availability }
         // frame_views: XY, FFT, Persist, Math, Cursors, measurements need a captured record; reason in roll
pub struct HorizontalState { /* existing */ pub roll: bool }
pub struct PulseState { pub condition: PulseCondition, pub width: FormattedValue, pub verified: bool }
pub struct TriggerState { /* existing */ pub kind: TriggerKindName /* edge | pulse */, pub pulse: Option<PulseState> }
pub struct TriggerPatch { /* existing */ pub kind: Option<TriggerKindName>, pub condition: Option<PulseCondition>, pub width_text: Option<String> }   // bad text rejects the patch
pub enum MathOperation { Add, Subtract, Multiply, InvertA }
pub enum MathStatus { Ok, Mixed { detail: String }, Unavailable { reason: String } }
pub struct MathState { pub enabled: bool, pub operation: MathOperation, pub source_a: u8, pub source_b: Option<u8>, pub scale: ScaleChoice, pub unit: String,
                       pub position: FormattedValue, pub position_divs: f64, pub status: MathStatus, pub label: String }   // label: "A−B"; InstrumentState gains `math: MathState`
pub struct MathPatch { enabled, operation, source_a, source_b, position_divs }                                     // all optional
pub enum SettingTarget { /* existing */ MathScale, MathPosition }
Requests: ArmSingle, StartAutoset, SetMath { patch }                                                          // all return ()
```
Availability reasons (exact, backend text): roll: `Trigger is not available in roll mode (500 ms/div and slower). Use a faster time base to trigger.`, `Single needs a captured record; not available in roll mode.`, `XY, FFT, persistence, math and cursors need a captured record; not available in roll mode.`;
auto-setup running: `Auto-setup is running`; `autoset` without channels: `Enable a channel first`; `record` outside roll: `Recording needs roll mode (500 ms/div or slower)`.

### 3.4 `display`
```rust
pub enum Persistence { Off, Ms500, S1, S2, Infinite }     // serde: "off","500ms","1s","2s","infinite"
DisplayState { revision, interpolation, automatic, pub persistence: Persistence }
Request SetPersistence { value } ; Event DisplayRollItem { bytes: Arc<Vec<u8>> }        // SSL1, ADR 0017 item 5
```
`SSF1` unchanged (ADR 0011); channel 5 (mask bit 4) is the math trace, scaled `up = position_divs + volts / volts_per_div`, `y = 4 - up`, never inverted. The persistence setting changes no byte of any frame.

### 3.5 `measurement`
`ChannelMeasurements.channel` is 1 to 5 (5 = M, column header `M`); `MeasurementsState.warnings: Vec<Warning>` with
`Warning { id: String /* "alias-ch2" */, severity: Warning, channel: Option<u8>, short: String /* "Alias CH1" */, message: String /* the tooltip, backend text */ }`.
Alias rule: `f > Fs / 5` raises, `f <= 0.95 x Fs / 5` clears, per channel, only from a valid measured frequency. Message: `Possible aliasing on CH1. Its frequency (62.0 MHz) is more than one fifth of the sample rate (250 MS/s), so the display may show a wrong shape or frequency.`
then one cause sentence: time base limited (interval = time base / 250): ` Use a faster time base to raise the sample rate.`; ADC limited with more than one channel on: ` Enable fewer channels to raise the sample rate.`; otherwise none.

### 3.6 `spectrum`
```rust
pub enum SpectrumScale { Dbv, Linear }  pub enum SpectrumWindow { Rectangular, Hann, Hamming, Blackman }
pub struct Tick { pub at: f64, pub text: String }
pub struct SpectrumState { revision, active: bool, window, scale, bin_spacing: FormattedValue, span: FormattedValue, sample_rate: FormattedValue, record_points: FormattedValue,
                           top_label: String, ticks_x: Vec<Tick>, ticks_y: Vec<Tick>, availability: Availability }
pub struct SpectrumPatch { active: Option<bool>, window: Option<SpectrumWindow>, scale: Option<SpectrumScale> }
Requests SetSpectrum { patch }, GetSpectrum -> SpectrumState ; Events SpectrumChanged(SpectrumState), SpectrumFrameReady { bytes }     // SSS1, ADR 0018 item 4
```

### 3.7 `recording`
```rust
pub struct RecordingFailure { pub id: u64, pub text: String, pub file_name: String, pub bytes: u64 }
pub struct RecordingState { revision, active: bool, file_name: Option<String>, duration: FormattedValue, bytes_written: FormattedValue, failure: Option<RecordingFailure> }
Requests StartRecording -> Option<SavedFile>, StopRecording, GetRecording -> RecordingState ; Event RecordingChanged(RecordingState)     // at most 1 per second
trait RecordFiles { fn create(&self, path: &Path) -> io::Result<Box<dyn RecordWriter>>; }   trait RecordWriter: Write { fn sync(&mut self) -> io::Result<()>; fn still_linked(&self) -> bool; }
```
`OsRecordFiles` is the production implementation (`create_new`-or-truncate through the picker's overwrite confirmation; `still_linked` compares device and inode of the open file with the path). Tests pass a failing writer for disk full.

### 3.8 Shell: commands, events, streams (WP14; ADR 0003 manifest, permissions and `commands.ts` regenerate)
| Kind | Name | Payload |
|---|---|---|
| command | `arm_single`, `start_autoset` | none |
| command | `set_math` | `{ patch: MathPatch }` |
| command | `set_persistence` | `{ value: Persistence }` |
| command | `set_spectrum` | `{ patch: SpectrumPatch }` |
| command | `start_recording` (returns `SavedFile` or null), `stop_recording` | none |
| command | `open_roll_stream`, `close_roll_stream`, `open_spectrum_stream`, `close_spectrum_stream` | a `Channel` for the open ones |
| command (extended) | `set_trigger` (`TriggerPatch` fields), `step_setting` and `reset_setting` (`MathScale`, `MathPosition`) | |
| event | `spectrum_changed`, `recording_changed` | whole slice with revision |
| events (extended payloads) | `instrument_changed` (math, roll, controls, pulse), `status_changed` (phase `done`, autoset), `display_changed` (persistence), `measurements_changed` (M column, warnings) | |
| snapshot | `get_snapshot` returns eight slices (adds `spectrum`, `recording`) | |
`FrameStreams` becomes generic over the encoded-bytes event (`DisplayFrameReady`, `DisplayRollItem`, `SpectrumFrameReady`): ordered (roll, normal subscription with the loss rule of 3.1) or latest-wins (frames, spectrum). `stamp_sent` applies to `SSF1` and `SSS1` (offset 16), not to `SSL1`.

### 3.9 Frontend contracts
```ts
// api: StreamKind gains 'roll' | 'spectrum'; commands.ts regenerated
// app/waveform
decodeRollItem(buffer: ArrayBuffer): RollItem                  // { sequence, kind: 'samples'|'gap'|'closed', session, mask, firstIndex, count, missed, closeReason, channels: {channel, samples: Float32Array}[] }
decodeSpectrum(buffer: ArrayBuffer): DecodedSpectrum           // { sequence, mask, linear, window, bins, binHz, top, unitsPerDiv, setupRevision, channels: {channel, peakBin, levels: Float32Array, divisions: Float32Array}[] }
createRollWindow(samplesPerChannel = 2500): RollWindow         // push(item): void (clears on a new session, NaN in gaps and before the first sample); view(): { channels: {channel, samples: Float32Array}[]; gaps: {from: number; to: number; missed: number; approximate: true}[]; gapCount: number }
createRollFeed(client): Readable<RollView | undefined>         // owns the roll stream; counts skipped sequence numbers
createSpectrumFeed(client): Readable<DecodedSpectrum | undefined>
// app/views
viewStore: 'yt' | 'xy' | 'fft'  (set via setView; leaving or entering 'fft' sends set_spectrum { active } ; entering xy picks defaults)
xyAxesStore: { x: ChannelNumber; y: ChannelNumber }  (setX, setY refuse the other axis' channel; default first two enabled)
peakMarkerStore: { mode: 'off'|'auto'|'manual'; channel; bin }  (moveBy(bins), place(bin); mode becomes manual on user placement)
persistenceEpoch: Readable<number> + clearPersistence()   // incremented by Clear; the graph also clears on setup revision, interpolation and size change
// app/formatting adds: formatFrequency(hz) four digits ("1.001 kHz"), formatDbv(db) ("−6.9 dBV"), shared golden vectors
// ui/graph GraphFrame gains: ticksX?, ticksY?: { at: number; text: string }[]; xy?: { x: ArrayLike<number>; y: ArrayLike<number>; xChannel; yChannel };
//                           persistence?: { decayMs: number | 'infinite'; epoch: number; now: () => number }; gaps?: { from: number; to: number; label: string }[]; NaN samples break the line
// MainWindowModel gains: views { tab, xy, fft }, persistence { value, options, tag, available }, roll { active, window, gaps, gapCount }, recording { chip, available, failure }, autoset { busy, noSignal },
//   math { row, chip, unavailableReason }, alias warnings per channel, trigger.pulse, status chip texts `Waiting for trigger` and `Done`
```

## 4. Work packages
Files are disjoint between packages that can run at the same time. A package that must edit a file of an earlier package says so and runs after it has merged.

### 4.1 Packages
| WP | Owner | Files and crates | Depends on | Done when |
|---|---|---|---|---|
| B1 contracts, stubs | developer-1 | `backend/kernel/src/contracts/**`, `lib.rs`, `testing.rs` (event recorder for the new messages), `tests/**`; `acquisition/src/{messages.rs,lib.rs}`, `display/src/{messages.rs,lib.rs}`, `measurement/src/{messages.rs,lib.rs}`; new crates `features/{spectrum,recording}` (Cargo, `lib.rs`, complete `messages.rs`, inert `module.rs`; they depend on the kernel only until their own package adds a lib); in `acquisition`: stub files `domain/math.rs`, `module/derive.rs`, `domain/autoset_run.rs`, `module/autoset.rs` and the call sites in `domain/instrument.rs`, `domain/view.rs`, `module/core.rs` that delegate to them; one-line mechanical updates of every `TriggerSetup` and `FrameCaptured` literal in the workspace; ts-rs exports and `api/generated/**` | none | Section 3 compiles exactly; every existing test passes; `FrameCaptured`, `RollCaptured` round trip in a kernel test; `cargo test --workspace --exclude sweepscope-app`, `npm run check:repo` pass |
| B2 libs/fft | developer-2 | `backend/libs/fft/**` | none | Radix-2 and windows exact; 1 kHz sine at bin centre reads its rms within 0.5 % for every window; DC bin; Nyquist; non power of two rejected; peak skips the lobe; property: Parseval within 1 % for rectangular |
| B3 math-channel and formatting | developer-3 | `backend/libs/math-channel/**`, `backend/libs/engineering-format/{src/**,tests/**}`, `frontend/src/app/formatting/**` | none | `apply` for the four operations incl. length mismatch; `parse_seconds` table of valid and invalid inputs; new quantities with golden vectors read by Rust and TypeScript; `formatFrequency` and `formatDbv` |
| B4 libs/autoset | developer-4 | `backend/libs/autoset/**` | none | A synthetic-device test harness (closure from plan to observation) reaches `Done` with 2 to 6 periods for 10 Hz, 100 Hz, 1 kHz, 10 kHz, 100 kHz, 1 MHz, 10 MHz and amplitudes from 5 mV to 20 V (except the documented 14 to 15 band, where it picks the nearest), `NoSignal` for flat input, `Failed` for a signal below 5 Hz, at most 14 probes |
| B5 device simulator and pulse | developer-5 | `backend/features/device/src/simulator/**`, `backend/features/device/src/hantek/{convert.rs,device_tests.rs}`, `backend/features/device/tests/**` | B1 | `SimulatorControl::set_signal(channel, SignalSpec)` with frequency, amplitude, offset, pulse train; pulse match with polarity and the four conditions (`=` within 5 %); `config_from_setup` maps `Pulse` to the driver's `TriggerKind::Pulse { width_ns, condition }` (width rounded to ns, clamped 10 ns to 10 s) with tests against the driver's types |
| B6 device roll loop | developer-6 | `backend/features/device/src/module/{control.rs,worker.rs,mod.rs,timeout.rs}` and tests | B1 | Worker rolls when the applied setup is streaming and capturing: `stop`, `configure`, `start_roll`, poll every 50 ms, count indices, publish `RollPolled`, `Overrun` becomes a gap with `missed = elapsed / interval`; setup change restarts; Stop and disconnect end it; no thread leak in 20 cycles |
| B7 acquisition core | developer-7 | `backend/features/acquisition/src/{domain/{instrument,model,status,view,setup_file,defaults,geometry}.rs,module/{core,handlers,commands,effects,shared,mod,setup_files,last_setup}.rs}`, `tests/**` | B1 | Roll mode and sessions, `RollCaptured` with `Opened` and `Closed`, availability texts of 3.3, single (`ArmSingle`, sweep override, `Done`, held frame), pulse state and `set_trigger` fields with `parse_seconds`, setup file keys, steppers enter roll, hooks to B8 and B9 kept; all section 5 behaviours |
| B8 acquisition math | developer-10 | `acquisition/src/{domain/math.rs,module/derive.rs}`, `acquisition/tests/math_*.rs` | B1, B3, B7 hooks | `MathState` rules, defaults, scale table, `Mixed`, `Unavailable`, `derive_trace` with `libs/math-channel`, step and reset of scale and position |
| B9 acquisition auto-setup | developer-4 (after B4) | `acquisition/src/{domain/autoset_run.rs,module/autoset.rs}`, `acquisition/tests/autoset_*.rs` | B1, B4, B7 | Adapter per ADR 0020 item 3 and 4 against `SimulatedDevice` for the seven frequencies, no signal, failure restores the setup, disconnect aborts, budget with `FakeClock`, no `FrameCaptured` during the run |
| B10 display | developer-1 (after B1) | `backend/features/display/src/{module,wire,convert,mode}.rs`, new `roll.rs`, `tests/**` | B1 | Persistence state with `DisplayChanged` deduplicated; channel 5 in `SSF1`; `SSL1` encoder and the consumer rule of 3.1; golden fixtures `frame-v1.bin` unchanged and new `roll-v1.bin` |
| B11 measurement | developer-3 (after B3) | `backend/features/measurement/src/**`, `tests/**` | B1 | M column from `derived`; alias warnings with hysteresis and cause sentences; slice rate limit unchanged |
| B12 spectrum | developer-2 (after B2) | `backend/features/spectrum/**` | B1, B2 | Slice, `active` gating, per channel and M spectra, top and tick texts, `SSS1` golden fixture `spectrum-v1.bin`, at most 4 slices per second, thread stops on drop |
| B13 recording | developer-5 (after B5) | `backend/features/recording/**`, `docs/formats/roll-record-v1.md`, `tools/roll-record-reader.py` (reference reader, run by a test through the repository's `python3` if present, else skipped with a message) | B1 | `.ssroll` v1 per ADR 0017, segments, gaps from index jumps, once-per-second flush, `still_linked`, failure slice, end on `Closed`, rejects start outside a roll session |
| B14 shell | developer-6 (after B6 to B13) | `src-tauri/**`, `frontend/src/api/generated/{commands.ts,StateSnapshot.ts}` | B6 to B13 | Section 3.8; manifest, capability, `build.rs` and registered commands equal; streams forward with their policies; `get_snapshot` has eight slices; app starts with the simulator in a shell test |
| F1 client, stores, decoders | developer-4 (after B4) | `frontend/src/api/{backend-client,tauri-client,fake-client,index}.ts`, `frontend/src/app/instrument/**`, `frontend/src/app/waveform/**` | B1 | Section 3.9 decoders, `RollWindow`, feeds, stores `spectrum` and `recording` with revision guards, commands, fake client scripts roll items and spectra |
| F2 model skeleton and wiring | developer-8 | `frontend/src/app/main-window/{types,actions,live-actions,live-main-window,live-app,mock-main-window,mock-model,mock-signals,select-common,index}.ts`, `frontend/src/app/views/**`, `frontend/src/main.ts`; empty selector stubs `select-views.ts`, `select-math.ts`, `select-peak.ts`, `select-roll.ts`, `select-autoset.ts`, `select-recording.ts` | B1, F1 interfaces | Model types of 3.9 and every new action (`setView`, `setPersistence`, `clearPersistence`, `armSingle`, `startAutoset`, `setMath`, `setTrigger` pulse, `setSpectrum`, `startRecording`, `stopRecording`, peak marker, XY axes); stubs return disabled or empty; M2 tests pass |
| F3 graph | developer-9 | `frontend/src/ui/graph/**`, `frontend/src/ui/marker/**`, `frontend/src/pages/gallery/ScopeSections.svelte`, `frontend/e2e/graph.spec.ts` additions | none | Ticks inside the frame, NaN line breaks, XY renderer (square divisions, straight segments), persistence layer (3.10), `Marker` kinds `peak` and `axis="x"`; glitch visible at 512 px width; budget probe under 33 ms with four channels and `Infinite` |
| F4 small ui and tokens | developer-10 | `frontend/src/ui/{screen-note,status-chip,button,measurement-matrix,icon}/**`, `frontend/src/design-system/{tokens.css,tokens.generated.ts,icons/**}`, `frontend/src/pages/gallery/{ControlSections,FeedbackSections,IndicatorSections,mock-data}.*`, `docs/design-system/**` | none | New tokens (design spec 7.1, category `opacity`), `ScreenNote`, `StatusChip` kind `record` and text `Done`, `Button` busy (`aria-busy`, static `...` under reduced motion), `columnHeader` snippet, icons `record`, `pulse-positive`, `pulse-negative`, `peak`, `info`; token contrast tests pass |
| F5 stage and views | developer-9 (after F3) | `frontend/src/pages/main-window/{ScopeStage,HorizontalGroup,HorizontalReadout,Sidebar}.svelte`, new `PeakPanel.svelte`, `RollGaps.svelte`; `frontend/src/app/main-window/{select-graph,select-horizontal,select-views,select-peak,select-roll}.ts` and tests | F2, F3, F1, F4 | Yt, XY and FFT screens, tags, `ScreenNote` cases of design 3.5 and 4.3, FFT Horizontal rows and window menu, Peak marker panel with Auto and Manual, roll readout and gap overlay with `aria-live`, held-frame look (full intensity when stopped, 40 % when frozen) |
| F6 toolbar and run controls | developer-10 (after B8) | `frontend/src/pages/main-window/{Toolbar,MainWindow}.svelte`, `shortcuts.ts`; `frontend/src/app/main-window/{select-toolbar,select-banners,select-autoset,select-recording}.ts`, `frontend/src/app/{autoset,recording}/**` and tests | F2, F4, F1 | Persist menu (radio, Clear), View segmented, Single, Autoset busy and Toasts, `No signal` tag state, Record toggle and chip, error Banner, Math toggle mirrors row M, chips `Waiting for trigger` and `Done`, `A` key |
| F7 dock | developer-8 (after F2) | `frontend/src/pages/main-window/{FrontPanelDock,VerticalGroup,TriggerGroup}.svelte`, new `MathRow.svelte`; `frontend/src/app/main-window/{select-trigger,select-math}.ts`; `frontend/src/app/layout/layout-tier.ts` (+ test); `frontend/e2e/tiers.spec.ts` thresholds; `docs/specs/scaffold-and-design-system/layout-calibration.md` | F2, F4 | Row M (fixed fifth row, one line at every tier and 200 % text scale), Pulse rows, Width `NumberField` with error and reserved two-line slot, roll reason line, thresholds recalibrated with the final UI (section 7) in Chromium and WebKit |
| F8 alias, strip, status, matrix | developer-3 (after B11) | `frontend/src/pages/main-window/{MainStatusBar,ScaleStrip,MeasurementsPanel}.svelte`; `frontend/src/app/main-window/{select-channels,select-measurements,select-status}.ts` and tests | F2, F4, B11 | `Alias` badges in four places with one tooltip, status `Fs` amber, `M` column and chip, role tags `X` and `Y`, `Mixed` and `Unavailable` chip texts, FFT axis tag, no measurement values in roll |

### 4.2 Waves and critical path
```
Wave 0: B1 (critical)   B2  B3  B4  F3  F4
Wave 1 (after B1): B5  B6  B7  F1  F2  B10  B11  B12 (after B2)  B8 (after B3; the hook call sites exist from B1)
Wave 2: B9 (after B4, B7)  B13 (after B5)  F5 (after F2, F3, F1)  F6 (after F2, F4)  F7 (after F2, F4)  F8 (after F2, B11)
Wave 3: B14 (after B6 to B13), integration, hardware checklist (human)
```
Critical path B1 -> B7 -> B8 -> B14, and B1 -> F2 -> F5. Fewer developers: B1 then B7; F2 then F7; B2 and B3 and B4 are small libs one person can take in a row; F3 and F4 are independent frontend starters.

### 4.3 Branches and worktrees
As M2: integration branch `feat/m3-analyse`; per package `feat/m3-analyse-<wp>-<slug>` from the integration branch after its dependencies merged; merge in wave order, B1 first. A package never edits a file owned by another package; a needed change is a message to the owner or the staff engineer. Commits are local until G4.

## 5. Test strategy (given / when / then)
Names follow `given_..._when_..._then_...`. Backend tests use the simulator, `FakeClock`, `ScriptedPicker`, `RecordFiles` fakes and a temp directory. No screenshot-comparison tests anywhere; canvas tests read pixels or recorded draw calls.

**FFT lib (B2).** Given a 1 kHz sine of 1 V peak sampled so 1 kHz is a bin centre when every window runs then the peak bin reads 0.707 V rms within 0.5 %. Given a constant input then only the DC bin is non-zero and equals the constant. Given a 1 kHz square at 50 kS/s and 4096 samples then odd harmonics at 3, 5, 7 kHz are within one bin and about 1/3, 1/5, 1/7 of the fundamental within 10 %, and even harmonics are 40 dB below (#29 AC4). Given 3000 samples then `NotPowerOfTwo`. Given Hann then `peak_bin` ignores bins 0 to 2 even when DC is the largest.
**Math lib (B3).** Given A and B when added, subtracted, multiplied then each sample equals the operation; given invert then B is ignored; given different lengths then `None`. `parse_seconds`: `500 us`, `500µs`, `0.5 ms`, `2e-3`, `1.5 s`, `10 ns` valid; empty, `abc`, `-1 ms`, `5 m`, `NaN` invalid. Golden rows: `-6.9 dB`, `1.001 kHz`, `V²/div` steps.
**Auto-setup lib (B4).** Given the harness and a 10 Hz square of 3 V then the plan shows 2 to 6 periods and 3 to 7 divisions; the same for the seven frequencies and amplitudes 5 mV, 50 mV, 500 mV, 5 V, 20 V; given 14.5 mV peak to peak then the nearest range is chosen and the result is `Done` with 2.9 or 7.25 divisions; given a flat input then `NoSignal` with 1 V/div and 1 ms/div; given a clipped start then the range goes up; given 3 Hz then `Failed`; given 14 probes without a result then `Failed`.
**Simulator and device (B5, B6).** Given the calibrator and Pulse `Greater 400 us` then every frame is triggered; `Greater 600 us` then none (`TimedOut`); `Less 600 us` triggers, `Equal 480 us` and `Equal 520 us` trigger, `Equal 450 us` does not, `NotEqual 200 us` triggers, `NotEqual 500 us` does not; negative polarity on an inverted pulse train. Given a pulse of 0.5 ms and the Hantek mapping then `width_ns` is 500 000 and the condition matches the driver enum (driver mock). Given a streaming setup and capturing then `RollPolled` samples arrive with contiguous `first_index`; given `inject_overrun` then a gap with `after` equal to the last index and `missed` near elapsed over interval, and the next samples start at `after + 1 + missed`; given a setup change then the worker stops, configures and restarts; given disconnect then the worker ends and no thread leaks over 20 cycles.
**Acquisition (B7 to B9).** Given a 500 ms/div setup then `horizontal.roll` is true, trigger, single, force, autoset and frame views are unavailable with the texts of 3.3 and `record` is available only while running. Given roll running when the user stops then `Closed { Stopped }` follows and no later item is published; given the time base steps to 200 ms/div then `Closed { LeftRoll }`; given a vertical change then `Closed { SetupChanged }`, a new session and `Opened`; given a stale `RollPolled` revision then nothing is published. Given `ArmSingle` then the phase is `WaitingForTrigger` and the setup sent to the device has `Normal` sweep while the published sweep is unchanged; given the first triggered frame then exactly one `FrameCaptured`, capturing off, phase `Done`, sweep restored; given a timed-out capture then still waiting; given Force while armed then one free-run frame and `Done`; given Run after `Done` then running resumes. Given `set_trigger { kind: pulse, width_text: "500 us", condition: greater }` then state shows `500 µs` and the setup carries 0.0005 s; given `"abc"` then the command is rejected with `Invalid width. Enter a time such as 500 µs. Applied: 500 µs.` and nothing changes (#25 AC3); given 5 us or 50 ms the width is accepted and `verified` is false outside 50 us to 5 ms; given a setup file with and without pulse keys then both load. Given math enabled with CH1 and CH2 then `FrameCaptured.derived` equals A-B in probe-scaled volts; given CH2 off then `derived` is none and the status is `Unavailable { reason: "CH2 is off" }`; given scales 500 mV and 20 mV then the status is `Mixed` with the detail text, the unit is `V` and the scale is explicit; given Multiply then the unit is `V²`; given roll then math is unavailable. Given the seven frequencies on the simulator when `StartAutoset` then the final setup shows 2 to 6 periods (measured by `libs/measurements`) and 3 to 7 divisions (amplitude band excepted), the screen holds the previous frame until the final one, no dialog-like state exists; given a flat input then `no_signal` is true, an outcome `NoSignal`, 1 V/div and 1 ms/div; given a later frame with signal then `no_signal` clears; given a failure then the previous setup is restored and the outcome is `Failed` with the text `Auto-setup could not find a stable setting. Previous setup kept.`; given a disconnect during the run then it aborts.
**Display (B10).** Given `SetPersistence` twice with the same value then one `DisplayChanged`; given a frame with `derived` then channel 5 follows `y = 4 - (position + volts / volts_per_div)` and the mask has bit 4; given `RollCaptured` items then `SSL1` bytes equal the golden file and sequence numbers rise by one; given an index jump then a gap item precedes the samples (consumer rule).
**Measurement (B11).** Given a derived trace then column 5 has twelve rows like a channel and no clip flag. Given 62 MHz measured at 250 MS/s then `alias-ch1` is raised with the exact message and cause sentence; given 49.9 MHz after it then it stays (hysteresis); given 47 MHz then it clears; given an invalid frequency then no warning; given 1 kHz at 100 ms/div (2.5 kS/s) then it is raised with the time-base sentence (checklist H-09 on hardware).
**Spectrum (B12).** Given `active = false` then no spectrum frame is produced; given active and a 1 kHz square at 200 us/div on CH1 then the frame has CH1 only, `bins = 2049`, `bin_hz` equals sample rate / 4096 within 1e-9 and the peak bin is the fundamental; given math then bit 4 is present; given Linear and a peak above the top then the top steps up once; given dBV then the top is +10 and ticks are every division; golden `spectrum-v1.bin` equals the decoder's expectation.
**Recording (B13).** Given a roll session of 100 chunks when recording then the file has the header, one segment, the samples records and an end record, and the reference reader reproduces every code; given an injected gap then a gap record at the same index and size; given a setup change then a second segment; given a writer that fails with disk full then the recording stops, the slice has `failure` with the text `the disk is full` and `RollCaptured` consumption continues; given `still_linked` false then it stops with `the file was removed`; given 3 hours of fake time then memory of the queue stays bounded (queue length at its limit, oldest dropped, gap record written); given `Closed` then an end record and `active = false`; given no roll session then `StartRecording` is rejected with `Recording needs roll mode (500 ms/div or slower)`; given a picker that cancels then `None` and no file.
**Shell (B14).** Manifest, capability, `build.rs` and registered commands are equal; the roll forwarder delivers every item in order (no drop) and the frame forwarder stays latest-wins; `get_snapshot` has eight slices; `recording_changed` and `spectrum_changed` reach a recording emitter.
**Frontend (F1 to F8).**
- decoders: given the golden `roll-v1.bin` and `spectrum-v1.bin` then fields match; given an unknown magic or a wrong length then they throw.
- `RollWindow`: given pushes totalling more than 2500 samples then the view holds exactly 2500 and the newest at the right; given a gap then the covered samples are NaN, a gap entry has `from` and `to` as fractions of the window and scrolls out when `after + missed` leaves it; given a new session then it clears; given 10 000 pushes then the allocated size is constant (#27 AC4 in the unit).
- stores: given an event with a lower revision then it is ignored (`spectrum`, `recording`); given the FFT view opened then `set_spectrum { active: true }` is sent once and `false` on leaving.
- views: given XY entered first then X = CH1 and Y = CH2 or the first two enabled; given X chosen equal to Y's channel then it is refused; given XY then back to Yt then trigger, vertical, time base, cursors and persistence values are unchanged (#31 AC3); given Y off then the selector says so and the model asks for the `ScreenNote` with `CH2 (Y) is off.` and the action `Enable CH2`; given one channel on then `XY needs two enabled channels`.
- peak marker: given Auto then the marker is on the backend peak bin of the selected channel and follows a new spectrum; given a click then Manual and the bin; given Shift+Right then 10 bins; given the marker bin then the readout is `bin x binHz` and `levels[bin]` formatted (`1.001 kHz`, `−6.9 dBV`) and marker, tag and panel show the same text.
- toolbar: given Persist `1 s` then one `set_persistence`; given XY, FFT or roll then the button is disabled with the backend reason; given Single then the chips read `Waiting for trigger` and, after done, `Stop` and `Done`; given autoset running then the button is busy (`aria-busy`), Run, Single, Force are disabled with `Auto-setup is running`, and on the outcome `done` one Toast `Auto-setup done. CH1 500 mV/div, 200 µs/div.`, `noSignal` the `No signal` tag until the backend clears it and one Toast, `failed` one warning Toast with `Retry`; given Record then the backend picker runs and the chip reads `Rec 00:00:03 · 12 kB`; given a `failure` then one error Banner with `Choose new file` and `Dismiss`, not repeated for the same id.
- dock: given Pulse then polarity, condition (`>`, `<`, `=`, `≠` with `aria-label` and tooltips) and Width are shown; given text `500 us` then one `set_trigger` with `widthText`; given a rejected command then the field shows the error and the applied value and `Esc` restores; given Video then it is disabled with `Exp`. Given row M off then it is dimmed in place and never removed; given `Mixed` then the badge and its tooltip; given `Unavailable` then the chip text `M  A−B  unavailable: CH2 is off`; at H3 and H4 the tabs include `M`.
- alias: given a warning then the four places show it and one tooltip; given it clears then all four clear together; the Frequency and Period cells of that channel carry the footnote.
- graph (jsdom with the recording context plus Playwright pixel scans): given a one-sample glitch in a 2501-point frame drawn at 512 px width then a pixel scan finds ink at the glitch column in the live layer and in a ghost; given persistence `500 ms` and frames then ghost buckets age in 8 steps by the injected clock, expire after the decay, and `Infinite` keeps 0.60; given a setup revision change then ghosts clear; given stopped then ageing pauses; given XY then the first and last points land at `0.5 + (4 - X) / 10` and `Y / 8` of the screen; given NaN samples then the path breaks; ticks are inside the frame; budget probe with four channels, `Infinite`, 20 frames per second: no frame over 33 ms (informative on CI, measured by QA).
- tiers: the table of section 7 with one test per threshold at and 0.01 rem below.
- e2e (QA owns): the live window against the fake Tauri stub with scripted events for each story; axe on the three new screens.

**Hardware.** [hardware-checklist.md](hardware-checklist.md), run by the human; outcomes only go to the QA report (ADR 0008).

## 6. Risks and hardware limits
| Risk | Effect | Mitigation |
|---|---|---|
| R1 Roll was only verified by the driver for 22 s with 4 channels; `Overrun` was never provoked on hardware | A real gap may not be detected or may be reported late | H-02 provokes it with `SIGSTOP`; gaps are estimates and drawn soft (ADR 0009) |
| R2 Spike condition: Rust RSS growth under software rendering | #27 AC4 may fail for a reason outside our code | H-03: one hour with a real display plus a blank-page run; document the cause if it stays above 2 MiB per hour |
| R3 Single re-arms after every capture timeout; the driver has no cancel | A few ms of dead time per timeout; Stop is immediate on screen but the device finishes its capture (up to 1.5 record durations) | Documented; H-05 measures that a trigger shortly after a re-arm is not lost more than rarely |
| R4 Trigger position accuracy +6 to -6 samples depending on channels and time base (A.4); AC limit 8 | Two samples of margin; untested time bases | H-05 at 1, 2 and 4 channels; the check measures the edge in the data |
| R5 Auto-setup duration: each probe costs one `configure` (over 100 ms) plus a capture of up to 328 ms (the 20 ms/div rung) | 3 to 6 s typical, 15 s limit | Busy state, budget, `Failed` restores the setup; measured in H-07 |
| R6 Pulse trigger verified on CH4 with 500 us pulses at 20 us, 200 us and 1 ms per division only; `NotEqual` tolerance assumed to be the complement of the 5 % window | Other widths and the `≠` edge are unverified | `?` badge outside 50 us to 5 ms; H-06 includes `≠` near the boundary |
| R7 FFT dynamic range: 8-bit ADC about 48 dB plus 33 dB processing gain; 4096 bins | Weak harmonics sit in the noise | No claim below -75 dBV; the axis never shows more than 80 dB |
| R8 Persistence memory: eight offscreen canvases | 4 MB each at 1100 x 880 CSS px, 35 MB total at ratio 1; at ratio 2 it would be 140 MB | CSS resolution for ghosts; released when persistence is off or the view leaves Yt |
| R9 Row M may wrap at W3 or 200 % text scale | Dock taller than measured, breaks AC-L1 | F7 test: row M is one line at every tier and text scale; thresholds measured with the final UI |
| R10 Alias from the measured frequency cannot see frequencies above Fs/2 (the measurement itself aliases) | A very fast signal may be reported as a lower frequency without a warning | Documented limit; the tooltip says "may" |
| R11 An open input is not zero (PRD 5.2): Auto-setup may see pickup instead of `No signal` | A floating input may produce a setting | H-07 uses a shorted input for `No signal` and records what an open input does |
| R12 `acquisition` grows (roll, single, pulse, math, autoset) | Maintainability | Disjoint submodules and files (B7 to B9); split a crate if one outgrows it (ADR 0010 item 1) |

### 6.1 Driver feasibility (checked against `docs/protocol.md` and the driver source)
| Feature | Driver support | Verified | Consequence |
|---|---|---|---|
| Roll | `Scope::start_roll`, `poll_roll` (returns new samples, `Error::Overrun` when elapsed time is at least 95 % of the RAM span), `stop`; stale first 11 to 16 frames dropped; trigger ignored; state word stays `0xC100` | Streaming rates at 200 ms (1247 per s), 500 ms (499), 2 s (123), 20 s; 22 s with 4 channels across 1.68 wraps (A.7); streaming time bases 24, 25, 27, 30 | Time bases 25 to 35 (500 ms/div and slower) are roll; RAM span 32.8 s with 3 or 4 channels and 131 s with one at 500 ms/div, so polling every 50 ms is far inside it. Overrun detection is untested on hardware (R1). Roll flag has no visible effect |
| Pulse trigger | `TriggerKind::Pulse { width_ns, condition }` with `Slope` as polarity; width 10 ns to 10 s; `Equal` accepts +-5 %; the level (`07`) is shared with the edge trigger; capture is aligned to the leading edge | 500 us pulses on CH4, all four conditions, both polarities, at 20 us, 200 us, 1 ms per division (A.5) | The Level control applies to Pulse; `=` means within 5 %; `NotEqual` presumed the complement; width unit is nanoseconds in the API |
| Single shot | No driver call: `capture` arms with `Sweep::Normal` and waits for `timeout`, then stops; it is not cancellable; trigger position verified at 10, 50, 90 % (7.4) | Position accuracy +-6 samples (A.4) | Single = Normal sweep plus one accepted triggered frame; re-arm after each timeout (R3) |
| Force trigger (already M1) | None; adapter reconfigures with `Auto` | M1 | Force while armed uses the same path |

## 7. Dock height tiers re-measured
Method (as `layout-calibration.md`): throwaway Playwright script (scratchpad, not committed) against the dev server at text scale 100 %, Chromium, current mock model. Need = toolbar + status bar + (content region minus scope stage) + stage minimum 445.6 px.
Row M was approximated by cloning the last channel row (26 px, one line); the Pulse layout by one more `Segmented` line under Level and the note replaced by a reserved two-line slot (`Invalid width. Enter a time such as 500 µs. Applied: 500 µs.`). At H3 and H4 row M is a fifth channel tab, so it adds nothing there (Designer asked, ask A).
| Column | Tier | Dock now (px) | Need now (rem) | Dock with M + Pulse (px) | Need with M + Pulse (rem) | New threshold (rem) | Old (rem) |
|---|---|---|---|---|---|---|---|
| wide | h1 | 148.5 | 45.51 | 178.5 | 47.38 | **47.75** | 47.25 |
| wide | h2 | 132.5 | 44.51 | 158.5 | 46.13 | **46.5** | 46.25 |
| wide | h4 | 94 | 42.10 | 94 | 42.10 | 45 (floor) | 45 |
| narrow | h1 | 295 | 56.16 | 353 | 59.79 | **60.25** | 59 |
| narrow | h2 | 263 | 54.16 | 313 | 57.29 | **57.75** | 57 |
| narrow | h3 | 197.5 | 48.57 | 223.5 | 50.19 | **50.5** | 50.25 |
| narrow | h4 | 94 | 42.10 | 94 | 42.10 | 45 (floor) | 45 |
Rule: threshold = measured need + 0.25 rem, rounded up to 0.25 rem. Row M alone adds 26 px at h1 and 22 px at h2 (small rows). The Pulse layout makes the Trigger group 178 px high at h1, 4 px taller than Vertical with M, so it sets the wide dock height; in the narrow column the second dock row (Horizontal and Trigger) grows from 148 to 178 px and Vertical with M is 174 px.
Findings: (1) the floor 45 rem holds, h4 needs 42.10 rem; (2) the earlier calibration table was measured on an older dock: the existing UI now needs about 1.3 rem less at h1 than the old table says (M1 and M2 changed the dock), so the old thresholds carry slack; the new ones follow the rule on the current measurement plus row M and Pulse; (3) windows 740 to 744 px high at 100 % that were h2 become h4 (group tabs); (4) F7 repeats the measurement with the real row M and Pulse UI in Chromium and WebKit and writes the final values in `layout-tier.ts`, `tiers.spec.ts` and the calibration document; the values above are the expected result and the test tolerance is 0.25 rem.

## 8. Dependencies
None new. Considered and rejected: `rustfft` (ADR 0018), a charting or canvas library for XY and persistence (the Canvas renderer already exists, ADR 0006), a file-streaming crate for `.ssroll` (std `File` and `BufWriter`).

## 9. Engineering answers to the design questions (design spec section 11)
| ID | Answer |
|---|---|
| Q-M3-1 | FFT of every enabled channel plus the math trace as channel M (colour `color-math`). Up to five 4096-point transforms per frame, well under a millisecond in total, and only while the FFT view is open (`active`) |
| Q-M3-2 | dBV axis fixed: top +10 dBV, 10 dB/div, 8 divisions (+10 to -70). Linear: V rms, automatic 1-2-5 top that changes only when the largest peak exceeds the top or falls below a quarter of it. Tick texts come from the backend (`ticksX` every 2 divisions, `ticksY` every division) |
| Q-M3-3 | dBV = 20 log10(V rms / 1 V); linear is V rms; the window's coherent gain is corrected so a bin-centred sine reads its true value; DC bin is `|mean|`; Auto peak skips DC and the main-lobe bins (1, 2, 2, 3); peak frequency is the bin centre (no interpolation, "resolution is one bin") |
| Q-M3-4 | Yes: Time/div sets the sample interval exactly as in Yt (`max(ADC interval, time base / 250)`), so it sets the span (Fs/2) and the bin spacing in FFT and the points per cycle in XY. The tooltip says so |
| Q-M3-5 | Measurements panel stays and shows the time-domain numbers of the same frame in FFT and XY (assumption for the PM, no cost); in roll it shows `-` with `Measurements need a captured record; not available in roll mode` |
| Q-M3-6 | XY draws straight segments only between the points `display` sent (already sin(x)/x upsampled at fast time bases; Dots does not apply to XY). Channel position moves the figure on its axis (the samples include the position), so the X ground marker in the bottom gutter is meaningful and draggable through the channel's position command |
| Q-M3-7 | Persistence is Yt only (D11) and unavailable in XY, FFT and roll with the reason. It resets on a setup revision change (scale, time base, position, channel set), an interpolation change, a screen resize, and Clear. Ageing pauses while stopped, frozen or disconnected |
| Q-M3-8 | Ghosts are drawn at the design opacities of their age bucket, never above the live trace (`persist-opacity-newest` 0.85). Accumulation cap: nothing accumulates beyond the bucket opacity because each bucket is blitted once. QA verifies contrast with a pixel scan of the final renderer (design 5.2 numbers) |
| Q-M3-9 | Auto-setup: about 3 to 6 s (limit 15 s or 14 probes); locks Run, Single, Force, time base steppers and the setup commands with `Auto-setup is running`; not cancellable (the capture cannot be cancelled); unavailable in roll with the reason; ends in Run; outcomes through the status slice |
| Q-M3-10 | Alias is per channel, in the measurement slice (`warnings`), from the measured frequency, threshold Fs/5, cleared at 0.95 of it. Hint wording is chosen by cause (section 3.5): time base limited, ADC limited, or no cause sentence. Above Fs/2 the measured frequency is itself an alias (documented) |
| Q-M3-11 | Yes: the Level control is the pulse threshold (one level register for edge and pulse). `=` accepts widths within +-5 % of the setting; `≠` is everything outside that window (presumed, H-06); `>` and `<` are exact comparisons at the hardware's resolution (8 ns). Accepted width 10 ns to 10 s (the driver range), stored in seconds; the error text is `Invalid width. Enter a time such as 500 µs. Applied: 500 µs.`; `?` badge outside 50 us to 5 ms |
| Q-M3-12 | Dock height measured (section 7). Pulse needs one more line and the error slot is two lines; the accuracy-note readline moves to the Type row tooltip in Pulse as designed. The new thresholds are in section 7 |
| Q-M3-13 | Extension `.ssroll`, suggested name `roll-YYYY-MM-DD-HHMMSS.ssroll`. Recording ends (no resume) on Stop, roll end, leaving roll or disconnect, with a Banner naming the reason and the size; a setup change starts a new segment. Chip data: elapsed time from the injected clock and bytes written, once per second, from the `recording` slice |
| Q-M3-14 | Math: defaults `A-B`, A = CH1, B = CH2 (first two enabled channels when CH1 and CH2 are not both on). Scale table 1-2-5 from 1 mV to 500 (V/div or V²/div), initial value the nearest at or above the larger source scale (Multiply: the product). `A×B` unit `V²`. One math channel, not saved in setups, unavailable in roll. Row M at H3 and H4 is a fifth channel tab |
| Q-M3-15 | Confirmed: a stopped or finished frame stays at full intensity; 40 % is only for disconnected and offline (`graphState`). `Done` is the existing `triggered` chip kind with new text |

## 10. Rollout and documentation at approval
- `docs/architecture.md`: tree (libs `fft`, `math-channel`, `autoset`; features `spectrum`, `recording`), 4.8 (roll as run mode, items and the loss rule), Q-E2 closed with `warnings`, Q-E11 note, 9 table lines for the four new ADRs; done by the PM or the Staff Engineer in the approval commit.
- `docs/formats/roll-record-v1.md` (B13), `README` mention of `.ssroll` (PM).
- ADR 0009 item 6 is superseded by ADR 0017 item 5; recorded in the ADR index line of 0017.
- No data migration. `last-setup.json` and setup files stay version 1 with optional pulse keys (a file without them loads as Edge).

## 11. Review and QA rulings
(Empty; rounds are appended here as in the earlier milestones.)
