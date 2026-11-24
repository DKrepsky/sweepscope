# QA report: M4 Polish (#33, #34, #37, #38, #50)

Build or commit: first round `feat/m4-polish` at `5bdd38a`; retest round after the fix round at `7ab8ff1` (unpushed). Debug builds of `sweepscope-app` and `sweepscope-cli` made from the tree of each round.
Date: 2026-11-24
Test plan: [test-plan.md](test-plan.md). Hardware checklist for the human: [hardware-checklist.md](hardware-checklist.md).

## Summary

Verdict: **pass** (software), after the retest round at `7ab8ff1` (section Retest round 2 below). No bug is open. BUG-31 and C-1 to C-5 and C-7 (partly) are fixed; C-6 (tag cut off at 150 % and 200 % text) and two small leftovers remain with the PM. Nothing blocks the release of the software. Hardware behaviour (shorted-input thresholds, offset independence, CLI on a real scope, `.deb` install) is not proven and needs the human's checklist before the calibration is called verified on the 6254BC.

What was exercised:

- Automated: `cargo test --workspace` 1201 passed, 0 failed (3 ignored, tree clean afterwards); vitest 1382 passed (110 files); `npm run check` green (fmt, clippy, eslint, stylelint, prettier, svelte-check, e2e tsc, architecture, test names); `npm run test:tools` 46 passed; Playwright (chromium and webkit) 660 passed, 1 skipped, 1 failed. The failure is the known webkit Toast timing test (`feedback.spec.ts`), which passes alone with `--workers=1` (12 of 12). New file `frontend/e2e/m4-polish.spec.ts`: 37 cases per browser, all green; the case for BUG-31 uses `test.fail` and flips red when the bug is fixed.
- Real window (Tauri app under Xvfb `:77`, openbox, Mesa software rendering, isolated HOME, `SWEEPSCOPE_NO_HARDWARE=1`, `SWEEPSCOPE_SIMULATOR_SIGNAL=shorted` or default, USB directories hidden by `unshare -rm`; the real 6254BC on this machine was never opened): the whole wizard with the real backend, persistence across restart, application of the offsets, cancel and device-lost paths, real keyboard events, text scale 150 % and 200 %.
- CLI (copy of the binary, same USB hiding): exit codes 0, 1, 2, 10, 11, 12.
- Static: package contents of an existing `.deb`, the udev rule, the accuracy page against the capability table, repository greps for markers and stale docs.

Wiring (BUG-1 class): no gap found. The four commands are in the main capability and manifest, `createCalibrationCommands` is called from `live-app.ts`, the shell composes `CalibrationModule`, the `zeroCalibration` snapshot member and the `zero_calibration_changed` event reach the real window, and the vite log showed no `not allowed` errors. The real session drove the tag, dialog, toolbar chip and acquisition pause, which a stub cannot prove.

## Retest round 2 (`7ab8ff1`, after the fix round)

Automated, each run alone, tree checked afterwards (only `frontend/e2e/m4-polish.spec.ts` modified by QA): `cargo test --workspace` 1209 passed, 0 failed, 3 ignored (tree clean); vitest 1377 passed in 110 files (5 fewer than before: the unused `connectionChip` tests were removed with the code); `npm run check` green; `npm run test:tools` 46 passed; Playwright chromium and webkit on spare ports 5273/4273: 660 passed, 1 skipped, 1 failed. The failure is the known probe `graph.spec.ts` "slowest frame is reported against the 33 ms budget" (chromium, load): it passes alone with `--workers=1` in both browsers. `m4-polish.spec.ts` 74 of 74 passed (37 cases x 2 browsers) with the `test.fail` for BUG-31 removed. Logs: `.claude/tmp/m4/cargo-test2.log`, `vitest2.log`, `check2.log`, `tools2.log`, `pw-full2.log`.

