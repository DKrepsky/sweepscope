# ADR 0012: Hantek adapter, hotplug-driven sessions, one worker thread per opened device

Status: accepted
Date: 2026-11-22
Issue: #5, #6, #7, #11, #12; amends ADR 0007

## Context
`hantek6xx4-lib` 0.2.0 (published; contains `DeviceMonitor`) is blocking: `Scope::open_device(DeviceInfo)`, `configure(&Config)` (the whole
sequence of protocol.md 7.1, including relay pauses of 4 + 50 ms and 10 ms per channel position, so well over 100 ms), `capture(opts)` (arm, poll every 20 ms,
read back; not cancellable), `start_roll`, `poll_roll`, `stop`. The driver has no partial update, no force trigger, no state callback during a capture,
and no model name or firmware version in `ScopeInfo` (`name` is the family string `6000`, plus PCB revision, FPGA version, and `driver_version()` read from
the device). A scope appears twice after plug-in; the monitor reports `Connected` only after the 2 s settle time and never reports a scope that vanished
before that. Opening is only allowed after `Connected` (PRD 5.2). A permission failure surfaces as `Error::Usb(rusb::Error::Access)`. Opening during
the re-enumeration window can wedge the device.

## Decision
1. **Adapter.** `HantekCatalog` (wraps `DeviceMonitor::channel()`, forwards `Connected` and `Disconnected` as `CatalogEvent` to every subscriber and replays the
   currently attached set to a new subscriber, as the simulated catalog does) and `HantekDevice` (wraps `Scope<UsbTransport>`, generic over the driver's
   `Transport` so the mock can drive tests) live in `device/src/hantek/`. They are the only code that names `hantek6xx4-lib`. The `device` crate adds
   `hantek6xx4-lib = "0.2.0"` and `rusb = "0.9"` (the same version the driver uses; needed only to recognise `rusb::Error::Access`). Both stay out of
   every other crate.
2. **Tables are shared.** The capability tables (36 time bases, 12 ranges, verified flags from PRD 5.1) move from `simulator/profile.rs` to
   `device/src/profile.rs`, used by the simulator and the adapter; a test asserts that every entry equals the driver's `TimeBase::from_index` and
   `VoltsPerDiv::from_index` values.
3. **Conversions are pure functions with tests** (in `hantek/convert.rs`): setup to `Config`; vertical position in divisions to the driver position byte by inverting
   `zero_code_for_position` for the channel mode (clamped to 0 to 255, so the achievable span is about +-4 divisions and only about +-1.6 divisions on the
   50 mV, 500 mV and 5 V ranges: published as `VerticalRange::max_position_divs`); trigger level in volts to an ADC code with the zero model of protocol.md 9
   (clamped to 0 to 228); driver `Frame` to kernel `Frame` (`trigger_sample` becomes `Triggered { sample_index }`, an untriggered frame becomes `Auto`);
   driver `Error` to `DeviceError` as in ADR 0007 plus the new variant below.
4. **`DeviceError::PermissionDenied`** (category `Recoverable`) is added; the adapter maps `Error::Usb(rusb::Error::Access)` to it. Every other
   `Usb` or `Protocol` error maps to `Io`.
5. **Identity.** Model: `Hantek 6xx4 (family <name>)` with the PCB revision, because the device does not report a model and the USB id alone cannot tell a 6254BC
   from a 6254BD; the product text may say 6254BC only for the unit PRD 5.1 verified. Firmware: the device's `driver_version()` value labelled as such
   (the closest the device reports). FPGA version: `0x%04X` of `ScopeInfo::fpga_version`. Calibration: always `Valid` for an opened scope, because the driver
   refuses to open one whose position table is invalid (`BadCalibration` becomes `CalibrationInvalid`, a connection error).
6. **`ScopeDevice::force_capture(timeout)`** is added (required method). Simulator: one auto-style frame. Hantek: the driver has no force, so the adapter
   re-configures with `Sweep::Auto`, captures, and remembers `needs_restore`; the next `capture` re-applies the requested sweep first. Cost: one extra
   `configure` per force from normal sweep (not measured; the hardware checklist measures it). A native force in the driver is a change request, not a
   dependency.
