# Test plan: M4 Polish (#33, #34, #37, #38, #50)

Branch `feat/m4-polish` at `5bdd38a` (all work packages merged, local). Sources: issues, [design-spec.md](design-spec.md) rev 4, [implementation-spec.md](implementation-spec.md), [work-plan.md](work-plan.md), ADRs 0021 to 0024, `docs/prd.md` 5.

## Scope and risks

| Risk | Why | Where it is checked |
|---|---|---|
| Composition gaps hidden by stubs (BUG-1 class) | The e2e stub acknowledges every calibration command; a capability, a snapshot member, an event name or a `main.ts` wiring may be missing in the real window | R-1 to R-6: real Tauri window under Xvfb with the simulator |
| Calibration applied to the wrong thing | Offsets must reach frames, measurements, exports, and the trigger | R-7, C-9 (simulator shorted, tag, measured values) |
| Marker removal incomplete | 14 places plus data model, docs, screenshots, exports | M-1 to M-6 |
| Real hardware is plugged in on this machine | The app always composes `HantekCatalog`; the CLI too unless discovery is off | Every real run: `SWEEPSCOPE_NO_HARDWARE=1` plus `unshare -rm` with tmpfs over the USB directories. The real scope is never opened |
| Shorted-input thresholds tuned on one unit | Simulator cannot prove them | Hardware checklist only |
| CLI exit codes | Five distinct codes, easy to confuse | L-1 to L-8 |
| Packages | Rule, icons, dependencies are declared, not tested by running | P-1 to P-6 (static) |

## Test cases

Kind: E = automated e2e (Playwright, stub), R = real window (Xvfb, simulator), U = UI check (manual with screenshot or DOM), S = static or command line check, H = hardware (human). "Needs scope" is only for H.

### #33 calibration dialog and tag

| ID | Given / when / then | Kind | Criterion | Needs scope |
|---|---|---|---|---|
| C-1 | Given a ready device with no offsets, when the main window shows, then the status bar shows `NO CALIBRATION` (button, aria-label "No calibration. Open calibration dialog"), right end of the bar | E, R | 33-1 | no |
| C-2 | Given offsets stored for all channels, when shown, then the tag reads `CALIBRATED` with the check icon | E, R | 33-1 | no |
| C-3 | Given no device (disconnected), when shown, then the tag is `Calibration -`, disabled, title carries the reason; click does nothing | E | 33-1 (design 2, D6) | no |
| C-4 | Given `NO CALIBRATION`, when clicked, then a modal dialog "Zero calibration", `Step 1 of 6`, "Why calibrate", three bullets, `Cancel` and `Start` | E, R | 33-2 | no |
| C-5 | Given step 1, when Start, then step 2 shows "Channel 1: short the input", the three instructions, the open-input note, channel chips with CH1 current, and `Calibrate`; focus is on the heading, never on `Calibrate` | E, R | 33-3 | no |
| C-6 | Given a channel step, when Calibrate, then the measuring step shows a progressbar (determinate with `aria-valuenow`, or indeterminate), `Cancel`, disabled `Measuring...`, and the dialog moves to the next channel; chips for done channels have a check and an accessible name | E, R | 33-4 | no |
| C-7 | Given channels 1 to 4 done, when the last finishes, then step 6 shows "Calibration completed", the 12 by 4 table (2 mV/div to 10 V/div), and `Done`; the tag becomes `CALIBRATED` | E, R | 33-6 | no |
| C-8 | Given an input that is not shorted (simulator default signal), when Calibrate, then the not-shorted alert with the measured lines, "Nothing was stored", `Try again`, no Skip; stays on the channel; nothing stored (tag unchanged, no offsets file) | E, R | 33-5 | no |
| C-9 | Given the simulator shorted, when the wizard completes, then the offsets are stored (offsets file in the isolated config folder, one unit), applied (a measurement or frame of the shorted input reads about 0 V), and survive an app restart (tag `CALIBRATED` after restart) | R | 33-7 | no |
| C-10 | Given `CALIBRATED`, when the tag is clicked, then the dialog opens at step 1 with "This device is calibrated" and "Last calibrated at <date>" | E, R | 33-8 | no |
| C-11 | Given acquisition running, when the dialog opens, then the toolbar chip reads `Paused (calibration)` behind the scrim; when cancelled, the run state is restored | E, R | 33-9 | no |
| C-12 | Given every step before Done, when Cancel or Esc, then the dialog closes and nothing is stored (offsets file unchanged, tag unchanged), including after three channels were measured | R (E for the UI half) | 33-9 | no |
| C-13 | Given the Done step, when Esc or Done, then the dialog closes, offsets stay | E, R | 33-6 | no |
| C-14 | Given the device is lost while the dialog is open, then the body shows "Device disconnected. Calibration cancelled. Nothing was stored." as an alert, with one `Close` | E, R | design 3.3 | no |
| C-15 | Given the recording is active, when the tag is shown, then it is unavailable with "Stop recording first" | E | design / impl 1.7 | no |
| C-16 | Keyboard: Tab order inside the dialog stays in the dialog, never reaches the page behind; Enter on step 1 does not start; on close focus returns to the tag; tag opens with Enter and Space | E | design 8 | no |
| C-17 | Tooltip: hover (500 ms) and keyboard focus show the multi-line tooltip; Esc hides it; not calibrated shows typical-value rows and "Click to calibrate."; calibrated shows date, rows, "Click to calibrate again." | E, R | 34-3 | no |
| C-18 | Done table: scroll region focusable, named, sticky header, scrolls at 200 % text scale; dialog stays inside the window at 1024 x 720 and 200 % | E, U | design 3.4, 8 | no |
| C-19 | Dialog, tag and connection item look in light of tokens: colours from tokens (`status-warning`, `status-ok`, scrim), no colour-only state | U | design 2, 8 | no |

