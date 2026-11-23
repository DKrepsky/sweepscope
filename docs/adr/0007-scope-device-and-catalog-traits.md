# ADR 0007: Two small device traits, `ScopeDevice` and `DeviceCatalog`, in the `device` crate, with blocking calls and a clock-driven simulator

Status: accepted
Date: 2026-11-22
Issue: #3 (also constrains #4)

## Context
PRD DEV-5 and architecture 4.2 rule 5 require the hardware behind a trait with a simulator. The only real
implementation, `hantek6xx4-lib`, is synchronous and blocking: `Scope::configure(&Config)`, `Scope::capture(opts) ->
Frame` (arm, poll the state every 20 ms, read back, up to a timeout), `start_roll`, `poll_roll` (returns
`Error::Overrun` from elapsed time, then resynchronises), `stop`, plus hotplug through `DeviceMonitor` events
(`Connected` after a 2 s settle time, `Disconnected`). Opening a device outside the monitor is forbidden (PRD 5.2).
The trait must be device neutral (no `hantek6xx4-lib` types), carry capabilities as data (Q-E3: the UI never knows the
scale tables), and be implementable by a simulator that runs on a fake clock in tests.

Open design questions, answered here:
1. Where does the trait live, given that `acquisition` must not import `device`?
2. Blocking or async?
3. How does a simulator reproduce "the same errors and events as the real adapter"?
4. Stop latency (ACQ-5) with a blocking capture.

## Decision
1. **Location.** `ScopeDevice`, `DeviceCatalog`, their value types and `SimulatedDevice` live in the feature crate
   `backend/features/device` (`sweepscope-device`), as architecture 4.2 rule 5 already says. `HantekDevice` joins it in
   the connection story; it will be the only user of `hantek6xx4-lib`. No feature depends on this crate. The device
   module (M1) owns the device thread, runs the capture loop and publishes `FrameCaptured` (to be listed in
   `kernel::contracts` then). `acquisition` gets frames by event and sends commands by request, so nothing outside
   `device` ever names `ScopeDevice`. M0 ships the traits and the simulator only, no module, no bus messages.
2. **Blocking, `&mut self`, `Send`.** USB is blocking and the device module owns a dedicated thread (architecture 4.6).
   No async runtime in this crate.
3. **Two traits (interface segregation).**
   - `ScopeDevice`: one opened instrument.
     `identity() -> &DeviceIdentity`, `capabilities() -> &Capabilities`,
     `configure(&AcquisitionSetup) -> Result<()>`, `capture(timeout: Duration) -> Result<Frame>`,
     `start_roll()`, `poll_roll() -> Result<Frame>`, `stop()`.
   - `DeviceCatalog`: what exists and when it appears or goes away.
     `events() -> Receiver<CatalogEvent>` (`Attached(DeviceId)`, `Detached(DeviceId)`) and
     `open(&DeviceId) -> Result<Box<dyn ScopeDevice>>`. The Hantek catalog wraps `DeviceMonitor` (settle time, open only
     after `Connected`); the simulated catalog is driven by `SimulatorControl`.
4. **One error type.** `DeviceError` = `Disconnected | NotFound | Timeout | Overrun | InvalidSetting(String) |
   CalibrationInvalid | Io(String)`, with `category() -> ErrorCategory` and `From<DeviceError> for KernelError`
   (`Disconnected` and `NotFound` map to `DeviceLost`; the rest to `Recoverable`). The adapter story maps the driver's
   `Error` into it one to one: `Disconnected` to `Disconnected`, `NotFound` to `NotFound`, `Timeout(_)` to `Timeout`,
   `Overrun` to `Overrun`, `InvalidParameter` to `InvalidSetting`, `BadCalibration` to `CalibrationInvalid`,
   `Usb` and `Protocol` to `Io`. A simulated unplug returns `Disconnected` from every call and emits
   `Detached`, as the driver does; an injected overrun makes `poll_roll` return `Overrun` and resynchronise.
5. **Settings are data, not driver types.** `AcquisitionSetup` refers to time base and vertical range by index into
   `capabilities().time_bases` and `capabilities().vertical_ranges` (ordered, with `verified` flags). Trigger level is in
   volts; the device converts to its own codes. Sample interval is derived from capabilities data
   (`adc_interval_ns` by enabled channel count, `samples_per_div`, formula `max(adc, time_base / samples_per_div)`), so
   the UI can show the effective rate before a capture (Q-E1).
