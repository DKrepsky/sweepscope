# Spike #4: roll-mode streaming path

Issue #4. Spec: [m0-spike implementation spec](../../specs/m0-spike/implementation-spec.md) section 2.6. Policy: [ADR 0008](../../adr/0008-spike-policy-and-measurement-reports.md).
Status: measured, decision proposed, waiting for the human (it touches a pass rule).

## 1. Question and decision rule

Can a roll stream (500 ms/div, four channels) run for one hour with constant Rust memory (ring buffer), a frontend that keeps only the visible
window, a recording file that grows while process memory does not, and gaps (overruns) that reach the frontend as data? Is the pattern
"poller, ring buffer, Tauri `Channel` of binary items" accepted for the roll story (#27)?

Rules fixed before measuring (spec 2.6):

| Rule | Value |
|---|---|
| Rust RSS growth after the first 5 minutes | at most 2 MiB over the remaining time |
| WebKitWebProcess RSS growth after the first 5 minutes | at most 10 MiB |
| Recorded file | grows linearly; final size within 1 % of expectation; every injected gap present as a gap record |
| Gaps reaching the frontend | every injected gap seen once, at the right absolute index |
| Items | no sequence number skipped on the Channel |
| CPU | poller and recorder below 2 % of one core (informative) |

## 2. Method

- Spike branch `spike/4-roll-streaming`, tag `spike-4-final` (local until pushed), tip `1a5ffb5`. Code: `spikes/4-roll-streaming/` (standalone workspace, uses the #3 simulator by path).
- Acceptance run: `spikes/4-roll-streaming/tools/soak.sh app`. The Tauri app under Xvfb (1280x720, software rendering), isolated HOME and XDG directories,
  real time (speed 1), 3600 s, simulator with four channels at 500 ms/div (2 ms interval, 499 samples per second per channel), poll every 100 ms,
  `inject_overrun()` every 600 s.
- Items: 32 byte header plus `u8` codes, about 2 KB per item, 10 items per second (about 2.3 KB/s). A `Channel` carries them, so each item takes the
  large-payload path of Tauri (spec 2.6; payloads over 1 KiB go through fetch, not eval).
- `memory.csv`: every 10 s, RSS of the main process (`/proc/self/statm`, Rust side and the GTK/WebKit UI-process code in the same process), RSS of
  `WebKitWebProcess`, ring fill, file size. `check.json` applies the memory rules from the first sample at or after 300 s.
- Frontend: `RollWindow` keeps only the visible span (5 s, 2500 samples) in absolute sample indices and logs every gap marker with its x position.
- Sequence numbers and recorded file are checked by `verify_recording` (`recording-summary.json`).
- Known limits: one machine, software rendering under Xvfb (no GPU path, which makes the UI-process cost pessimistic or different from a real display); one
  run; the simulator, not hardware; the missed-sample count of a gap is an estimate from elapsed time (the driver cannot do better, PRD 5.2). Rust RSS
  includes the whole UI process (WebKitGTK UI-process code, GTK), not only our code. CPU was not recorded (informative rule, see section 5). The natural
  overrun (poller paused longer than 95 % of the buffer span) is covered by a unit test (`poller.rs`), not by this run; all six gaps here are injected.

## 3. Environments

| machine-id | OS and kernel | CPU | Display | WebKitGTK, Tauri |
|---|---|---|---|---|
| `i5-9600kf-xvfb-software` | Linux Mint 22.3, 7.0.0-43 | i5-9600KF 6 cores, 32 GiB | Xvfb 1280x720x24, software GL, X11 backend, DMABUF renderer off | 2.52.6, 2.12.7 |

Full record: [environment.json](results/4/i5-9600kf-xvfb-software/environment.json). Raw data in
[results/4/i5-9600kf-xvfb-software/](results/4/i5-9600kf-xvfb-software/): `check.json`, `recording-summary.json`, `memory.csv` (360 rows),
`frontend-gaps.json` (the recording path was removed, it contained a local path).

## 4. Results

### 4.1 Memory (samples from 300 s to 3590 s, KiB)

| Process | at 300 s | at 3590 s | growth | min | median | p95 | max |
|---|---|---|---|---|---|---|---|
| Rust (main process) | 177,304 | 180,704 | **3,400** (3.3 MiB) | 177,304 | 179,066 | 180,593 | 182,348 |
| WebKitWebProcess | 202,108 | 209,864 | 7,756 (7.6 MiB) | 202,108 | 210,366 | 213,868 | 216,572 |

Rust RSS slope (least squares, KiB per hour) by period: 300 to 600 s 3,871; 600 to 1800 s 3,735; 1800 to 3600 s 3,636. The WebKit slope falls from
39,695 (300 to 600 s) to 8,361 (600 to 1800 s) and 4,346 (1800 to 3600 s): it settles, which is garbage collector and cache behaviour.
The Rust growth does not settle: median step +12 KiB per 10 s, the same in every period, about 1.2 KiB/s, or about 101 bytes per item (36,001 items).
Before 300 s the process grew 7.2 MiB (start-up, ring and window allocation, first draws).

### 4.2 Ring buffer and frontend window

| Check | Result |
|---|---|
| Ring fill | reaches 300,000 samples per channel at 600 s and stays at 300,000 for the other 3,000 s |
| Frontend | keeps the visible window only (spec 2.6: ring of 2500 samples per channel); WebKit RSS settles (4.1) |

### 4.3 Recorded file

| Check | Expected | Measured |
|---|---|---|
| Final size | 8,350,836 bytes | 8,350,836 bytes (exact) |
| Growth | linear | 2,319.7 B/s, largest deviation from the line 395 bytes |
| Sample records, gap records, close records | | 35,994, 6, 1 |
| Samples per channel delivered plus missed | 1,800,000 | 1,799,695 + 305 estimated = 1,800,000, index continuous |
| Process memory while the file grew by 8.3 MB | flat | the file is not in RSS (page cache); Rust growth is 4.1 |

### 4.4 Gaps

| # | `after` (index) | `missed` (samples, estimate) | start_roll result | recording | frontend marker | x in window when it appeared |
|---|---|---|---|---|---|---|
| 1 | 299,950 | 50 | same | same | same | 0.9800 |
| 2 | 599,949 | 51 | same | same | same | 0.9796 |
| 3 | 899,949 | 51 | same | same | same | 0.9796 |
| 4 | 1,199,949 | 51 | same | same | same | 0.9796 |
| 5 | 1,499,949 | 51 | same | same | same | 0.9796 |
| 6 | 1,799,949 | 51 | same | same | same | 0.9796 |

One gap is about 100 ms (50 or 51 samples at 2 ms): the time since the last poll. Items: 36,001, **0 skipped sequence numbers**.

### 4.5 Pass rules

| Rule | Result |
|---|---|
| Rust RSS growth at most 2 MiB | **FAIL**: 3.3 MiB |
| WebKit RSS growth at most 10 MiB | pass: 7.6 MiB |
| File linear, size within 1 %, gaps in the file | pass: exact size, 6 of 6 gap records |
| Gaps in the frontend, right index | pass: 6 of 6, identical |
| No skipped sequence number | pass |
| CPU below 2 % | not measured |

### 4.6 Accelerated and headless runs (smoke, not acceptance)

| Run | Items | Rust RSS growth | Note |
|---|---|---|---|
| Headless backend, 60x clock, one simulated hour in about 1 minute | 35,915 | 64 KiB | no WebView; 12 samples, window of seconds |
| App under Xvfb, 60x clock | 36,001 | 160 KiB | 6 samples, growth over 45 s |

Both runs deliver the same number of items as the real-time hour in about a minute, with almost no growth. These runs are short in wall time, so they are
not comparable to the hour as a leak test. They do show that growth is **not proportional to the number of items sent** (36k items cost 3.4 MiB in
one case and 0.16 MiB in the other); it follows wall time. The earlier reading "headless 64 KiB against app 3.4 MiB for the same items" compares a
60 s run with a 3600 s run, so it does not isolate the Channel (finding 3).

## 5. Findings

1. **Constant memory by construction works.** The ring stays at its capacity from minute 10 on; the frontend window is bounded; the file grows 8.3 MB with exact size.
2. **Gaps travel as data and survive every hop.** The six gaps are identical in `start_roll` result, recording and frontend markers, at the right absolute index.
   The chunk after a gap starts at `after + missed`, so the time axis stays continuous.
3. **The Rust RSS growth is real, linear in wall time, small, and not yet attributed.** 3.3 MiB per hour (1.2 KiB/s), steady in every period, no step larger than
   3 MiB. It is not per item (4.6). The measurement cannot tell the cause: Rust RSS here includes the whole UI process under software rendering (painting,
   WebKit UI-process caches, allocator fragmentation), the poller, the sampler and the Channel. The headless comparison available is accelerated, so it
   proves nothing about wall-time growth. What would settle it is a real-time headless run (section 7). Extrapolated: about 85 MiB per day on a 180 MiB
   process; a leak worth finding, not one that stops a roll story.
4. **The 5 minute exclusion is too short for a process that is still allocating.** The process grew 7.2 MiB before 300 s and the ring only filled at 600 s. For a ring of 10
   minutes the baseline must be taken after the ring is full. This does not change the result (growth after 600 s is 3.1 MiB, still above 2 MiB).
5. **Natural overrun and CPU were not part of this run.** Natural overrun is unit tested; the CPU rule is informative and was not recorded.

## 6. Decision

**The pattern (poller thread, ring buffer with absolute sample indices, ordered binary items with sequence numbers on a Tauri `Channel`, gap items
carrying `after` and an estimated `missed`, bounded recorder queue that writes gap records) is accepted for the roll story (#27), with one condition
on the Rust RSS rule.**

Reasons: every functional rule passed with zero loss over 36,001 items; the WebKit rule passed; the failing rule fails by 1.4 MiB in an hour at a rate of
1.2 KiB/s, flat in shape, with no sign of per-item cost, and the cause is more likely the UI process than the pattern. Rejecting a pattern whose only
defect is unattributed 1 KiB/s of process growth would be disproportionate; accepting it silently would break ADR 0008 point 7.

**The rule is not changed to fit the result.** Options for the human (the failed rule is the human's decision, ADR 0008 point 8):

| Option | What | For | Against |
|---|---|---|---|
| A (recommended) | Accept with condition: keep the 2 MiB rule as written; #27's definition of done repeats the one-hour soak and requires either growth within the rule or a documented cause outside our code. Before M3 a 25 minute real-time headless run (section 7) is made to attribute the growth | Honest, cheap, no rule bent | Roll story carries a risk until then |
| B | Accept and change the rule to a rate: at most 4 MiB per hour measured after the ring is full (about 1.1 KiB/s), with the exclusion window moved to 15 minutes | Rule matches the shape of the data; usable as regression test | Loosened after seeing the result; the rate hides a real leak of this size |
| C | Do not accept yet; run the attribution first (section 7, 25 to 60 minutes of unattended time), then decide | Decision rests on a cause | Delay of an hour or less; the answer may be inconclusive again |

Recommendation: A, plus the section 7 run done now because it costs no human time. If that run shows the headless backend growing at the same rate, the pattern
(not the UI) leaks and the decision becomes C with a fix in the roll story before acceptance.

## 7. Changes requested and open points

Changes requested (proposals, none applied):

1. **Gap item in the product stream** (ADR to be written after the human approves this report): keep the layout of spec 2.6 (kind, sequence, absolute index, `after`,
   `missed` as estimate); the roll story's requirements must say the size is an estimate (PRD 5.2) and the marker is drawn as an uncertain band.
2. **PRD 8.3 'Memory in roll recording'**: state the baseline point as "after the ring is full" and the Rust rule as agreed in section 6.
3. No change to the driver or to ADR 0007: `poll_roll` plus `Overrun` was enough.

Open points:

- **Attribution run (no human needed, about 25 minutes unattended, any session, headless):** run the backend in real time without WebView and with the
  same simulator and poll period: `tools/soak.sh real` with `SPIKE_DURATION_S=1500` and, if the script cannot take the override, a copy of its `roll_headless` call.
  Compare the Rust RSS slope from the moment the ring is full (at 600 s in a 3600 s run; for a short run use a ring sized to fill in 3 minutes). A second
  run of the app with the Channel send disabled would split the Channel from the UI process; needs Xvfb only. Both are developer-2 work.
- CPU of poller and recorder was not recorded; add it to the attribution runs.
- Hardware: roll with the real device is verified nowhere in the driver (`poll_roll` timing, overrun detection by elapsed time); first M3 hardware check.

## 8. How a gap marker is placed on screen

A gap item carries `after` (absolute index of the last sample before the gap, exact) and `missed` (samples lost, estimate). The frontend keeps the window in
absolute sample indices: `start = max(0, head - windowSamples)`.

```
x(index)      = (index - start) / windowSamples          // 0..1 inside the view, outside it when scrolled off
marker.x      = x(after)                                 // left edge of the band
marker.width  = missed / windowSamples                   // right edge x(after + missed)
```

In pixels, `x_px = (index - start) / samplesPerPixel`, band width `missed / samplesPerPixel` (about 100 ms, so 50 samples, 2 percent of a 5 s window).
Draw a hatched, translucent band, and no line segment between the last sample before and the first after the gap (break the trace). The head moves to
`after + missed` when the gap arrives, so the next chunk continues without a shift. Keep a gap while `after + missed > start`; a gap partly scrolled
out is clipped at the left edge. The spike measured `x = 0.98` at arrival, as expected (the band sits just inside the right edge and scrolls left).
Because `missed` is an estimate, draw the right edge as soft (gradient or dashed edge) and show the sample count in the tooltip as "about".
