# ADR 0010: Three instrument modules (`device`, `acquisition`, `display`) talking through a short list of kernel contracts

Status: accepted
Date: 2026-11-22
Issue: #5, #7, #8, #9, #10, #11, #12 (milestone M1); amends ADR 0007

## Context
M1 turns the M0 traits into a working instrument. Three things have to be decided that the architecture leaves open:

1. Who owns what. `docs/architecture.md` 3 lists `device`, `acquisition`, `channels`, `timebase-trigger` and `display` as separate feature
   crates "created by their stories". ADR 0007 says the `device` module owns the device thread and capture loop and publishes
   `FrameCaptured`, and that `acquisition` talks to it by event and request.
2. Features may not import each other (P2), yet `acquisition` needs the setup and frame types that ADR 0007 put in `device`, and `display`
   needs frames. Messages used by more than one module live in the kernel (4.2 rule 3).
3. The bus runs request handlers on the caller's thread (`Bus::request`), subscriptions are polled queues without blocking waits, and the
   driver applies a whole `Config` at once (there is no per-setting call).

## Decision
1. **Three feature crates for M1.**
   - `sweepscope-device`: connection state machine, sources (hardware catalog, simulator catalog), one worker thread per opened device that
     applies setups and runs the capture loop. It is the only crate that names `ScopeDevice`, `DeviceCatalog` and the driver.
   - `sweepscope-acquisition` (new): the domain state of the instrument: channel settings, time base, trigger, probe factors, run intent,
     the horizontal window geometry, availability of controls, instrument facts (sample rate, capture rate, dead time) and the status
     (armed, waiting, triggered, auto). It validates, restores the setup after a reconnect and publishes the state slices.
   - `sweepscope-display` (new): turns a captured frame into the wire frame of ADR 0011 (scaling to divisions, window crop, invert).
   `channels` and `timebase-trigger` are **not** separate crates in M1: the hardware applies one atomic setup, so splitting them would need a
   fourth party to assemble it. They are submodules of `acquisition`. Later features (pulse trigger, math channels) extend the submodules; if one
   outgrows the crate it is split then.
2. **Kernel contracts.** `kernel::contracts` gains the device-neutral data that more than one module needs, moved out of `device` without
   behaviour change: `Capabilities`, `TimeBaseStep`, `VerticalRange`, `DeviceIdentity`, `CalibrationStatus`, `AcquisitionSetup` (with
   `ChannelSetup`, `TriggerSetup`, `Coupling`, `Slope`, `Sweep`), `Frame`, `ChannelData`, `TriggerOutcome`. `device` re-exports them so its
   public names do not change. New cross-module messages, seven in total, listed in the implementation spec section 3.1: events
   `ConnectionChanged`, `CaptureCompleted`, `FrameCaptured`; requests `ApplySetup`, `SetCapturing`, `ForceTrigger`. The traits `ScopeDevice`
   and `DeviceCatalog` stay in `device` (ADR 0007 decision 1 stands for them).
3. **Frame path.** `device` worker publishes `CaptureCompleted { setup_revision, outcome }`; `acquisition` discards outcomes whose
   `setup_revision` is not the current one, updates status and rate, and publishes `FrameCaptured { frame, context }`; `display` subscribes
   latest-wins and publishes the encoded frame. A frame is never drawn with a scale it was not captured with: that is why the setup revision
   travels with every capture.
4. **Run intent and connection are different facts.** `acquisition` owns `running`. The device state machine has `Disconnected`,
   `Connecting`, `Ready` and `Error`. The `Acquiring` state of PRD DEV-1 and architecture 4.5 is the selector `ready && running` in the frontend
   (rule: derive, do not copy). `Settling` is not a device state in M1 (see ADR 0012).
5. **Module state and threads.** `Bus::request` runs the handler in the caller's thread, so each module keeps its state in one private
   `Mutex`, locks it briefly inside handlers and its own threads, and publishes events after unlocking. A lock is never held across a bus call
   or a device call. Handlers never block on hardware: they store the intent and wake the worker. This is the settings module's pattern,
   made explicit; architecture 4.6 ("not shared behind a mutex") is read as "not shared between modules".
6. **Kernel bus additions** (small, no dependency): `Subscription::recv_timeout(Duration) -> Option<E>` (blocking wait on a condition
   variable) and `Bus::subscribe_latest::<E>()` (capacity one, newest wins, `lagged()` counts replaced items: the latest-wins policy of
   architecture 4.3).
7. **Contracts are reviewed, not grown.** Adding an eighth cross-module message needs a line in the spec of its story and the same review.

## Alternatives considered
- **One `device` crate for connection, setup, run state and facts.** Fewest messages (only the frame crosses), but one crate would hold
  hardware access, domain rules and presentation facts, and every later feature that needs the setup (measurement, autoset) would depend on the
  device crate or need yet another contract. It also ignores ADR 0007's explicit module split.
- **Separate `channels`, `timebase-trigger` and `acquisition` crates now.** Matches the architecture list, but each would send a partial
  setup to a coordinator, and the coordinator is `acquisition` anyway.
- **`ScopeDevice` in the kernel so `acquisition` runs the loop.** Rejected in ADR 0007 (hardware vocabulary in the kernel); unchanged.
- **Actor threads with message inboxes for every module.** Cleaner ownership, but needs a multiplexed blocking inbox in the kernel
  (two event types and requests into one wait). The mutex pattern costs nothing new and matches the existing settings module.
- **Keep `Acquiring` as a device state.** Duplicates the run intent in two modules and lets them disagree for the duration of an in-flight
  capture.

## Consequences
- WP1 of the M1 spec is a mechanical move plus new types; `device` tests keep passing through the re-exports.
- Seven cross-module messages are in the kernel; ts-rs exports only those the frontend needs (`DeviceIdentity`, enums).
- The frontend derives `Acquiring`; the PRD state list is satisfied by the UI, not by one backend enum.
- A slow subscriber of `CaptureCompleted` (capacity 256, drop oldest) loses frames silently except for `lagged()`; acceptable because frames
  are latest-wins by nature, and the acquisition thread does constant work per frame.
- The first `display` and `acquisition` crates are created by this milestone, as the architecture asks.
