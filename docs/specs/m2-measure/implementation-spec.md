# Implementation spec: M2 Measure (#13, #14, #15, #18, #19, #20)

Status: draft, waiting for G3
Links: PRD 4.2, 5.2, 6.5, 6.6, 8.2 (US-06, 12, 13, 17, 18); [architecture](../../architecture.md) (Q-E5, Q-E6, Q-E8); [design](../../design.md) 6, 8, 9, 10;
M1 spec [m1-live-view](../m1-live-view/implementation-spec.md); ADRs [0011](../../adr/0011-frame-wire-format-and-window-crop.md), [0013](../../adr/0013-state-slices-revisions-and-command-style.md),
new [0014](../../adr/0014-measurement-module-and-contract-additions.md), [0015](../../adr/0015-interpolation-in-the-backend-display-module.md), [0016](../../adr/0016-files-dialogs-atomic-writes-and-setup-format.md);
driver `docs/protocol.md` (sections 6, 9, 10, A.6); [hardware checklist](hardware-checklist.md).
Branch `feat/m2-measure`; each developer works in a worktree branch off it (section 4.3).
Exit criterion (PRD 9, M2): US-06, 12, 13, 17, 18 pass; calibrator numbers (1.000 kHz, about 1.9 V, rise time 240 to 256 ns) pass on hardware, everything else on the simulator in automated tests.
Out of scope (as approved): #16, #17, #21, statistics (MEA-3), zoom and pan, offline captures, auto-setup.

## 1. Scope

### 1.1 What is delivered
| Story | Deliverable |
|---|---|
| #13 interpolation | `libs/interpolation`, `display` owns the mode (ADR 0015), sin(x)/x enabled in the Horizontal group, automatic default below 250 samples per division |
| #14 cursors | `ui/cursor` (draggable, keyboard), frontend `cursors` store and readouts (X1, X2, Y1, Y2, deltas, 1/ΔX), TypeScript formatter port with shared golden vectors, `C` key and toolbar tool |
| #15 measurements | `libs/measurements`, feature `measurement` (ADR 0014), twelve quantities per enabled channel, validity with reasons, clip flag, unverified marker, accuracy note, pin menu, engineering notation |
| #18 export | feature `export`: CSV with metadata, raw binary `SSR1` with format description, error causes, nothing half-written |
| #19 screenshot | frontend composer on an offscreen canvas at device resolution, PNG saved by `export` |
| #20 setups | JSON setup file save and load with a report of what the device cannot do, last setup autosaved and restored at start (ADR 0016) |
| Simulator | calibrator becomes 0 to 1.91 V with 250 ns (10 to 90 %) edges so rise time, amplitude and the sinc criterion are testable without hardware |

### 1.2 Decisions the human must make (G3)
Each has a recommendation; the spec is written for the recommendation.

