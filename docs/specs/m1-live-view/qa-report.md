# QA report: M1 Live view (#5, #6, #7, #8, #9, #10, #11, #12; bugs #40, #41, #42)

Build or commit: `feat/m1-live-view` at b8e9048 (WP1 to WP12), plus uncommitted QA test code (see Artefacts).
Date: 2026-11-22
Plan: [test-plan.md](test-plan.md). Hardware items: [hardware-checklist.md](hardware-checklist.md) (not run by QA, human only).

## Summary
Verdict: **fail (not releasable)**. One high bug blocks release: the real app never receives any backend event, so none of the live behaviour of
stories #5 to #12 reaches the window (BUG-1). It was found only by running the real Tauri app; all automated suites are green because the browser
stub accepts the call the real capability refuses. With the one-line capability fix applied to a scratch copy (not to the repository), the real app
passed the live flow on the simulator, so the fix is expected to be small. Three more acceptance criteria are not met (BUG-2, BUG-3, BUG-4).

What blocks release: BUG-1 (high); BUG-2 and BUG-3 (medium, acceptance criteria of #8 and #10); BUG-4 (low, #6 AC2). Visual baselines must be
regenerated on CI after the push (see below). The human hardware checklist is still to be run.

Safety incident to know about: the first start of the real app opened the real scope. `SWEEPSCOPE_SIMULATOR=1` only starts the simulator plugged in;
`HantekCatalog` is always composed and the 6254BC is plugged into this machine, so the log showed `hantek6xx4_lib::scope initialised scope ... pcb=P1.06
fpga=0xd001`. I killed the app at once (about 15 s). No command beyond the driver's normal open was sent, no `--ignored` test was run, and every later
run of the app used a private mount namespace that hides `/dev/bus/usb` and `/sys/bus/usb/devices`. Replug the scope or restart it if it behaves oddly.
Recommendation to the staff engineer: add an env flag (for example `SWEEPSCOPE_NO_HARDWARE=1`) that leaves out the Hantek catalog, so test runs can never touch a connected scope.

## Results

### Gate checks
| ID | Result | Notes |
|---|---|---|
| G-1 | Pass | `cargo test --workspace`: 471 passed, 0 failed, 3 ignored (the hardware smoke tests, not run). Device, acquisition, display and kernel crates repeated 15 times: no failure, no flaky test. |
| G-2 | Pass | `npm run check` (fmt, clippy -D warnings, eslint, stylelint, prettier, svelte-check, architecture, test names) clean; `test:tools` 42 pass; vitest 595 pass (51 files). |
| G-3 | Pass with known failures | Playwright chromium and webkit: 361 passed, 13 skipped, 12 failed. All 12 failures are the stale visual baselines (8 `visual.spec.ts`, 4 `graph.spec.ts` scope-screen). No other regression. |

### #5 connection and identity
| ID | Result | Notes |
|---|---|---|
| C5-1 | Pass (simulator) | Real app: status bar `Connected`, `Simulator`, `6254BC (simulated)` immediately at start. Hardware part: H-03, H-07. |
| C5-2 | Pass | e2e `live.spec.ts`: identity and simulated marker in the status bar. Note: model, FW, FPGA and calibration show in the chip and its tooltip; not all four are visible text (by design 1.4). |
| C5-3 | Pass (e2e) | Disconnected banner with `Retry` and `Use simulator`; both send their commands. In the real app `use_simulator(false)` then `(true)` reconnects. Cold start with no simulator not run in the real app (a connected scope cannot be avoided safely: see incident). |
| C5-4 | Pass | Communication error: banner and `Error` state, window stays usable (e2e). |
| C5-5 | Pass | `given_20_unplug_and_replug_cycles...` and `module_hotplug_threads` green; additionally 20 cycles through `use_simulator` in the real app: no panic, thread count 37 before and after, state `Ready` and running at the end. |
| C5-6 | Not tested | Needs the scope: H-04. |
| C5-7 | Pass | Fault injection tests (`simulator_faults`, `module_capture`) green. |

### #6 USB permission help
| ID | Result | Notes |
|---|---|---|
| C6-1 | Pass (e2e) | Banner names the rule file and message, shows `Copy command` and `Retry`. Real `PermissionDenied` on hardware: H-05. |
| C6-2 | **Fail (BUG-4, low)** | Clipboard holds the exact command (Chromium); the control gives no confirmation. WebKit clipboard not testable in Playwright. |
| C6-3 | Pass | `Retry` sends `retry_connection`; backend `given_a_session_that_failed_when_retry_arrives...` green. Real replug after the rule: H-06. |
| C6-4 | Pass | `packaging/udev/99-hantek6xx4.rules` (vendor 04b5, product 6cde, `plugdev`, `uaccess`) and `docs/usb-permissions.md` exist; the Banner command copies the shipped file. |

### #7 run and stop
| ID | Result | Notes |
|---|---|---|
| C7-1 | **Fail in the unpatched real app (BUG-1)**, pass with the fix applied | Unpatched: Run starts the backend and a frame arrives, but the toggle stays `Run`, chip `Stop`. Patched copy: toggle `Stop`, chips `Run` and `Trig'd`, `Armed`, `Waiting for trigger`, `Auto` all seen. |
| C7-2 | Pass (patched), unit | Stop freezes the last frame, toggle `Run`, chip `Stop`. Backend stop timing in `module_capture` green; the 250 ms bound on the device is hardware (H-10). |
| C7-3 | Partly | Clicks stayed responsive during runs; the slow read case only exists on hardware (H-15). |
| C7-4 | Pass (patched) | Status bar `Capture 23.7 Hz dead time 92.2 %`, then 23.1 and 23.6 Hz after setting changes: live, not constant. Hardware rate: H-14. |
| C7-5 | Pass | `given_a_device_that_failed_mid_capture...` green; error banner in e2e. |

### #8 live waveform
| ID | Result | Notes |
|---|---|---|
| C8-1 | Not valid here | The probe runs in the dev window and prints `frame budget {frames, meanIntervalMs, maxIntervalMs}` every 10 s. Under Xvfb with software rendering it printed mean 41 ms, max 72 ms; software numbers are not valid and are not kept. Needs the human on a real GPU (H-41). |
| C8-2 | Pass | `live.spec.ts`: a one-sample 3.5 division glitch in a 2501-sample frame adds visible pixels; display crate pins the crop. |
| C8-3 | **Fail (BUG-2, medium)** | Canvas 2D only and the status bar reads `Renderer Canvas 2D` (pass). The DMABUF workaround is not documented outside the PRD, the spike specs and the checklist (README and docs have no mention). |
| C8-4 | Pass | 10 x 8 graticule, per channel scale chips, readable at 100 to 200 % text scale in e2e and in the WebKitGTK window at 100 %. HiDPI scale snapshots are blocked by the stale baselines (graticule line checks pass). |
| C8-5 | Pass | Frames on CH1 and CH3 draw only those channels; disabled channels draw nothing. |
| C8-6 | Pass (e2e, patched app) | Disconnect: trace stays, dimmed, state `frozen`, chip `Offline`, status `Disconnected`, banner. Real app on `use_simulator(false)`: all of it within about 1 s of the command (screenshot taken 1.2 s later). Return restores `live`. |

### #9 vertical
| ID | Result | Notes |
|---|---|---|
| C9-1 | Pass (patched app, unit) | CH2 stepped 1 V to 500 mV/div: trace doubled, label updated. 12 ranges reachable in the backend tests. |
| C9-2 | Pass | Probe 10x: label `5 V/div` for 500 mV; backend test pins the multiply. |
| C9-3 | Pass (patched app) | CH2 enabled from the row switch: chip and trace appear (CH2 initially hidden under CH1 because the simulator gives every channel the same sine, not a bug). Disabled channels are absent from frames (backend and e2e). AC/DC, BW, invert wiring verified by e2e command assertions and vitest, not by eye in the real app. |
| C9-4 | Pass | Distinct colours, labels, enable switches (e2e, screenshots). |
| C9-5 | Pass (unit) | Backend `unverified` text and the `?` badge; not stepped to 2 mV in the real app. |
| C9-6 | Pass (unit) | Last channel cannot be disabled (`domain_instrument`). |
| C9-7 | Pass (unit) | Position clamps (`given_a_position_beyond_the_limit...`); marker tooltip not checked by eye. |

### #10 horizontal
| ID | Result | Notes |
|---|---|---|
| C10-1 | Pass (patched app) | 8 steps from 200 us to 100 ms/div, the 9th reached 200 ms/div, and the stepper `+` is then disabled with tooltip "Roll mode arrives in a later release". The 2 ns end is covered by backend tests. |
| C10-2 | Pass (unit, e2e) | `set_trigger_position` with the fraction; marker drag emits once on release. 10, 50, 90 % in the real app not driven. |
| C10-3 | **Fail (BUG-3, medium)** | At 100 ms/div (index 23, unmeasured) no marker is shown anywhere. The backend marks it unverified; the frontend model has no field for it. |
| C10-4 | Pass (patched app) | `Fs`, `Record 4.10 kpts 16.4 div` change with the time base (`Fs 2.50 kS/s` at 100 ms/div, `1.25 kS/s` at 200 ms/div; 2 channel count effect only visible at fast bases, not driven). |

### #11 trigger
| ID | Result | Notes |
|---|---|---|
| C11-1 | Pass (patched app) | Dragging the right trigger marker from 0 V to the top: a ghost follows the pointer, one command on release, label `T1 rising 3.74 V` and the level field read 3.74 V. The tag keeps the confirmed value during the drag (design 8.6). |
| C11-2 | Pass (patched app) | Normal sweep, level above the signal: chip `Waiting for trigger`, the last frame stays (0 differing pixels over 3 s). |
| C11-3 | Pass (patched app) | Auto sweep, same level: chip `Auto`, frames keep changing. |
| C11-4 | Pass (patched app) | Force in the waiting state produced exactly one new frame (5257 pixels changed, then 0). |
| C11-5 | Pass (unit, e2e) | `set_trigger` carries source and slope; disabled-channel source rejected and source moves on disable (domain tests). |
| C11-6 | Pass | Single, Autoset, sin(x)/x, Pulse are disabled with the later-release reason. |

### #12 unplug and replug
| ID | Result | Notes |
|---|---|---|
| C12-1 | Pass (patched app, unit) | Freeze with `Disconnected` within about 1 s, no crash; real USB unplug: H-11. |
| C12-2 | Pass (patched app, unit) | After `use_simulator(true)`: reconnects, same level (3.74 V), sweep and time base, running resumed without user action (D4). |
| C12-3 | Unit only | No hang in the simulator; real power-up re-enumeration: H-13. |
| C12-4 | Pass (sim) | 20 cycles in the real app: no hang or crash, 37 threads before and after. Hardware: H-13. |
| C12-5 | Pass (unit) | Stopped stays stopped. |

### Absorbed bugs
| ID | Result | Notes |
|---|---|---|
| B40-1 | Pass | e2e `channel-rows.spec.ts` green in both browsers; the WebKitGTK window shows aligned columns and a disabled row border equal to the others. Close #40 after BUG-1 is fixed (it is verified, not blocked). |
| B41-1 | Pass | `marker-a11y.spec.ts` green in both browsers (drag and Alt+Arrow). The real window was not checked with a screen reader. |
| B42-1 | Pass at 100 % | In the WebKitGTK window the four text-size options are one line each. At 200 % text scale in a 1440 px window the Settings popover is cut off by the window edge (observation O-1). 125 % and 150 % not checked in the real window. |

### Cross-cutting
| ID | Result | Notes |
|---|---|---|
| X-1 | Pass | `tiers.spec.ts` (all calibrated thresholds, both browsers) green with live content. |
| X-2 | Pass | Tab order and focus outline tests green; disabled controls show their reason as tooltip (seen for the roll steps). |
| X-3 | Partly | axe on the main window, popovers and gallery green; axe with a banner showing was not run. |
| X-4 | See below | Baselines not regenerated. |
| X-5 | Pass | Isolated HOME, no panic in the log. |
| X-6 | Pass | CSP preview tests green (5). The real window's IPC under the dev CSP works for commands and the frame channel. |

## Bugs
| Bug | Severity | Status | Owner |
|---|---|---|---|
| BUG-1 main capability lacks `core:event:allow-listen`: no backend event reaches the real app (evidence `.claude/tmp/m1/vite-event-listen.log`, `app-bug-events.png`, `app-p4.png` after the local fix) | High, blocks release | Open. Hypothesis: add `core:event:allow-listen` and `core:event:allow-unlisten` to `src-tauri/capabilities/main.json`; verified on a scratch copy only | developer-3 |
| BUG-2 DMABUF workaround not documented (#8 AC3) | Medium | Open | developer-3 |
| BUG-3 no unverified marker for an unmeasured time base (#10 AC3) | Medium | Open | developer-8 (cc developer-6) |
| BUG-4 Copy command has no confirmation (#6 AC2); e2e carries `test.fail` | Low | Open | developer-6 |
| #40, #41, #42 | n/a | Verified fixed; PM can close after the release | |

Observations for the PM (not caused by this change, no bug sent): O-1 at 200 % text scale in a 1440 x 900 window the Settings popover is clipped
by the window edge; the layout scrolls horizontally there. O-2 the sidebar, cursor and measurement panels and the "Illustrative numbers" note still
show mock data by design. O-3 the app starts stopped (D5) so the first view is empty until Run is pressed.

## Visual baselines (X-4)
Not regenerated, and I recommend nobody replaces them with a local render. The memory rule is that baselines must come from the ubuntu-24.04 CI
runner; that needs a pushed commit with a temporary workflow change, which was not allowed here (no commit, no push, and the branch is not on the
remote). Local runs differ from CI by host fonts: the gallery `Tokens` section differs only in glyph rendering (nothing in the gallery was changed except the Marker and Segmented fixes), so the gallery baselines are probably not stale but cannot be judged on this host.
Judgement of the main-window changes from the local renders: intended. The live graph is empty without frames (the mock traces and cursor overlays are gone), the dock shows
the stub snapshot (CH1 to CH3 on, CH4 off), the status bar shows `Connected`, `Simulator`, identity, `Fs -`, `Capture - -` and `Renderer Canvas 2D`, and the toolbar shows the live chips.
`graph.spec.ts` scope-screen snapshots now push a deterministic three-channel frame first, so they still cover trace drawing; their baselines also need the CI regeneration.
Procedure for the PM after the push: temporary workflow change to run `npm --prefix frontend run test:e2e -- --project=chromium e2e/visual.spec.ts e2e/graph.spec.ts --update-snapshots all --retries=0`,
upload `frontend/e2e/__screenshots__/chromium/`, download, copy, revert the workflow.

## Not tested and why
- Everything on the real 6254BC (checklist H-01 to H-42, `--ignored` tests): human only. In particular: real `PermissionDenied`, real replug timing, 20 real plug-ins, power-up re-enumeration, force latency, real capture rate, real trigger position accuracy, the unverified range accuracy.
- #8 AC1 frame budget (33 ms) on a real GPU: only a software-rendered dry run. The probe works.
- Real window at 125 % and 150 %, HiDPI real window, Wayland, the compact tier in the real window.
- Banner states (permission, error) in the real window: they need a real or faked backend error; covered in e2e only.
- Backend ranges at 2 mV and 10 V in the real window; trigger position 10, 50, 90 % in the real window.
- axe with banners; keyboard use of the steppers in the real window.
- The fixes for BUG-1 to BUG-4 (retest after the developers answer).

## Screenshots and logs
Kept in `.claude/tmp/m1/` (not committed): `cargo-test.log`, `pw-full.log`, `pw-full2.log`, `app-bug-events.png`, `vite-event-listen.log`, `app-patched.log`, `app-*.png`.

## Artefacts added or changed by QA (uncommitted)
`docs/specs/m1-live-view/test-plan.md`, this report, `frontend/e2e/live.spec.ts` (13 cases), `frontend/e2e/support/frames.ts`, `frontend/e2e/support/tauri-stub.ts`
(event emission, frame channel, callback registry; `emit` and `sendFrame` helpers), `frontend/e2e/graph.spec.ts` (frame before the snapshot).