### #33 and connection item (#50 design 2.1)

| ID | Given / when / then | Kind | Criterion | Needs scope |
|---|---|---|---|---|
| N-1 | Given the five states (Ready, Ready simulator, Connecting, Disconnected, Error), when shown, then the item is dot (warning icon for Error), bold word, muted detail; no `Simulator` chip, no `StatusChip` | E, R | design 2.1 | no |
| N-2 | Given the status bar at 1024 px and 200 % text scale, when shown, then connection item and tag are both visible; Renderer drops first | E, U | design 2 | no |
| N-3 | The `Accuracy` button and popover are gone | E | design 4.1 row 11 | no |

### #50 markers

| ID | Given / when / then | Kind | Criterion | Needs scope |
|---|---|---|---|---|
| M-1 | Given the main window (all docks, a flagged time base in the stub), when shown, then no text, aria-label, title or class matches `unverified`, `partially verified`, `not verified`, or a `?` badge | E, R | 50-1 | no |
| M-2 | Given every Select, Menu and range selector (V/div, time base, trigger, pulse width), when opened, then no marker item or badge | E | 50-1 | no |
| M-3 | Given the gallery pages, when shown, then no marker demo | E | 50-1 | no |
| M-4 | Given the whole repository (`frontend/src`, `backend`, `src-tauri`, `cli`, `docs`), when searched, then no marker identifier or text remains except history and the removal notes; `Exp` badge text has no "verified" | S | 50-1, 50-3 | no |
| M-5 | Given a flagged range (50 mV/div and below, time bases above 30), when stepped to, then behaves as before (the step is accepted, a frame arrives in the simulator) | R | 50-2 | no |
| M-6 | Given the PRD, `docs/design.md` and `docs/architecture.md`, when read, then marker requirements are amended (grep and read) | S | 50-3 | no |
| M-7 | Given the marker unit tests, when the whole test suite runs, then all green and no `unverified` test left | S | 50-4 | no |
| M-8 | Screenshot and CSV, raw export: no ` ?` after values, no `range_verified`; `calibrated` flag instead | S, R | 50-1 | no |

### #34 accuracy

| ID | Given / when / then | Kind | Criterion | Needs scope |
|---|---|---|---|---|
| A-1 | Given `docs/accuracy.md`, when read, then it lists every range (12) and time base (36) with Verified, Partly verified or Not verified, agreeing with the PRD 5.1 capability table | S | 34-1 | no |
| A-2 | Given the page, then it states the one-unit limitation honestly, uses "about", never a tolerance | S | 34-2 | no |
| A-3 | The accuracy page guard test (`accuracy_page.rs`) passes and fails when the page and the table differ (mutate a scratch copy) | S | 34-1 | no |
| A-4 | Tooltip: see C-17 | E, R | 34-3 | no |

### #38 CLI