7. **Session lifecycle.** `DeviceModule` owns a connection manager thread and, per opened device, one worker thread. `Attached(id)` while no session exists:
   state `Connecting`, `catalog.open(id)` on the manager thread, then `Ready` with identity and capabilities and a worker is spawned. `Detached(id)` of the
   active device or any `Disconnected` from the worker: state `Disconnected`, the worker is told to stop, joined with a 250 ms deadline (then detached; a
   capture cannot be cancelled), and the handle is dropped. `NotFound` or `Disconnected` while opening is not an error banner (the device went away): state
   `Disconnected`. Any other open failure: state `Error` with a kind (`PermissionDenied`, `CalibrationInvalid`, `Communication`); the next `Attached` or a
   `RetryConnection` request tries again. A device handle never survives re-enumeration (ADR 0007).
8. **Sources.** The module takes a hardware catalog and an optional simulator catalog with its `SimulatorControl`. The simulator starts unplugged in the
   product; `UseSimulator { enabled }` plugs it in or out. One device is active: a hardware `Attached` ends a simulator session and wins; the simulator
   is only opened when no hardware session exists. `SWEEPSCOPE_SIMULATOR=1` in the environment starts with the simulator plugged in (demos, e2e).
9. **Worker loop** (one thread, serial access to the handle):
   ```
   wait until stop | pending setup | capturing        (condition variable, no polling)
   take pending setup -> configure; remember its revision          (latest wins, coalesces bursts of changes)
   if capturing: capture(timeout) or force_capture; publish CaptureCompleted
        Ok(frame)         -> outcome Frame
        Err(Timeout)      -> outcome TimedOut           (normal sweep: no trigger yet)
        Err(Disconnected) -> report to the manager, end
        Err(other)        -> report to the manager (state Error), stop capturing, keep the session
   ```
   `timeout = max(250 ms, 1.5 x record duration)` (ADR 0007 decision 8). A capture that finishes after `SetCapturing { on: false }` is dropped by the worker.
   Setup changes are applied whether or not capturing, so a stopped scope shows the new setting at the next Run without delay.
10. **Dependencies and build.** The driver needs the system `libusb-1.0` development files (`libusb-1.0-0-dev` and `pkg-config` on Ubuntu); CI installs them in the
    `rust` and `shell` jobs. The driver's `vendored` feature is available through a passthrough feature `device/vendored`, not enabled by default (packaging, M4,
    decides). `env_logger` in the shell turns `RUST_LOG` into driver and adapter logs for hardware sessions.
11. **udev.** The repository ships `packaging/udev/99-hantek6xx4.rules`: the driver's rule plus `TAG+="uaccess"`, so the seat's user gets access without group
    membership. The connection error carries the exact command (built from the shipped file) and the doc path.

## Alternatives considered
- **Open the device inside the monitor callback.** The callback blocks further events and mixes policy into the driver thread; rejected.
- **A single long-lived worker for all sessions.** The handle dies with the session, and a leaked-thread test (20 cycles) is simpler with one thread per session.
- **Poll `UsbTransport::devices()` ourselves to show `Settling`.** Duplicates the monitor's settle logic and is a second code path that may open or list devices;
  rejected. The honest alternative is a driver event for "appeared, settling" (change request below); M1 shows `Disconnected` during the 2 s and `Connecting` while opening.
- **Detect permission errors from the message text.** Brittle; `rusb` as a dependency is cheap and exact.
- **Vendored libusb by default.** No system package needed, but statically links an LGPL library and changes packaging; deferred.
- **Raw protocol access for a native force trigger.** Forbidden by PRD 5.2 (no raw requests from the application).

## Consequences
- Driver change requests for the maintainer (not blocking M1): a force-trigger call, a partial trigger update (level drag then costs one command instead of a
  full `configure`), an `appeared` event for the settle period, `Error::PermissionDenied`, model and firmware strings.
- Each setting change costs one full `configure`; bursts coalesce, but a drag or wheel gesture feels as slow as the hardware (measured in the checklist, PRD 5.2).
- Stop is immediate for the user (the display stops taking frames) but the device finishes its capture first; the next Run waits for it. At slow time bases that is up to
  1.5 record durations.
- `device` grows by the adapter, the session module and `PermissionDenied`; its tests stay hardware-free (simulator, driver mock).
- Time bases of 500 ms/div and slower are frame captures of 8 s and more until roll mode (#27): see the implementation spec for the proposed M1 limit.
