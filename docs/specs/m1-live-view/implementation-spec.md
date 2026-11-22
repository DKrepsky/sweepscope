# Implementation spec: M1 Live view (#5, #6, #7, #8, #9, #10, #11, #12)

Status: approved by the human at G3 (D1-D7 as recommended)
Links: PRD sections 5, 6, 7 (US-01, 03, 04, 09), 9 (M1); [architecture](../../architecture.md); [design](../../design.md) sections 3, 8, 9, 10;
ADRs [0006](../../adr/0006-graph-renderer-canvas-first.md), [0007](../../adr/0007-scope-device-and-catalog-traits.md), [0009](../../adr/0009-roll-stream-gap-item.md),
new [0010](../../adr/0010-instrument-modules-and-kernel-contracts.md), [0011](../../adr/0011-frame-wire-format-and-window-crop.md),
[0012](../../adr/0012-hantek-adapter-and-device-session.md), [0013](../../adr/0013-state-slices-revisions-and-command-style.md);
spike reports in `docs/spikes/m0/`; driver `docs/protocol.md` (`/home/kr/workspace/hantek6xx4-lib`); [hardware checklist](hardware-checklist.md).
Exit criterion (PRD 9): US-01, US-03, US-04, US-09 pass on hardware. Everything else passes on the simulator in automated tests.

## 1. Scope

### 1.1 In
One feature branch `feat/m1-live-view`; each developer works in a separate worktree and branch off it (see section 4.3).

| Story | What M1 delivers |
|---|---|
| #5 connection and identity | Device module with hotplug (open only after `DeviceMonitor` says `Connected`), state `Disconnected`, `Connecting`, `Ready`, `Error`, identity in the status bar, simulator selection, real-hardware adapter |
| #6 USB permission help | `PermissionDenied` connection error with Banner: rule name, install command, `Copy command`, `Retry`; udev rule file and docs page |
| #7 run and stop | Acquisition worker thread, `Run`/`Stop`, status chips (armed, waiting, triggered, auto), capture rate and dead time measured, driver errors stop the run |
| #8 live waveform | Frame stream (ADR 0011), 10 x 8 graticule with live traces on Canvas 2D, frozen state on disconnect, frame-budget probe |
| #9 vertical | Per channel enable, 12 ranges (step and reset), position, AC/DC, bandwidth limit, probe 1x/10x/100x, invert, unverified-range marker |
| #10 horizontal | 36 time base steps (see D1 for the 11 roll steps), trigger position in percent of the screen, unverified marker, sample rate and record length |
| #11 edge trigger | Source, slope, level (drag, stepper, `Alt+Up/Down`), sweep auto and normal, force trigger, trigger status |
| #12 unplug and replug | Freeze on unplug within 1 s, automatic reconnect, restore of the full setup, no hang and no leaked thread over 20 cycles |
| Bugs #40, #41, #42 | Absorbed (D6): they live in files M1 rewires |

### 1.2 Out (unchanged from the stories)
Single shot, pulse and video trigger, roll mode, cursors and measurements (the sidebar stays on mock data, marked as such), interpolation beyond
linear and dots, persistence, WebGL2, zoom and pan, zero calibration, setups on disk, aliasing warning, auto-setup, export.

### 1.3 Decisions the human must make (G3)
Each has a recommendation; the spec is written for the recommendation.