| Item | Result | Evidence |
|---|---|---|
| BUG-31 chips on the Done step | Fixed | Real window (Tauri app, real backend, signal shorted): at Step 6 the DOM has `Channel 1..4, done` chips and the screenshot `r2-done.png` shows the row with four ticks above the success note; e2e case without `test.fail` is green in both browsers |
| C-1 device-lost text | Fixed | Real window: `use_simulator` off during step 2 gives `Device disconnected. Calibration cancelled. Nothing was stored.` and one `Close` button (e2e asserts the same text) |
| C-2 zero as `0 V` | Fixed | Real Done table: ranges whose offset is exactly zero show `0 V`; non-zero cells keep the readout format (`663 uV`, `1.06 mV`, `-7.41 uV`) |
| C-3 CLI empty `--out` folder | Fixed (one leftover) | No device (exit 10), never triggers (exit 11), invalid setup (exit 1): the new `--out` folder does not exist afterwards. An existing empty folder is kept; an existing target file (exit 1) is untouched; success still writes the folder. Leftover: with a new nested folder (`--out a/b/c`) only the leaf is removed, `a/b` stay (cosmetic, new) |
| C-4 accuracy page rows | Fixed | `docs/accuracy.md` now has Analog bandwidth, Bandwidth limit, ADC 8-bit HMCAD1511 and Hotplug (2 s settle) rows in the status table, wording in line with PRD 5.1; guard test `accuracy_page` is part of the green cargo run |
| C-5 design.md | Fixed | Greps for TaskPanel and "no modal dialog" find none in `docs/design.md` (J10, line 199, AC-D1 amended) |
| C-7 `.deb` Maintainer and Depends | Fixed (name left) | `dpkg -I`: `Maintainer: SweepScope contributors <https://github.com/DKrepsky/sweepscope>`, `Depends: libusb-1.0-0, libwebkit2gtk-4.1-0, libgtk-3-0` (no duplicates). Package name is still `sweep-scope` and the maintainer field is a URL, not an e-mail (cosmetic, with the PM) |
| First-frame offsets race (CLI) | Fixed | 12 runs, `--count 1`, stored offsets for `simulator`, USB hidden: all exit 0 and every first CSV says `ch1_zero_offset_applied: true`; config folder checksum unchanged. Without stored offsets the CSV and raw runs still exit 0 (no wait on a missing event) |
| Offsets validation on load | Pass | CLI: a stored offset of 40 V and -40 V is accepted; 40.5 V, 1e9 V and -75 V are rejected (`zero_offset_applied: false`, exit 0). App: with 1e9 V in the file, the real window starts with `NO CALIBRATION`, the tooltip shows the standard values, and `zero-offsets.json.corrupt` is kept beside the file. The CLI leaves no `.corrupt` copy (read-only use of the folder, as designed) |
| Packages at HEAD | Partly | See #37 criterion 1 below |
| AppImage start | Pass | AppImage under Xvfb with isolated HOME, `SWEEPSCOPE_NO_HARDWARE=1`, USB directories hidden, extract-and-run: it started, window `SweepScope`, WM_CLASS `sweepscope` (equals `StartupWMClass`), icon set, config folder `io.github.dkrepsky.sweepscope`, simulator Ready, tag `NO CALIBRATION` (`appimage.png`) |

Note on the packages: the `.deb` and AppImage in `agent-a5c710d93cdc20369/target/release/bundle/` were built from `15cf2bd`, which has the packaging and docs fixes but not the backend fix round (`f17417e`; the binary lacks the new device-lost text). The packaging inputs are the same as at `7ab8ff1`; the code in the bundles is older than HEAD. A final release build at the tag is still needed (H-M4-9).

## Verdict per acceptance criterion

### #33 Calibration dialog