| # | Decision | Recommendation | If decided otherwise |
|---|---|---|---|
| D1 | **Q-E6, origin of cursor X values.** The default of Q-E6 says "first sample of the record". The record starts 3.2 divisions left of the screen and pan (#32) will move the window, so that origin is invisible to the user. | X readouts are Δ-based; absolute X is the **time from the left edge of the screen** (the first sample of the displayed window), never the trigger. The prototype already shows "X origin: left edge of the window". PRD 5.2 holds (sample-relative, not trigger). The panel footnote says it. | Record origin: the readout would need the window offset in the frame stream (a protocol change) and would show values like 3.2 div + x. |
| D2 | **Export content** (#18 AC1 "selected channels"). No channel picker exists in the design. | Export the **whole captured record** (4096 points, not only the 10 visible divisions) of **every enabled channel**; the metadata says so. A picker is a later refinement. **Reads AC1 as "enabled channels": PM to confirm (G1 if not).** | Add a channel chooser dialog (design needed). |
| D3 | **Raw binary "format description"** (#18 AC2). | Format `SSR1` (self-describing JSON header, then `u8` codes) documented in `docs/formats/frame-raw-v1.md` with a short reader; each export also writes a sidecar `<name>.format.txt` with the same description. A Rust decoder and a golden file prove reload. | Only the repository document, no sidecar. |
| D4 | **Statistics block of the sidebar** (MEA-3 is its own story). | The live sidebar hides the `Stats` mode and the statistics block; the gallery and mock keep them. | Keep mock numbers (misleading beside real ones: not recommended). |
| D5 | **New UI that design.md does not show:** a `File` menu in the toolbar (Save setup, Load setup, Export CSV, Export raw, Screenshot) and a pin menu for measurement rows (the existing `+` button). Both reuse `Menu`; shortcuts follow design 10.4 (`Ctrl+S`, `Ctrl+E`, `Ctrl+P`, `C`). | Accept without a design round (PM and Designer informed). | Designer round before WP15 and WP13. |
| D6 | **Right-click menu on the screen** (design 10.2: add X or Y cursor here, set trigger level here). Not in any acceptance criterion. | Defer. Cursors are added by the Cursors tool or `C` (all four at default places) and removed one by one (double-click, `Delete`) or all at once. | Add after M2 as a small story. |
| D7 | **New dependency** `tauri-plugin-dialog` (Rust only), section 7. | Accept. | Without it the dialogs need `rfd` directly and a window handle: more code, same dependency tree. |
| D8 | **Simulator change.** Calibrator high level 5 V becomes 1.91 V; ideal edges become 250 ns edges (matches the driver measurements). | Accept; two M1 tests that pin 5 V change. | Hardware-only verification of AC2 of #13 and #15. |
| D9 | **Accuracy note wording** (#15 AC4, Q-E4; PM owns the words). Draft for the PM: `Voltage readings are accurate to about 2 to 3 counts (about 5 % of a division at extreme positions). Ranges below 50 mV/div and above 1 V/div are unverified. Verified on one unit.` | PM confirms or rewrites; the sentence lives in one backend constant (`measurement`) and is quoted in the README until the M4 accuracy page exists. | Placeholder text ships. |
| D10 | **Run state at start.** The last setup is restored at start but acquisition stays stopped (M1 D4 resumes only after a replug, where the user had it running). | Stopped at start. | One line in `acquisition` defaults. |

### 1.3 Rulings that touch acceptance criteria (for the PM; none removes a criterion)
| Story, AC | Problem or ambiguity | Ruling in this spec |
|---|---|---|
| #13 AC1 "when a new view opens ... default" | There is one view; "opens" means app start and time-base change while the user has not chosen. | Automatic mode (ADR 0015): sin(x)/x below 250 samples per division (2 ns to about 500 ns), linear above; the user's pick is sticky until the app restarts. |
| #13 AC2 "visible rise time consistent with 240 to 256 ns" | At 2 ns per division the screen is 20 ns wide; the edge is not visible. | Tested on the simulator at 20 ns/div (4 channels, 4 ns) up to 100 ns/div and 1 channel: 10 to 90 % of the sent frame within 240 to 256 ns; hardware item H-03. Narrower bases show part of the edge by physics. |
| #14 AC3 "values follow the selected channel and update" | Positions are screen fractions (presentation); readouts depend on scales. | Cursors stay on the screen where they are; the readout is recomputed when the scale, position or selected channel changes. |
| #14 AC4 | "frozen or offline" | Works while the graph state is `live` or `frozen`; disabled with `No waveform` only when `empty`. Offline mode does not exist yet. |
| #14 AC5 | Record in architecture | Done by the staff engineer after approval (Q-E5, Q-E6, Q-E8 in section 11 of `docs/architecture.md`). |
| #15 AC1 "at least eight pinned" | Pinning is presentation state. | All twelve are computed and offered; all are pinned at first run; the choice persists. |
| #15 AC2 "1.000 kHz within 0.01 %" | Crossings are quantised by one sample; with 3 to 4 periods in the record that bounds the error by about 1 sample over 3 periods (0.03 % at 200 us/div worst case). Exact when the signal is locked to the sample clock (the simulator, and the driver measured the calibrator to 1 ppm against it). | Verified on the simulator and on hardware (H-01, H-02); if the hardware misses 0.01 % at some time base the result is reported, not hidden (R2). |
| #15 AC3 | "fewer than one period, clipped" | Reason codes of ADR 0014 item 4; rise and fall also need three samples across the edge (reason `EdgeTooFast`): at the default 200 us/div the calibrator edge shows `-` with that reason. |
| #18 AC2 | See D3. | |
| #19 AC1 | "scale labels and measurements" | The image holds the screen (graticule, traces, cursors, trigger guides), a header line with the scales and a table of the pinned measurements with their footnotes and the accuracy note. |
| #20 AC2 | "anything the device cannot do is reported" | Load result lists each unavailable value (ADR 0016 item 5), shown as a warning Toast. |

## 2. Design of the change

### 2.1 Modules and crates touched
| Crate or folder | Change |
|---|---|
| `backend/kernel` | `contracts::FormattedValue` (moved), `FrameContext` +3 fields, `files.rs` (`FilePicker`, `write_atomic`, `read_bounded`), `testing` scripted picker |
| `backend/libs/interpolation` (new) | `upsample_window` |
| `backend/libs/measurements` (new) | `measure`, `is_clipped` |
| `backend/libs/engineering-format` | shared golden vectors file `tests/golden.tsv`, test reads it; new quantities `Frequency` and `Period` (four significant digits, section 9 round 1) with their own Rust-only vectors `tests/golden_measurement.tsv` |
| `backend/features/device` | `simulator/signal.rs` calibrator (1.91 V, 250 ns edges) and its tests |
| `backend/features/display` | mode state, sinc, re-encode, `DisplayChanged`, requests |
| `backend/features/measurement` (new) | module, slice, accuracy note |
| `backend/features/export` (new) | CSV, raw, screenshot save, raw decoder |
| `backend/features/acquisition` | setup file, last-setup autosave and restore, `save_setup`, `load_setup`, `setup_notice`, `FrameContext` fill |
| `src-tauri` | composition, picker implementation, commands (one binary), events, snapshot, permissions, manifest |
| `frontend/src/api` | `sendBytes`, new events |
| `frontend/src/app` | `instrument` (two stores), `formatting` (new), `cursors` (new), `files` (new), `main-window` (model, selectors, actions) |
| `frontend/src/ui` | `cursor` (new), `screenshot` (new), `menu` (checkable item), `graph` untouched |
| `frontend/src/pages/main-window` | cursor and measurement panels, File menu, scope stage, scale strip, horizontal group, shortcuts |
| `docs/formats/frame-raw-v1.md` (new), `docs/architecture.md` | format document; Q-E5, Q-E6, Q-E8 and file-tree updates after approval |

### 2.2 State ownership
| State | Owner | Notes |
|---|---|---|
| Interpolation mode (`DisplayState`) | backend `display` | domain: it decides what is sent (ADR 0015) |
| Measurements slice | backend `measurement` | derived from frames, revisioned, at most 4 per second |
| Setup, last-setup file, `setup_notice` | backend `acquisition` | ADR 0016 |
| Last captured frame for export | backend `export` | latest-wins subscription, never mutated |
| Cursor positions (four optional screen fractions) | frontend `cursors` store | presentation; not persisted in M2 |
| Pinned measurement rows | frontend, persisted in the UI-state blob (namespace `measurements`, version 1) | presentation |
| Cursor readouts, PNG composition | frontend (selector, composer) | trivial arithmetic on data held (rule 4.5.2); PNG per rule 4.5.4 |
| Toast shown flags for `setup_notice` | frontend in memory | per notice id |

### 2.3 Data flow
```
acquisition ──FrameCaptured (+probe, verified, seconds_per_div)──┬─▶ display ──DisplayFrameReady──▶ shell ──▶ frame Channel ──▶ Graph
                                                                 ├─▶ measurement ──MeasurementsChanged (≤4/s)──▶ shell ──▶ `measurements_changed`
                                                                 └─▶ export (keeps the last frame)
frontend ── set_interpolation ──▶ display (re-encodes the last frame) ── display_changed
frontend ── export_frame {csv|raw} ──▶ export ── FilePicker ── write_atomic ──▶ Option<SavedFile>
frontend ── save_screenshot (PNG bytes) ──▶ export ── FilePicker ── write_atomic
frontend ── save_setup / load_setup ──▶ acquisition ── FilePicker ── write_atomic / read_bounded
acquisition ── Storage (`last-setup.json`) at start, ≤1 per second after a change, on stop
```

## 3. Interfaces (fixed before work starts; WP1 implements all Rust ones)
Rust signatures are the contract; `TS` means exported with ts-rs to `frontend/src/api/generated/` (camelCase).

### 3.1 Kernel
```rust
// contracts (FormattedValue moved from acquisition, re-exported there; TS file unchanged)
pub struct FormattedValue { pub value: f64, pub text: String, pub unit: String }          // TS
pub struct FrameContext { pub setup_revision: u64, pub window_start: usize, pub window_len: usize, pub invert: [bool; 4],
    pub probe: [f64; 4], pub range_verified: [bool; 4], pub seconds_per_div: f64 }          // new: last three
pub struct SavedFile { pub file_name: String }                                               // TS; a cancelled dialog is `None` in `Option<SavedFile>`

// files
pub struct FileFilter { pub label: String, pub extensions: Vec<String> }
pub trait FilePicker: Send + Sync {
    fn pick_save(&self, suggested_name: &str, filter: &FileFilter) -> Result<Option<PathBuf>, KernelError>;
    fn pick_open(&self, filter: &FileFilter) -> Result<Option<PathBuf>, KernelError>;
}
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), KernelError>;       // temp in the same folder, sync, rename, temp removed on failure
pub fn read_bounded(path: &Path, max_bytes: u64) -> Result<Vec<u8>, KernelError>; // larger file: Recoverable "file is too large"
// testing: ScriptedPicker { save: Vec<Option<PathBuf>>, open: Vec<Option<PathBuf>> } (queues), implements FilePicker
```
Error texts of `write_atomic` and `read_bounded` read `cannot write <file name>: <cause>`; causes: `permission denied`, `no space left on the device`, `the folder does not exist`, `the path is a folder`, otherwise the `io::ErrorKind` words.

### 3.2 `libs/interpolation` and `libs/measurements`
```rust
// interpolation: output.len() == (len - 1) * factor + 1; output[k * factor] == record[start + k]; factor >= 1, len >= 2; reads the record outside the window
pub fn upsample_window(record: &[f32], start: usize, len: usize, factor: usize) -> Vec<f32>;

// measurements
pub enum Invalid { NoSignal, FewerThanOnePeriod, NoCompleteEdge, EdgeTooFast, NoCompletePulse }
pub struct Measured { pub vmax: f64, pub vmin: f64, pub vpp: f64, pub vmean: Result<f64, Invalid>, pub vrms: Result<f64, Invalid>,
    pub frequency: Result<f64, Invalid>, pub period: Result<f64, Invalid>, pub positive_width: Result<f64, Invalid>,
    pub negative_width: Result<f64, Invalid>, pub duty_cycle: Result<f64, Invalid> /* percent */, pub rise_time: Result<f64, Invalid>, pub fall_time: Result<f64, Invalid> }
pub fn measure(volts: &[f64], sample_interval_s: f64, volts_per_count: f64) -> Measured;    // empty input: every Result is Err(NoSignal)
pub fn is_clipped(codes: &[u8]) -> bool;                                                      // any code 0 or 255
```
Algorithm (all pinned by tests):
- `mid = (min + max) / 2`; swing below `10 * volts_per_count` is `NoSignal` for every timing quantity (amplitudes and mean stay valid).
- Crossings use a hysteresis of `max(10 % of vpp, 2 * volts_per_count)` around `mid`; the crossing time is linearly interpolated between the two samples that straddle `mid`.
- `period` = (last rising crossing - first rising crossing) / (count - 1); needs two rising crossings, else `FewerThanOnePeriod`. `frequency = 1 / period`.
- `positive_width` / `negative_width` = mean of the complete high or low pulses (rising to falling, falling to rising) in the record; none: `NoCompletePulse`. `duty_cycle = 100 * positive / (positive + negative)`.
- `vmean`, `vrms`: samples between the first and last rising crossing when there are at least two (whole periods), else the whole record.
- `rise_time`, `fall_time`: 10 to 90 % of the span between the base level (median of samples below `mid`) and the top level (median above `mid`), mean over the complete edges; fewer than 3 samples between the 10 % and 90 % crossings gives `EdgeTooFast`; no complete edge gives `NoCompleteEdge`.

### 3.3 `display` additions (WP1 writes `messages.rs`, WP5 the logic)
```rust
pub enum InterpolationMode { Sinc, Linear, Dots }                                              // TS, camelCase strings "sinc" | "linear" | "dots"
pub struct DisplayState { pub revision: u64, pub interpolation: InterpolationMode /* effective */, pub automatic: bool }   // TS
pub struct DisplayChanged(pub DisplayState);  impl Event          // published when the state changes (deduplicated)
pub struct GetDisplay;            // Response = DisplayState
pub struct SetInterpolation { pub mode: InterpolationMode }       // Response = (); sets automatic = false
pub const COMMANDS: &[CommandDescriptor];                         // set_interpolation
```
Rules: effective mode of an automatic state = Sinc when `(window_len - 1) / 10 < 250` for the newest frame, else Linear (before any frame: Linear). `factor = clamp(ceil(250 / spd), 1, 512)`.
The module keeps the last `FrameCaptured` and re-encodes it on `SetInterpolation`.

### 3.4 `measurement`
```rust
pub enum MeasurementId { Frequency, Period, Vpp, Vmax, Vmin, Vmean, Vrms, PositiveWidth, NegativeWidth, DutyCycle, RiseTime, FallTime }   // TS "frequency", ... "positiveWidth", "dutyCycle"
pub enum InvalidReason { NoSignal, FewerThanOnePeriod, NoCompleteEdge, EdgeTooFast, NoCompletePulse, Clipped }                              // TS
pub struct Invalid { pub reason: InvalidReason, pub text: String }                                                                          // TS
pub struct Measurement { pub id: MeasurementId, pub label: String, pub value: Option<FormattedValue>, pub invalid: Option<Invalid> }        // TS; exactly one of value, invalid
pub struct ChannelMeasurements { pub channel: u8, pub clipped: bool, pub unverified: Option<String>, pub measurements: Vec<Measurement> /* always 12, order above */ }   // TS
pub struct MeasurementsState { pub revision: u64, pub channels: Vec<ChannelMeasurements> /* enabled channels of the last frame, ascending */, pub accuracy_note: String }  // TS
pub struct MeasurementsChanged(pub MeasurementsState);  impl Event;   pub struct GetMeasurements;   // Response = MeasurementsState
pub struct MeasurementDeps { pub bus: Bus, pub clock: Arc<dyn Clock> }
impl MeasurementModule { pub fn new(deps: MeasurementDeps) -> Result<Self, KernelError>; }   // thread; Drop stops and joins
```
Units and formatting: frequency `Hertz`, period and widths and rise and fall `Seconds`, amplitudes `Volts`, duty `Percent`; `unverified` text is `Unverified range, accuracy up to about 5 % of a division.` when `range_verified` is false for that channel (also published as `unverified`). Texts of `InvalidReason`: `Signal is flat or too small`, `Fewer than one period in the record`, `No complete edge in the record`, `Edge too fast for the sample interval (needs 3 samples across the edge)`, `No complete pulse in the record`, `Signal is clipped at the converter limits`.
Volts of a channel: `(code - zero_code) / counts_per_div * volts_per_div * probe`; `volts_per_count = volts_per_div * probe / counts_per_div`.

### 3.5 `export`
```rust
pub enum ExportFormat { Csv, Raw }                                                             // TS "csv" | "raw"
pub struct ExportFrame { pub format: ExportFormat }          // Response = Option<SavedFile>; Recoverable "No waveform to export" before the first frame
pub struct SaveScreenshot { pub png: Vec<u8> }               // Response = Option<SavedFile>; Recoverable when not a PNG or over 32 MiB
pub struct ExportDeps { pub bus: Bus, pub picker: Arc<dyn FilePicker>, pub unix_time_us: fn() -> f64 }
impl ExportModule { pub fn new(deps: ExportDeps) -> Result<Self, KernelError>; }
pub fn decode_raw(bytes: &[u8]) -> Result<RawCapture, KernelError>;                             // public: the reload path, used by tests and later by offline mode
pub const COMMANDS: &[CommandDescriptor];                                                     // export_frame, save_screenshot
```
CSV (UTF-8, `\n`, dot decimal): metadata as `# key: value` lines (`format: sweepscope-csv 1`, `timestamp_utc` ISO 8601 of the moment the frame reached the backend, `time_base_s_per_div`, `sample_interval_s`, `record_points`, `trigger_index` or `none`, `time_origin: first sample of the record`, `volts: probe scaled, invert not applied`, then per channel `chN_volts_per_div` (at the input), `chN_probe`, `chN_range_verified`), one header row `time_s,ch1_v,...` (enabled channels only), then one row per sample: time with 12 decimals, volts with 6 decimals. Suggested names `sweepscope-YYYYMMDD-HHMMSS.csv|.bin|.png`.
Raw (`.bin`): bytes 0..4 magic `SSR1`; 4..8 `u32` little-endian header length `H`; then `H` bytes of UTF-8 JSON (`format`, `version: 1`, `timestamp_utc`, `sample_interval_s`, `seconds_per_div`, `record_points`, `counts_per_div`, `trigger_index` (null when none), `channels: [{ channel, volts_per_div, probe, zero_code, inverted, range_verified }]` in ascending channel order); then, per listed channel in order, `record_points` bytes of raw codes. `decode_raw` returns the header fields and the code vectors; the sidecar `<name>.format.txt` holds the text of `docs/formats/frame-raw-v1.md` section "Layout". Both files are written atomically; if the second fails the first is removed.

### 3.6 `acquisition` additions
```rust
pub struct SaveSetup;                                          // Response = Option<SavedFile>
pub struct LoadSetup;                                          // Response = Option<SetupLoaded>
pub struct SetupLoaded { pub file_name: String, pub warnings: Vec<String> }   // TS
pub struct SetupNotice { pub id: u64, pub text: String }       // TS; InstrumentState gains `setup_notice: Option<SetupNotice>`
pub struct SetupFile { .. }                                    // serde, pretty JSON, ADR 0016 item 4 (crate-private, in domain/setup_file.rs)
impl Instrument { pub fn to_setup_file(&self) -> SetupFile; pub fn apply_setup_file(&mut self, file: &SetupFile) -> Result<SetupReport, KernelError>; }   // SetupReport { warnings }
pub struct AcquisitionDeps { pub bus: Bus, pub clock: Arc<dyn Clock>, pub storage: Arc<dyn Storage>, pub picker: Arc<dyn FilePicker> }   // two new fields
pub const COMMANDS: ..;                                         // + save_setup, load_setup
```
Rules: `SaveSetup` is available once `instrument` exists (a device was ready once); `LoadSetup` needs a ready device, else Recoverable with the standard reason. A file is parsed completely and validated before any value is applied (all or nothing for structure; per value for availability). The `controls` of `InstrumentState` gain `files: Availability` (load and save) so the UI states the reason.

### 3.7 Commands and events added (frontend to backend)
| Command | Args | Returns | Handled by |
|---|---|---|---|
| `get_snapshot` | none | `StateSnapshot` now also `display` and `measurements` (type moves to the shell, see WP9) | shell |
| `set_interpolation` | `{ mode: InterpolationMode }` | `()` | display |
| `export_frame` | `{ format: ExportFormat }` | `SavedFile \| null` | export |
| `save_screenshot` | raw body: the PNG bytes | `SavedFile \| null` | export |
| `save_setup` | none | `SavedFile \| null` | acquisition |
| `load_setup` | none | `SetupLoaded \| null` | acquisition |
Events (shell, Tauri): `display_changed` (`DisplayState`), `measurements_changed` (`MeasurementsState`). The three file commands and `save_screenshot` are `async` in the shell (dialog blocks). Manifest descriptor of the binary command has `args: "Uint8Array"`.

### 3.8 Frontend contracts
`api`: `BackendClient.sendBytes<R>(command: string, bytes: Uint8Array): Promise<R>` (Tauri: `invoke(command, bytes)`; the fake records calls).
`app/instrument` gains stores `display: Readable<DisplayState | undefined>` and `measurements: Readable<MeasurementsState | undefined>` (revision guard as the others; `start()` subscribes to `display_changed` and `measurements_changed` before `get_snapshot`) and the command `setInterpolation(mode)`.
`app/formatting` (WP4): `formatQuantity(value: number, quantity: 'volts' | 'seconds' | 'hertz'): { text: string; unit: string }`; same rules as the Rust crate for these three quantities (three significant digits, trailing zeros kept, SI prefix, U+2212 minus, `-` for non-finite). Shared vectors: `backend/libs/engineering-format/tests/golden.tsv`, columns `value<TAB>quantity<TAB>text<TAB>unit`, read by one Rust and one TypeScript test. The four-digit `Frequency` and `Period` quantities exist only in Rust (measurement table cells); the port and `golden.tsv` do not change.
`app/cursors` (WP11 state, WP12 selector):
```ts
export type CursorId = 'x1' | 'x2' | 'y1' | 'y2';
export interface CursorsState { x1?: number; x2?: number; y1?: number; y2?: number }   // x from the left, y from the top, 0..1 of the screen
export interface CursorsStore extends Readable<CursorsState> { toggle(): void /* none present: add all four (x .3 .7, y .3 .7); else clear */; move(id: CursorId, position: number): void; remove(id: CursorId): void; clear(): void }
export const createCursorsStore: () => CursorsStore;
```
`app/files` (WP11 stub, WP15 body): `createFileActions({ client, notify, composeScreenshot }): FileActions` with `FileActions { saveSetup(): Promise<void>; loadSetup(): Promise<void>; exportCsv(): Promise<void>; exportRaw(): Promise<void>; saveScreenshot(): Promise<void> }`; `composeScreenshot: () => Promise<Uint8Array>` is injected by `main.ts`. Toasts: saved: info `Saved <file name>`; cancelled: nothing; failure: error Toast with the backend message; load warnings: warning Toast listing them.
`app/main-window` (WP11 owns `types.ts`, lands first):
```ts
export interface CursorLineModel { id: CursorId; axis: 'x' | 'y'; position: number; tag: string; valueText: string /* spoken */ }
export interface CursorsModel { enabled: boolean; available: Availability /* reason "No waveform" */; source: ChannelNumber; values: CursorValue[]; originNote: string;
  lines: CursorLineModel[]; delta?: { tag: string; position: number } }
export interface MeasurementsModel { columns; rows /* pinned only */; footnotes; frozenLabel?; accuracyNote: string; pinnable: { id: string; label: string; pinned: boolean }[]; statistics?: ... /* now optional */ }
ChannelModel.clipped?: boolean;  ToolbarModel.files: { saveSetup: Availability; loadSetup: Availability; exportCsv: Availability; exportRaw: Availability; screenshot: Availability };
MainWindowActions += toggleCursors(); moveCursor(id: CursorId, position: number); removeCursor(id: CursorId); clearCursors(); setMeasurementPinned(id: string, pinned: boolean);
  saveSetup(); loadSetup(); exportCsv(); exportRaw(); saveScreenshot();   // and setInterpolation takes 'sinc' | 'linear' | 'dots'
```
`ui/cursor/Cursor.svelte` props: `axis: 'x' | 'y'`, `position: number`, `label: string`, `valueText: string`, `onmove(position)`, `onremove()`, `disabled?`, `disabledReason?`, snippet `tag`. Behaviour (design 8.1, 9, 10.2): line with `color-cursor` 1 px dashed 6/4, 8 px hit margin, `role="slider"` with `aria-orientation`, `aria-valuenow` (the position), `aria-valuetext` (the readout), pointer capture drag that moves in the same frame, arrows move one step (x: 1/250 of the width, y: 1/200 of the height), `Shift` ten steps, `Delete` and double-click call `onremove`, focus ring, `Esc` ends a drag. `ui/menu` `MenuItem` gains `checked?: boolean` (renders `role="menuitemcheckbox"` and a check glyph).
`ui/screenshot/draw-screenshot.ts` (WP16): `drawScreenshot(ctx, spec: ScreenshotSpec, ratio: number): void` and `ScreenshotSpec { graph: GraphFrame; size: PixelSize-like css width/height; header: string[]; cursors; trigger markers; table: MeasurementsModel-like; footnotes; accuracyNote }`; `pages/main-window/compose-screenshot.ts` builds the spec from `MainWindowModel`, creates an `OffscreenCanvas` or `<canvas>` of `css size x devicePixelRatio`, calls the function and returns PNG bytes.

## 4. Work packages
Files are disjoint between packages that can run at the same time. A package that must edit a file of an earlier package says so and runs after it has merged.

### 4.1 Packages
| WP | Owner | Files and crates | Depends on | Done when |
|---|---|---|---|---|
| WP1 contracts, stubs | developer-1 | `backend/kernel/src/{contracts/**,files.rs,lib.rs,testing.rs}`, `backend/kernel/tests/**`; `backend/features/acquisition/src/{messages.rs,lib.rs,domain/instrument.rs}` (re-export `FormattedValue`, fill the 3 `FrameContext` fields, add the 3.6 message types and the two new state fields with inert defaults) and their tests; new crates `backend/libs/{interpolation,measurements}` and `backend/features/{measurement,export}` with `Cargo.toml`, `lib.rs`, complete `messages.rs`, stub modules; `backend/features/display/src/{messages.rs,lib.rs}` additions (3.3 types only); generated TS | none | Section 3 compiles exactly; M1 tests pass; `write_atomic` and `read_bounded` tested (success, missing folder, read-only folder, failed rename leaves no temp, large file); ts-rs exports committed; `cargo test --workspace --exclude sweepscope-app` and `npm run check:repo` pass |
| WP2 interpolation and simulator | developer-2 | `backend/libs/interpolation/src/**`, `tests/**`; `backend/features/device/src/simulator/signal.rs` and the device tests that pin 5 V | WP1 (crate skeleton) | `upsample_window` exact on grid points, reproduces a sampled sine within 1 % at 4x, edge of window uses outside samples; calibrator 0 to 1.91 V with 250 ns (10 to 90 %) rising and falling edges, tests updated |
| WP3 measurements lib | developer-3 | `backend/libs/measurements/src/**`, `tests/**` | WP1 (crate skeleton) | Section 3.2 algorithm; synthetic square, sine, triangle, DC, noisy, clipped, one-period and sub-period inputs; property: frequency of a sine with N whole periods within 0.01 % |
| WP4 formatting port | developer-4 | `backend/libs/engineering-format/{src/**,tests/**}` (`golden.tsv`, `golden_measurement.tsv`, reader tests; `src` for the round 1 quantities), `frontend/src/app/formatting/**` | none | Golden vectors cover volts, seconds, hertz across the prefix boundaries, negative values, zero, non-finite; Rust and TypeScript tests read the same file and pass; round 1: `Frequency` and `Period` vectors pass in Rust (section 9) |
| WP5 display | developer-1 (after WP1) | `backend/features/display/src/{convert.rs,module.rs,wire.rs,mode.rs}`, `tests/**` | WP1, WP2 | ADR 0015 behaviour: automatic rule, factor, upsampled frame header and trigger index, re-encode on `SetInterpolation`, `DisplayChanged` deduplicated, rise-time test over simulator frames |
| WP6 measurement module | developer-3 (after WP3) | `backend/features/measurement/src/**`, `tests/**` | WP1, WP3, WP4 (round 1 quantities) | Section 3.4: slice, formatting, clip, unverified, accuracy note, rate limit 4/s with `FakeClock`, `GetMeasurements`, thread stops on drop |
| WP7 export | developer-5 | `backend/features/export/src/**`, `tests/**` (fixture `frame-raw-v1.bin`), `docs/formats/frame-raw-v1.md` | WP1 | Section 3.5: CSV and raw, sidecar, screenshot save, error causes, atomicity, `decode_raw` round trip and golden file |
| WP8 setups | developer-6 | `backend/features/acquisition/src/{domain/setup_file.rs,domain/instrument.rs,domain/mod.rs,module/**,lib.rs}`, `tests/**` (WP1 has already added `InstrumentState.setup_notice`, `ControlsAvailability.files` and the request and result types to `messages.rs`) | WP1 | Section 3.6 and ADR 0016 items 4 to 6; all behaviours of section 5 |
| WP9 shell | developer-6 (after WP8, WP5, WP6, WP7) | `src-tauri/**` (composition, `ipc/**`, `manifest.rs`, `build.rs`, `capabilities/main.json`, `permissions/**`, `Cargo.toml`, `src/picker.rs` new), `frontend/src/api/generated/{commands.ts,StateSnapshot.ts}`; two lines in `acquisition/src/{lib.rs,messages.rs}` to remove the old `StateSnapshot` | WP5 to WP8 | New modules composed (construction order: settings, measurement, export, acquisition, device, display); `get_snapshot` returns the six slices; commands of 3.7 registered with permissions, `save_screenshot` takes the raw body; picker on `tauri-plugin-dialog`; events forwarded; manifest and permission tests pass; app starts with the simulator in a shell test |
| WP10 frontend client and stores | developer-7 | `frontend/src/api/{backend-client,tauri-client,fake-client,index}.ts` and tests, `frontend/src/app/instrument/**` | WP1 | `sendBytes`; two stores with revision guards; `setInterpolation`; fake client supports bytes and the new events |
| WP11 model and wiring | developer-8 | `frontend/src/app/main-window/{types.ts,actions.ts,live-actions.ts,live-main-window.ts,mock-*.ts,select-cursors.ts,select-measurements.ts}` (last two as stubs returning the disabled or empty model) and tests, `frontend/src/app/cursors/**`, `frontend/src/app/files/{types.ts,index.ts,file-actions.ts}` (stub), `frontend/src/pages/main-window/{MainWindow.svelte,Sidebar.svelte,CursorsPanel.svelte,MeasurementsPanel.svelte,compose-screenshot.ts}` (panels and composer as stubs), `frontend/src/main.ts` | WP1, WP10 interfaces | Section 3.8 types and actions; model wired to the new stores; cursors store complete with tests; shortcuts `C`, `Ctrl+S`, `Ctrl+E`, `Ctrl+P` call actions (inactive in text fields); `setInterpolation` goes to the backend; Sidebar renders the two panels; statistics hidden in the live model (D4); the M1 tests still pass |
| WP12 cursors UI | developer-9 | `frontend/src/ui/cursor/**`, `frontend/src/app/main-window/select-cursors.ts` (replaces the stub) and `*.test.ts` for it, `frontend/src/pages/main-window/{CursorsPanel.svelte,CursorOverlay.svelte,ScopeStage.svelte}` | WP11, WP4 | Readouts of section 5; drag and keyboard in `Cursor`; ScopeStage mounts the cursors and the interpolation badge reads `sin(x)/x SW`, `Linear SW` or `Dots` from the model; `No waveform` tooltip when disabled |
| WP13 measurements UI | developer-8 (after WP11) | `frontend/src/app/main-window/select-measurements.ts` (replaces the stub), `select-channels.ts` (clip flag), `frontend/src/pages/main-window/{MeasurementsPanel.svelte,ScaleStrip.svelte}`, `frontend/src/ui/menu/**` (checked items) | WP11, WP10 | Rows from the slice, pin menu with persistence, footnotes for reasons, unverified `?` and amber edge, accuracy note under the table, `Frozen` label, `Clip` badge in the scale chip |
| WP14 interpolation UI | developer-4 (after WP4) | `frontend/src/app/main-window/select-horizontal.ts`, `frontend/src/pages/main-window/HorizontalGroup.svelte` | WP11, WP10 | sin(x)/x option enabled and selected by the backend state; the three options call the action; tooltips state automatic mode |
| WP15 files and menu | developer-5 (after WP7) | `frontend/src/app/files/file-actions.ts` (replaces the stub) and tests, `frontend/src/pages/main-window/Toolbar.svelte` | WP7, WP10, WP11 | File menu with the five items and their availability reasons; use cases of 3.8 with Toasts; no data: items disabled with a reason |
| WP16 screenshot composer | developer-2 (after WP2) | `frontend/src/ui/screenshot/**`, `frontend/src/pages/main-window/compose-screenshot.ts` (replaces the stub), `frontend/e2e/screenshot.spec.ts` | WP11 | `drawScreenshot` draws graticule, traces, cursors, header, table, footnotes; canvas size is css size times the device pixel ratio; e2e decodes the PNG (no image comparison) and checks size and hairline crispness by pixel scan |

### 4.2 Waves and critical path
```
Wave 0: WP1 (critical path)   WP4
Wave 1 (after WP1): WP2  WP3  WP7  WP8  WP10  WP11
Wave 2: WP5 (after WP2)  WP6 (after WP3)  WP12 (after WP11, WP4)  WP13 (after WP11)  WP14  WP15 (after WP7, WP11)  WP16 (after WP2, WP11)
Wave 3: WP9 (after WP5 to WP8), integration, hardware checklist (human)
```
Fewer developers: WP1 then WP5; WP3 then WP6; WP6 and WP13 are one person's two halves; WP4, WP14 and WP12 can be one frontend developer; WP2 and WP16 can swap owners.

### 4.3 Branches and worktrees
As M1: integration branch `feat/m2-measure`; per package `feat/m2-measure-wpN-<slug>` from the integration branch after its dependencies merged; merge in wave order, WP1 first. A package never edits a file owned by another package; a needed change is a message to the owner or the staff engineer. Commits are local until G4.

## 5. Test strategy (given / when / then)
Names follow `given_..._when_..._then_...`. Backend tests use the simulator, `FakeClock`, `ScriptedPicker` and a temp directory. No screenshot-comparison tests anywhere; the PNG test reads bytes and pixels.

**Kernel (WP1).** Given a read-only folder when `write_atomic` runs then the error says `permission denied` and no temp file remains. Given a rename that fails then the temp file is removed and the old file is intact. Given a missing folder then `the folder does not exist`. Given a file over the limit when `read_bounded` runs then it fails without reading it.
**Interpolation (WP2).** Given a window when upsampled by 8 then the grid points equal the input exactly. Given a sine sampled at 4 points per cycle when upsampled by 16 then the result is within 2 % of the sine. Given a window at the start of the record then no index is read out of range. Simulator: given the calibrator when sampled at 1 ns then the 10 to 90 % rise is 250 ns within 1 % and the high level is 1.91 V.
**Measurements lib (WP3).** Given a 1 kHz square of 1.9 V at 800 ns interval over 4096 samples then frequency is 1000 Hz within 0.01 %, period 1 ms, duty 50 % within 0.5, widths 500 us. Given a sine with 3.28 periods then `vmean` is within 1 % of 0 (whole periods used). Given a flat line then timing results are `NoSignal` and `vpp` is 0. Given less than one period then `FewerThanOnePeriod` and amplitudes are valid. Given an edge spanning two samples then `EdgeTooFast`; given 40 samples then 250 ns within 2 %. Given noise of 2 counts on a 60 count square then crossings are not doubled. Given codes with a 255 then `is_clipped`.
**Display (WP5).** Given automatic mode and 4 ns per sample at 20 ns/div then the frame carries about 2501 points and the interval is divided by the factor; given 200 us/div then it carries the original 2501 points. Given the user picks Dots at 20 ns/div then the original window is sent and `automatic` is false. Given a stopped acquisition when the mode changes then the last frame is re-published in the new mode. Given the same effective mode twice then one `DisplayChanged`. Given simulator calibrator frames at 20, 50 and 100 ns/div then the 10 to 90 % rise of the encoded trace is 240 to 256 ns (#13 AC2). Given a trigger sample then its index in the sent frame is multiplied by the factor.
**Measurement (WP6).** Given simulator calibrator frames at 200 us/div then frequency reads `1.000 kHz`, period `1.000 ms` and Vpp `1.91 V` (#15 AC2); given widths and edge times then they keep three digits (`500 us`, `250 ns`), rise time is invalid with `EdgeTooFast`. Given 20 us/div with one channel then rise time reads about 250 ns. Given a frame with a rail code then Vpp, rise and fall are `Clipped`, frequency is still valid, `clipped` is true. Given a range with `range_verified` false then `unverified` is set and values stay. Given probe 10x then volts are 10 times. Given a disabled channel then it is absent. Given 60 frames in one second then at most 4 events and none when no text changed. Given 2 ns/div then frequency is invalid `FewerThanOnePeriod` (#15 AC3).
**Export (WP7).** Given a frame and a picker returning a path when CSV is exported then the file has the metadata keys, a header with the enabled channels and one row per sample whose volts equal the conversion. Given raw then `decode_raw` returns the codes and header of the frame (round trip) and equals the golden file; the sidecar exists. Given a picker that cancels then nothing is written and the reply is `None`. Given a read-only folder then the error names `permission denied` and no file, including the sidecar, remains (#18 AC3). Given no frame then `No waveform to export`. Given a non-PNG body then it is refused; given a PNG then it is written byte for byte.
**Setups (WP8).** Given a ready simulator when a setup is saved then the file is pretty JSON with `format` and `version` and the values in volts and seconds. Given that file and a changed instrument when loaded then the instrument equals the saved one. Given a file with `volts_per_div` of 3 V when loaded then the current value stays and `warnings` names it (#20 AC2). Given invalid JSON, a wrong `format` or `version: 2` then the error gives the reason and the setup is unchanged (#20 AC4). Given a change when one second has passed on the `FakeClock` then `last-setup.json` exists; given the module is dropped then it is flushed. Given a new module on the same storage when the first device becomes ready then the instrument equals the last setup and acquisition is stopped; given a corrupt `last-setup.json` then defaults, `.corrupt` kept, and `setup_notice` set. Given no device when load is requested then it fails with `No scope connected`.
**Shell (WP9).** Manifest, capability, `build.rs` and registered commands are equal; `get_snapshot` returns display and measurements; `display_changed` and `measurements_changed` reach a recording emitter; `save_screenshot` with a raw body reaches `export` and with a JSON body is refused; error serialisation has category and message without paths.
**Formatting (WP4).** Every row of `golden.tsv` passes in Rust and in TypeScript. Given `Frequency` or `Period` then four significant digits, trailing zeros kept, same prefix and rounding rules: `1000 Hz` is `1.000 kHz`, `999.96 Hz` is `1.000 kHz`, `9999.6 Hz` is `10.00 kHz`, `100e3 Hz` is `100.0 kHz`, `25e6 Hz` is `25.00 MHz`, `1e-3 s` is `1.000 ms`, `0.5 Hz` is `500.0 mHz`, `NaN` is `-`; given `Hertz` or `Seconds` then the rows of `golden.tsv` are unchanged (capture rate, sample interval, cursors stay at three digits).
**Frontend.**
- stores: given an event with a lower revision then it is ignored (display, measurements); given `set_interpolation` then one `send` with the mode.
- cursors store: given none when `toggle` then four cursors at the defaults; given four when `toggle` then none; given `move` beyond 1 then it clamps; given `remove` then only that one disappears.
- cursor readouts (selector): given X1 at 0.35 and X2 at 0.55 at 2 ms/div then `X1 700 us`, `X2 1.10 ms`, `ΔX 400 us`, `1/ΔX 2.50 kHz` (the prototype numbers); given Y1 at 0.625 and Y2 at 0.375 on 500 mV/div with position -2 div then the volts follow `(divisions above the centre - position) x volts per division`; given the scale changes then the readouts change and the lines do not move (#14 AC3); given ΔX of 0 then `1/ΔX` is `-`; given the selected channel is off then Y readouts are `-`; given graph state `empty` then `available` is disabled with `No waveform` and `toggle` does nothing; given `frozen` then cursors work (#14 AC4).
- `Cursor` component: given a drag then `onmove` fires in the same step and `aria-valuenow` follows; given ArrowRight then one step, with Shift ten; given Delete or double-click then `onremove`; given disabled then the reason is the tooltip.
- measurements selector: given the slice with a valid and an invalid cell then `state` valid with `1.91 V`-style text and `invalid` with a numbered footnote carrying the backend text; given `unverified` then state `unverified`; given a pinned subset then only those rows, and the choice is saved to the UI-state blob and restored; given graph `frozen` then `frozenLabel` is `Frozen`; given no data then no columns.
- interpolation: given `display.interpolation` sinc then the segmented value is `sinc` and the option is enabled; given a click on Dots then `setInterpolation('dots')`.
- files: given `exportCsv` and a `SavedFile` then an info Toast with the name; given `null` then no Toast; given a rejected command then an error Toast with the backend text; given `loadSetup` with warnings then one warning Toast listing them; given `empty` graph then the export and screenshot items are disabled with `No waveform`; given a `setup_notice` with a new id then one Toast, not repeated.
- screenshot: given a model when composed at ratio 2 then the canvas is twice the css size and the recording context shows graticule, a trace per channel, header text and the table text (unit test with the recording context); e2e: the PNG decodes to the expected size and a graticule line is exactly one device pixel wide.
- model and pages: the M1 component tests keep passing; shortcuts do nothing while a text field has focus.
- e2e (QA owns): the live window against the fake Tauri stub with scripted events: cursors and measurements follow, File menu actions call the right commands.

**Hardware.** [hardware-checklist.md](hardware-checklist.md), run by the human; outcomes only go to the QA report (ADR 0008).

## 6. Risks and hardware limits
| Risk | Effect | Mitigation |
|---|---|---|
| R1 Volts accuracy 2 to 3 counts, one unit measured, ranges outside 50 mV to 1 V per division unchecked (PRD 5.2) | Numbers are approximate | `?` and amber edge on unverified ranges, accuracy note under the table (D9); the numbers are never rounded to more than three digits |
| R2 Frequency quantisation: crossings are interpolated between samples; at 200 us/div the interval is 800 ns and the edge (250 ns) is sharper than one sample, so a crossing can be off by up to a sample | 0.01 % (1 sample over 3 periods = 0.03 %) is not guaranteed for an unlocked signal at slow time bases | Exact for signals locked to the sample clock (the calibrator was measured to 1 ppm against it); reported by H-02; follow-up in the statistics story (average over frames) if the hardware misses it |
| R3 Trigger accuracy about +-6 samples (PRD 5.2) | Nothing in M2 uses the trigger as an origin (D1) | Cursor origin is the screen edge, export time origin is the first sample, trigger index is metadata only |
| R4 Sinc rings at sharp edges and needs the record outside the window | Overshoot of a few percent on the displayed edge, edge of the record is replicated | Documented in the mode tooltip; tested; Linear and Dots stay available |
| R5 Measurement basis is the whole record while the screen shows 10 of 16.4 divisions | A clip or an edge off screen affects a number | Stated in the panel footnote; ADR 0014 item 3 |
| R6 Native dialogs on Linux (GTK through the plugin) blocking a command thread; WebKitGTK and Wayland differences | A hung command or a dialog behind the window | Commands are `async` on a blocking task, the picker is tested only with a fake, dialog behaviour is hardware checklist H-09 and H-10; parent window given to the plugin |
| R7 Canvas to PNG in WebKitGTK at fractional device pixel ratios | Blurred lines in the image | Hairlines snapped to the device grid as in `ui/graph`; e2e pixel scan on the gallery at ratio 2; checklist H-10 on the human's display |
| R8 Raw binary command body (`save_screenshot`) up to 32 MiB through `invoke` | Slow or refused on a very large screen | Size limit and a clear error; a 4K screenshot is about 1 MiB |
| R9 `FrameContext` fields grow for in-process use only | None on the wire | ADR 0011 layout unchanged, golden fixture still passes |
| R10 Stale or racing frame for export | Export holds the newest frame at request time, which can be one newer than the displayed one | Accepted; the metadata timestamp is the receive time |
| R11 `last-setup.json` written while the app is killed | Torn file | `write_atomic`; a corrupt file is renamed and ignored |

Limits applied from PRD 5.2: sin(x)/x default at fast time bases, unverified range markers and accuracy note, time cursors relative to samples (not the trigger), roll bases remain disabled (M1 D1) so exports and measurements only see frame captures.

## 7. Dependencies
| Dependency | Where | Why not write it | Licence, status, size |
|---|---|---|---|
| `tauri-plugin-dialog` 2.x | `src-tauri` only, Rust side, no JS package, no window capability | Native file dialogs on GTK, macOS and Windows; the plugin wraps `rfd` and handles the parent window | MIT or Apache-2.0, maintained by the Tauri project; `rfd` and GTK are already in the tree through `wry` on Linux |
| `serde_json` | `acquisition` and `export` (already used by `settings` and the shell) | Setup and raw header files | MIT or Apache-2.0 |
| none new in the frontend | | PNG encoding is the browser's | |
No `csv`, `toml`, `png` or `base64` crates: values are numeric and written by hand; JSON is used for the setup.

## 8. Rollout and migration
- Merge order as in 4.3; QA test plan before the developers hand over; code review on the whole feature branch.
- After G3: ADRs 0014 to 0016 move to `accepted`; the staff engineer updates `docs/architecture.md`: section 3 (settings holds UI-state blobs, setups belong to `acquisition`; `measurement`, `export`, libs), section 4.5 (interpolation mode is domain state), section 10 items 3 and 5, and section 11: **Q-E5** (formatting stays in Rust; cursor readouts use a TypeScript port of three quantities checked against shared golden vectors, three significant digits by default and four for measured frequency and period (`Quantity::Frequency`, `Quantity::Period`; round 1 of section 9), because a backend round trip cannot meet the same-frame feedback of AC-MK1), **Q-E6** (D1), **Q-E8** (ADR 0014 item 4).
- Docs: `docs/formats/frame-raw-v1.md`; README keys and file formats; the accuracy sentence (D9) in the README until M4's page.
- Persisted data: `last-setup.json` (version 1) and the UI-state namespace `measurements` (version 1) are new; M1 stored nothing else, so there is no migration.

## 9. Review and QA rulings
(rounds are appended here, approved sections are not rewritten)

### Round 1: frequency and period at four significant digits (#15 AC2)
Product ruling: the criterion stands, `1.000 kHz` must be displayable. WP6 as built showed `1.00 kHz` because every quantity used three digits.
- **Rule.** Digits are a property of the quantity in `libs/engineering-format` (`Digits` in `quantity.rs`, the only place). Two new variants `Quantity::Frequency` (Hz) and `Quantity::Period` (s) use four significant digits; all other quantities are unchanged: volts, widths, rise and fall time, time cursors, sample interval stay at three.
- **Why new variants and not `Hertz`/`Seconds`.** `Hertz` also formats the capture rate and `Seconds` the sample interval, the cursor ΔX and widths; changing them would alter M1 readouts and golden vectors without a requirement. Frequency and period are the only measurements whose AC is tighter than the display (0.01 % needs 4 digits; 1.000 kHz vs 1.00 kHz is 0.5 %).
- **Cursors.** `1/ΔX` stays `Hertz` at three digits (`2.50 kHz`): it derives from a ΔX shown with three digits, so a fourth digit would be false precision, and #14 has no digit criterion. The TypeScript port and `golden.tsv` are untouched. The check of section 5 holds.
- **Golden vectors.** Four-digit rows go in `tests/golden_measurement.tsv` (Rust only), not in the shared `golden.tsv`, so the TypeScript test needs no skip logic or unused port code. Add `period` and `frequency` columns values to the Rust reader test only.
- **Assignments.** developer-4 (WP4): add the two variants, parametrise `scale.rs` by digit count (replace the `SIGNIFICANT_DIGITS` constant by a value from `Digits`), add `golden_measurement.tsv` and its reader, keep every existing vector green; first, so WP6 can consume it. developer-3 (WP6): map `Id::Frequency` to `Quantity::Frequency` and `Id::Period` to `Quantity::Period` in `measurement/src/channel.rs`; update `tests/module.rs:133` to `1.000 kHz`, add the period assertion `1.000 ms`; widths and edge times stay `Seconds`. Frontend: no change (`measurements` carry backend text; the gallery mocks already show `1.000 kHz`).
- **Other criteria.** None affected. #15 AC2 accuracy (0.01 %) is unchanged and still depends on R2/H-01; four digits show a 0.01 % error only as the last digit (1.000 vs 1.001 kHz is 0.1 %), so a display miss of 0.01 % is invisible and the numeric `value` field carries the exact number for tests. Design numbers (`1.000 kHz` mocks, `2.50 kHz` cursor, `1.91 V`, 240 to 256 ns) hold.