6. **Frames.** `Frame` carries raw 8-bit codes per enabled channel, the data to convert to volts (`zero_code`,
   `counts_per_div`, `volts_per_div`), `sample_interval_s` and a trigger outcome (`Triggered { sample_index }`, `Auto`,
   `None` for roll). Roll chunks reuse `Frame` with `None`, as the driver does. Variable length: a roll chunk may be empty.
7. **Time is injected.** The simulator takes `Arc<dyn Clock>`. `Clock` gains `sleep(Duration)` (`SystemClock` sleeps,
   `FakeClock` advances), so capture time, dead time, roll cadence and overrun are testable without real waiting.
   Blocking waits in the simulator sleep in slices of at most the 20 ms poll interval and check the unplug flag, as the
   driver's poll loop does.
8. **Stop latency.** `capture` cannot be interrupted. The caller picks `timeout = max(250 ms, record duration x 1.5)`
   and checks its stop flag between captures, which is exactly ACQ-5 ("one frame time or 250 ms, whichever is larger").
   A `Timeout` in normal sweep means "no trigger yet", not an error to show. No cancel token until the driver can honour one.

## Alternatives considered
- **Trait in the kernel.** Lets `acquisition` hold a `Box<dyn ScopeDevice>`, but puts hardware vocabulary into the
  shared kernel that architecture 4.1 keeps feature-free, and makes the kernel change for every device quirk.
- **Async trait.** Wraps blocking USB in `spawn_blocking` for no gain, adds a runtime dependency to the crate.
- **Split-phase API (`arm`, `poll_state`, `read_frame`).** Would allow stop mid-capture, but the driver exposes only the
  composite `capture`; the raw requests are off limits. Revisit if the driver adds it (PRD open question 5).
- **Time base and range as SI floats.** Match-by-float on 1-2-5 values is brittle; indexes into published tables are not.
- **One trait with hotplug inside.** The real device can only be opened after the monitor says so, so discovery and
  the open instrument are different lifetimes.

## Consequences
- Adding a device means a new `ScopeDevice` and `DeviceCatalog` in `device`, plus its composition line; no feature changes.
- The simulator constants (frame dead time) are assumptions until measured on hardware; they are marked unverified.
- `Clock::sleep` touches the kernel and `FakeClock` (small, tested).
- A conformance test for any `ScopeDevice` is deliberately not built yet; the adapter story adds one when a second
  implementation exists.
- If the driver adds cancellation or split-phase capture, `capture` gains a sibling method; no breaking change.
- `DeviceCatalog::events()` first delivers `Attached` for every device already present, as the real hotplug monitor does
  for scopes plugged in when it starts; every subscriber gets this initial event, so consumers need no separate "list" call.
- A `ScopeDevice` handle dies with re-enumeration: after `unplug()` it stays `Disconnected` even when the device is plugged in
  again; the caller opens a new handle after the next `Attached` (PRD 5.2, the device returns with a new address).
- In `Sweep::Auto` the capture `timeout` is ignored: auto sweep always returns a frame within the record time (`TriggerOutcome::Auto`
  when no edge was found). Only `Sweep::Normal` returns `Timeout`.

## Amendment (2026-11-22, M1 planning, see ADR 0010 and ADR 0012)
- Decision 1 and 6: the data types `Capabilities`, `TimeBaseStep`, `VerticalRange`, `DeviceIdentity`, `CalibrationStatus`, `AcquisitionSetup`
  (with its parts), `Frame`, `ChannelData` and `TriggerOutcome` move to `kernel::contracts` because `acquisition` and `display` need them and features
  may not import each other. `device` re-exports them. The traits stay in `device`. `VerticalRange` gains `max_position_divs`.
- Decision 3: `ScopeDevice` gains `force_capture(timeout) -> Result<Frame, DeviceError>`.
- Decision 4: `DeviceError` gains `PermissionDenied` (category `Recoverable`).
- The "M1 device module" of decision 1 is split: `device` keeps the thread, sessions and capture loop; run intent and setup state belong to the new
  `acquisition` crate.