| # | Criterion | Verdict | Evidence |
|---|---|---|---|
| 1 | Bar shows `CALIBRATED` when every channel has offsets, `NO CALIBRATION` otherwise | Pass | C-1, C-2 (e2e, real window: tag flips after Done and after restart; file deleted gives NO CALIBRATION) |
| 2 | Click on `NO CALIBRATION` opens a dialog explaining why, with Start | Pass | C-4 (e2e, real window, real Enter key) |
| 3 | Start shows channel 1, short-the-input instructions (never an open input), Calibrate | Pass | C-5; focus on the heading, not on Calibrate; note "An open input is not zero..." |
| 4 | Calibrate measures, moves to the next channel, repeats to all done | Pass | C-6 (real: about 5.1 s per channel, progress 0 to 92 % in 12 steps, then next channel) |
| 5 | Not shorted: says so, stores nothing, lets me retry | Pass | C-8: real open input gives the alert with the backend line, `Try again`, no Skip, no file written; e2e covers retry to the next channel. Retry-then-success on the real backend was not run (signal is fixed per process); module tests cover it (cargo) |
| 6 | Done message with offset values per channel and the accuracy summary, status `CALIBRATED` | Pass | C-7: done note, chips row (BUG-31 fixed, retest 2), 12 x 4 table with `0 V` zeros (C-2 fixed), tag `CALIBRATED`. Accuracy summary is in the tag tooltip by the G2 decision (design 10) |
| 7 | Applied per channel and range, persists across restarts for the same unit | Pass | C-9: Mean of a shorted CH1 at 2 mV/div reads 655 uV uncalibrated and 26, -53, 38, -22 uV (noise) after calibration; offsets file holds the unit `simulator`; tag is `CALIBRATED` after a restart. Other units (hardware) not tested |
| 8 | Click on `CALIBRATED` reopens the dialog | Pass | C-10 (e2e; the intro shows "This device is calibrated" and the date) |
| 9 | Acquisition paused while open, cancel at any step stores nothing | Pass | C-11, C-12: running acquisition goes to `stopped` (chip `Paused (calibration)`) and returns to running after Cancel; instrument state identical before and after; Cancel after 3 channels, Esc and Cancel mid-measurement, cancel then immediate reopen: nothing stored, no stale result |

Also verified: device lost during a session gives one `Close` note and the tag becomes disabled `Calibration -` with title "No device", recovers on replug (C-14); recording blocks calibration with "Stop recording first" (e2e only, not in the real window); focus returns to the tag on close; Tab never reaches the page behind (real keys: the third Tab reaches the browser UI, the known native dialog behaviour); 200 % text scale at 1024 x 720 keeps the dialog inside the window with the footer reachable.

### #34 Accuracy page and tooltip

| # | Criterion | Verdict | Evidence |
|---|---|---|---|
| 1 | Accuracy page lists each verified, partly verified, unverified capability and range with status | Pass | A-1, A-3: 12 ranges and 36 time bases agree with the capability table and PRD 5.1; the guard test fails when I change a range status or a measured time base index (restored). C-4 fixed: Hotplug, Bandwidth and ADC rows added (retest 2) |
| 2 | States the one-unit limitation honestly | Pass | A-2: "one 6254BC unit", "not tolerances", "about" |
| 3 | Hover shows the accuracy summary: standard values when not calibrated, values with stored offsets when calibrated | Pass | C-17: e2e hover (500 ms) and keyboard focus; real snapshot text: typical zero error rows and "Click to calibrate." when not calibrated, `CH1 +0.67 mV at 2 mV/div` rows, the accuracy notes and "Click to calibrate again." when calibrated |

### #37 Packages

| # | Criterion | Verdict | Evidence |
|---|---|---|---|
| 1 | Release build produces an AppImage and a .deb | Partly (built at `15cf2bd`, retest 2: both inspected, AppImage started) | `tauri build` was not run (long, writes outside scratch). A `.deb` and an AppImage exist from the WP7 commit `aefeac8` in an agent worktree; the packaging inputs (`tauri.conf.json`, `packaging/`, icons) are identical to HEAD (`git diff` empty), but the binaries are not from HEAD |
| 2 | `.deb` installs the udev rule and the app starts | Partly | `dpkg -c`: `/lib/udev/rules.d/99-hantek6xx4.rules` identical to `packaging/udev`, `udevadm verify` passes, postinst reloads rules, desktop file has `StartupWMClass=sweepscope`. Install and start not run (no install on this machine); the debug app starts under Xvfb (above). Needs H-M4-9 |
| 3 | Dependencies declared and resolved | Pass (declared) | `dpkg -I` (retest 2): `libusb-1.0-0, libwebkit2gtk-4.1-0, libgtk-3-0`, no duplicates. Resolution on a clean machine not run (H-M4-9) |
| 4 | Final identifier and icon replace placeholders | Pass | Identifier `io.github.dkrepsky.sweepscope` in `tauri.conf.json`; no `dev.sweepscope.app` in the built binary; the running app wrote its files to `~/.config/io.github.dkrepsky.sweepscope`; deb icons 16 to 256 are byte-identical to `src-tauri/icons`, 512 is `icon.png`. The window class and launcher icon were not inspected |

### #38 CLI

