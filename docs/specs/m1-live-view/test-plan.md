# Test plan: M1 Live view (#5, #6, #7, #8, #9, #10, #11, #12; bugs #40, #41, #42)

Spec: [implementation-spec.md](implementation-spec.md) (section 5 and the rulings in 1.4). Hardware items: [hardware-checklist.md](hardware-checklist.md).
Build under test: branch `feat/m1-live-view` at b8e9048 (WP1 to WP12 merged).

## Scope and risks
In: connection and identity, USB permission help, run/stop and status, live waveform and graticule, vertical, horizontal and trigger controls, unplug and replug,
the three absorbed bugs. Out: anything that needs the real 6254BC (the human runs the checklist; agents never open the scope and never run the `--ignored` tests).

Risks driving the plan: (1) the e2e stub used to be static, so live states (frames, events, disconnect) were never exercised in a browser; (2) the Tauri window is
WebKitGTK, which Playwright WebKit only approximates (bug #42); (3) timing claims (1 s freeze, 5 s ready, 250 ms stop) are only checkable on the simulator;
(4) visual snapshot tests were dropped (no baselines to maintain).

Kinds: `unit` = cargo test or vitest (already written by the developers, re-run and spot-checked); `e2e` = Playwright against the stubbed Tauri bridge;
`app` = the real Tauri binary against the backend simulator (`SWEEPSCOPE_SIMULATOR=1`) under Xvfb with an isolated HOME; `UI` = visual check; `HW` = needs the scope, human only.

## Test cases
### Gate checks (whole product, regression)
| ID | Given / when / then | Kind | Covers | HW |
|---|---|---|---|---|
| G-1 | Given the branch, when `cargo test --workspace` runs (and the device, acquisition, display and kernel crates 15 more times), then everything passes, no flaky test | unit | all | no |
| G-2 | Given the branch, when `npm run check`, `npm run test:tools` and vitest run, then all clean | unit | DoD | no |
| G-3 | Given the full Playwright suite (chromium and webkit), when it runs, then no regression outside the stale snapshots | e2e | regression | no |

### #5 connection and identity
| ID | Given / when / then | Kind | Covers | HW |
|---|---|---|---|---|
| C5-1 | Given the simulator plugged at start, when the app starts, then status bar reads `Ready`/`Simulated` within 5 s, with model, FW, FPGA, calibration visible (item and tooltip) | app | AC1, AC2 | H-03, H-07 |
| C5-2 | Given a ready snapshot with identity, when the status bar renders, then model, firmware, FPGA, calibration are all present (stub) | e2e | AC2 | no |
| C5-3 | Given no scope and no simulator flag, when the app starts, then `Disconnected`, the hint "Connect a scope or use the simulator" and a `Use simulator` action show; pressing it connects the simulator | app, e2e | AC3 | H-01 |
| C5-4 | Given `ConnectionState::Error` with a communication issue, when rendered, then an error state is visible and the app keeps running | e2e | AC5 | no |
| C5-5 | Given 20 simulated unplug and replug cycles, when finished, then `Ready`, no hang, thread count at baseline (cargo `module_hotplug_threads`) | unit | AC4 (sim part) | H-04 |
| C5-6 | Given the real scope re-enumerating, when plugged in 20 times, then no wedge | HW | AC4 | H-04 |
| C5-7 | Given a driver error mid-session in the simulator (fault injection tests), when it occurs, then `Error` or `Disconnected` and no panic | unit | AC5 | no |

### #6 USB permission help
| ID | Given / when / then | Kind | Covers | HW |
|---|---|---|---|---|
| C6-1 | Given a snapshot with a `PermissionDenied` issue and help, when rendered, then an error Banner names the rule file and shows the install command, with `Copy command` and `Retry` | e2e | AC1 | H-05 |
| C6-2 | Given the Banner, when `Copy command` is pressed, then the clipboard holds the command and the control confirms (label changes) | e2e | AC2 | H-06 |
| C6-3 | Given the Banner, when `Retry` is pressed, then `retry_connection` is invoked; backend retry after failure reconnects without restart (cargo test) | e2e, unit | AC3 | H-06 |
| C6-4 | Given the repository, when inspected, then `packaging/udev/99-hantek6xx4.rules` and `docs/usb-permissions.md` exist, the install command in the Banner refers to the shipped file and the rule matches vendor 04b5 product 6cde | manual | AC4 | no |

### #7 run and stop
| ID | Given / when / then | Kind | Covers | HW |
|---|---|---|---|---|
| C7-1 | Given ready, when Run is pressed (app: real simulator), then the toggle reads `Stop` and the phase chip changes among armed, triggered, auto, waiting | app, e2e | AC1 | H-10 |
| C7-2 | Given running, when Stop is pressed, then `set_running(false)`, toggle reads `Run`, chip `Stop`, the last frame stays; the device stops within 250 ms on the simulator (cargo tests) | app, e2e, unit | AC2 | H-10 |
| C7-3 | Given a slow device read, when the UI is used, then it stays responsive (acquisition on its own thread; click latency measured in the app run) | app | AC3 | H-15 |
| C7-4 | Given running, when the status bar is read twice, then capture rate and dead time are live numbers, not constants (differ with time base and channel count) | app | AC4 | H-14 |
| C7-5 | Given a driver error mid-capture, when it occurs, then running becomes false, the state is visible, no crash | unit, e2e | AC5 | no |

### #8 live waveform
| ID | Given / when / then | Kind | Covers | HW |
|---|---|---|---|---|
| C8-1 | Given frames at 60 per second, when the app draws them, then no animation-frame interval exceeds 33.3 ms (frame-budget probe, dev build, simulator) | app | AC1 | H-41 |
| C8-2 | Given a one-sample glitch in a wide record, when drawn, then it is visible (graph canvas pixel test and the display crate test) | e2e, unit | AC2 | no |
| C8-3 | Given the app, when it starts, then it uses Canvas 2D only (D2), status shows `Canvas 2D`, docs describe the DMABUF workaround | UI, manual | AC3 | H-42 |
| C8-4 | Given the screen, when inspected at 1080p and HiDPI and text scales 100 to 200 %, then 10 x 8 graticule and per channel scale labels are readable | UI, e2e | AC4 | H-40 |
| C8-5 | Given a decoded golden frame (fixture `frame-v1.bin`), when fed through the stub frame stream, then the graph draws the trace on the right channel colour and nothing is drawn for disabled channels | e2e | AC1 | no |
| C8-6 | Given a connected scope, when it disconnects, then the last frame stays (frozen, dimmed) with a visible `Disconnected` state | e2e, app | AC5 | H-11 |

### #9 vertical
| ID | Given / when / then | Kind | Covers | HW |
|---|---|---|---|---|
| C9-1 | Given a channel, when the V/div stepper is used (button, wheel, keyboard), then `step_setting` carries the range target and direction, labels update from the next state event; 12 ranges reachable by stepping and reset (cargo + app) | e2e, app, unit | AC1 | H-21 |
| C9-2 | Given probe 10x, when set, then the label is multiplied by 10 and the scale labels follow (backend view tests, app) | unit, app | AC2 | H-22 |
| C9-3 | Given a channel, when enabled, disabled, inverted, AC/DC, BW limit toggled, then the matching `set_channel` patch is sent and the row reflects the state event; a disabled channel is absent from the frame | e2e, app | AC3 | H-23, H-24, H-25 |
| C9-4 | Given each channel, when inspected, then it has its own colour, label and enable switch | e2e, UI | AC4 | H-27 |
| C9-5 | Given a range outside 50 mV to 1 V per div, when selected, then the unverified marker with reason text is shown | e2e, app | AC5 | H-21 |
| C9-6 | Given the last enabled channel, when disabled, then it is refused with the reason | e2e, app | rule | no |
| C9-7 | Given the ground marker dragged beyond the limit on a 50 mV range, when released, then it clamps near 1.6 div with a tooltip | e2e, unit | rule | H-26 |

### #10 horizontal
| ID | Given / when / then | Kind | Covers | HW |
|---|---|---|---|---|
| C10-1 | Given the time base stepper, when stepped from 2 ns to 200 ms/div, then 25 steps are reachable and the axis labels update; the 11 steps from 500 ms/div are disabled with the reason "Roll mode arrives in a later release" | unit, app | AC1 | H-35 |
| C10-2 | Given the trigger position marker, when dragged or set to 10, 50, 90 %, then `set_trigger_position` carries the fraction and the tick follows the next event | e2e, app | AC2 | H-36 |
| C10-3 | Given an unmeasured time base step, when selected, then the `Unverified` marker shows | unit, app | AC3 | H-35 |
| C10-4 | Given the time base, when the status bar is read, then effective sample rate and record length are shown and change with the time base and channel count | app | AC4 | H-38 |

### #11 trigger
| ID | Given / when / then | Kind | Covers | HW |
|---|---|---|---|---|
| C11-1 | Given a repeating signal, when the level marker is dragged and released, then one `set_trigger_level` is sent, a ghost shows during the drag, the marker tag shows level and slope, the simulator display stabilises | e2e, app | AC1 | H-30 |
| C11-2 | Given normal sweep and a level above the signal, when frames are awaited, then the last frame stays and the chip reads `Waiting for trigger` | app, unit | AC2 | H-32 |
| C11-3 | Given auto sweep and no event, when the timeout passes, then the chip reads `Auto` and frames keep coming | app, unit | AC3 | H-31 |
| C11-4 | Given running, when Force is pressed, then one frame is captured immediately (simulator) and Force is disabled while stopped | app, unit | AC4 | H-33 |
| C11-5 | Given the source and slope controls, when changed, then `set_trigger` carries them; a disabled channel cannot be the source and disabling the source moves it | e2e, unit | AC5 | H-34 |
| C11-6 | Given Single, Autoset, sin(x)/x, Pulse trigger, when inspected, then disabled with "Arrives in a later release" (out of scope controls) | e2e | rule | no |

### #12 unplug and replug
| ID | Given / when / then | Kind | Covers | HW |
|---|---|---|---|---|
| C12-1 | Given running, when the simulator is unplugged, then `Disconnected` within 1 s, frozen last frame, no crash (cargo and e2e; app via the simulator control if reachable) | unit, e2e | AC1 | H-11 |
| C12-2 | Given a replug, when the settle time passes, then the setup (channels, scales, time base, trigger) is restored and running resumes (D4) without user action | unit | AC2 | H-12 |
| C12-3 | Given an unplug during power-up re-enumeration, when it occurs, then no hang | unit, HW | AC3 | H-13 |
| C12-4 | Given 20 cycles, when finished, then no hang, no crash, no leaked thread | unit, HW | AC4 | H-13 |
| C12-5 | Given Stopped and an unplug and replug, when reconnected, then it stays stopped | unit | D4 | H-16 |

### Absorbed bugs
| ID | Given / when / then | Kind | Covers | HW |
|---|---|---|---|---|
| B40-1 | Given four channel rows at 1440 x 900, when measured, then equal column widths and equal border of a disabled row (e2e `channel-rows.spec.ts`, both browsers, plus the Tauri window) | e2e, UI | #40 | no |
| B41-1 | Given a marker, when dragged or moved with Alt+Arrow, then `aria-valuenow` follows during and after (e2e `marker-a11y.spec.ts`) | e2e | #41 | no |
| B42-1 | Given the Settings popover in the real Tauri window (WebKitGTK), when opened at 100, 125, 150, 200 %, then every text-size option is on one line (22 px high) | app, UI | #42 (C-5) | no |

### Cross-cutting
| ID | Given / when / then | Kind | Covers |
|---|---|---|---|
| X-1 | Given the main window at all calibrated tiers, themes, text scales and the 1024 x 720 minimum, when rendered live, then no clipping or overlap in the dock, status bar and banners | e2e, UI |
| X-2 | Given keyboard only, when tabbing through the toolbar and dock, then focus is visible, disabled controls announce their reason (tooltip or aria) | e2e, UI |
| X-3 | Given axe on the live main window with banners, when run, then no violations | e2e |
| X-4 | Dropped: visual snapshot tests were removed; the main window is checked by layout, bounding-box and token tests | n/a |
| X-5 | Given a fresh HOME, when the app starts and exits, then no config residue besides settings and no error in the log | app |
| X-6 | Given the CSP of the production bundle, when loaded, then no console errors, frame stream and events work under the CSP (`ipc:`) | e2e, app |

## Environment
Simulated device (backend simulator and the stub bridge, extended by QA with event emission and the golden frame fixture); Chromium and WebKit via Playwright 1.65;
real Tauri debug binary under Xvfb `:77` with openbox, isolated HOME, Mesa software rendering, `WEBKIT_DISABLE_DMABUF_RENDERER=1`. Linux (kernel 7.0). No 6254BC is touched.

## Entry and exit criteria
Entry: WP1 to WP12 on `feat/m1-live-view`, clean build. Exit: every case has a result in the QA report; no open high severity bug; hardware items listed as not tested and handed to the human.

## Retest round 1 (e065a37)
Added to the plan, all executed: R-1 BUG-1 in the real window with `SWEEPSCOPE_SIMULATOR=1`, `SWEEPSCOPE_NO_HARDWARE=1` and the USB-hiding namespace (app, no scope); R-2 live flow: Run, Stop, V/div and time base steppers, trigger sweep and level (app); R-3 unplug and replug through `use_simulator`, then 20 cycles with the thread count (app); R-4 C10-3 at 100 ms/div (app, now covers the badge placement); R-5 C6-2 Copy confirmation (e2e); R-6 permission-help command text, dry check only: syntax with `bash -n` and `sh -n`, `printf` part into a scratch file compared byte for byte with the shipped rule, `udevadm verify` (manual, no scope, never installs); R-7 docs for BUG-2 and the no-hardware flag (manual); R-8 full suites run alone: cargo, `npm run check`, vitest, Playwright. Results are in the QA report.