| # | Decision | Recommendation | If the human decides otherwise |
|---|---|---|---|
| D1 | **Roll time bases in M1.** 11 of the 36 steps (500 ms/div and slower, `streaming = true`) are roll mode (#27, M3). A frame capture at 500 ms/div takes 8 s and at 1000 s/div 4.5 hours, and `capture` cannot be cancelled, so Stop could not work. Story #10 AC1 says "all 36 steps are reachable". | All 36 steps are **shown**; the 11 roll steps are disabled in M1 with the reason "Roll mode arrives in a later release". AC1 is amended to "all 25 frame-mode steps reachable now, the 11 roll steps with #27". **This removes part of an acceptance criterion: G1 for #10 again.** | Enabling them means Stop and Run can stall for up to 1.5 record durations (up to hours); the checklist would need a "do not use" note. Not recommended. |
| D2 | **Renderer.** Spike #2 gave a conditional go for WebGL2 whose value is persistence (#26, M3); Canvas 2D passes the frame-rate rules. Story #8 AC3 reads "falls back to Canvas 2D when WebGL is unavailable". | M1 ships **Canvas 2D only** behind `TraceRenderer`, the status bar shows `Canvas 2D`, and the DMABUF workaround is documented. WebGL2 and automatic fallback arrive with persistence (M3). ADR 0006 is accepted with that scope. AC3 holds trivially (the app never needs WebGL) and PRD DSP-9 stays open until M3. | Adding WebGL2 now is one more package (about the size of WP13) with the NVIDIA and WebKitGTK risk of the spike, for no M1 feature. |
| D3 | **Force trigger.** The driver has no force. | M1 forces through a re-configure with auto sweep (ADR 0012 item 6): works, but costs a full `configure` (over 100 ms, measured in the checklist). Ask the driver maintainer for a native call. | Waiting for the driver would leave AC4 of #11 unmet in M1. |
| D4 | **Run state after replug.** US-03 says the setup is restored "without user action", not whether acquisition resumes. | If it was running at unplug, it resumes after the restore. | Stay stopped: one line in `acquisition` (`resume_after_reconnect = false`). |
| D5 | **First-run defaults** (nothing persists yet, #20): CH1 on, CH2 to CH4 off, 1 V/div, DC, probe 1x, position 0 div; time base 200 us/div; trigger edge on CH1, rising, level 0 V, auto sweep, trigger at 50 % of the screen. | As listed (CH1 only gives 1 GS/s at fast bases; the driver default has all four on). | Change the constants in `acquisition/src/domain/defaults.rs`. |
| D6 | **Bugs #40, #41, #42.** #40 (channel row columns) is in `ChannelRow.svelte`, which M1 rewires. #41 (`Marker` `aria-valuenow` while dragging) is in the component that carries the trigger drag of #11 and AC-MK1. #42 (Settings popover wrapping) is one line in `Segmented`, next to the toolbar work. | Absorb all three, each fixed test first; #40 in WP11, #41 and #42 in WP12. | Leave open: #41 would still block a correct #11 accessibility story. |
| D7 | **udev rule.** The driver's rule grants group `plugdev` only. | Ship the driver's rule plus `TAG+="uaccess"` (the logged-in seat user gets access with no group change) and an install command built from the shipped file. Verify on hardware (checklist H-06). | Ship the driver's rule unchanged and add `sudo usermod -aG plugdev $USER` and a re-login to the Banner text. |
| D8 | **Human environment steps before hardware work.** `sudo apt install libusb-1.0-0-dev` on the dev machine (the driver needs it to build; `pkg-config` reports it missing today), user in `plugdev` or the udev rule installed. Also: the 6254BC is plugged into the dev machine now; agents will not touch it, only the human runs the checklist and the `--ignored` smoke test. | Do both before WP4 integration. | Without libusb, only `--features device/vendored` builds. |

### 1.4 Acceptance criteria that are ambiguous, infeasible or need a ruling
These are rulings the spec assumes; the PM confirms or sends them back.

| Story, AC | Problem | Ruling in this spec |
|---|---|---|
| #5 AC1 and PRD US-01 | Design J1 has a `Settling` chip. The driver hides the 2 s settle: no event exists until the scope is stable, so the app cannot show `Settling`. | During the settle the app shows `Disconnected`; `Connecting` shows while opening (well under the 5 s). A driver event "appeared" would fix it (change request, ADR 0012). |
| #5 AC2 | The device reports no model name and no real firmware version. | Model reads `Hantek 6xx4 (family 6000, PCB P1.06)` (6254BC is not provable from the device); "firmware" is the device's `driver_version()` value, labelled `FW`; FPGA `0xD001`; calibration `Valid` whenever the scope opens (an invalid table fails the open and is an error state). PM may reword; one function in `device/src/hantek/identity.rs`. |
| #5 AC3 | "the simulator can be selected": no control in design.md. | A `Use simulator` action on the info Banner shown while disconnected; with the simulator active the status bar item reads `Simulated`. No control to leave the simulator in M1 (a real scope attaching takes over). Designer to confirm (message sent). |
| #5 AC4 and #12 AC4 | "no wedge in 20 plug-ins" and "20 cycles" cannot be proven by automation on a real scope. | Automated: 20 simulated unplug/replug cycles, no hang, no leaked thread. Hardware: checklist H-04, H-11, H-12 run by the human. |
| #7 AC2 | "Stop within one frame time or 250 ms": the in-flight capture cannot be cancelled and at a normal sweep without a trigger a capture waits up to `max(250 ms, 1.5 x record duration)`. | For the user Stop is immediate (display freezes on the last shown frame, state `Stop`); the device finishes its capture first, and a Run pressed meanwhile waits for it. Worst case 1.5 record durations: 250 ms up to 40 ms/div, 4.9 s at 200 ms/div. Stated plainly in the checklist. |
| #7 AC4 | "capture rate and dead time, not a constant": undefined. | Capture rate = frames per second over the last 2 s. Dead time = `1 - rate x record duration`, clamped to 0 to 1 (the share of time the scope was not sampling; near 100 % at fast time bases). Both show `-` until two frames exist. |
| #8 AC1 | "redraws at the display rate (60 fps) with no frame over 33 ms on the reference laptop". The device delivers tens of frames per second, and no second machine exists (spike #2 criterion 1 was not met). | Interpreted as: the animation loop never produces an interval over 33.3 ms while frames arrive at 60 per second from the simulator, and draws only when a new frame exists. Measured on the dev machine with the frame-budget probe (WP10, checklist H-14); the laptop and integrated-GPU run stays open until a second machine exists and DSP-7 is not marked met. |
| #8 AC2 | "min/max per column". | The window sent holds at most about 2501 points per channel and the graph draws every sample, so a one-sample glitch is drawn and visible (the DSP-3 intent); `libs/decimation` waits for the first story that exceeds the pixel count (ADR 0011). A test pins the glitch. |
| #9 AC "12 ranges are offered" | Offered through a stepper, not a list. | All 12 reachable by step and reset (ADR 0013 item 5). |
| #9 and PRD VER-2 | "custom" probe factor has no control in the channel row. | M1 offers 1x, 10x, 100x (cycle button); custom waits for a design. Position limits: on the 50 mV, 500 mV and 5 V ranges the hardware zero spans only about +-1.6 divisions (protocol.md 9); the ground marker clamps and the tooltip says so. |
| #10 AC2 | The trigger position is a percentage of what? The record is 16.4 divisions, the screen 10. | Percent of the screen (the marker is in the screen gutter). The backend maps it to the hardware record position (section 3.2). |
| #11 AC4 | No force in the driver. | D3. |
| #12 AC2 | Resume of running. | D4. |
| PRD 5.2 trigger accuracy | The driver's trigger index is nominal and the real edge is within about +-6 samples. At 2 ns to 100 ns per division that is several divisions of screen. | The marker shows the setting, the trace shows what was captured; the accuracy note slot stays a placeholder (Q-E4, PM). |

### 1.5 Bugs #40 to #42 and other findings for the PM
Absorbed as in D6. New findings for the PM: PRD US-01 bullet 3 (CAL hint) belongs to #17, not M1; the shipped udev rule and `docs/usb-permissions.md` are new
artefacts of #6; the driver change requests of ADR 0012 should be filed as issues in the driver repository (human action, publishing rule of the workflow).

## 2. Design of the change

### 2.1 Modules and crates touched
| Crate or folder | Change |
|---|---|
| `backend/kernel` | `Subscription::recv_timeout`, `Bus::subscribe_latest`; new `contracts/` module (moved types, 7 messages, `ConnectionSnapshot`) |
| `backend/libs/engineering-format` (new) | Q-E5 formatter: SI prefixes, 3 significant digits, units per quantity |
| `backend/features/device` | Types moved out (re-exported), `profile.rs` shared tables, `PermissionDenied`, `force_capture`, `max_position_divs`; new `module/` (sessions) and `hantek/` (adapter) |
| `backend/features/acquisition` (new) | `domain/` (pure rules) and `module/` (threads, handlers, events) |
| `backend/features/display` (new) | scaling, crop, wire encoding, module |
| `src-tauri` | composition of all modules, commands, event and frame forwarding, snapshot, env flags, logging |
| `packaging/udev`, `docs/usb-permissions.md`, `README.md`, `.github/workflows/ci.yml` | udev rule, docs, libusb prerequisite |
| `frontend/src/api` | events and streams in the client, regenerated contract |
| `frontend/src/app` | `instrument` (stores, commands), `waveform` (decode, feed, probe), `main-window` (live view model and actions) |
| `frontend/src/pages/main-window`, `ui/marker`, `ui/segmented` | wiring of every control to actions, bug fixes |

### 2.2 State ownership
| State | Owner | Notes |
|---|---|---|
| Connection (state, source, identity, error, simulator flag) | backend `device` | slice `connection` |
| Setup (channels, time base, trigger), probe, invert, run intent, window geometry, availability, facts, status | backend `acquisition` | slices `instrument`, `status`, `facts`; also the restore after reconnect |
| Wire frame | backend `display` | latest-wins stream |
| Acquiring (`ready` and `running`), graph state `live`/`frozen`/`empty`, last frame | frontend selectors | derived, never stored twice |
| Channel colour and label, selected channel, interpolation `linear`/`dots`, drawer, text scale | frontend presentation stores | colour and label are fixed by tokens; interpolation becomes domain state when sin(x)/x (#13) needs the backend; the backend never needs these |
| Trigger and ground marker ghosts | frontend (design 8.6) | ends when the confirmed slice arrives |

### 2.3 Data flow
```
HantekCatalog | SimulatedCatalog ──CatalogEvent──▶ device::module (manager thread) ──ConnectionChanged──▶ acquisition ──▶ (shell) connection_changed
                                                       │ open → worker thread per session                    │ ApplySetup, SetCapturing, ForceTrigger (requests)
                                                       ▼                                                     ▼
                                       worker: configure / capture loop ◀───────────────────────── device handlers (store intent, wake worker)
                                                       │ CaptureCompleted { setup_revision, Frame | TimedOut }
                                                       ▼
                       acquisition: drop stale revision, status + rate meter, FrameContext ──FrameCaptured (latest-wins)──▶ display
                                                       │ instrument_changed, status_changed, facts_changed                │ DisplayFrameReady (latest-wins)
                                                       ▼                                                                  ▼
                                              shell forwards as Tauri events                          shell forwards on the frame Channel ──▶ frontend decode ──▶ rAF feed ──▶ Graph
```
Frontend to backend: Tauri commands (section 3.4) call `bus.request(...)`; replies are `()` or `KernelError`; state comes back as events.

## 3. Interfaces (fixed before work starts, WP1 implements them)
Rust signatures are the contract; `TS` means exported with ts-rs to `frontend/src/api/generated/` (camelCase). The 3-level `export_to` path of
`kernel/src/error.rs` applies inside the kernel crate.

### 3.1 Kernel (`sweepscope_kernel`)
```rust
impl<E> Subscription<E> { pub fn recv_timeout(&self, wait: Duration) -> Option<E>; }          // blocks on a Condvar, wakes on publish
impl Bus { pub fn subscribe_latest<E: Event>(&self) -> Subscription<E>; }                     // capacity 1, drops the older item, counts lagged()

pub mod contracts {
    // moved from sweepscope_device without behaviour change; re-exported there
    pub struct Capabilities { channel_count, record_len, time_bases: Vec<TimeBaseStep>, vertical_ranges: Vec<VerticalRange>, adc_interval_ns, samples_per_div }
    pub struct VerticalRange { pub volts_per_div: f64, pub verified: bool, pub max_position_divs: f64 }   // new field: 4.0, or 1.6 for 50 mV, 500 mV, 5 V
    pub struct DeviceIdentity { .. }  pub enum CalibrationStatus { .. }
    pub struct AcquisitionSetup { .. } /* ChannelSetup, TriggerSetup, Coupling, Slope, Sweep */  pub struct Frame { .. } /* ChannelData, TriggerOutcome */

    pub enum ConnectionState { Disconnected, Connecting, Ready, Error }                                  // TS
    pub enum DeviceSource { Hardware, Simulator }                                                        // TS
    pub enum ConnectionIssueKind { PermissionDenied, CalibrationInvalid, Communication }                 // TS
    pub struct ConnectionIssue { pub kind: ConnectionIssueKind, pub message: String, pub help: Option<PermissionHelp> }   // TS
    pub struct PermissionHelp { pub rule_file: String, pub install_command: String, pub doc: String }    // TS
    pub struct ConnectionSnapshot { pub revision: u64, pub state: ConnectionState, pub source: Option<DeviceSource>,
        pub identity: Option<DeviceIdentity>, pub issue: Option<ConnectionIssue>, pub simulator_active: bool }   // TS

    // events
    pub struct ConnectionChanged { pub snapshot: ConnectionSnapshot, pub capabilities: Option<Capabilities> }   // device publishes
    pub enum CaptureOutcome { Frame(Arc<Frame>), TimedOut }
    pub struct CaptureCompleted { pub setup_revision: u64, pub outcome: CaptureOutcome }                        // device worker publishes
    pub struct FrameContext { pub setup_revision: u64, pub window_start: usize, pub window_len: usize, pub invert: [bool; 4] }
    pub struct FrameCaptured { pub frame: Arc<Frame>, pub context: FrameContext }                               // acquisition publishes
    // requests handled by device, sent by acquisition; the handler stores intent and returns at once
    pub struct ApplySetup { pub setup_revision: u64, pub setup: AcquisitionSetup }   // Response = ()
    pub struct SetCapturing { pub on: bool }                                           // Response = ()
    pub struct ForceTrigger;                                                           // Response = ()
}
```
`ConnectionSnapshot.revision` is assigned by `device`; the other slices have their own counters (ADR 0013).

### 3.2 Horizontal geometry (pure function in `acquisition/src/domain/geometry.rs`)
```rust
pub struct Geometry { pub samples_per_div: f64, pub window_len: usize, pub window_start: usize, pub record_divs: f64 }
pub fn geometry(record_len: usize, sample_interval_s: f64, seconds_per_div: f64) -> Geometry;
//  spd = seconds_per_div / sample_interval_s;  window_len = clamp(round(10 * spd) + 1, 2, record_len)
//  window_start = (record_len - (window_len - 1)) / 2;  record_divs = record_len / spd
pub fn record_percent(g: &Geometry, record_len: usize, screen_fraction: f64) -> f64;   // 100 * (window_start + s * (window_len - 1)) / record_len -> trigger.horizontal_position_percent
pub fn trigger_index_in_window(g: &Geometry, sample_index: usize) -> Option<usize>;    // None when outside 0..window_len
```
Examples that tests pin (record 4096, 250 samples/div): `window_len = 2501`, `window_start = 797`, screen 50 % gives record percent 50.0, screen 0 % gives 19.46, screen 100 % gives 80.54.
At 2 ns/div with four channels (interval 4 ns): `spd = 0.5`, `window_len = 6`.

### 3.3 State slices published by `acquisition` (TS; `FormattedValue` in ADR 0013)
```rust
pub struct FormattedValue { pub value: f64, pub text: String, pub unit: String }
pub struct Availability { pub enabled: bool, pub reason: Option<String> }
pub struct ScaleChoice { pub index: u8, pub label: FormattedValue, pub verified: bool, pub faster: Availability, pub slower: Availability }   // faster = smaller V/div or time/div

pub struct InstrumentState { pub revision: u64, pub running: bool, pub channels: Vec<ChannelState> /* always 4 */,
    pub horizontal: HorizontalState, pub trigger: TriggerState, pub controls: ControlsAvailability }
pub struct ChannelState { pub number: u8, pub enabled: bool, pub range: ScaleChoice /* label already multiplied by the probe factor */,
    pub position: FormattedValue /* "div" */, pub position_divs: f64, pub max_position_divs: f64, pub coupling: Coupling, pub bandwidth_limit: bool,
    pub probe: f64 /* 1, 10 or 100 */, pub probe_text: String /* "10x" */, pub invert: bool,
    pub unverified: Option<String> /* reason text when the range is outside 50 mV to 1 V per div */, pub enable: Availability }
pub struct HorizontalState { pub time_base: ScaleChoice, pub trigger_position: FormattedValue /* percent of the screen */, pub trigger_position_fraction: f64,
    pub window_text: String /* "10.0 of 16.4 div" */, pub window_start_fraction: f64, pub window_size_fraction: f64, pub trigger_tick_fraction: f64 }
pub struct TriggerState { pub source: u8, pub slope: Slope, pub sweep: Sweep, pub level: FormattedValue /* probe-scaled volts */,
    pub level_divisions: f64 /* from the screen centre, up is positive */, pub level_tag: String, pub position_tag: String }
pub struct ControlsAvailability { pub run: Availability, pub force: Availability, pub vertical: Availability, pub horizontal: Availability, pub trigger: Availability }

pub enum Phase { Stopped, Armed, WaitingForTrigger, Triggered, Auto }                                          // TS
pub struct AcquisitionStatus { pub revision: u64, pub phase: Phase }
pub struct InstrumentFacts { pub revision: u64, pub sample_rate: Option<FormattedValue>, pub sample_interval: Option<FormattedValue>,
    pub record_points: Option<FormattedValue>, pub capture_rate: Option<FormattedValue>, pub dead_time: Option<FormattedValue> }
pub struct StateSnapshot { pub connection: ConnectionSnapshot, pub instrument: Option<InstrumentState>, pub status: AcquisitionStatus, pub facts: InstrumentFacts }   // shell assembles
```
`instrument` is `None` until the first device became ready (capabilities unknown). After that the last state stays, with `controls` unavailable and `running = false` while no device is ready.

Rules of `acquisition` (all in `domain/`, pure, unit tested):
- Availability policy (ADR 0013 item 8): no ready device means every control reads `enabled: false, reason: "No scope connected"` (or `"Connecting"`, or the error text).
  `force` is enabled only while running. Roll time bases are unavailable for stepping with `"Roll mode arrives in a later release"` (D1).
- The last enabled channel cannot be disabled (`reason: "At least one channel must stay on"`). Disabling the trigger source moves the source to the first enabled channel.
- Enabled channel count changes the sample interval (`Capabilities::sample_interval_s`), so the geometry and the facts are recomputed on every setup change.
- Position is clamped to `+- max_position_divs` of the channel's range; the ground marker reads the clamped value. A range change re-clamps.
- Trigger level is stored in volts at the input: `level_volts = (level_divisions - position_divs_of_source) x volts_per_div_of_source`; the label is `level_volts x probe` formatted. A range change keeps the volts.
- `unverified`: `range.verified == false` gives `"Unverified range: <below 50 mV | above 1 V> per div. Accuracy up to about 5 % of a division."`; a time base with `verified == false` makes `ScaleChoice.verified` false (the stepper badge says `Unverified`).
- Status phase: Run gives `Armed`; `Frame(Triggered)` gives `Triggered`; `Frame(Auto)` gives `Auto`; `TimedOut` gives `WaitingForTrigger`; Stop and loss give `Stopped`. Only changes are published.
- Facts are published at most every 250 ms and only when a text changed.
- Restore: on `ConnectionChanged` with `Ready`, the setup is re-validated against the new capabilities (indexes in range, positions re-clamped), `ApplySetup` is sent with a new setup revision, and `SetCapturing { on: true }` if `running` was true at the loss and D4 holds.
- Any setup change increments `setup_revision` and sends `ApplySetup`; `probe`, `invert` and `running` changes do not.
- A `CaptureCompleted` whose `setup_revision` is not the current one is dropped; one received while `running == false` is dropped.

### 3.4 Commands (frontend to backend, Tauri `invoke`; one `CommandDescriptor` each; all return `()` or `KernelError` except where noted)
| Command | Args | Handled by |
|---|---|---|
| `get_snapshot` | none, returns `StateSnapshot` | shell (asks `GetConnection` and `GetInstrument`) |
| `set_running` | `{ running: bool }` | acquisition |
| `force_trigger` | none | acquisition |
| `set_channel` | `{ channel: u8, patch: ChannelPatch }` with `ChannelPatch { enabled?, position_divs?, coupling?, bandwidth_limit?, probe?, invert? }` | acquisition |
| `step_setting` | `{ target: SettingTarget, direction: -1 or 1 }` | acquisition |
| `reset_setting` | `{ target: SettingTarget }` | acquisition |
| `set_trigger_position` | `{ fraction: f64 }` (screen, 0 to 1) | acquisition |
| `set_trigger_level` | `{ divisions: f64 }` (from the screen centre) | acquisition |
| `set_trigger` | `{ patch: TriggerPatch }` with `TriggerPatch { source?, slope?, sweep? }` | acquisition |
| `use_simulator` | `{ enabled: bool }` | device |
| `retry_connection` | none | device |
| `open_frame_stream` | `{ onFrame: Channel }` | shell |
| `close_frame_stream` | none | shell |

`SettingTarget` (TS, tagged): `{ kind: 'range', channel: u8 } | { kind: 'timeBase' } | { kind: 'triggerPosition' } | { kind: 'triggerLevel' }`. Step sizes: range and time base one table
entry (clamped at the first or last available entry, no error), trigger position 5 % of the screen, trigger level 0.2 division. Reset targets: 1 V/div, 200 us/div, 50 %, level 0 V (D5).
Errors: `KernelError` of category `Recoverable` with a user-presentable message (no ready device, last channel, value not finite, channel out of 1 to 4).
The existing `load_ui_state` and `save_ui_state` stay.
Events (shell, Tauri, payload = the slice): `connection_changed`, `instrument_changed`, `status_changed`, `facts_changed`. The manifest generator
(`src-tauri/src/manifest.rs`) renders `EVENTS` with their payload types and `STREAMS` (`frames` with `open_frame_stream` and `close_frame_stream`) into `commands.ts`.

### 3.5 Device module (`sweepscope_device`)
```rust
pub struct SimulatorSource { pub catalog: Box<dyn DeviceCatalog>, pub control: SimulatorControl }
pub struct DeviceDeps { pub bus: Bus, pub clock: Arc<dyn Clock>, pub hardware: Option<Box<dyn DeviceCatalog>>, pub simulator: Option<SimulatorSource> }
impl DeviceModule { pub fn new(deps: DeviceDeps) -> Result<Self, KernelError>; }   // registers handlers, starts the manager thread; Drop stops and joins (250 ms deadline per worker)
pub struct UseSimulator { pub enabled: bool }  pub struct RetryConnection;  pub struct GetConnection;   // requests; GetConnection -> ConnectionSnapshot
pub const COMMANDS: &[CommandDescriptor];
// trait changes (WP1): ScopeDevice::force_capture(&mut self, timeout: Duration) -> Result<Frame, DeviceError>;  DeviceError::PermissionDenied
// hantek module (WP4), crate-private except these two:
pub struct HantekCatalog;   impl HantekCatalog { pub fn start() -> Result<Self, DeviceError>; }          // implements DeviceCatalog
pub struct HantekDevice<T: Transport>;                                                                        // implements ScopeDevice; tests use the driver's MockTransport
```
Behaviour: ADR 0012 items 7 to 9. The worker never holds the state lock while calling the device.

### 3.6 Acquisition and display modules
```rust
pub struct AcquisitionDeps { pub bus: Bus, pub clock: Arc<dyn Clock> }
impl AcquisitionModule { pub fn new(deps: AcquisitionDeps) -> Result<Self, KernelError>; }      // handlers for 3.4 acquisition commands as requests, plus GetInstrument; one consumer thread
pub const COMMANDS: &[CommandDescriptor];
// domain (WP5) entry points used by the module (WP6): all pure
pub struct Instrument { .. }                                       // owns setup, probe, invert, run intent, capabilities
impl Instrument {
    pub fn new(defaults: Defaults) -> Self;
    pub fn on_ready(&mut self, caps: Capabilities) -> Restore;                // Restore { setup: AcquisitionSetup, resume: bool }
    pub fn on_lost(&mut self);
    pub fn set_running(&mut self, on: bool) -> Result<Change, KernelError>;
    pub fn set_channel(&mut self, channel: u8, patch: ChannelPatch) -> Result<Change, KernelError>;
    pub fn step(&mut self, target: SettingTarget, direction: i8) -> Result<Change, KernelError>;
    pub fn reset(&mut self, target: SettingTarget) -> Result<Change, KernelError>;
    pub fn set_trigger_position(&mut self, fraction: f64) -> Result<Change, KernelError>;
    pub fn set_trigger_level(&mut self, divisions: f64) -> Result<Change, KernelError>;
    pub fn set_trigger(&mut self, patch: TriggerPatch) -> Result<Change, KernelError>;
    pub fn state(&self, revision: u64) -> Option<InstrumentState>;
    pub fn frame_context(&self, setup_revision: u64) -> FrameContext;
    pub fn setup(&self) -> Option<&AcquisitionSetup>;
}
pub struct Change { pub setup_changed: bool, pub state_changed: bool }
pub fn next_status(phase: Phase, event: StatusEvent) -> Phase;  pub struct CaptureMeter;  // record(now), rate(now), dead_time(record_duration)

pub struct DisplayDeps { pub bus: Bus, pub unix_time_us: fn() -> f64 }
impl DisplayModule { pub fn new(deps: DisplayDeps) -> Result<Self, KernelError>; }               // subscribe_latest::<FrameCaptured>, publish DisplayFrameReady
pub struct DisplayFrameReady { pub bytes: Arc<Vec<u8>> }                                         // event, display-owned
pub fn encode(frame: &Frame, context: &FrameContext, sequence: u32, produced_us: f64) -> Vec<u8>;   // ADR 0011 layout; the shell overwrites bytes 16..24
pub fn stamp_sent(bytes: &mut [u8], sent_us: f64);
```

### 3.7 Frontend contracts
`api`: `BackendClient.openStream(kind: StreamKind, onMessage: (data: ArrayBuffer) => void): StreamHandle` with `StreamHandle { close(): void }`; `EventName` and `EventPayload` generated.

`app/instrument` (WP9):
```ts
export interface InstrumentStores {
  connection: Readable<ConnectionSnapshot | undefined>; instrument: Readable<InstrumentState | undefined>;
  status: Readable<AcquisitionStatus | undefined>; facts: Readable<InstrumentFacts | undefined>;
  start(): Promise<void>;                       // subscribe to the four events, then get_snapshot, apply by revision
  stop(): void;
}
export const createInstrumentStores: (deps: { client: BackendClient }) => InstrumentStores;
export interface InstrumentCommands { setRunning(on: boolean): void; forceTrigger(): void; setChannel(n: ChannelNumber, patch: ChannelPatch): void;
  step(target: SettingTarget, direction: -1 | 1): void; reset(target: SettingTarget): void; setTriggerPosition(fraction: number): void;
  setTriggerLevel(divisions: number): void; setTrigger(patch: TriggerPatch): void; useSimulator(enabled: boolean): void; retryConnection(): void }
export const createInstrumentCommands: (deps: { client: BackendClient; notify: (kind: 'error', text: string) => void }) => InstrumentCommands;   // rejected promise to error Toast, never throws
```
Derived selectors (same folder): `isAcquiring`, `graphState`, `connectionChip`, `statusChips`.

`app/waveform` (WP10): `decodeFrame(buffer: ArrayBuffer): DecodedFrame` (typed views, rejects unknown magic); `createFrameFeed({ client, requestFrame, now }): { frame: Readable<DecodedFrame | undefined>, start(): void, stop(): void }` (keeps the newest, publishes at most once per `requestFrame` callback); `createFrameBudgetProbe(...)` (rAF interval and draw time, console summary every 10 s in dev builds).

`app/main-window` (WP11 owns the type files, lands them first): `MainWindowModel` keeps its shape and gains `availability?: { enabled: boolean; reason?: string }` on each control group and channel, `step: { faster: Availability; slower: Availability }` on `voltsPerDiv` and `timePerDiv`, banners with `actions: { id: 'copy-command' | 'retry' | 'use-simulator'; label: string }[]`, `graph.channels[].samples` as `ArrayLike<number>` (typed array), `status.connection.simulated: boolean`.
`MainWindowActions` (plain callbacks, no store access in pages): `toggleRun`, `forceTrigger`, `setChannelEnabled(n, on)`, `stepVoltsPerDiv(n, dir)`, `resetVoltsPerDiv(n)`, `setChannelPosition(n, divisions)`, `commitChannelPosition(n, text)`, `setCoupling(n, 'ac'|'dc')`, `toggleBandwidth(n)`, `cycleProbe(n)`, `toggleInvert(n)`, `selectChannel(n)`, `stepTimePerDiv(dir)`, `resetTimePerDiv()`, `setTriggerPosition(fraction)`, `stepTriggerPosition(dir)`, `setTriggerLevel(divisionsFromCentre)`, `stepTriggerLevel(dir)`, `resetTriggerLevel()`, `setTriggerSource(n)`, `setTriggerSlope`, `setTriggerSweep`, `setInterpolation('linear'|'dots')`, `bannerAction(id)`, `copyText(text)`.
`createLiveMainWindow(deps): Readable<MainWindowModel>` builds the model from the four stores, the frame feed and the presentation stores; sidebar cursors and measurements stay mock data, labelled as such in the code.

## 4. Work packages

### 4.1 Packages
Files are disjoint between packages. A package that creates a crate lists the one-line stubs it leaves for later packages; WP1 creates all stubs so no two packages edit one file.

| WP | Owner | Files and crates | Depends on | Done when |
|---|---|---|---|---|
| WP1 contracts and interfaces | developer-1 | `backend/kernel/src/{bus.rs,lib.rs,contracts/**}`, `backend/kernel/tests/**`; `backend/features/device/{Cargo.toml,src/{lib.rs,capabilities.rs,catalog.rs,device.rs,error.rs,frame.rs,identity.rs,setup.rs,profile.rs,simulator/**},tests/**}` (moves, re-exports, `PermissionDenied`, `force_capture`, `max_position_divs`; adds dependencies `hantek6xx4-lib`, `rusb` and feature `vendored`); stubs `device/src/module/mod.rs`, `device/src/hantek/mod.rs`; new crate `backend/features/acquisition` (`Cargo.toml`, `src/{lib.rs,messages.rs}` complete, stubs `src/domain/mod.rs`, `src/module/mod.rs`); new crate `backend/features/display` (`Cargo.toml`, `src/{lib.rs,messages.rs}` complete, stubs `src/{convert.rs,wire.rs,module.rs}`); generated TS in `frontend/src/api/generated/**` except `commands.ts` | none | Section 3 compiles exactly; all existing device tests pass through re-exports; bus tests for `recv_timeout` and `subscribe_latest`; ts-rs exports committed; `cargo test --workspace --exclude sweepscope-app` and `npm run check:repo` pass |
| WP2 engineering-format | developer-2 | `backend/libs/engineering-format/**`, remove `backend/libs/.gitkeep` | none | `format(value, Quantity) -> Formatted { value, text, unit }`; quantities volts, volts per div, seconds, seconds per div, hertz, samples per second, percent, divisions, points; 3 significant digits with trailing zeros kept, except the per-division scale quantities which show the step value exactly (`1 V/div`, `200 us/div`, design 6); U+2212 minus; `-` for non-finite; golden table tests; no dependency |
| WP3 device module | developer-3 | `backend/features/device/src/module/**`, `backend/features/device/tests/module_*.rs` | WP1 | ADR 0012 items 7 to 9 behave as tested in section 5 with the simulator; 20-cycle hotplug test without leaked thread; `COMMANDS` listed |
| WP4 Hantek adapter | developer-4 | `backend/features/device/src/hantek/**`, `backend/features/device/tests/hantek_*.rs`, `.github/workflows/ci.yml` (libusb step only) | WP1, D8 | `HantekCatalog` and `HantekDevice` per ADR 0012 items 1 to 6; every conversion unit tested; table equality test against the driver; `HantekDevice` passes the same behaviour tests as the simulator where the mock allows; `#[ignore]` smoke test `hardware_*` (open, identity, capture 1 and 4 channels) for the human; CI installs libusb |
| WP5 acquisition domain | developer-5 | `backend/features/acquisition/src/domain/**`, `backend/features/acquisition/tests/domain_*.rs` | WP1 (types), WP2 (formatter, may start on a stub formatter trait and switch) | Section 3.3 rules and 3.2 geometry, `Instrument`, `next_status`, `CaptureMeter`, defaults (D5), probe and invert, formatting through WP2; all pure, tested with `FakeClock` |
| WP6 acquisition module | developer-2 (after WP2) | `backend/features/acquisition/src/module/**`, `backend/features/acquisition/tests/module_*.rs` | WP1, WP5 signatures (3.6), WP3 for the integration test | Handlers, consumer thread (`recv_timeout` on `CaptureCompleted`, drains `ConnectionChanged`), events at the rates of 3.3, restore after reconnect, stale-revision drop; integration test with `DeviceModule` and simulator |
| WP7 display | developer-1 (after WP1) | `backend/features/display/src/{convert.rs,wire.rs,module.rs}`, `backend/features/display/tests/**` incl. `fixtures/frame-v1.bin` | WP1 | ADR 0011 layout; scaling and crop formulas tested; invert; the fixture is produced by a test and committed |
| WP8 shell | developer-3 (after WP3) | `src-tauri/**` (composition, `ipc/{commands,events,frames,mod}.rs`, `manifest.rs`, `build.rs`, `capabilities/main.json`, `permissions/autogenerated/**`, `Cargo.toml`, `tauri.conf.json` only if needed), `frontend/src/api/generated/commands.ts`, `packaging/udev/99-hantek6xx4.rules`, `docs/usb-permissions.md`, `README.md` | WP1; module constructors from WP3, WP6, WP7 for the final wiring | All commands of 3.4 registered with permissions; event forwarders; `get_snapshot`; frame forwarder with `stamp_sent`; `SWEEPSCOPE_SIMULATOR=1`; `env_logger`; manifest test compares everything; the app starts with the simulator and answers `get_snapshot` in a shell test |
| WP9 frontend client and stores | developer-7 | `frontend/src/api/{backend-client.ts,tauri-client.ts,fake-client.ts,index.ts}` and their tests, `frontend/src/app/instrument/**` | WP1 (generated types), WP8 for the manifest (or hand-edited `commands.ts` until it lands) | Stores with the revision guard, commands, selectors, `openStream`, fake client supports events and streams; all tested without Tauri |
| WP10 frontend live model and waveform | developer-8 | `frontend/src/app/main-window/{live-main-window.ts,select-*.ts,*.test.ts}` (new files only), `frontend/src/app/waveform/**` | WP9 interfaces (3.7), WP11 type files | Model built from stores; frame decode with the golden fixture; frame feed coalescing to rAF; frame-budget probe; graph state selector |
| WP11 frontend pages and types | developer-6 | `frontend/src/pages/main-window/**`, `frontend/src/app/main-window/{types.ts,actions.ts,mock-*.ts,index.ts}` and their tests, `frontend/src/main.ts` (last, after WP9 and WP10) | none for step 1 (types, actions, mock updated); WP9, WP10 for `main.ts` | Every control calls its action; disabled controls show reasons; banners with actions (`Copy command`, `Retry`, `Use simulator`); status bar slots; trigger and ground marker drags emit actions; bug #40 fixed test first; `main.ts` wires the real stores |
| WP12 component bugs | developer-9 | `frontend/src/ui/marker/**`, `frontend/src/ui/segmented/**`, `frontend/e2e/marker-a11y.spec.ts` (new) | none | #41 (aria-valuenow during drag and keyboard moves, with the e2e test) and #42 (no wrapping in Segmented options) fixed test first |

### 4.2 Waves and critical path
```
Wave 0:  WP1 (critical path, hand over first)   WP2 (engineering-format)   WP11 step 1 (types, actions, mocks)   WP12   D8 (human)
Wave 1 (after WP1):  WP3  WP4  WP5  WP7  WP9
Wave 2:  WP6 (after WP2, WP5)   WP8 (after WP3; final wiring after WP6, WP7)   WP10 (after WP9 and WP11 step 1)   WP11 rest
Wave 3:  WP11 main.ts, integration, hardware checklist (human), perf probe run
```
The PM may run fewer developers: WP2 then WP6 (developer-2), WP1 then WP7 (developer-1), WP3 then WP8 (developer-3) are already sequential by design; WP9 and WP10 can be one
developer, WP12 can follow WP11.

### 4.3 Branches and worktrees
Feature branch `feat/m1-live-view` is the integration branch. Each package works in its own worktree on `feat/m1-live-view-wpN-<slug>` created from the feature branch after
its dependencies merged into it; packages merge back in wave order, WP1 first. A package never edits a file owned by another package; a needed change is a message to the owner or the
staff engineer. Developers may commit locally; QA starts when the feature branch contains WP1 to WP11.

## 5. Test strategy (given / when / then)
Names follow `given_..._when_..._then_...`. Backend tests use the simulator, `FakeClock` for pure logic and `SystemClock` with a bounded `eventually(timeout)` helper
(added to `kernel::testing` by WP1) for the few tests that run real threads. No test touches hardware except the `#[ignore]` smoke test.

**Kernel (WP1).** Given a subscriber when `recv_timeout` waits and an event is published from another thread then it returns it before the timeout. Given a latest subscription when three
events are published then only the last is received and `lagged` is 2. Given the moved types when compared with the exported TypeScript then the committed files equal the generated ones.

**Engineering format (WP2).** Given 0.5 V per div then text `500` unit `mV/div`; given 1 V per div then `1` and `V/div` (no trailing zeros for scales). Given 1.9 V then `1.90 V`. Given 250e6 samples per second then `250 MS/s`. Given 2e-9 s then `2.00 ns` (trailing zeros kept, 3 significant digits). The table of golden
vectors is the test. Given NaN then `-`. Given a negative value then U+2212.

**Device module (WP3).**
- given a simulator plugged in when the module starts then the snapshot goes `Disconnected`, `Connecting`, `Ready` with identity and capabilities in the event.
- given a ready device when it is unplugged then `Disconnected` is published within 1 s (real clock) and the worker thread has ended.
- given an unplugged simulator when it is plugged in again then the state returns to `Ready` with a new session and `ApplySetup` is accepted.
- given 20 unplug and replug cycles (the simulator control) when they finish then the state is `Ready`, no call hung, and the process thread count (Linux `/proc/self/task`) is back to its baseline.
- given a catalog whose `open` returns `NotFound` after `Attached` when the module handles it then the state is `Disconnected` and no issue is published; given `PermissionDenied` then `Error` with kind `PermissionDenied` and the help command; given `Io` then kind `Communication`.
- given an error state when `RetryConnection` arrives and the open now succeeds then the state is `Ready`.
- given a pending `ApplySetup` burst of five revisions when the worker wakes then only the last is configured and `CaptureCompleted` carries that revision.
- given capturing on and a capture in flight when `SetCapturing { on: false }` arrives then the finished frame is not published.
- given normal sweep and no trigger when a capture times out then `CaptureCompleted` carries `TimedOut` and the loop continues; given `Disconnected` mid-capture then the state is `Disconnected`; given another error then `Error` and capturing stops.
- given `UseSimulator { enabled: true }` with no hardware then a simulator session opens; given a hardware `Attached` while it runs then the simulator session ends and the hardware session opens.
- given a request while no session exists when `ApplySetup` arrives then it returns a `Recoverable` error naming the reason.

**Hantek adapter (WP4).** Pure conversions: given each range index then the driver `VoltsPerDiv` has the same volts (all 12) and each time base index the same seconds per division (all 36); given a position of +2 div in four channel
mode then the driver position byte reproduces +2 div through `zero_code_for_position` within one byte step; given +4 div on a narrow range then it clamps to the limit in `max_position_divs`; given a trigger level of 0 V at position 0 then the ADC code
equals the zero code; given a level above the window then the code clamps to 228; given a driver frame with `triggered` and a trigger sample then `Triggered { sample_index }`, without `Auto`; given each driver `Error` then the `DeviceError` of ADR 0012, with `Usb(Access)` giving
`PermissionDenied`. Catalog: given `Connected` and `Disconnected` monitor events (the driver's `MonitorOptions { polling: Some(..) }` against a scripted source is not public; use the forwarding function directly) then `CatalogEvent` in order, and a late subscriber is replayed the attached set.
Device on `MockTransport`: given a scripted open, configure and capture then the kernel frame equals the driver frame converted; given `force_capture` from normal sweep then a configure with auto sweep precedes the capture and the requested sweep is re-applied before the next `capture`. `hardware_*` tests are `#[ignore]` and print what they read.

**Acquisition domain (WP5, pure).** Geometry (3.2 examples). Defaults of D5. Given the 36 time bases and 12 ranges when stepping then the first and last step are impossible with a reason, and roll steps are blocked. Given a last enabled channel when disabling it then rejected. Given the trigger source disabled then the source moves.
Given a position beyond the limit then it clamps. Given a range change then the trigger level in volts is unchanged and `level_divisions` changes. Given probe 10x then the range label and level label are multiplied by 10 and the frame scale is not. Given an unverified range or time base then the marker text and `verified` flag. Given no ready device then every availability has a reason.
Given `on_ready` after `on_lost` then the restored setup equals the setup before the loss and `resume` follows D4. Status machine: every transition of 3.3. `CaptureMeter` with `FakeClock`: given frames every 50 ms then 20 per second; given none for 2 s then `-`; dead time clamped.

**Acquisition module (WP6).** Given a ready simulated device and `set_running(true)` then `status_changed` shows `Armed` then `Triggered` or `Auto`, `facts_changed` shows a measured capture rate that is not a constant (changes with the number of enabled channels at a fast time base only if the simulator's cycle changes; otherwise at least the field is `Some` and moves with the clock) and `FrameCaptured` events appear. Given `set_running(false)` then no further `FrameCaptured`
and `Stopped`. Given a setup change then exactly one `ApplySetup`, and frames of the older revision are not published. Given an unplug then `running` becomes false within 1 s and `controls` are unavailable; given replug then the setup equals the earlier one and acquisition resumes (D4).
Given a driver error mid-capture (injected) then running is false, the state is visible and no panic. Given `force_trigger` in normal sweep with no trigger then a frame arrives. Given the instrument slice then its revision grows by one per change and the snapshot equals the last event.

**Display (WP7).** Given a frame and context when encoded then header fields and `f32` values match the ADR 0011 table; golden fixture equality; given invert then the trace mirrors around the ground level; given an off-screen sample then the value lies outside 0 to 8 unclamped; given a window that excludes the trigger sample then the index is -1; given `stamp_sent` then bytes 16 to 24 change and the rest not; given two frames in a latest-wins subscription then only the newer is encoded.

**Shell (WP8).** Manifest, capability, `build.rs` and command lists equal (existing test extended); `get_snapshot` on a composed backend with the simulator; every slice event reaches a recording emitter; the frame forwarder sends the newest frame and stops on `close_frame_stream`; error serialisation of the new commands has category and message without paths.

**Frontend (WP9 to WP12).**
- stores: given an event with a lower revision than the applied one then ignored; given the snapshot arriving after a newer event then the event wins; given a backend error then a Toast and the store unchanged.
- commands: given `step(range 1, +1)` then one `step_setting` call with that target; given a rejected command then `notify` with the message.
- selectors: given ready and running then `isAcquiring`; given disconnected with a last frame then `graphState = frozen`; given no frame ever then `empty`; given `Phase.WaitingForTrigger` then the chip `Waiting for trigger`.
- decode: given the golden fixture then channels, sample count, trigger index and values equal the Rust test; given an unknown magic then it throws a typed error.
- frame feed: given 60 frames in one animation frame then one publish with the newest; given `stop` then no more publishes.
- model: given the four slices then every field of `MainWindowModel` (strings from the backend, positions as fractions); given `connection.issue` of kind `PermissionDenied` then an error banner with `Copy command` and `Retry`.
- pages (component tests with the real model builder and the fake client): given a click on the channel enable toggle then `setChannelEnabled`; given the stepper wheel then `stepVoltsPerDiv`; given a drag of the trigger level marker then a ghost during the drag and one `setTriggerLevel` on release; given a disabled control then `aria-disabled` and the reason as tooltip; given the graph frozen then the `Disconnected` state is visible.
- bugs: #40 given four rows then equal column widths and the disabled row border equals the others (computed style test); #41 given a drag or `Alt+Arrow` then `aria-valuenow` changes in the same step (e2e); #42 given the Settings popover then each option is one line.
- e2e (QA owns): the main window against the fake Tauri stub with scripted events and frames covering US-01, 04, 09 and the unplug freeze; no hardware.

**Hardware.** Checklist [hardware-checklist.md](hardware-checklist.md), run by the human on the 6254BC; it closes US-01, 03, 04 and 09. Measured figures go to the local notes of the person who ran it and rule outcomes to the QA report (ADR 0008 style: outcomes, not figures, in committed files).

## 6. Risks and hardware limits
| Risk | Effect | Mitigation |
|---|---|---|
| Each setting change is a full `configure` (over 100 ms; relays 54 ms, positions 4 x 10 ms, ADC registers) | Dragging the trigger level or wheeling the time base lags; frames pause meanwhile | Coalescing latest-wins, ghosts in the UI, measured in the checklist; driver change request for a partial trigger update |
| `capture` cannot be cancelled; normal sweep waits up to 1.5 record durations | Stop is delayed on the device (user sees it at once); a Run right after waits | ADR 0012 item 9; stated in AC ruling #7; roll steps disabled (D1) |
| Opening during re-enumeration wedges the scope (PRD 5.2) | Needs a replug | Only `DeviceMonitor` events open; 20-plug-in checklist item |
| Real frame cycle is unknown (simulator's 15 + 5 ms are guesses) | Capture rate and dead time on screen differ from the demo | Measured by the facts; checklist H-13 records it; update `simulator/timing.rs` constants afterwards |
| libusb development package missing on the dev machine | The workspace does not build | D8 |
| Hotplug mode of libusb (callbacks vs polling fallback) differs | Different detach latency | `DeviceMonitor::mode()` is logged at start; US-03 timing checked on hardware |
| Trigger position accuracy about +-6 samples, offset depends on channels and time base | Edge not exactly at the marker; at 2 ns to 100 ns per div the edge may fall outside the 6 to 51 sample window | PRD 5.2 note; trigger index -1 when outside; no sub-sample claims |
| Position span is narrow on the 50 mV, 500 mV and 5 V ranges | Ground marker cannot reach +-4 div on common ranges | `max_position_divs`, clamp, tooltip |
| Volts accuracy 2 to 3 counts, one unit measured | Numbers are approximate | `Unverified` markers; accuracy note slot is a placeholder (Q-E4) |
| A frame in flight at a setup change is dropped | One frame lost per change | Intended (ADR 0010 item 3) |
| Frame-time outliers seen in spike #2 on one machine; no second machine | DSP-7 cannot be signed off for the laptop class | Frame-budget probe (WP10), repeated runs on the dev machine early in the wave (H-14), DSP-7 stays open until a second machine exists |
| Frames and events share the Tauri main thread with the WebView | Rare stalls on slow machines | Latest-wins feed; one publish per animation frame; status and facts events are deduplicated and rate limited |
| Driver `Error` is `non_exhaustive` | New variants map to `Io` | Wildcard arm tested |
| Normal sweep with no trigger and no way to see why | User thinks the app froze | Chip `Waiting for trigger` (design J6); Force button |

Limits applied from PRD 5.2: frame based capture with dead time shown, effective sample rate shown (1, 2, 4 ns by enabled channels), time bases up to 500 ns share the ADC interval,
trigger accuracy note, unverified range markers, open only through `DeviceMonitor`, no raw requests, udev help, hotplug with settle time.

## 7. Dependencies
| Dependency | Where | Why not write it | Licence, status, size |
|---|---|---|---|
| `hantek6xx4-lib` 0.2.0 | `device` only | It is the device layer (PRD 8.1), our own project | MIT, published, small |
| `rusb` 0.9 | `device` only | Pattern-match `Error::Usb(rusb::Error::Access)` exactly; same version the driver pulls, so the lockfile does not grow | MIT, maintained |
| `libusb-1.0` (system) | build and run | The driver's USB backend; `vendored` feature available through `device/vendored` | LGPL-2.1, dynamic link |
| `env_logger` 0.11 | `src-tauri` only | `RUST_LOG` for driver and adapter logs during hardware sessions; a hundred lines otherwise | MIT or Apache-2.0, maintained |
| none new in the frontend | | | |
Everything else (threads, condition variables, `Arc`, binary encoding) is `std`.

## 8. Rollout and migration
- Merge order: WP1, then wave 1 to 3 packages as they finish; QA test plan before the developers hand over (workflow step 8); code review on the whole feature branch.
- After G3 approval: ADRs 0010 to 0013 move to `accepted`; ADR 0006 is accepted with the D2 scope; `docs/architecture.md` sections 4.2, 4.7, 9 and 11 are updated (device and acquisition split, `Acquiring` derived, snapshot format decided, Q-E1 implemented, Q-E3 answered by ADR 0013, Q-E7 policy decided).
- Docs touched by the packages: `docs/usb-permissions.md`, README prerequisites and commands, `docs/design.md` untouched (designer reviews the simulator action and the `Settling` limit).
- PRD edits for the PM after M1: DEV-1 state list note (`Acquiring` is derived), the ACQ-5 reading, the record of the roll limit.
- No data migration: nothing is persisted by M1.

## 9. Review and QA rulings
(empty; rounds are appended here, approved sections are not rewritten)
