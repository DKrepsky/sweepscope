# Test plan: M2 Measure (#13 #14 #15 #18 #19 #20)

Spec: [implementation-spec.md](implementation-spec.md) (sections 5 and 9), [hardware-checklist.md](hardware-checklist.md). Branch `feat/m2-measure`.

## Scope and risks
In scope: interpolation (#13), cursors (#14), measurements (#15), CSV and raw export (#18), screenshot (#19), setups and last-setup restore (#20), and the shell wiring (commands, events, capability, dialog).
Out of scope: statistics, zoom and pan, offline captures, right-click menu (D6).

Risks driving the plan:
- Capability gaps that browser stubs hide (M1 BUG-1 type): new commands, `display_changed` and `measurements_changed` events, native file dialog. Checked in the real window.
- R2: frequency quantisation. The numeric `value` of the simulator calibrator is read against 0.01 % and the real figure reported.
- Four-digit frequency and period ruling (section 9, round 1): `1.000 kHz` must show.
- Cursor tags can overlap (known minor).
- Files: nothing half-written, no path leaks in toasts.
- Stray folders (`.claude/frontend`) after test runs.

Guards for every real-app run: `SWEEPSCOPE_NO_HARDWARE=1`, USB hidden (`unshare -rm` with tmpfs over `/dev/bus/usb` and `/sys/bus/usb/devices`), isolated HOME, Xvfb, Mesa forced. The real 6254BC is never opened; hardware items H-01 to H-10 stay with the human.

## Test cases
Kinds: A = automated (cargo, vitest, Playwright), E = Playwright e2e added by QA in `frontend/e2e/m2-measure.spec.ts`, R = real Tauri window under Xvfb (manual), U = UI check. HW = needs the real scope (none run by QA).

| ID | Given / when / then | Kind | Criterion | HW |
|---|---|---|---|---|
| T-01 | Given the whole repo, when `cargo test --workspace`, vitest, `npm run check`, `npm run test:tools` run, then all pass | A | all | no |
| T-02 | Given the full Playwright suite on spare ports (chromium and webkit), when run, then no regression | A | all | no |
| T-03 | Given the real window, when it starts, then no capability error appears in the dev log and `get_snapshot` returns display and measurements | R | shell | no |
| T-04 | Given the real window, when `set_interpolation`, `export_frame`, `save_setup`, `load_setup`, `save_screenshot` are invoked from devtools, then none is refused by the capability | R | shell | no |
| T-05 | Given the real window, when listening to `display_changed` and `measurements_changed`, then events arrive (readout changes) with no "not allowed" error | R | shell | no |
| T-06 | Given the 20 ns/div to 100 ns/div bases on the simulator, when the window starts, then Horizontal shows sin(x)/x (automatic) and Linear and Dots can be chosen | R, E | #13 AC1 | no |
| T-07 | Given dots mode, when drawn, then only points show (pixel count lower than linear, no connected runs) | R, U | #13 AC3 | no |
| T-08 | Given interpolation changed while running, when picked, then it applies without a restart (acquisition keeps running, `set_interpolation` sent once) | E, R | #13 AC4 | no |
| T-09 | Given the simulator calibrator, when measured, then rise time reads 240 to 256 ns at a base where the edge is resolved (backend tests cargo) | A | #13 AC2 | HW H-03 |
| T-10 | Given a frame, when the Cursors tool or `C` is used, then two vertical and two horizontal cursors appear | E, R | #14 AC1 | no |
| T-11 | Given a focused cursor, when ArrowRight/Left/Up/Down, Shift and Delete are pressed, then it moves one step, ten steps, and is removed | E | #14 AC1 | no |
| T-12 | Given a cursor, when dragged with the mouse, then the position and the readout follow in the same step | E, R | #14 AC1 | no |
| T-13 | Given cursors, when placed, then the readout shows X1, X2, ΔX, 1/ΔX, Y1, Y2, ΔY with units | E, R | #14 AC2 | no |
| T-14 | Given the cursors, when the scale or the channel changes, then readouts update while the lines stay | E | #14 AC3 | no |
| T-15 | Given a frozen graph, when cursors are used, then they work; given an empty graph, then the Cursors tool is disabled with tooltip `No waveform` | E | #14 AC4 | no |
| T-16 | Given the docs, when read, then Q-E6 origin decision is recorded in `docs/architecture.md` | U (doc) | #14 AC5 | no |
| T-17 | Given cursors at close places, when tags are drawn, then the overlap is noted (known minor) | R, U | #14 | no |
| T-18 | Given the calibrator on the simulator, when measured, then twelve rows are offered and pinning is possible | E, R | #15 AC1 | no |
| T-19 | Given the simulator calibrator at 200 us/div, when read, then frequency text is `1.000 kHz`, period `1.000 ms`, Vpp about `1.91 V`, and the numeric frequency is within 0.01 % (the real figure is reported) | R, A | #15 AC2 | HW H-01 |
| T-20 | Given a signal with fewer than one period or a clipped one, when measured, then the cell shows `-` and a reason (footnote) | E, R | #15 AC3 | no |
| T-21 | Given a range outside 50 mV to 1 V per division, when measured, then the `?` marker and amber edge show and the accuracy note is under the table | E, R | #15 AC4 | no |
| T-22 | Given values, when shown, then engineering notation with units (`1.91 V`, `500 us`) | E, R | #15 AC5 | no |
| T-23 | Given a pinned subset, when the app restarts, then the pin choice is restored | E, R | #15 AC1 | no |
| T-24 | Given a frame, when `Export CSV` is chosen in the File menu, then `export_frame` with `csv` is sent, an info toast `Saved <name>` shows, no path in the text | E, R | #18 AC1 | no |
| T-25 | Given the real window with a file dialog replaced by the default path, when a CSV is exported, then the file has the metadata and 4096 rows per enabled channel and values match the frame | R | #18 AC1 | no |
| T-26 | Given a raw export, when read with the reader of `docs/formats/frame-raw-v1.md`, then codes reload, and the sidecar format file exists | R | #18 AC2 | no |
| T-27 | Given a failing command (read-only folder), when exporting, then an error toast names the cause and no file or temp file remains | E (toast), A (cargo), R | #18 AC3 | no |
| T-28 | Given an empty graph, when the File menu opens, then the export and screenshot items are disabled with `No waveform` | E | #18 AC4 | no |
| T-29 | Given a frame, when `Ctrl+P` or the toolbar button is used, then `save_screenshot` is called with PNG bytes and a toast shows | E, R | #19 AC1 | no |
| T-30 | Given the composed PNG at ratio 1 and 2, when decoded, then size is css size times ratio, graticule lines one device pixel wide, header and table text drawn (vitest and existing e2e) | A | #19 AC1, AC2 | HW H-09 |
| T-31 | Given a failing screenshot save, when it occurs, then an error toast names the cause | E | #19 AC3 | no |
| T-32 | Given a setup, when saved, then the file is human-readable JSON with channel, time base and trigger values | R, A | #20 AC1 | no |
| T-33 | Given a setup file, when loaded, then settings apply; a value the device cannot do (3 V range) is reported in a warning toast and the current value stays | R, E, A | #20 AC2 | HW H-10 |
| T-34 | Given the app closed and started again (same HOME), when the simulator is ready, then the last setup is restored and acquisition is stopped | R | #20 AC3 | no |
| T-35 | Given a corrupt or version 2 file, when loaded, then the current setup stays and a toast gives the reason; given a corrupt `last-setup.json`, then defaults, `.corrupt` kept, one notice toast | R, A | #20 AC4 | no |
| T-36 | Given the File menu, when opened with keyboard, then items are navigable, Escape closes, shortcuts `Ctrl+S`, `Ctrl+E`, `Ctrl+P`, `C` work and do nothing while a text field has focus | E | D5 | no |
| T-37 | Given a disconnect (`use_simulator false`) during live capture, when cursors and measurements are on, then the trace freezes, measurements read `Frozen`, cursors work | R, E | #14 AC4, #15 | no |
| T-38 | Given light and dark themes and text scales 100 and 150, when the panels render, then no overflow and tokens hold | U, E | design | no |
| T-39 | Given the test runs ended, when the repo is listed, then no `.claude/frontend` or `frontend/.claude` folder exists and `git status` has only intended changes | A | hygiene | no |
| T-40 | Given the stack, when 20 cycles of `use_simulator` toggling run in the real app, then no thread growth and no errors (neighbouring M1 regression) | R | regression | no |
| T-41 | Given `tools/check-test-names.mjs`, when run on the new e2e file, then it passes | A | convention | no |

## Environment
- Simulated device only. Linux, Tauri debug build of `sweepscope-app`, Vite dev on a spare port where possible. Xvfb from extracted debs in the scratchpad, openbox, `GDK_BACKEND=x11`, Mesa forced, isolated HOME, `SWEEPSCOPE_NO_HARDWARE=1`, USB hidden with `unshare -rm`.
- Playwright on spare ports (a local, untracked config), chromium and webkit.
- Real 6254BC: not used. A scope is plugged into this host; it is hidden by the two guards above.

## Entry and exit criteria
Entry: all work packages merged on `feat/m2-measure`; `cargo build` succeeds.
Exit: all cases executed or listed under "not tested"; no open bug of severity high or critical; a verdict is written in `qa-report.md`.