| # | Criterion | Verdict | Evidence |
|---|---|---|---|
| 1 | Setup file and N: captures N frames and writes the files | Pass | L-1: `--count 3 --simulator` gives `capture-0001..3.csv`, exit 0; L-2 raw: `.bin` files and `capture.format.txt`; L-7: with stored offsets the CSV says `ch1_zero_offset_applied: true` and the raw header `"calibrated":true`, the config folder is unchanged (checksums); default folder from `XDG_CONFIG_HOME` works |
| 2 | No device: distinct exit code | Pass | L-3: exit 10 after 1.0 s with `--device-wait 1` ("no scope became ready within 1s") |
| 3 | Never triggers: distinct exit code after the timeout | Pass | L-4: Normal sweep with the level above the signal, `--timeout 1`: exit 11 after 1.0 s, "0 of N written" |
| 4 | Success exits 0 | Pass | L-1, L-2 |

Other codes: existing target gives 1 and the file is untouched; bad option, missing subcommand, missing `--setup`, count 0, bad format give 2 with the usage text; invalid or missing setup gives 1; `/sys/kernel/foo` (permission) gives 12; `--help` gives 0. `ldd` shows no WebKit, GTK or Tauri. C-3: failed runs leave an empty output folder behind (cosmetic).

### #50 Verification markers

| # | Criterion | Verdict | Evidence |
|---|---|---|---|
| 1 | No UNVERIFIED or PARTIALLY VERIFIED tag, badge or marker on any screen, setting or range selector | Pass | M-1 to M-4, M-8: DOM scan (text, aria-label, title, class) clean in e2e (unmeasured time base stub, every dock select and menu, gallery) and in the real window while stepping the time base across its whole range (5 ns to 1000 s) and all 12 ranges on 4 channels; the real snapshot carries no marker field; the `Exp` badge has no verified wording. Prose sentences remain in the accuracy tooltip and docs ("not verified outside that range"); they are statements, not markers |
| 2 | Previously flagged ranges and features behave as before | Pass | M-5: in the real window every time base and range step was accepted (flagged ones included), frames kept arriving |
| 3 | PRD and design docs amended | Pass | PRD VER-3, 5.2, goal 3 and architecture amended; design.md J4 and 3.1 amended. C-5 fixed in retest 2 (stale TaskPanel and "no modal dialog" text removed) |
| 4 | Old marker tests removed or updated, suite green | Pass | M-7: all suites green; `MainWindow.test.ts` asserts no `verified` field is read by a component |

## Results

| Test ID | Result | Notes |
|---|---|---|
| C-1, C-2 | Pass | e2e + real |
| C-3 | Pass | e2e (disabled, title reason, click ignored); real: after use_simulator off |
| C-4, C-5 | Pass | e2e + real |
| C-6 | Pass | e2e + real; indeterminate bar and determinate value in e2e |
| C-7 | Pass | BUG-31 fixed (retest 2) |
| C-8 | Pass | e2e + real open input |
| C-9 | Pass | real: offsets applied (655 uV to noise level), persisted, survives restart |
| C-10, C-13 | Pass | e2e |
| C-11 | Pass | e2e + real |
| C-12 | Pass | real (3 channels measured then Cancel; Esc and Cancel mid-measure; reopen) + e2e |
| C-14 | Pass | e2e + real (copy differs from design: C-1 cosmetic) |
| C-15 | Pass | e2e only |
| C-16 | Pass | e2e + real keys (Enter on the tag opens, Enter on the heading does nothing, Esc closes, focus returns to the tag) |
| C-17 | Pass | e2e + real data |
| C-18 | Pass | e2e at 100 % and 200 % at 1024 x 720; real 200 % at 1440 x 900 (screenshots r200-*.png) |
| C-19 | Pass | tag colour equals the `status-ok` token in e2e; visual check of the screenshots: no colour-only state, scrim, tag and table consistent with the design |
| N-1 | Pass | e2e (Ready, Ready real scope, Connecting, Disconnected, Error); real: Ready simulator and Disconnected; no chips |
| N-2 | Pass | e2e; C-6 limits at 150 % and 200 % |
| N-3 | Pass | e2e |
| M-1 to M-3 | Pass | e2e |
| M-4 | Pass | grep: only the hardware capability flag (`verified` in `TimeBaseStep` and `VerticalRange`, used by autoset and the accuracy guard), test names, ADRs and docs remain |
| M-5 | Pass | real |
| M-6 | Pass | C-5 fixed |
| M-7 | Pass | |
| M-8 | Pass | CSV and raw header carry `calibrated` and `zero_offset_applied`; no `range_verified` in new files (old files: export unit tests in cargo) |
| A-1 to A-3 | Pass (C-4) | |
| L-1 to L-8 | Pass | L-6 verified with `/sys` (rootless namespace root bypasses chmod) |
| P-1, P-3, P-4(partial), P-6 | Pass | static, existing deb |
| P-2 | Partly | deb and AppImage rebuilt at `15cf2bd` (packaging fixes, not the backend round); AppImage starts under Xvfb (retest 2) |
| P-5 | Not run | no install |
| X-1 | Pass | see Summary |
| X-2, X-3 | Pass | no capability errors; all callers present |
| X-4 | Pass with known issue | C-6 (tag off screen at 150 % and 200 % in a 1440 px window) |
| X-5 | Pass | tree clean after `cargo test --workspace` |
| X-6 | Pass | cargo export tests (old `range_verified` files decode) |

