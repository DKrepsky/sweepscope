# QA report: M3 Analyse (#22 to #31)

Build: branch `feat/m3-analyse`, commit 96cef59 (debug build of `sweepscope-app`, simulator).
Date: 2026-11-24.
Test plan: [test-plan.md](test-plan.md). Evidence (not committed): `.claude/tmp/m3-analyse/`.

## Summary

**Verdict: fail.** One blocker (BUG-M3-1) stops release; five further bugs are open (two medium, three low).

The blocker: the live app is built without its analysis wiring. `frontend/src/main.ts` calls `createLiveApp` without the `analysis` dependency, so the model gets `inertAnalysisSources()` and `inertAnalyseCommands`. In the real window **Single, Autoset (button and `A`), Persist, Math, Record, and the FFT view's data do nothing, and roll mode shows no trace**, although the backend, the IPC capabilities and both new streams work. Unit tests (1249) and Rust tests (996) cannot see this. Everything below marked "with QA shim" was verified only after I wired the missing sources and commands through a QA-only overlay in the scratchpad (no product file changed). Those results show what works once BUG-M3-1 is fixed; they must be retested against the real fix.

What passes in the product as wired: alias warning (four places), Pulse trigger, XY view, step and reset of the new targets, backend commands and capabilities (checked with direct IPC calls in the real window).

Counts: `cargo test --workspace` 996 passed, 0 failed, tree clean afterwards. `npx vitest run` 1249 passed (100 files). `npm run typecheck` clean. Playwright (Chromium and WebKit, spare ports): 552 passed, 1 skipped, 1 failed (the WebKit info Toast timing test, flaky under load: it passed 4 of 4 times alone with one worker). The baseline run before my changes had 3 failures: the gallery token test in both browsers (test bug, fixed, see below) and the 33 ms frame budget probe (35 ms under three parallel workers; passed in the later full run and in the isolated rerun).

## Results

Kinds: REAL = real Tauri window under Xvfb (USB hidden, `SWEEPSCOPE_NO_HARDWARE=1`); SHIM = real window with the QA wiring overlay; E2E = `frontend/e2e/m3-analyse.spec.ts` (42 cases, both browsers; cases that document a bug use `test.fail` and flip red when fixed); LIB = existing Rust or vitest suites, read and executed.

### #22 Auto-setup
| ID | Result | Notes |
|---|---|---|
| T-22-1 | Pass (SHIM) | From 5 ms/div and a wrong range the run ended in 1.3 s at 500 us/div and 500 mV/div: 5 periods, 4.1 divisions. Toast `Auto-setup done. CH1 500 mV/div, CH2 500 mV/div, 500 us/div.` Without the shim the button does nothing (BUG-M3-1). The #22 AC1 reading (3 to 7 divisions where the range table allows) is the human-approved one. |
| T-22-2 | Pass (LIB) | Seven frequencies and amplitudes, 14.5 mV band, clipped start, 3 Hz fail, 14 probes: all autoset tests green. |
| T-22-3 | Fail (partly) | Toast pass (E2E). The `No signal` tag is never rendered (BUG-M3-6; E2E case is `test.fail`). Backend flag, 1 V/div and 1 ms/div pass (LIB). Shorted-input behaviour is H-07. |
| T-22-4 | Pass (E2E, LIB) | Failed outcome Toast text, `Retry`; backend restores the previous setup (LIB). The `Retry` click is not covered live (BUG-M3-1). |
| T-22-5 | Pass (SHIM, E2E) | `aria-busy`, Run and Single disabled, `Auto-setup is running`, zero dialogs. |
| T-22-6 | Fail (E2E, `test.fail`) | `A` key and the button send no `start_autoset` (BUG-M3-1). |
| T-22-7 | Pass (E2E) | Same outcome id twice gives one Toast. |

