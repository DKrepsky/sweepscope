# Implementation spec: M0 Spike (#2 frame streaming and rendering, #3 ScopeDevice and simulator, #4 roll mode)

Status: approved by the human at G3 on 2026-11-22
Links: stories #2, #3, #4; [prd.md](../../prd.md) 4.3, 5, 8.2, 8.3, 9; [architecture.md](../../architecture.md) 4.2, 4.3, 4.6 to 4.8, 5.7;
ADRs [0003](../../adr/0003-typed-contract-with-ts-rs.md), [0006](../../adr/0006-graph-renderer-canvas-first.md),
[0007](../../adr/0007-scope-device-and-catalog-traits.md), [0008](../../adr/0008-spike-policy-and-measurement-reports.md).
Design: not needed for any of the three.

## 1. Scope

### In
- **#3 (product code, merged normally):** new crate `sweepscope-device` with `ScopeDevice`, `DeviceCatalog`, value types,
  `DeviceError`, and `SimulatedDevice` + `SimulatedCatalog` + `SimulatorControl`; `Clock::sleep` in the kernel; unit tests
  without hardware. Confirms the type-generation choice of ADR 0003 (see 2.4).
- **#2 (spike, code not merged):** a throwaway Tauri app that streams synthetic 4 x 4096 frames at 60 messages per
  second over a `Channel`, draws them with Canvas 2D and WebGL2 (with and without persistence), measures dropped frames,
  latency and frame time, and compares `WEBKIT_DISABLE_DMABUF_RENDERER` on and off. A report with a go/no-go for WebGL2.
- **#4 (spike, code not merged):** a throwaway Tauri app that polls the simulator in roll mode at 500 ms/div,
  keeps a fixed Rust ring buffer, records to a file, sends ordered chunks with `Gap` items over a `Channel`, and keeps only the
  visible window in the frontend. One-hour run, memory and file growth recorded. A report that accepts or rejects the pattern.

### Out
- The Hantek adapter, the `device` module (thread, bus messages), `acquisition`, `display`, kernel streams, any UI.
- A wire format for the product frame stream: decided in an ADR after the #2 report, not now.
- Spike results are not product requirements; they may propose changes (section 8).

### Scope notes and findings for the PM and human
1. **Type-generation tool (M0 roadmap item).** ts-rs was chosen in the scaffold (ADR 0003, "to evaluate in M0") and works in CI. No
   story covers the roadmap item. This spec closes it through #3: the new identity and capabilities types are exported with
   ts-rs, CI diffs them, and ADR 0003 moves to accepted if that works. tauri-specta is still a release candidate
   (checked 2026-11-22), so the reason to reject it stands. No acceptance criterion is touched.
2. **Real-time and hardware.** #2 needs real display sessions on two machines (human-run, section 7). #4 needs one hour of wall
   time but no special hardware (developer-run in the background, in an isolated Xvfb session).