| ID | Given / when / then | Kind | Criterion | Needs scope |
|---|---|---|---|---|
| L-1 | Given a setup file and `--count 3 --simulator`, when run, then three `capture-000N.csv` files and exit 0; stdout lists files | S | 38-1, 38-4 | no |
| L-2 | `--format raw`: `.bin` files decode (SSR1), layout document written once | S | 38-1 | no |
| L-3 | Given no device (simulator off, hardware off) and `--device-wait 1`, then exit 10 | S | 38-2 | no |
| L-4 | Given a flat input and a Normal sweep with `--timeout 1`, then exit 11; frames already written stay | S | 38-3 | no |
| L-5 | Given an existing target file, then exit 1 and the file is untouched; given a bad option or no subcommand, exit 2 with usage; given an unreadable or invalid setup, exit 1 | S | design | no |
| L-6 | Given an unwritable output folder, then exit 12 | S | impl 3 | no |
| L-7 | Given stored offsets, then the CSV says `zero_offset_applied` true and the config folder is unmodified (hash before and after) | S | impl 4 | no |
| L-8 | Superseded by the single executable section below (one binary, which links WebKitGTK; runs without a display) | S | ADR 0022 | no |

### #37 packages

| ID | Given / when / then | Kind | Criterion | Needs scope |
|---|---|---|---|---|
| P-1 | Given the build config, then identifier is `io.github.dkrepsky.sweepscope`, no `dev.sweepscope.app` or placeholder left (grep), icons list complete | S | 37-4 | no |
| P-2 | Given a built `.deb` (only if one exists for this commit; otherwise `tauri build` is not run on this machine and the case is reported as untested), `dpkg -c` shows udev rule, icons 16 to 512, desktop file, binary; `dpkg -I` shows Depends with webkit2gtk 4.1, gtk3, libusb | S | 37-1 to 37-3 | no |
| P-3 | Given the udev rule file, then it passes `udevadm verify` and matches the vendor and product ids the driver uses | S | 37-2 | no |
| P-4 | Given the CI workflow, then a manual job builds both packages and publishes nothing | S | 37-1 | no |
| P-5 | Given the app run from an installed layout (not done: no install on this machine) | not run | 37-2 | no |
| P-6 | Icon: `icon.png`, `.ico`, `.icns` exist, 16 to 512, the window title and class use the final identifier in the real window | S, R | 37-4 | no |

### Regression and cross cutting

| ID | Given / when / then | Kind |
|---|---|---|
| X-1 | Full suites: `cargo test --workspace`, vitest, Playwright (chromium and webkit), `npm run check`, `npm run test:tools` | S |
| X-2 | Real window smoke: console log of the dev server has no `not allowed` or capability error; every calibration command is in the capability | R |
| X-3 | Wiring: each new stream, event and command has a non-test caller (`grep` of `zero_calibration_changed`, `zeroCalibration`, `createCalibrationCommands`, `compose` of calibration in the shell) | S |
| X-4 | Text scale 200 %: dialog and bar usable, no horizontal scroll of the page, footer buttons reachable | U, E |
| X-5 | Clean tree after `cargo test --workspace` | S |
| X-6 | Old frames and raw files with `range_verified` still decode (export unit tests) | S |

### Hardware (human, short list in [hardware-checklist.md](hardware-checklist.md))

H-M4-1 real short and Done table under 40 s per channel; H-M4-2 open input beside the calibrator is rejected; H-M4-3 shorted input reads 0 V within one code at 2, 5, 10 mV/div; H-M4-4 offsets independent of time base, one channel, AC coupling, bandwidth limit; H-M4-5 replug and restart keep calibration; H-M4-6 CLI exit codes on the real scope; H-M4-7 `.deb` on a clean machine.

## Single executable (#38, #37 packaging), addendum and work-plan section 10

Tree under test: `feat/m4-polish` at `2a510e8`. Replaces L-8 (there is no `sweepscope-cli` binary any more; the CLI is the application binary and links WebKitGTK). No design states (CLI and packaging only). Every run: `SWEEPSCOPE_NO_HARDWARE=1`, USB directories hidden where the start mode allows it, isolated `HOME` and XDG folders, no `DISPLAY` or `WAYLAND_DISPLAY` for the CLI, Xvfb only for the window. No case needs the real scope; the device-side checks stay in H-M4-6.