### #23 Sample rate and aliasing
| ID | Result | Notes |
|---|---|---|
| T-23-1 | Pass (REAL) | At a fast time base: 1 channel 1.00 GS/s, 2 channels 500 MS/s, 3 and 4 channels 250 MS/s; Record shown; both update on each toggle (also back down). |
| T-23-2 | Pass (REAL, E2E) | 1 kHz at 100 ms/div (2.50 kS/s): `Fs 2.50 kS/s Alias CH1` in the status bar, `Alias` badge in the scale chip and the CH1 measurement header, `[1]` footnote on Frequency and Period with the backend text including the time-base sentence. No alias at 20 ms/div. |
| T-23-3 | Pass (REAL, E2E) | Back at 20 ms/div the warning list is empty and all four places clear together. Hysteresis band not exercised live (LIB covers it). |
| T-23-4 | Pass (REAL, E2E) | Tooltip is the plain-language backend message (status bar `title`, badges `aria-describedby`). A real pointer hover was not performed. |

### #24 Single
| ID | Result | Notes |
|---|---|---|
| T-24-1 | Pass (SHIM) | Armed, triggered, phase `done`, chips `Stop` and `Done`, one frame. Without the shim Single does nothing (BUG-M3-1). |
| T-24-2 | Pass on simulator, hardware open (H-05) | Nine cases (1, 2, 4 channels x 10, 50, 90 %): header trigger index equals the chosen position exactly (250, 1250, 2250 of 2501); the rising edge found in the data is 3 to 4 samples before it (limit 8). |
| T-24-3 | Pass (SHIM, E2E) | Level above the signal: phase `waitingForTrigger`, chip `Waiting for trigger`, Single pressed; Stop and a second Single press cancel at once. |
| T-24-4 | Pass (SHIM) | After `Done` the frame stays on screen at full intensity (`s08-single-armed.png`). |
| T-24-5 | Pass (SHIM) | Run after `Done` resumes (phase `triggered`). |

### #25 Pulse trigger
| ID | Result | Notes |
|---|---|---|
| T-25-1 | Pass (REAL, E2E) | Polarity (`Positive pulse`, `Negative pulse`), `>`, `<`, `=`, `≠` with names and tooltips, Width field. |
| T-25-2 | Pass (REAL via IPC) | All ten H-06 cases on the simulator's 500 us pulses: `> 400`, `< 600`, `= 500`, `= 480`, `= 520`, `≠ 200` trigger; `> 600`, `< 400`, `= 450`, `≠ 500` wait. Real hardware H-06 open. |
| T-25-3 | Pass with BUG-M3-4 | `abc`: field `aria-invalid`, previous value 500 kept, Escape restores. The error text is drawn twice and one copy crosses into the sidebar. |
| T-25-4 | Pass (E2E) | Video is disabled and has the `Exp` badge. |
| T-25-5 | Fail (E2E, `test.fail`) | BUG-M3-4. |

### #26 Persistence
| ID | Result | Notes |
|---|---|---|
| T-26-1 | Pass (REAL, E2E) | Menu: radio items `Off`, `0.5 s`, `1 s`, `2 s`, `Infinite` and `Clear persistence`. |
| T-26-2 | Partial | SHIM: choosing `1 s` sets the backend value, button reads `Persist: 1 s` (on look), screen tag `PERSIST 1 s`, readout `PERSISTENCE 1 s`. Ghost accumulation and 8-bucket ageing rest on the pixel tests in `graph.spec.ts` (green in both browsers); I did not watch ghosts fade live. The design's muted scale-strip note (`Persistence builds from successive frames...`) is not present (design 5.1, low); `UPDATE n /s` is the known unbuilt item. |
| T-26-3 | Not run | Cursors and measurements with persistence on were not exercised live. |
| T-26-4 | Pass (E2E) | One-sample glitch at 512 px, live layer and ghost (`graph.spec.ts`). |
| T-26-5 | Pass (informative) | Four channels, Infinite, 60 frames: slowest 35 ms in the baseline run under load, within 33 ms in the later runs. Software rendering, not a product claim. |
| T-26-6 | Fail (E2E, `test.fail`) | Menu choice sends no `set_persistence` (BUG-M3-1). |

