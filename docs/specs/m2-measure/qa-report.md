# QA report: M2 Measure (#13 #14 #15 #18 #19 #20)

Build or commit: `feat/m2-measure` at 6361535 (the first real-window pass ran on d4f85e7; after the review fixes landed, the cargo, vitest, check, full Playwright run and the export checks were repeated at 6361535).
Date: 2026-11-23 (host clock 2026-11-24 UTC)
Plan: [test-plan.md](test-plan.md)

## Summary
Verdict: **fail** (release blocked). The feature works end to end in the real window, but four defects block release: BUG-11, BUG-12 (acceptance criteria of #14), BUG-13 (pin menu unusable) and BUG-15 (`cargo test` dirties the tree and rewrites committed generated files with broken imports). Everything else passed or is a minor finding.

Guards held in every real-app run: `SWEEPSCOPE_NO_HARDWARE=1` (log line `hardware discovery is off`), USB hidden with `unshare -rm` and tmpfs over `/dev/bus/usb` and `/sys/bus/usb/devices` (0 devices visible; a 6254BC is plugged into this host), isolated HOME, Xvfb :77, Mesa forced. The real scope was never opened.

## Results
| Test ID | Result | Notes |
|---|---|---|
| T-01 | pass | `cargo test --workspace` 0 failed (twice); vitest 68 files, 797 tests; `npm run check` ok (after QA fixed formatting of its own files); `npm run test:tools` 42 pass |
| T-02 | pass | full Playwright, chromium and webkit, spare ports 5273 and 4273: 445 passed, 1 skipped (pre-existing), including 56 runs of the new spec |
| T-03, T-04, T-05 | pass | real window: no `not allowed` or capability error in the vite log, the app log or the devtools console. `get_snapshot`, `set_interpolation`, `export_frame` (csv, raw), `save_screenshot`, `save_setup`, `load_setup` all ran; `display_changed` and `measurements_changed` visibly updated the window; native GTK dialogs open in front and work |
| T-06 | pass | at 20 ns/div the window starts with sin(x)/x (automatic badge `SIN(X)/X SW`); at 200 us/div it is Linear; Linear and Dots can be picked. 100 ns/div not stepped to in the real window |
| T-07 | pass | Dots mode draws a dotted trace and the `DOTS` badge |
| T-08 | pass | switched sin(x)/x, Linear, Dots while running: capture rate kept updating; e2e: one `set_interpolation` with `dots` |
| T-09 | pass (simulator) | calibrator rise time at 20 to 100 ns/div is in the cargo suite (green); hardware H-03 is the human's |
| T-10 | pass | `C` adds four cursors; two vertical, two horizontal |
| T-11 | pass | e2e: arrow = 1/250, Shift = 10 steps, Delete removes |
| T-12 | pass | drag in real window and e2e; value follows in the same step |
| T-13 | pass | real window: `X1 600 us, X2 1.40 ms, ΔX 800 us, 1/ΔX 1.25 kHz, Y1 1.60 V, Y2 -1.60 V, ΔY -3.20 V` at 200 us/div, 1 V/div (checked by hand against the grid) |
| T-14 | pass | readouts follow channel scale (Y values doubled at 2 V/div while the lines stayed) |
| T-15 | **fail** | frozen (disconnect) works; empty screen: C adds cursors (BUG-12); toolbar tool has no disabled state or `No waveform` tooltip (BUG-11) |
| T-16 | **fail** | Q-E6 in `docs/architecture.md` is still "Open (cursor story)"; the spec assigns this to the staff engineer after approval (#14 AC5 open) |
| T-17 | known minor | dragging X2 next to X1 makes the X tags overlap (`X1 6..` hidden by `X2 692 us`) |
| T-18 | pass | pin menu offers twelve rows (BUG-13 for its placement); five pinned by default (see BUG-16 note) |
| T-19 | pass with finding | text `1.000 kHz`, `1.000 ms`, Vpp `1.99 V` for the simulator default sine (2.09 V at 2 V/div). Numeric value, see "R2 figures" below: misses 0.01 % |
| T-20 | pass | at 20 ns/div: Period and Rise time `-` with footnote `[1] Signal is flat or too small`; e2e: invalid cell with reason |
| T-21 | pass | at 2 V/div: `?` marker and amber edge on values, accuracy note under the table, `UNVERIFIED` badge on the channel chip |
| T-22 | pass | `1.000 kHz`, `1.000 ms`, `707 mV`, `183 us` |
| T-23 | pass | unpinned Frequency, restarted the app: choice restored (`settings.json` holds `measurements`) |
| T-24, T-25 | pass | CSV via File menu and dialog; toast `Saved <file>` (no path); file has 11 metadata lines, header `time_s,ch1_v`, 4096 rows, values equal the converter codes |
| T-26 | pass | raw file read with a Python reader built from `docs/formats/frame-raw-v1.md`: magic SSR1, 4096 codes, volts match CSV; sidecar `.format.txt` written. Note: the repository doc says `.bin`; the dialog filter is `bin` |
| T-27 | pass | `/sys` (read-only) target: error toasts `cannot write qa-blocked.csv: permission denied` and for raw also the sidecar; nothing left behind. A chmod 555 folder cannot be used under `unshare -rm` (the user-namespace root bypasses it) |
| T-28 | pass | empty screen: Export CSV, Export raw and Save screenshot disabled, tooltip `No waveform`; setups enabled |
| T-29 | pass | PNG saved (928 x 1038 at ratio 1) with graticule, traces, cursors, trigger guides, header lines (scales, cursors, trigger), measurement table, accuracy note |
| T-30 | pass | vitest and the existing e2e pixel checks (ratio 2, one device pixel lines); HiDPI on a real display is H-09 |
| T-31 | pass | e2e: rejected `save_screenshot` shows an error toast with the cause. Clipped note in the PNG: BUG-14 |
| T-32 | pass | `setup1.json`: pretty JSON, `format`, `version: 1`, channels, time base, trigger in volts and seconds |
| T-33 | pass | loaded a file with CH1 at 3 V: warning toast `Loaded setup3v.json with changes: CH1 volts/div 3.00 V is not available; kept 2.00 V`; saved values restored on load of the good file |
| T-34 | pass | restart with the same HOME: 500 us/div and 2 V/div restored, acquisition stopped |
| T-35 | pass | version 2 file: `The setup file was written by a newer SweepScope (version 2)`; invalid JSON: `The file is not valid JSON, so it is not a setup file`; corrupt `last-setup.json`: one warning toast, defaults, `.corrupt` kept |
| T-36 | pass | e2e: File menu keyboard, Escape, Ctrl+S, Ctrl+E, Ctrl+P, shortcuts ignored with a text field focused |
| T-37 | pass | `use_simulator false`: trace frozen, `FROZEN` label, cursors keep working; 20 unplug and replug cycles: threads 36 to 37, acquisition resumed, no errors |
| T-38 | partly | existing tier, a11y and gallery e2e (both text scales, both browsers) green; no dedicated light-theme run of the new panels (app is dark in the stub) |
| T-39 | pass for `.claude/frontend` and `frontend/.claude` (absent) / **fail** for `backend/frontend` | see BUG-15 |
| T-40 | pass | see T-37 |
| T-41 | pass | `check-test-names` ok |

### R2 figures (known risk R2, #15 AC2)
- Real window, simulator default signal (1 kHz sine, 2 mV noise) at 200 us/div, 40 reads of the numeric `value`: frequency error flips between **-0.040 %, 0.000 % and +0.040 %** (period 1.0004 ms, 1.0000 ms, 0.9996 ms). The text shows `1.000 kHz` or `999.6 Hz` accordingly. This is above 0.01 %.
- Calibrator-like wave (0 to 1.91 V, 250 ns edges, 8-bit codes) swept over 200 phases of the record (new test `backend/libs/measurements/tests/qa_calibrator_frequency.rs`): worst error 0.011 % (100 us/div), 0.022 % (200 us/div), 0.017 % (500 us/div), 0.023 % (1 ms/div and 2 ms/div); mean 0.005 to 0.012 %. At 100 us/div 71 of 200 phases are `FewerThanOnePeriod`. So `1.000 kHz` is displayed, but 0.01 % is not met at an arbitrary phase; it is met on average only. The simulator's calibrator cannot be selected from the window (no command), so these two figures come from the real window (sine) and the library test (calibrator shape).
- PM decision needed: accept 0.03 % as the M2 bound (spec R2) or schedule averaging. Hardware H-01 and H-02 remain with the human.

## Bugs
| Bug | Severity | Status |
|---|---|---|
| BUG-11 toolbar Cursors tool has no handler and no `No waveform` state (#14 AC1, AC4) | medium | open, sent to developer |
| BUG-12 `C` adds cursors on an empty screen (#14 AC4) | medium | open, sent to developer |
| BUG-13 measurement pin menu opens off the right edge, labels invisible (#15) | medium | open, sent to developer |
| BUG-14 screenshot clips the accuracy note, T1 label overlaps the centre line (#19 AC1) | low | open, sent to developer |
| BUG-15 `cargo test --workspace` creates `backend/frontend/` and rewrites 8 generated TS files with broken imports (kernel `export_to` depth, commit 5661779) | high (CI, clean checkout) | open, sent to developer |
| BUG-16 scaffold placeholders (`Illustrative numbers`, `Accuracy note slot (wording TBD)`) remain in the live window; default pins are five, not twelve | low | open, sent to developer; pins decision for PM |
| C-6 cursor tags overlap when cursors are close (known) | cosmetic | not filed |
| C-7 File button tooltip is cut at the right window edge; File menu shows no shortcut hints; 9-sample steps in the trace are 8-bit quantisation, not a defect | cosmetic | not filed |
| Doc gaps: Q-E6 not recorded (#14 AC5); README accuracy sentence, keys and file formats not written (spec section 8); `frame-raw-v1.md` says `.bin` | docs | for staff engineer and PM |

e2e cases that pin the bugs use `test.fail` (BUG-11, BUG-12, BUG-13) so the suite stays green and turns red when they are fixed; remove the annotations in the retest.

## Not tested and why
- Hardware checklist H-01 to H-10 (real 6254BC): the agents never touch the scope; human item.
- HiDPI and 200 % OS scale, dialog behaviour on a real compositor and Wayland: not available under Xvfb. Under Xvfb with openbox the native dialog did not take keyboard focus by itself (needed `windowfocus`), and the File menu stayed open behind it; not reproduced as a defect on a real desktop.
- Light theme for the new panels and four-channel measurement columns; 100 ns/div step in the real window; screenshot save failure in the real window (covered by e2e); the exported file at a failing disk-full condition.
- Selecting the calibrator in the real app (no UI or command for it).
- Stub-only note: the e2e stub's time base is not a real value, so time cursor text in browser tests shows odd prefixes (`600 s`); the real window shows correct `600 us`.

## Screenshots and logs
Kept in `.claude/tmp/m2/` (not committed): `01-start.png` to `45-sys-raw.png`, `shot-saved.png`, `data/` (CSV and raw sample, cargo and Playwright logs).
New test code: `frontend/e2e/m2-measure.spec.ts` (28 cases x 2 browsers), `frontend/e2e/support/tauri-stub.ts` (scripted command results), `frontend/e2e/csp.spec.ts` (preview port from `E2E_PREVIEW_PORT`), `backend/libs/measurements/tests/qa_calibrator_frequency.rs`.