| ID | Given / when / then | Kind | Criterion |
|---|---|---|---|
| S-1 | Given no display, when the binary gets each of `capture`, `--help`, `-h`, `--version`, `-V`, then it runs the CLI and exits with its code | E, S | dispatch |
| S-2 | Given a setup file and `--count N` (csv and raw), then N files and exit 0 | S | 38-1, 38-4 |
| S-3 | Given no device (no `--simulator`) and `--device-wait`, then exit 10 | S | 38-2 |
| S-4 | Given a Normal sweep that never triggers and `--timeout`, then exit 11 | S | 38-3 |
| S-5 | Given mistakes after `capture` (unknown option, missing value, count 0, bad format, no `--setup`), then exit 2 with the usage | S | addendum |
| S-6 | Existing target (1, file untouched), invalid or missing setup (1), unwritable folder (12), stored offsets applied and config folder unchanged | S | design |
| S-7 | Given no argument, `captur`, `Capture`, `--Help`, `--headless`, `-v`, `s.json`, `--simulator capture`, under Xvfb, then the window opens and nothing is captured | R | dispatch |
| S-8 | Window command line with no display: fails and ends, no hang, nothing written | S | limitation |
| S-9 | Edges: non-UTF-8 argument, output to a closed pipe, `--help` after `capture`, Ctrl+C during a capture | S | robustness |
| S-10 | Given a non-AppImage binary with inherited `APPDIR`, `PWD`, `OWD` from elsewhere, then they are ignored | S | 10.4 |
| S-11 | The window starts and shows the live simulator view (Xvfb), capability errors absent | R, U | 37-2 |
| P-7 | `npm run build -- --bundles deb,appimage` at this commit produces both | S | 37-1 |
| P-8 | `.deb` extracted (not installed): `/usr/bin/sweepscope` is the only program, no `sweepscope-cli`; `.desktop` Exec has no arguments; udev rule, icons, Depends | S | 37-2, 37-3 |
| P-9 | The extracted `.deb` binary runs `--version`, `--help`, `capture` without a display | S | 38, 37 |
| AI-1 | AppImage, normal start, from another folder: relative `--setup`, relative `--out`, default out, relative `--config-dir` | S | 10.4 |
| AI-2 | The same with `--appimage-extract-and-run` and with `APPIMAGE_EXTRACT_AND_RUN=1` | S | 10.4 |
| AI-3 | AppImage exit codes 0, 2, 10, 11, 12; folder with spaces; started through `~/.local/bin/sweepscope` on `PATH`; stale or foreign `PWD`/`OWD` | S | 10.4, install.md |
| AI-4 | AppImage window starts (Xvfb), both start modes | R | 37-2 |
| D-1 | Commands and claims of `docs/cli.md` work as written (with `--simulator`) | S | docs |
| D-2 | Commands of `docs/install.md` work as written, as far as possible without root | S | docs |
| D-3 | Hardware checklist H-M4-6 works as written with the simulator (device-only parts noted) | S | docs |
| X-7 | Full suites: `cargo test --workspace` (with `cli_mode`), vitest, `npm run check`, `npm run test:tools`, Playwright | S | regression |
| X-8 | No CI step, script or doc that is still current names the removed binary; architecture rules cover the new dependency | S | 10.2 |
| S-12 | Given a clamped setup, then warnings are on stderr and stdout lists only written files (BUG-33) | S | 10.5 |
| S-13 | Given a non-UTF-8 argument after a CLI word, then exit 2 with usage and nothing created; as first argument, the window | E, S, R | 10.5 |
| AI-5 | Given non-UTF-8 `PWD` and `APPDIR`, then relative paths still resolve to the caller's folder | E, S | 10.5 |
| D-4 | The extract-and-run folder behaves as `cli.md` describes (BUG-34) | S | 10.5 |

## Environment

- Simulator only. Real 6254BC is plugged in on this machine (`lsusb` 04b5:6cde): never opened. App and CLI run under `unshare -rm` with tmpfs over `/dev/bus/usb` and `/sys/bus/usb/devices`, `SWEEPSCOPE_NO_HARDWARE=1`, isolated `HOME` and XDG folders.
- Xvfb `:77` (extracted debs, not the user display), openbox, Mesa software rendering, 1440 x 900.
- Spare ports: vite 5373 for the debug app, Playwright 5273 and 4273 through an untracked `playwright.qa.config.ts`.
- Scratch under `.claude/tmp/m4`; no screenshot-comparison tests.

## Entry and exit criteria

Entry: all work packages merged, tree clean, builds. Exit: every case run or reported as not run with the reason, every bug has a status, verdict per acceptance criterion in the report. Release blockers: any open Critical or Major bug, or a wiring gap that leaves a criterion dead in the real window.