## Bugs

| Bug | Severity | Status |
|---|---|---|
| BUG-31 Done step shows no channel chips row (design 3.1, wireframe step 6) | Minor | Fixed, retested at `7ab8ff1` |
| C-1 Device-lost text differs from design copy | Cosmetic | Fixed, retested |
| C-2 Done table shows `0.00 V` (design `0 V`) | Cosmetic | Fixed, retested |
| C-3 CLI leaves an empty `--out` folder after a failed run | Cosmetic | Fixed, retested; leftover: empty parents of a nested new `--out` stay (cosmetic, new) |
| C-4 `docs/accuracy.md` omits Hotplug, Bandwidth, ADC rows | Minor (docs) | Fixed, retested |
| C-5 `docs/design.md` stale TaskPanel and "no modal dialog" text | Minor (docs) | Fixed, retested |
| C-6 Tag cut off at 150 % and 200 % text scale in a 1440 px window (scroll tier; pre-existing floor) | Minor | With PM, decision needed (not part of the fix round, not retested) |
| C-7 `.deb` package name `sweep-scope`, Maintainer `github`, duplicate Depends | Cosmetic | Maintainer and Depends fixed, retested; package name `sweep-scope` unchanged and the Maintainer is a URL, not an e-mail (cosmetic, with the PM) |

## Not tested and why

- Real 6254BC: forbidden for agents. Shorted-input thresholds, the assumption that offsets do not depend on mode, position, coupling or bandwidth limit, the unit identity field, CLI codes on a device: see [hardware-checklist.md](hardware-checklist.md) (H-M4-1 to H-M4-10).
- `tauri build` at `7ab8ff1` (the bundles are from `15cf2bd`), `.deb` install and dependency resolution on a clean machine and the launcher icon in a desktop menu: need a clean machine (H-M4-9). The AppImage start, window title, class and icon were checked in retest 2.
- Calibration while recording in the real window (needs a roll recording and a file dialog): e2e only for the tag, module tests for the hold.
- Retry after a not-shorted result with a real shorted signal in the real window (the simulator signal is fixed per process): covered by e2e and the calibration module tests.
- Hot unplug of the simulator during Measuring on different channels, text scale 125 %, light theme (the app has the dark set only), WebKit real window (the app window is WebKitGTK; the e2e ran in chromium and the WebKit test browser).
- Two units with different identities (needs two scopes).
- Screenshot-comparison tests are not used (decision of the human).

## Screenshots and logs
Kept in `.claude/tmp/m4/` (not committed): `r-intro.png`, `r-notshorted.png`, `r-done.png`, `r200-*.png` (intro, channel, measuring, done, lost, status), `r150-status.png`, `cargo-test.log`, `vitest.log`, `pw-full.log`, `check.log`. Scratch tools (Xvfb recipe, bridge config, CLI wrapper) live in the session scratchpad; the recipe is in `.claude/memory/qa.md`.

Test code added by QA: `frontend/e2e/m4-polish.spec.ts` (37 cases) and an extension of `frontend/e2e/support/tauri-stub.ts` (zero calibration session in the page, option `zero`, helper `zeroStub`).
