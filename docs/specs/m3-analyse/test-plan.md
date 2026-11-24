# Test plan: M3 Analyse (#22 to #31)

Branch `feat/m3-analyse`, commit 96cef59. Specs: [design-spec](design-spec.md), [implementation-spec](implementation-spec.md), [hardware-checklist](hardware-checklist.md); ADRs 0017 to 0020.
Written before execution. Results are in [qa-report.md](qa-report.md).

## Scope and risks

In scope: acceptance criteria of #22 to #31 on the simulator, the design states of the three new views (Yt persistence, XY, FFT) and the touchpoints (roll, Single, Pulse, Auto-setup, alias, math, recording), regressions of M1 and M2.
Out of scope for QA: everything in the hardware checklist (the human runs H-01 to H-10); agents never touch the scope.

| Risk | Why | How it is covered |
|---|---|---|
| R-A The real window does not do what the unit tests say (BUG-1 class: capabilities, streams, wiring) | Two new streams, nine new commands, an `analysis` dependency of the live app | Real Tauri window under Xvfb driven through devtools-level IPC and DOM, with USB hidden |
| R-B Roll is only ever fed by a simulator that cannot be made to fall behind | Gap marker AC | `SIGSTOP` of the app process (the simulator infers an overrun from elapsed time), recording file gap record |
| R-C Layout: toolbar and dock grew (Single, Persist, Math, Record, Pulse rows, row M) | Floor is 1024 x 720 | Measure toolbar overflow in Yt, XY, FFT, roll at 1440 x 900 and 1024 x 720 |
| R-D Alias and Auto-setup numbers only on 1 kHz sine | Simulator signal is fixed (calibrator not selectable in the app) | Backend and lib tests cited; frequency sweep is a backend test (cargo), hardware H-07, H-09 |
| R-E Hardware-only behaviour (trigger position within 8 samples, one-hour RSS, SIGSTOP on hardware) | Cannot be proven on the simulator | Marked `HW`, not executed, listed in the report |

## Environment

- Simulator build of the debug app (`cargo build -p sweepscope-app`), `SWEEPSCOPE_NO_HARDWARE=1`, `SWEEPSCOPE_SIMULATOR=1`, USB directories masked with `unshare -rm` plus tmpfs, isolated `HOME`; a 6254BC is plugged in on this host and was never opened.
- GUI under Xvfb 1440 x 900 (extracted from .deb files, no sudo), openbox, Mesa software rendering, isolated HOME and XDG dirs.
- Control channel: a QA-only vite config in the scratchpad adds a bridge script so IPC and DOM can be driven in the real window (no product file changed).
- Playwright 1.65 on Chromium and WebKit, spare ports 5273 and 4273 (untracked `playwright.qa.config.ts`, deleted afterwards), IPC stub with scripted commands and events.
- Unit and backend suites: `cargo test --workspace`, `npx vitest run`, `npm run typecheck`.

## Test cases

Kinds: E2E = automated Playwright (file `frontend/e2e/m3-analyse.spec.ts` unless noted); REAL = manual in the real window; UI = visual check against the design spec; LIB = existing backend or library suite read and executed; HW = needs the real scope (human, not run).

### #22 Auto-setup
| ID | Given / when / then | Kind | Criterion | HW |
|---|---|---|---|---|
| T-22-1 | Given the simulator 1 kHz sine and a bad start (5 ms/div, 1 V/div), when Autoset runs, then 2 to 6 periods and 3 to 7 divisions result | REAL | AC1 | no |
| T-22-2 | Given periodic signals 10 Hz to 10 MHz, amplitudes 5 mV to 20 V, when the decision library runs, then every case ends `Done` in range (14.5 mV band excepted) | LIB (`cargo test -p autoset`, `-p acquisition` autoset tests) | AC1 | no |
| T-22-3 | Given a flat input, when Auto-setup runs, then a flat line, `No signal` tag and Toast | E2E (tag), LIB (backend sets `noSignal`, 1 V/div, 1 ms/div) | AC2 | shorted input H-07 |
| T-22-4 | Given no stable setting, when it fails, then the Toast says so with Retry and the previous setup stays | E2E (Toast), LIB (restore) | AC3 | no |
| T-22-5 | Given Autoset pressed while running, when it works, then no dialog opens, button busy, Run/Single/Force disabled with `Auto-setup is running` | REAL, E2E | AC4 | no |
| T-22-6 | Given the A key, when pressed, then Auto-setup starts | E2E | AC4 | no |
| T-22-7 | Given Auto-setup done, when the Toast shows twice with the same id, then one Toast | E2E | AC1 | no |