3. **End-to-end latency (#2 AC4) cannot be fully measured in software.** JavaScript can observe up to the frame after the
   draw call, not scan-out. The spike measures send to receive, send to draw submitted (with forced GPU completion) and
   send to the next animation frame callback, and adds one refresh interval as the estimate of the last stage. An optional
   external check (phone in slow motion filming the screen and a timestamp flash) is listed as a human step. The PM and human
   decide whether the in-app figure is acceptable for the 100 ms target (it is a very large margin, so the answer is
   unlikely to change).
4. **Dead time of the simulator (#3 AC3) is an assumption.** The driver documents the 20 ms poll interval but no measured
   frame cycle. The simulator uses documented constants marked unverified. An optional human step (section 7) measures the real
   cycle with `hantek6xx4-cli` on the 6254BC and corrects them.
5. **Sample payload type.** PRD 4.3 assumes 8-bit samples (16 KiB per frame), architecture 3 fixes `f32` payloads (64 KiB).
   #2 measures both; the report may change the convention.
6. **Stop latency finding (ADR 0007, item 8).** The driver cannot cancel a capture. ACQ-5 allows it (one frame time or 250 ms).
   Recorded as a driver wish (PRD open question 5), not a blocker.
7. **No acceptance criterion needs to change.** Where an AC is stricter than what software can measure, the measure and its limit are named
   in the report.

## 2. Design of the change

### 2.1 Crates and folders touched
| Area | Path | Change | Merged to main |
|---|---|---|---|
| Kernel | `backend/kernel/src/clock.rs`, `testing.rs` | `Clock::sleep(Duration)`; `FakeClock::sleep` advances time | yes |
| Feature | `backend/features/device` (`sweepscope-device`) | new: traits, types, simulator, tests | yes |
| Generated | `frontend/src/api/generated/` | ts-rs exports of identity and capabilities types | yes |
| Spike #2 | `spikes/2-frame-streaming/` | standalone Cargo workspace and plain web assets | no (branch `spike/2-frame-streaming`) |
| Spike #4 | `spikes/4-roll-streaming/` | same | no (branch `spike/4-roll-streaming`, based on the #3 branch) |
| Reports | `docs/spikes/m0/` | reports and results | yes (PR from `docs/spike-<n>-report`) |

The root workspace globs (`backend/features/*`) pick up the new crate. `check-architecture` rules apply: `sweepscope-device` depends only
on `sweepscope-kernel`, `serde`, `ts-rs`. It does not depend on `hantek6xx4-lib` yet (the adapter story adds it).

### 2.2 #3 interfaces (fixed before work starts)
Module layout of `sweepscope-device` (all `pub use` from `lib.rs`; `#![forbid(unsafe_code)]`):

```
src/lib.rs          re-exports only the items below plus `SignalKind`, `device_id()`, `SimulatedDevice::new(Arc<dyn Clock>, SimulatorControl)`,
                    `SimulatedCatalog::new(Arc<dyn Clock>, SimulatorControl)`, `SimulatorControl::{new, unplug, plug_in, inject_overrun, set_signal}`
src/identity.rs     DeviceIdentity, CalibrationStatus, DeviceId
src/capabilities.rs Capabilities, TimeBaseStep, VerticalRange
src/setup.rs        AcquisitionSetup, ChannelSetup, TriggerSetup, Coupling, Slope, Sweep
src/frame.rs        Frame, ChannelData, TriggerOutcome
src/error.rs        DeviceError
src/device.rs       trait ScopeDevice
src/catalog.rs      trait DeviceCatalog, CatalogEvent
src/simulator/      mod.rs, profile.rs (identity and capability tables), signal.rs, timing.rs, edge.rs, device.rs, control.rs, catalog.rs
                    (all modules private; only the lib.rs re-exports are public)
tests/              simulator_identity.rs, simulator_capture.rs, simulator_roll.rs, simulator_faults.rs, second_device.rs
```

```rust
pub trait ScopeDevice: Send {
    fn identity(&self) -> &DeviceIdentity;
    fn capabilities(&self) -> &Capabilities;
    fn configure(&mut self, setup: &AcquisitionSetup) -> Result<(), DeviceError>;
    /// Blocks until a frame is ready or `timeout` elapses (`Timeout`; in normal sweep that means "no trigger yet").
    fn capture(&mut self, timeout: Duration) -> Result<Frame, DeviceError>;
    fn start_roll(&mut self) -> Result<(), DeviceError>;
    /// Samples since the previous call, possibly none. `Overrun` once after data was probably lost; the next call continues at the newest data.
    fn poll_roll(&mut self) -> Result<Frame, DeviceError>;
    fn stop(&mut self) -> Result<(), DeviceError>;
}

pub trait DeviceCatalog: Send {
    fn events(&self) -> Receiver<CatalogEvent>;
    fn open(&self, id: &DeviceId) -> Result<Box<dyn ScopeDevice>, DeviceError>;
}
pub enum CatalogEvent { Attached(DeviceId), Detached(DeviceId) }

pub struct DeviceId(pub String);
pub struct DeviceIdentity {            // derives Serialize, TS
    pub model: String, pub firmware: Option<String>, pub fpga_version: Option<String>,
    pub calibration: CalibrationStatus, pub simulated: bool,
}
pub enum CalibrationStatus { Valid, Invalid }

pub struct Capabilities {              // derives Serialize, TS
    pub channel_count: u8, pub record_len: u32,           // 4 and 4096
    pub time_bases: Vec<TimeBaseStep>,                    // fast to slow
    pub vertical_ranges: Vec<VerticalRange>,
    pub adc_interval_ns: Vec<f64>,                        // index = enabled channels - 1; 3 and 4 equal
    pub samples_per_div: f64,                             // 250
}
pub struct TimeBaseStep { pub seconds_per_div: f64, pub streaming: bool, pub verified: bool }   // streaming = roll, from 500 ms/div
pub struct VerticalRange { pub volts_per_div: f64, pub verified: bool }
impl Capabilities { pub fn sample_interval_s(&self, enabled_channels: u8, time_base_index: u8) -> f64 }  // max(adc, tb / samples_per_div)

pub struct AcquisitionSetup { pub channels: [ChannelSetup; 4], pub time_base_index: u8, pub trigger: TriggerSetup }
pub struct ChannelSetup { pub enabled: bool, pub range_index: u8, pub position_divs: f64, pub coupling: Coupling, pub bandwidth_limit: bool }
pub struct TriggerSetup { pub source: u8, pub level_volts: f64, pub slope: Slope, pub sweep: Sweep, pub horizontal_position_percent: f64 }
pub enum Coupling { Ac, Dc }  pub enum Slope { Rising, Falling }  pub enum Sweep { Auto, Normal }

pub struct Frame {
    pub channels: [Option<ChannelData>; 4],               // None = disabled
    pub counts_per_div: f64, pub sample_interval_s: f64, pub trigger: TriggerOutcome,
}
pub struct ChannelData { pub codes: Vec<u8>, pub volts_per_div: f64, pub zero_code: f64 }
pub enum TriggerOutcome { Triggered { sample_index: usize }, Auto, None }
pub enum DeviceError { Disconnected, NotFound, Timeout, Overrun, InvalidSetting(String), CalibrationInvalid, Io(String) }
impl DeviceError { pub fn category(&self) -> ErrorCategory }  // plus From<DeviceError> for KernelError
```
Rust types are camelCase on the wire where exported (`#[serde(rename_all = "camelCase")]`, as the kernel does); only `f64`, `u8`, `u32`
cross to TypeScript (ADR 0003: no `u64`).

`Clock` gains `fn sleep(&self, duration: Duration)`.

### 2.3 Simulator behaviour (what the tests pin down)
| Behaviour | Rule | Source |
|---|---|---|
| Identity | model `"6254BC (simulated)"`, firmware and FPGA strings fixed, calibration `Valid`, `simulated = true` | DEV-4, #3 AC2 |
| Capabilities | 4 channels, 4096 per channel, 36 time bases 2 ns to 1000 s (1-2-5, `streaming` from index 25 = 500 ms/div), 12 ranges 2 mV to 10 V (`verified` only 50 mV to 1 V, indexes 4 to 8); time base `verified` follows PRD 5.1 (indexes 0 to 22, 24, 25, 27, 30 measured; the rest `false`), `adc_interval_ns = [1, 2, 4, 4]`, 250 samples/div | PRD 5.1 |
| Sample interval | `max(adc[ch-1], tb/250)`. Pins: 4 ch at 200 us/div is 800 ns; 500 ms/div is 2 ms (499 samples/s); 200 ms/div is 0.8 ms. Checked against the driver's `protocol::sample_interval_ns` (read only) | driver A.7 |
| Frame | 4096 codes per enabled channel, none for disabled; codes clipped to 0..255; `counts_per_div` 24.15, 21.6, 21.1 for 4, 2, 1 ADC channels as the driver | driver types |
| Signals | per channel `Sine`, `Square`, `Triangle`, `Calibrator` (1 kHz, 5 V square) plus seeded noise (own xorshift, no `rand`). Phase follows the simulated time, so frames are continuous across captures | #3 AC3 |
| Edge trigger | finds the first crossing of `level_volts` with the right slope at or after the arm time; places it at `horizontal_position_percent` of the record (within +-0 samples; the real +-6 sample error is not simulated and is documented) | PRD 5.2 |
| Sweep | `Auto`: no crossing within the record time gives `TriggerOutcome::Auto` (free-run frame). `Auto` ignores the capture `timeout` and always returns within the record time. `Normal`: no crossing, or a cycle longer than `timeout`, gives `Err(Timeout)` after `timeout` of clock time | ACQ-2 |
| Dead time | cycle = acquisition time (record duration) rounded up to the 20 ms poll interval, plus an assumed readback of 15 ms and re-arm of 5 ms, all `unverified` constants in `timing.rs`, documented in the module docs. `FakeClock` advances by exactly the cycle per `capture` | #3 AC3, PRD 5.2 |
| Roll | `start_roll` records the simulated pointer; `poll_roll` returns the samples produced since the last poll (whole frames), nothing when none; the buffer span is `65536 / stride x interval` (32.8 s at 500 ms/div with 4 channels); when elapsed time reaches 95 % of the span `poll_roll` returns `Overrun` and resynchronises, exactly as the driver | driver 6.4 |
| Faults | `SimulatorControl` (cloneable, `Send`): `unplug()`, `plug_in()`, `inject_overrun()` (next poll returns `Overrun` once), `set_signal(channel, SignalKind)`. After `unplug()` every device call returns `Disconnected` (also a `capture` that is waiting) and the catalog emits `Detached`; `plug_in()` emits `Attached`. A device handle opened before `unplug()` stays `Disconnected` after `plug_in()`; the caller re-opens. Non-finite setup values, invalid or unconfigured calls return `InvalidSetting` | #3 AC4 |
| Catalog | `SimulatedCatalog::new(clock, control)`; `events()` first sends `Attached` for a device already plugged in (as the real monitor); `open` of an unplugged simulator returns `NotFound` | ADR 0007 |

### 2.4 Type generation check (ADR 0003)
`DeviceIdentity`, `CalibrationStatus` and `Capabilities` (with `Vec`, `Option`, nested structs, enums, `f64`, `u8`, `u32`) derive `TS` with the
same `export_to` as the kernel and are exported by a test in `backend/features/device/tests/contract_export.rs` (same pattern as the
kernel's). CI already fails if `frontend/src/api/generated` changes. Done when the generated files are committed, CI passes and
they type-check in `frontend` unchanged. The developer reports any ts-rs limitation found; the staff engineer then moves ADR 0003 to accepted.

### 2.5 #2 spike design
```
spikes/2-frame-streaming/
  Cargo.toml  (own [workspace])   src-tauri/{Cargo.toml,tauri.conf.json,build.rs,src/*.rs,capabilities/}
  web/{index.html,main.js,scenarios.json,decode.js,metrics.js,render-canvas2d.js,render-webgl2.js,runner.js}
  tools/{run-all.sh,capture-env.sh,summarize.mjs}
```
Backend: a generator thread paces at 60 Hz on absolute deadlines (sleep, then spin the last 1 ms), builds a frame from synthetic
channels (sine, square, triangle, noise, phase advancing per sequence), encodes it, sends on the `Channel`, and records its own send
jitter. State: owned by the spike backend, one `Mutex<Option<StreamHandle>>`; no product rules apply.

Tauri commands (the whole contract between the two packages):
| Command | Arguments | Returns |
|---|---|---|
| `start_stream` | `channel: Channel<tauri::ipc::Response>`, `config: { payload: "u8" \| "f32" \| "json", rateHz: 60, durationS: number }` | `{ framesSent: number, sendJitterMs: { median, p99, max } }` when the stream ends |
| `stop_stream` | none | none |
| `environment` | none | `{ env: Record<string,string>, webkitVersion, tauriVersion }` |
| `save_result` | `name: string`, `json: string` | writes `<SPIKE_OUT>/<name>.json` |
| `exit_app` | none | closes the app |

Binary frame (little endian; the `u8` and `f32` payloads; both are above 1 KiB, so Tauri uses its fetch path, the one that matters for real frames):
| Offset | Size | Field |
|---|---|---|
| 0 | 4 | magic `0x31465353` ("SSF1") |
| 4 | 4 | `u32` sequence |
| 8 | 8 | `f64` sent timestamp, microseconds since the Unix epoch, taken immediately before `Channel::send` |
| 16 | 8 | `f64` produced timestamp, same clock, taken when the frame was generated |
| 24 | 1 | sample format: 0 = `u8` codes, 1 = `f32` scaled to divisions (-4..4) |
| 25 | 1 | channel count (4) |
| 26 | 2 | reserved, zero |
| 28 | 4 | `u32` samples per channel (4096) |
| 32 | n | channel-major samples (all of channel 0, then 1, ...). Header size 32 keeps `Float32Array` aligned |

Sizes: 16 416 bytes (`u8`), 65 568 bytes (`f32`). `payload: "json"` sends the same `u8` codes through `Channel<Vec<u8>>` as a JSON array
(the baseline that justifies binary; optional, run only if time allows).

Frontend (plain ES modules, no bundler): `runner.js` runs scenarios from `scenarios.json` in order (autorun when `SPIKE_AUTORUN=1`, filtered by
`SPIKE_SCENARIOS`), each for `durationS` (60) after a 2 s warm-up that is excluded from statistics, then calls `save_result`, and at the end
`exit_app`. `decode.js` returns typed-array views, no copies. The renderer draws the newest received frame once per animation frame (latest wins);
frames received and replaced before drawing count as `coalesced`.

Scenario matrix (per environment variant, per machine):
| id | renderer | persistence | payload | gpu sync |
|---|---|---|---|---|
| ipc-u8 | none | no | u8 | no |
| ipc-f32 | none | no | f32 | no |
| c2d-plain | canvas2d | no | f32 | no |
| c2d-persist | canvas2d | yes | f32 | no |
| gl2-plain | webgl2 | no | f32 | no |
| gl2-persist | webgl2 | yes | f32 | no |
| c2d-persist-sync | canvas2d | yes | f32 | yes |
| gl2-persist-sync | webgl2 | yes | f32 | yes |

Environment variants: `default` (nothing set) and `dmabuf-off` (`WEBKIT_DISABLE_DMABUF_RENDERER=1`). If `default` shows a blank view, crash or
no WebGL2 context, record it with a screenshot, then try `WEBKIT_DISABLE_COMPOSITING_MODE=1` as a third variant, and record which works. 16 runs of
62 s, about 17 minutes per machine. Canvas size 1200 x 600 CSS px at the window's device pixel ratio; the ratio is recorded.

Rendering techniques (the spike's variables; kept small):
- **Canvas 2D:** one path per channel (`moveTo`/`lineTo` of 4096 points, 1 device px, token-free colours), persistence by drawing a translucent black
  rectangle over the previous result before each frame (`alpha = 0.2`), then the traces.
- **WebGL2:** one `LINE_STRIP` per channel from a dynamic vertex buffer (x from index, y from the sample; `u8` codes uploaded as `UNSIGNED_BYTE`
  normalised or `f32` as is, matching the payload); persistence by ping-pong framebuffers with a fade pass. Line width is 1 px (WebGL limit); thick
  lines are out of scope and noted as a limit.
- `gpu sync`: after drawing, Canvas 2D reads one pixel (`getImageData(0,0,1,1)`), WebGL2 calls `gl.finish()` and reads one pixel. This forces GPU
  completion so `draw_ms` includes GPU time, at the cost of stalling the pipeline; so these runs are reported separately.

Metrics per run (JSON, `save_result`), definitions fixed here:
| Metric | Definition |
|---|---|
| `framesSent`, `framesReceived` | counts after warm-up; `seqGaps` = missing sequence numbers = dropped in transport; `coalesced` = received and replaced before a draw |
| `fps` | animation frames drawn per second, after warm-up |
| `intervalMs` | time between consecutive animation frame callbacks: median, p95, p99, max, count over 20 ms, count over 33.3 ms |
| `drawMs` | time from callback start to the end of the draw call (and the one-pixel read when `gpu sync`): **mean**, median, p99, max. Timer resolution is 1 ms, so the mean (error about 0.3 ms divided by the square root of the frame count) is the comparison figure; median and p99 are informative |
| `ipcMs` | `time of the channel.onmessage call - sent timestamp`: median, p99, max. It includes main-thread queueing: a message that arrives while the thread is drawing waits. In the ipc-only scenarios the thread is idle, so there it is the transport time |
| `e2eMs` | `(timestamp at the first animation frame callback after the draw) - sent timestamp`: median, p99, max |
| `sendJitterMs` | Rust pacing error against the absolute deadline: median, p99, max |
| `webgl2Available` | `canvas.getContext('webgl2') != null` |

Every timing metric (`intervalMs`, `drawMs`, `ipcMs`, `e2eMs`) also stores a histogram: 1 ms buckets from 0 to 99 ms plus an overflow count (ADR 0008 amendment). Percentiles use the nearest-rank method on the sorted samples. Clock for latency: Rust `SystemTime` and JavaScript `performance.timeOrigin +
performance.now()`; both are wall-clock on one machine. WebKit may coarsen `performance.now` (the report states the observed resolution; if it is
coarser than 1 ms the latency tables carry that as an error bar).

### 2.6 #4 spike design
```
spikes/4-roll-streaming/
  Cargo.toml  (own [workspace], depends on sweepscope-device and sweepscope-kernel by path)
  src-tauri/src/{main.rs,poller.rs,ring.rs,recorder.rs,stream.rs,memory.rs,scaled_clock.rs}
  web/{index.html,main.js,window.js,decode.js,gap.js}
  tools/{soak.sh,summarize.mjs}
```
Flow: `poller` thread on `SimulatedDevice` (4 channels enabled, 500 ms/div, so 2 ms interval, 499 samples per second per channel) calls
`poll_roll` every 100 ms. Each chunk goes to (a) `RollRing`, (b) the recorder, (c) the stream to the WebView. An `Overrun` becomes a gap item.

- **`RollRing`** (`ring.rs`): fixed-capacity ring per channel (default 10 minutes of samples, about 1.2 MB for four channels) with an absolute sample
  counter; `push` never allocates after construction; `window(range)` copies out for a late-joining view. Memory is constant by construction and the test
  proves it (capacity unchanged after 10 x capacity pushes).
- **Stream items** (binary on a `Channel`, little endian), ordered, with sequence numbers:
  | Offset | Size | Field |
  |---|---|---|
  | 0 | 4 | magic `0x31525353` ("SSR1") |
  | 4 | 1 | kind: 0 samples, 1 gap, 2 close |
  | 5 | 3 | reserved |
  | 8 | 4 | `u32` item sequence (a skipped number means a lost item) |
  | 12 | 4 | `u32` count per channel (samples), or 0 |
  | 16 | 8 | `f64` first sample index (samples) or `after` sample index (gap) |
  | 24 | 8 | `f64` missed samples per channel, estimate (gap) or 0 |
  | 32 | n | `u8` codes, channel-major (samples only) |
  The sample index is absolute and counts delivered plus estimated missed samples, so the time axis stays continuous: `t = index x sample_interval`. The chunk after a gap
  starts at `after + missed`. The missed count is an estimate from elapsed time (the driver cannot do better, PRD 5.2) and the report says so.
- **Gap marker placement** (the thing the report must show): the frontend maps `after` to x as `(after - windowStart) / samplesPerPixel` and draws a hatched
  band `missed / samplesPerPixel` wide; `gap.js` is the pure function and is unit-tested, including a gap partly scrolled out of the window.
- **Recorder**: a thread with a bounded queue (64 chunks) appending to a binary file through `BufWriter` (header with channel count, interval, start time; then records of the same item layout). When the
  queue is full the oldest item is dropped and a gap record is written, never a silent splice (architecture 4.3, ordered gap-marked policy).
- **Frontend window**: preallocated `Float32Array` ring per channel sized for the visible window only (10 divisions = 5 s = 2500 samples), drawn on a 2D
  canvas; no array grows with time.
- **Memory sampling** (`memory.rs`, std only): every 10 s, RSS of the process (`/proc/self/statm`) and of its child processes (`WebKitWebProcess`, found by parent pid in `/proc/*/stat`), plus
  ring fill and file size, appended to `memory.csv`.
- **Scaled clock** (`scaled_clock.rs`): a `Clock` wrapping `SystemClock` with a speed factor, so the 60x accelerated run (one simulated hour in one minute) can find logic faults quickly. It is a
  smoke test; the acceptance run is the real-time hour.
- **Overrun injection**: `SimulatorControl::inject_overrun()` at fixed times (every 10 minutes) and one natural overrun (the poller paused for longer than 95 % of the span, by a debug command) so
  both paths are seen.

Pass rules for #4 (fixed before measuring):
| Rule | Value |
|---|---|
| Rust RSS growth after the first 5 minutes | at most 2 MiB over the remaining time |
| WebKitWebProcess RSS growth after the first 5 minutes | at most 10 MiB (garbage collector noise) |
| Recorded file | grows linearly; final size within 1 % of `samples x channels x header-adjusted` expectation; every injected gap is present in the file as a gap record |
| Gaps reaching the frontend | every injected gap seen once, at the right absolute index (checked by the window code logging its markers) |
| Items | no sequence number skipped on the Channel |
| CPU | poller thread and recorder below 2 % of one core on average (informative; the stream is 2 KB/s) |

## 3. Work packages
Four developers; the PM may run them in fewer, in the order of the waves. Disjoint files are listed per package.

| WP | Owner | Files and crates | Depends on | Done when |
|---|---|---|---|---|
| WP3.1 interfaces | developer-1 | `backend/kernel/src/clock.rs`, `backend/kernel/src/testing.rs`, `backend/kernel/tests/clock.rs` (new), `backend/features/device/Cargo.toml`, `src/{lib,identity,capabilities,setup,frame,error,device,catalog}.rs`, `tests/contract_export.rs`, `frontend/src/api/generated/*` (new files) | none | Section 2.2 compiles exactly; `Clock::sleep` tested; `sample_interval_s` and `DeviceError` mapping tested; exports committed; `npm run check` and `cargo test --workspace` pass; handed over to WP3.2 and WP3.3 |
| WP3.2 simulator parts | developer-2 | `backend/features/device/src/simulator/{signal,timing,edge}.rs` and their unit tests inside those files; no trait use | WP3.1 types only to agree on names; can start in parallel on plain `f64`/`u8` functions | Pure functions: `signal::sample(kind, t)`, `timing::{sample_interval, record_duration, frame_cycle, roll_span}`, `edge::find_crossing(samples_fn, level, slope, from, to)`; unit tests for each rule of 2.3 |
| WP3.3 simulator device | developer-1 | `src/simulator/{mod,device,control,catalog}.rs`, `tests/simulator_*.rs`, `tests/second_device.rs` | WP3.1, WP3.2 | All behaviours of 2.3 covered by the tests in section 5; `second_device.rs` implements `ScopeDevice` with a 30-line scripted double and runs it through `Box<dyn ScopeDevice>`; crate docs list the simulator limits |
| WP2.1 stream backend | developer-3 | `spikes/2-frame-streaming/Cargo.toml`, `src-tauri/**`, `tools/*` | none | Commands of 2.5 work; codec round-trip test in Rust; `run-all.sh` runs every scenario and writes results; `capture-env.sh` strips host and user data |
| WP2.2 stream frontend | developer-4 | `spikes/2-frame-streaming/web/**` | none (payload and command contract fixed in 2.5) | Both renderers draw the synthetic stream; `metrics.js` and `decode.js` unit-tested with `node --test`; runner writes the JSON of 2.5 |
| WP2.3 harness check and data | developer-3 and developer-4 | `docs/spikes/m0/results/2/<machine>/**`, `docs/spikes/m0/2-frame-streaming.md` (data tables only) | WP2.1, WP2.2, human runs (section 7) | Harness verified under Xvfb (numbers flagged invalid); results of both machines committed; tables generated by `summarize.mjs` |
| WP4.1 roll backend | developer-2 | `spikes/4-roll-streaming/Cargo.toml`, `src-tauri/**`, `tools/soak.sh` | WP3.3 merged on the feature branch | Poller, ring, recorder, stream, memory sampler, scaled clock work; unit tests for ring (constant capacity), recorder (gap on full queue), stream codec; accelerated run clean |
| WP4.2 roll frontend | developer-4 | `spikes/4-roll-streaming/web/**` | WP4.1 contract (2.6), WP2.2 finished (reuses its draw code by copy) | Visible-window ring, gap markers, `gap.js` unit-tested |
| WP4.3 soak and data | developer-2 | `docs/spikes/m0/results/4/<machine>/**`, `docs/spikes/m0/4-roll-streaming.md` (data tables only) | WP4.1, WP4.2 | One real-time hour in an isolated Xvfb session finished, `memory.csv` and summary committed |
| WP5 decisions | staff-engineer | `docs/spikes/m0/*.md` (Decision, Changes requested), ADR 0003 status, workflow line of ADR 0008, follow-up ADRs for the product frame and stream wire format | WP2.3, WP4.3 | Reports complete; proposals for PRD or ADR changes ready for the human |

Waves: wave 1 = WP3.1, WP3.2, WP2.1, WP2.2 in parallel (four developers, no shared files). Wave 2 = WP3.3, WP2.3 harness check, WP4.2 draft. Wave 3 = WP4.1
(needs WP3.3), then human runs for #2 and the soak for #4. Package sizes are small; WP3.1 is the critical path and must be handed over first.

Branches: #3 on `feat/3-scope-device-and-simulator` (full flow, merges normally). `spike/2-frame-streaming` from `main`. `spike/4-roll-streaming` from the #3 branch (it uses
the simulator). Report PRs from `docs/spike-2-report` and `docs/spike-4-report` (ADR 0008).

## 4. State ownership and messages
- #3: no state shared with the frontend yet. The simulator owns its state (clock, pointers, faults); `SimulatorControl` shares an `Arc<Mutex<..>>` with the device inside the crate only.
  No bus messages in M0.
- #2 and #4: spike state is local to the spike backend; nothing here binds the product. Product streams are decided after the reports (architecture 4.3, 4.7, 4.8 stay as written until then).

## 5. Test strategy (given / when / then)
Product code (#3), module level, `FakeClock`, no hardware:
- given a simulated device when identity is read then model contains "simulated", `simulated` is true and firmware, FPGA version and calibration are present.
- given four enabled channels and an edge trigger on a 1 kHz sine at 200 us/div when a frame is captured then each channel has 4096 codes, the trigger sits at the configured position and the clock advanced by one frame cycle.
- given two captures when the clock is read then the time between returns equals the cycle (acquisition rounded to 20 ms plus the unverified readback and re-arm).
- given one, two and four enabled channels when the sample interval is asked then it is 1, 2 and 4 ns at 2 ns/div and `tb/250` at 200 us/div.
- given normal sweep and a level above the signal when capture is called then it returns `Timeout` after the timeout in clock time; given auto sweep then it returns `TriggerOutcome::Auto`.
- given a disabled channel when a frame is captured then its entry is `None`.
- given a sine with amplitude above the range when captured then codes clip at 0 and 255.
- given a roll at 500 ms/div when the clock advances 1 s and the device is polled then about 499 samples per channel are returned; polled again at once then an empty frame.
- given a roll when the clock advances beyond 95 % of the buffer span then `poll_roll` returns `Overrun` once and the next poll continues from the newest data.
- given `inject_overrun` when polled then `Overrun` once, then data.
- given a connected simulator when `unplug` is called then the next `configure`, `capture`, `poll_roll` return `Disconnected` and the catalog emits `Detached`; `plug_in` emits `Attached`; `open` while unplugged returns `NotFound`.
- given a capture waiting for a trigger when `unplug` is called from another thread then `capture` returns `Disconnected` within one poll interval (real clock, one test).
- given an invalid setup (range index out of table, no enabled channel) then `configure` returns `InvalidSetting`.
- given each `DeviceError` then `category()` and `KernelError` conversion are as ADR 0007.
- given a second `ScopeDevice` implemented in the test then it runs through `Box<dyn ScopeDevice>` with no change to the crate (#3 AC1).
- given the exported TypeScript files then they equal the committed ones (CI diff) and `frontend` type-checks.

Spike code (small trusted-number tests only): percentile and interval statistics (known arrays), frame codec round trip (Rust and JavaScript decode the same bytes), ring buffer constant capacity, recorder gap on full queue,
gap marker maths including a partly visible gap, window ring wrap. Run with `cargo test` in the spike workspace and `node --test` in `web/`.

QA gets the product test list above (WP3.x); spikes get no QA e2e. QA reviews the harness instructions for #2 and the results files for completeness against the report format.

## 6. Evaluation rules (fixed before measuring)
These are the pass and decision rules for the reports.

**#2, per machine, per scenario (after the 2 s warm-up):**
| Rule | Pass |
|---|---|
| Frame rate | at least 58 fps on a 60 Hz display (against the refresh rate when higher) |
| Frame time | no animation frame interval over 33.3 ms (PRD 8.3); at most 0.1 % over 20 ms is marginal |
| Transport | `seqGaps` = 0 in every scenario; `ipcMs` p99 below 16 ms **in the ipc-only scenarios** (idle main thread). In render scenarios `ipcMs` is informative (it includes queueing behind the draw); the latency rule below covers it |
| Latency | `e2eMs` p99 plus one refresh interval below 100 ms (PRD 8.3) |

**Decision for WebGL2 (go / no-go):**
- *Go* if WebGL2 creates a context and meets the rules on every measured machine (in the default or the documented workaround environment), and either Canvas 2D with persistence fails a rule on at least one machine, or, in the sync runs, Canvas 2D mean `drawMs` is at least 1.5 x that of WebGL2 **and** at least 4 ms (a quarter of the 16.7 ms frame budget; below that, a ratio of two small numbers is not a material gain and the 1 ms timer makes it unstable).
- *No-go for now* if Canvas 2D with persistence meets the rules everywhere and WebGL2 does not improve mean `drawMs` by 1.5 x or Canvas 2D mean `drawMs` stays under 4 ms: Canvas 2D stays primary and the report proposes making WebGL2 optional (a change to PRD DSP-7 and DSP-9, and ADR 0006 consequences).
- Canvas 2D stays as the fallback in both cases (DSP-9). If WebGL2 is unavailable or unstable on the NVIDIA machine even with the workaround, the report says so and proposes the product default.
- **DMABUF workaround** is confirmed if `dmabuf-off` turns a failing `default` (blank or unstable view, lost WebGL2) into a passing one, or shows no harm where `default` already passes; corrected otherwise (the report names what works instead, and where the product sets it: launcher environment or process start before the window is created).

**#4:** the pass rules of 2.6. Verdict on the stream pattern: accepted if all pass; otherwise the report names the failing rule and a proposed change.

## 7. Human-run steps and hardware
| # | Step | Needs | Who | Time |
|---|---|---|---|---|
| H1 | Run `spikes/2-frame-streaming/tools/run-all.sh` in a normal desktop session on **machine A: the dev machine** (NVIDIA RTX 4060 Ti, X11, WebKitGTK 2.52.6 present) | Real display, GPU | human | about 17 min, plus 3 minutes for a failure variant |
| H2 | Same on **machine B: a second Linux machine, preferably an integrated GPU laptop** (PRD 8.3 target; Wayland if available to cover the other display server) | Second machine, `libwebkit2gtk-4.1` runtime; a release binary built by the developer is copied there, or it is built there | human | about 17 min |
| H3 | Copy `results/` of each machine back, check `environment.json` contains no host or user names, commit through the developer | | human | few minutes |
| H4 (optional) | External latency check: film the screen with a phone at 240 fps while the spike flashes a square on every 30th frame together with a logged timestamp | Phone | human | 15 min |
| H5 (optional) | Measure the real frame cycle with `hantek6xx4-cli` on the 6254BC at 200 us/div and 2 ms/div, 1 and 4 channels, to replace the simulator's assumed readback and re-arm constants | 6254BC on the desk | human | 15 min |
| H6 | The hour-long roll soak (#4) is **not** human-run: developer-2 runs it in the background under Xvfb with an isolated `XDG_CONFIG_HOME`; never on the user's session | none | developer-2 | 1 h wall clock |

Xvfb runs for #2 verify the harness only; their numbers (software rendering) are marked invalid and are not committed as results.
Compatibility risk for H2: a binary built on this machine may need a newer glibc than machine B has; the fallback is building on machine B.

## 8. Risks and hardware limits
| Risk | Effect | Mitigation |
|---|---|---|
| `performance.now` coarsened by WebKit | Latency resolution above 1 ms | Report states resolution; use the Rust-side timestamps and sequence numbers for ordering; error bars |
| rAF capped at display refresh | Cannot show more than 60 fps on a 60 Hz panel | Rules compare against refresh rate; record it |
| GPU sync reads stall the pipeline | Distorts fps in sync runs | Reported separately; plain runs give fps |
| NVIDIA and WebKitGTK blank or crash (PRD 10) | WebGL2 and maybe the whole view unusable | Environment variants, screenshot, workaround ladder in 2.5; this is a result, not a failure of the spike |
| Spike unrepresentative of the product (no Svelte, no real layout) | Numbers slightly optimistic | Report states it; the product re-measures with its frame budget at the end of M1 |
| Driver cannot cancel a capture | Stop latency at slow time bases equals frame time | Allowed by ACQ-5; recorded as driver wish (ADR 0007) |
| Frame dead time is assumed | Simulator misleads UI demos on update rate | `unverified` constants and H5; PRD 5.2 says to publish the measured rate |
| One unit measured by the driver | Simulator mirrors one unit | Documented in crate docs |
| Roll gap size is an estimate | Marker width not exact | Report says so; marker uses `missed` as an estimate and carries `after` exactly |
| Hour-long soak on a loaded dev machine | Noisy memory data | Isolated session; record the machine load in `environment.json` |

PRD 5.2 limits applied: capture is frame based with dead time (simulator), 4096 records, channel-dependent sample rate, roll at 500 ms/div and slower,
overruns inferred from elapsed time, hotplug with settle time (catalog events), Linux only.

## 9. Dependencies
- Product: none new. `sweepscope-device` uses `serde` and `ts-rs`, which the kernel already uses; no `rand`, no async runtime.
- Spikes (branch only, not audited by CI): `tauri` 2.x (same line as the shell), `serde`, `serde_json`. No npm packages (plain ES modules, `withGlobalTauri`), no charting library, no `rand`. If a spike needs
  more, it asks first.

## 10. Rollout
- #3 merges through the normal gates (QA, review, G4).
- The report PRs carry only `docs/spikes/m0/`. After G4 of the reports: ADR 0003 accepted, ADR 0008 line added to `workflow.md` section 6, ADR 0006 and PRD DSP-7 and DSP-9 edited if the report says so, and the product frame and roll stream wire formats written as ADRs (staff engineer) before M1 starts.
- Architecture 11 open items touched later: "the stream pattern" and "WebGL2 and persistence".

## 11. Review and QA rulings
(empty; rounds are appended here, approved sections are not rewritten)

### Round 1 (#3 final, commit 30ef39c)
- Final public API of the simulator is exactly the `lib.rs` re-exports (section 2.2); `simulator::{edge,signal,timing}` are private. Spike #4 needs only `SimulatedDevice`, `SimulatorControl`, `SignalKind`.
- Verified flags follow PRD 5.1 for both time bases and vertical ranges (section 2.3).
- Catalog sends an initial `Attached`, handles die with re-enumeration, `Auto` sweep ignores the timeout (ADR 0007 consequences).
- ADR 0003 accepted; CI confirmation follows on the #3 pull request.

### Round 2 (QA review of the #2 harness at e8ef411, D3-1..7, D4-1..5)
Rulings by the staff engineer (rules in section 6 and the metric table in 2.5 are edited above; this lists what changes in code).
- **D4-1 (timer quantised to 1 ms): ruled.** The comparison figure is the mean `drawMs` in the sync runs with the absolute floor of 4 ms, never a ratio of p99 values. Frame rate and interval counts (over 20 ms, over 33.3 ms) are unaffected by the quantisation at that scale and stay the primary rules. Change for developer-4: add `mean` to `drawMs`; add the 1 ms histograms; `summarize.mjs` (developer-3) prints mean and the ratio with the floor applied.
- **D4-2 (ipcMs fails by construction): ruled.** A JavaScript receive stamp before the main thread is free does not exist (a Tauri channel is bound to the window, not to a worker). The transport rule is therefore evaluated on the ipc-only scenarios and on `seqGaps`; in render scenarios `ipcMs` is reported and labelled "includes main-thread queueing". Changes: developer-4 documents this in `web/` and the result JSON field comments; developer-3 labels the column in `summary.md` and the README. If the ipc-only p99 is above 16 ms on a real machine that is a real finding, not a harness defect.
- **D3-7 / D4-5 (per-sample series): ruled, ADR 0008 item 5 amended** (see the ADR): the harness stores summary statistics plus 1 ms histograms per timing metric, no CSV series. Percentiles, means and counts are reproducible from them, and a histogram shows stalls. The #4 `memory.csv` stays (it is a time series by nature).
- **D3-1: accepted, required before H1** (privacy): window-only capture (X11: `xdotool getactivewindow` or `xwininfo -id` with `import -window <id>`; Wayland: `grim -g` or `gnome-screenshot -w`), and the README tells the human to review any screenshot before copying it.
- **D3-2: accepted.** `--quick` (or `SPIKE_DURATION_S`) shortens runs; the duration is stamped in each JSON and `summarize.mjs` marks such runs INVALID for committed results; README says to run `--quick --scenarios gl2-plain` first (about a minute).
- **D3-3, D3-5: accepted** (README totals including build; what to do with a blank window, what Ctrl+C leaves behind; file name comments). **D3-4 (a to f): accepted** (validate arguments before running; no duplicate variant; unknown scenario ids and bare `--machine` fail with a message; non-zero exit when any variant crashed or timed out; clear the variant folder before re-running). **D3-6: accepted** (strip lspci flags and subsystem; whole-word matching in `strip()`).
- **D4-3: accepted** (probe the clock for a fixed 20 ms). **D4-4: accepted** (unknown scenario ids fail).
- Priority: D3-1, D3-2, D4-1, D4-2, D3-7/D4-5 before H1; the others in the same round if cheap, otherwise before H2.