### #27 Roll
| ID | Result | Notes |
|---|---|---|
| T-27-1 | Fail without shim, pass with shim | REAL: roll shows an empty screen with only the `now` tag; the backend streams 60 `SSL1` items in 3 s through `open_roll_stream` (BUG-M3-1). SHIM: trace drawn and scrolling (the simulator's 1 kHz sine aliases to a constant at 2 ms sampling, so the line is flat; scrolling shows in the moving gap band). |
| T-27-2 | Pass (SHIM) | `SIGSTOP` of the app for 40 s: band `GAP 40.0 s` with dashed amber edges, polite announcement `Data gap of 40.0 s`, trace continues after it, gap scrolls out after about 5 s. The recorded file holds the matching gap record (`missed 22509` for a 45 s stop). The simulator does report overruns. `Gaps n` in the status bar is the known unbuilt item. |
| T-27-3 | Fail (BUG-M3-2) | Force, trigger group, Single, Autoset, XY, FFT, Persist, Math are disabled and the exact backend reason is visible text under the Trigger group; measurements say `not available in roll mode`. Cursors is enabled once roll data is drawn. |
| T-27-4 | LIB, hardware open (H-03) | RollWindow constant size and device 20-cycle thread test pass. One-hour soak and the two-condition RSS rule are for the human. |
| T-27-5 | Fail (E2E, `test.fail`) | `open_roll_stream` is never called by the page (BUG-M3-1). |
| T-27-6 | Fail (E2E, `test.fail`; REAL) | BUG-M3-3, toolbar overflow. |

### #28 Recording
| ID | Result | Notes |
|---|---|---|
| T-28-1 | Pass (SHIM) | Record opens the native Save dialog; chip `REC 00:00:03 · 4.26 KIB`, then `REC 00:01:02 · 40.4 KIB`; button becomes `Stop rec`. Without the shim Record does nothing. The chip says KiB, the Banner says kB, the spec example says kB (low). |
| T-28-2 | LIB, hardware open (H-03) | Bounded queue test. |
| T-28-3 | Pass (SHIM) for a removed file | Deleting the file while recording: within 4 s the slice has `failure` `the file was removed`, error Banner `Recording stopped: the file was removed. 15.2 kB written to test1.ssroll. The display is still running.` with `Choose new file` and `Dismiss`; roll keeps scrolling. |
| T-28-4 | LIB, hardware open (H-04) | Disk full with the failing writer. |
| T-28-5 | Pass | `tools/roll-record-reader.py` reads the file: one segment (four channels, probe, volts per div), 711 sample records, contiguous indices except one gap record, an `end` record `stopped`. |
| T-28-6 | LIB | New segment on a setup change. |

### #29 FFT
| ID | Result | Notes |
|---|---|---|
| T-29-1 | Pass (SHIM) | dBV and Linear; Rectangular, Hann, Hamming, Blackman all apply in the backend; axis, top label, readout, screen tag follow. |
| T-29-2 | Partial | Auto marker on the peak; a click on the spectrum switched to Manual and read bin 883, 10.78 kHz, -75.9 dBV (marker, tag and panel equal). The arrow keys did not move the marker because focus stayed on the scope screen after the click; keyboard moves were not verified. For a 1.000 kHz 707 mV signal at Fs 1.25 MS/s with Blackman the Auto peak reads bin 4, 1.221 kHz, 541 mV (-2.3 dB): the lobe skip excludes bin 3. With Hann or at 50 kS/s the peak is 1.001 kHz, -3.1 dBV. See O-3. |
| T-29-3 | Pass (SHIM) | Bin spacing 305.2 Hz at 1.25 MS/s and 12.21 Hz at 50 kS/s, equal to Fs / 4096, shown in the readout, the Horizontal readline and the screen tag. |
| T-29-4 | Pass (LIB), hardware open (H-08) | Square-wave harmonic test in `fft`. The simulator offers only a sine, so no harmonics were seen live. |
| T-29-5 | Fail (E2E, `test.fail`) | `set_spectrum` is not sent and the stream not opened by the unwired page (BUG-M3-1). |
| T-29-6 | Fail (E2E, `test.fail`) | BUG-M3-5. XY and roll pass axe. |
| T-29-7 | Fail (low) | BUG-M3-7, marker glyph about 9 px left of the peak line. |

### #30 Math
| ID | Result | Notes |
|---|---|---|
| T-30-1 | Pass (SHIM) | M on with CH1 and CH2 shows a trace with its own `500 mV/div`; menu offers A+B, A-B, A x B, Invert A (only A-B run live). |
| T-30-2 | Pass (SHIM) | Column M with the twelve rows; flat A-B reads Vpp 24.8 mV noise. With four columns the values and units wrap to two lines (O-4). |
| T-30-3 | Pass (SHIM, E2E) | CH2 off: chip `M  A−B  unavailable: CH2 is off`, `UNAVAILABLE CH2 IS OFF` badge. |
| T-30-4 | Pass (SHIM, E2E) | CH2 at 200 mV/div: `MIXED` badge, backend detail `CH1 500 mV/div, CH2 200 mV/div. The math trace has its own scale, 500 mV/div.`, unit V. |
| T-30-5 | Fail (E2E, `test.fail`) | The M toggle sends no `set_math` (BUG-M3-1). |

### #31 XY
| ID | Result | Notes |
|---|---|---|
| T-31-1 | Pass (REAL, E2E) | Equal signals draw a diagonal; tags `Y CH2 ...` and `X CH1 ...`; role tags `X`, `Y` on the scale chips; readout `XY | X CH1 | Y CH2 | TIME BASE`. |
| T-31-2 | Pass (REAL, E2E) | CH2 off with CH3 on: `XY cannot draw`, `CH2 (Y) is off.`, button `Enable CH2` restores it; one channel: `XY needs two enabled channels`, `Only CH1 is on. Enable another channel.` |
| T-31-3 | Pass (E2E) | XY then Yt leaves the dock text identical and sends no `set_trigger`. |
| T-31-4 | Pass (E2E) | The other axis' channel is disabled in each selector. |
| T-31-5 | Pass (E2E) | Persist and Cursors disabled in XY. |

### Cross-cutting
| ID | Result | Notes |
|---|---|---|
| T-X-1 | Pass (REAL) | Called in the real window with no capability error (also none in the vite log): `arm_single`, `start_autoset`, `set_math`, `set_persistence`, `set_spectrum`, `start_recording`, `stop_recording`, `open_roll_stream`, `open_spectrum_stream`, `step_setting` and `reset_setting` on `mathScale` and `mathPosition`, `set_trigger` with pulse fields. `close_roll_stream` and `close_spectrum_stream` were not called explicitly. |
| T-X-2 | Fail | BUG-M3-1. |
| T-X-3 | Pass | Counts in the summary. |
| T-X-4 | Fail | BUG-M3-3: at 1024 x 720 Yt and FFT fit, roll overflows by 48 px (Screenshot button hidden behind Settings). Dock height tiers were not re-measured by QA (`tiers.spec.ts` passes in both browsers). |
| T-X-5 | Pass | `git status` clean of generated files after `cargo test --workspace`. |
| T-X-6 | Open | Hardware items H-01 to H-10 are for the human. |

## Bugs
| Bug | Severity | Owner | Status |
|---|---|---|---|
| BUG-M3-1 Live app never wires the analysis sources and commands: roll, FFT data, recording state, Single, Autoset, Persist, Math, Record are dead in the real window (e2e tag BUG-21) | **Blocker** | developer-8 | Open, sent on the bus |
| BUG-M3-6 `No signal` tag never rendered, only the Toast (e2e BUG-26) | Medium | developer-10 | Open, sent |
| BUG-M3-3 Toolbar wider than the window in roll mode (1440: File clipped to `Fil`, Screenshot lost; 1024: Screenshot behind Settings) and 11 px wider in Yt at 1440, which lets the UI shift when the screen takes focus (e2e BUG-23) | Medium | developer-10 | Open, sent |
| BUG-M3-2 Cursors enabled in roll mode while data is drawn (e2e case passes today only because of BUG-M3-1; found with the shim) | Medium | developer-10 | Open, sent |
| BUG-M3-4 Invalid pulse width error drawn twice, one copy overflows into the sidebar (e2e BUG-24) | Low | developer-8 | Open, sent |
| BUG-M3-5 `SW` badge in the selected FFT segment contrast 3.93 (needs 4.5) (e2e BUG-25) | Low | developer-10 | Open, sent |
| BUG-M3-7 Peak marker glyph about 9 px left of the peak line | Low | developer-9 | Open, sent |

Fixed by QA (test code): `frontend/e2e/gallery.spec.ts` compared every colour token to a hex value; the new translucent token `color-gap-fill` made the test fail in both browsers. The test now skips it in the exact comparison, like the overlay token.

### Observations and known items (no bug filed)
- O-1 Known item confirmed: the done text uses `us`, the design says `µs` (`Auto-setup done. CH1 500 mV/div, ..., 500 us/div.`). Low.
- O-2 Known items confirmed absent: `Gaps n` in the status bar, `UPDATE n /s` readout. Single's pressed state is inferred from phase and worked in both cases tested. Math changed while stopped was not exercised.
- O-3 Auto peak skips the window's main-lobe bins (Blackman: bins 0 to 3). At 305 Hz bin spacing (the H-08 setup with the CAL signal at 200 us/div) a 1 kHz fundamental sits in bin 3, so with Blackman the marker reads the next bin and about 2 dB low. For the human to judge at H-08.
- O-4 The M column makes the Measurements panel wrap values and units to two lines with four columns.
- O-5 Trigger guides (`T1 ↗ 0.00 V`, trigger position tag) stay on screen in roll mode where the trigger is unavailable.
- O-6 Persist menu: the popup stayed open after the view switched to XY in one run (s07-xy-pulse.png); a synthetic Escape did not close it. Not reproduced with a real key press, so not filed.
- O-7 The Measurements header reads `FROZEN` after a finished Single, while the design says a held frame is shown at full intensity (the graph is). Wording only.

## Not tested and why
- All hardware items H-01 to H-10 (agents never touch the scope; a 6254BC is plugged in on this host and stayed masked).
- Auto-setup on 10 Hz to 10 MHz and the shorted or open input: the simulator signal in the app is fixed at 1 kHz; covered by the lib and backend tests only.
- Disk-full recording and the one-hour soaks (RSS rule of #27, #28): not feasible here; lib tests only.
- Single position on real hardware; pulse `≠` edge at 480 and 520 us; FFT harmonics of the CAL square.
- Persistence ghosts fading live, Cursors and measurements with persistence on (T-26-3), arrow-key moves of the peak marker, A x B and Invert A live, M in the FFT, Math changed while stopped, Persist menu with a real pointer hover, `close_*_stream` commands.
- Real display session and WebKitGTK behaviour beyond Xvfb software rendering; text scale 150 and 200 % on the new rows.
- Everything marked SHIM must be repeated after BUG-M3-1 is fixed, on the real wiring.

## Screenshots and logs
Kept in `.claude/tmp/m3-analyse/` (not committed): `s02` to `s25` screenshots (`s05-inert`, `s04-roll-c`, `s06-backend-driven` show BUG-M3-1; `s16-gap-a` the gap band; `s20-roll-1024` the toolbar overflow; `s24-pulse-abc` the duplicate error), `cargo-test.log`, `e2e-baseline.log`, `e2e-full.log`, `e2e-m3-run*.log`, app and vite logs.