### #23 Sample rate and aliasing
| ID | Given / when / then | Kind | Criterion | HW |
|---|---|---|---|---|
| T-23-1 | Given 1, 2, 3, 4 channels at a fast time base, when the status bar is read, then Fs reads 1 GS/s, 500, 250, 250 MS/s and Record is shown, and both update on toggle | REAL | AC1 | no |
| T-23-2 | Given 1 kHz at 100 ms/div (2.5 kS/s), when measured, then `Alias CH1` shows in status bar, scale chip, measurement header (and footnote) | REAL, E2E | AC2 | H-09 |
| T-23-3 | Given the frequency below 0.95 x Fs/5 (time base back to 20 ms/div), when measured, then all four places clear together | REAL, E2E | AC3 | H-09 |
| T-23-4 | Given the warning, when hovered or read, then the tooltip is the plain-language backend text with the cause sentence | REAL, E2E | AC4 | no |

### #24 Single
| ID | Given / when / then | Kind | Criterion | HW |
|---|---|---|---|---|
| T-24-1 | Given Single pressed, when a trigger occurs, then one frame, phase Done, chips `Stop` and `Done` | REAL, E2E | AC1 | no |
| T-24-2 | Given 10, 50, 90 % position, when a trigger occurs, then the edge is within 8 samples | LIB (acquisition and device simulator tests) | AC2 | H-05 |
| T-24-3 | Given no trigger event (level above the signal), when waiting, then `Waiting for trigger`, Stop cancels, Single again cancels | REAL, E2E | AC3 | no |
| T-24-4 | Given a captured frame, when stopped, then it stays at full intensity until Run or Single | REAL | AC4 | no |
| T-24-5 | Given Done, when Run is pressed, then acquisition resumes | REAL | AC1 | no |

### #25 Pulse trigger
| ID | Given / when / then | Kind | Criterion | HW |
|---|---|---|---|---|
| T-25-1 | Given the Pulse type, when selected, then polarity, `> < = ≠` with labels and Width are offered | REAL, E2E | AC1 | no |
| T-25-2 | Given the simulated 500 us pulse train, when the ten H-06 cases are applied, then it triggers only on matching ones | REAL (invoke) | AC2 | H-06 |
| T-25-3 | Given an invalid width `abc`, when entered, then the field is invalid, error shows, previous value stays, Esc restores | REAL, E2E | AC3 | no |
| T-25-4 | Given the Video type, when the group is read, then disabled with `Exp` badge | E2E | AC4 | no |
| T-25-5 | Given a rejected width, when the error shows, then it appears once inside the dock | E2E | AC3 (design) | no |

### #26 Persistence
| ID | Given / when / then | Kind | Criterion | HW |
|---|---|---|---|---|
| T-26-1 | Given the Persist menu, when opened, then radio items Off, 0.5 s, 1 s, 2 s, Infinite and Clear | REAL, E2E | AC2 | no |
| T-26-2 | Given persistence on, when frames arrive, then ghosts accumulate and fade in 8 buckets | LIB (vitest, `graph.spec.ts` pixel tests) plus REAL (tag and setting) | AC1 | no |
| T-26-3 | Given persistence, when cursors or measurements are used, then they still work | REAL | AC3 | no |
| T-26-4 | Given a one-sample glitch at 512 px, when drawn, then it stays visible | E2E (`graph.spec.ts`) | AC4 | no |
| T-26-5 | Given 4 channels, Infinite, 20 fps, when drawn, then no frame over 33 ms | E2E (informative) | budget | no |
| T-26-6 | Given the Persist menu, when chosen, then `set_persistence` is sent | E2E | AC2 | no |

### #27 Roll
| ID | Given / when / then | Kind | Criterion | HW |
|---|---|---|---|---|
| T-27-1 | Given 500 ms/div, when running, then the trace scrolls and the `now` tag shows | REAL | AC1 | H-01 |
| T-27-2 | Given the host stops and the backend reports an overrun, when it resumes, then a `GAP n s` band, aria-live text and the trace continuing after it | REAL (SIGSTOP 40 s) | AC2 | H-02 |
| T-27-3 | Given roll, when the controls are read, then trigger, Force, Single, Autoset, XY, FFT, Persist, Math, Cursors are disabled with the backend reason as visible text | REAL, E2E | AC3 | H-01 |
| T-27-4 | Given a roll hour, when memory is observed, then constant | LIB (RollWindow unit, device 20-cycle leak test); soak is HW | AC4 | H-03 |
| T-27-5 | Given roll, when the page opens, then the roll stream is opened and items scroll | REAL, E2E | AC1 | no |
| T-27-6 | Given roll, when the toolbar is measured at 1440 and 1024 px, then no overflow | REAL, E2E | layout | no |

### #28 Recording
| ID | Given / when / then | Kind | Criterion | HW |
|---|---|---|---|---|
| T-28-1 | Given roll and Record, when a file is chosen in the native dialog, then chip `Rec 00:00:03 · size` and bytes grow | REAL | AC1 | no |
| T-28-2 | Given hours, when memory is observed, then constant | LIB (bounded queue test) ; hour soak HW | AC2 | H-03 |
| T-28-3 | Given the file is removed while recording, when it happens, then recording stops within 2 s, Banner with reason and size, Choose new file and Dismiss, display keeps rolling | REAL | AC3 | H-04 |
| T-28-4 | Given the disk is full, when it happens, then the same | LIB (failing writer) | AC3 | H-04 |
| T-28-5 | Given the file, when read by `tools/roll-record-reader.py`, then header, segments, contiguous samples, gap record, end record | REAL | AC4 | H-03 |
| T-28-6 | Given a setup change while recording, when it happens, then a new segment starts | LIB | AC4 | no |

### #29 FFT
| ID | Given / when / then | Kind | Criterion | HW |
|---|---|---|---|---|
| T-29-1 | Given the FFT view, when shown, then dBV or Linear and four windows are offered and the backend applies them | REAL | AC1 | no |
| T-29-2 | Given a spectrum, when the peak marker is Auto, Manual (click), keyboard (arrows, Shift = 10 bins), then frequency and level read the same in marker, tag, panel | REAL | AC2 | no |
| T-29-3 | Given the frequency axis, when labelled, then bin spacing equals Fs / 4096 | REAL (12.21 Hz at 50 kS/s, 305.2 Hz at 1.25 MS/s) | AC3 | H-08 |
| T-29-4 | Given a 1 kHz square, when displayed, then odd harmonics at 3, 5, 7 kHz | LIB (`cargo test -p fft`, `-p spectrum`) | AC4 | H-08 |
| T-29-5 | Given Yt, when FFT is chosen and left, then `set_spectrum` active true then false | E2E | AC1 | no |
| T-29-6 | Given the FFT view, when axe runs, then no WCAG AA violation | E2E | design | no |
| T-29-7 | Given the peak marker, when drawn, then it sits on the peak | UI | design | no |

### #30 Math
| ID | Given / when / then | Kind | Criterion | HW |
|---|---|---|---|---|
| T-30-1 | Given CH1 and CH2, when M is enabled, then a trace with its own scale and position shows; four operations are offered | REAL | AC1 | H-10 |
| T-30-2 | Given a math channel, when measured, then column M has twelve rows like a channel | REAL | AC2 | no |
| T-30-3 | Given CH2 off, when math is shown, then the chip says `unavailable: CH2 is off` | REAL, E2E | AC3 | H-10 |
| T-30-4 | Given sources on different scales, when math is on, then `Mixed` badge and an explicit scale | REAL, E2E | AC4 | no |
| T-30-5 | Given the M toggle, when pressed, then `set_math` is sent | E2E | AC1 | no |

### #31 XY
| ID | Given / when / then | Kind | Criterion | HW |
|---|---|---|---|---|
| T-31-1 | Given two channels, when XY is chosen, then X against Y with scale tags `X CH1 ...` and `Y CH2 ...`; equal signals draw a diagonal | REAL, E2E | AC1 | no |
| T-31-2 | Given a channel off in XY, when read, then the ScreenNote names channel and axis with `Enable CHn`; one channel gives `XY needs two enabled channels` | REAL, E2E | AC2 | no |
| T-31-3 | Given XY then Yt, when settings are compared, then unchanged and no `set_trigger` | E2E | AC3 | no |
| T-31-4 | Given XY, when the axis selectors are read, then the other axis channel is disabled | E2E | design | no |
| T-31-5 | Given XY, when Persist and Cursors are read, then disabled | E2E | design | no |

### Cross-cutting
| ID | Given / when / then | Kind |
|---|---|---|
| T-X-1 | Given the real window, when each new command and both new streams are called, then no capability error (BUG-1 class) | REAL |
| T-X-2 | Given the real window, when the live app is built, then every analysis control sends its command and every feed reaches the model | REAL, E2E |
| T-X-3 | Given the full suites, when run, then no regression: `cargo test --workspace`, vitest, typecheck, Playwright chromium and webkit | automated |
| T-X-4 | Given the layout floor 1024 x 720, when Yt, XY, FFT, roll are shown, then toolbar and dock fit | REAL |
| T-X-5 | Given the working tree after `cargo test --workspace`, when checked, then it is clean | automated |
| T-X-6 | Given hardware list H-01 to H-10, when this plan is executed, then none is run by QA | HW (human) |

## Entry and exit criteria

Entry: branch integrated, `cargo test --workspace`, vitest and typecheck green (reported by the PM; re-run by QA).
Exit: every case executed or listed under "not tested", every bug sent to its developer, report written. Verdict `pass` only with no open blocker or high bug; hardware items stay open for the human by design.
