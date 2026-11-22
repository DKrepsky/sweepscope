# Spike #4: roll-mode streaming path

Issue #4. Spec: [m0-spike implementation spec](../../specs/m0-spike/implementation-spec.md) section 2.6. Policy: [ADR 0008](../../adr/0008-spike-policy-and-measurement-reports.md).
Status: final. The human approved option A (accepted with condition) on 2026-11-22.
This report states conclusions and rule outcomes only. Measured figures and raw data are kept locally by the maintainer and are not committed (ADR 0008).

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

- Spike branch `spike/4-roll-streaming`, tag `spike-4-final` (local until pushed), tip `aea1a7d`; the acceptance hour ran at `1a5ffb5`, later commits add CPU columns,
  overrides and a Mesa EGL setting. Code: `spikes/4-roll-streaming/` (standalone workspace, uses the #3 simulator by path).
- Acceptance run: `spikes/4-roll-streaming/tools/soak.sh app`: the Tauri app under Xvfb with software rendering, isolated HOME and XDG directories, real time, one hour, the simulator with
  four channels at 500 ms/div, polled every 100 ms, with an overrun injected every 10 minutes.
- Items are binary, about 2 KB each, ten per second, on a Tauri `Channel`, so each takes the large-payload path (payloads over 1 KiB go through fetch, not eval).
- Memory is sampled every 10 s: RSS of the main process (Rust code plus the GTK and WebKit UI-process code in the same process) and of the `WebKitWebProcess`, with ring fill and file size. The memory rules
  are applied from the first sample at or after 5 minutes.
- The frontend keeps only the visible span in absolute sample indices and logs every gap marker with its position. Sequence numbers and the recorded file are verified after the run.
- Attribution runs after the decision: the backend alone, headless, in real time (25 minutes, short ring); and the app under Xvfb with Mesa software EGL and the Channel send disabled (25 minutes).
- Known limits: one machine and software rendering (no GPU path); one acceptance run; the simulator, not hardware; the missed-sample count of a gap is an estimate from elapsed time (the driver cannot
  do better, PRD 5.2); Rust RSS includes the whole UI process; the natural overrun (poller paused beyond 95 % of the buffer span) is covered by a unit test, all gaps in the soak are injected;
  the Channel-off run used Mesa EGL and the Channel-on hour used NVIDIA EGL, so they are not like for like; no Channel-on run with Mesa; the first attribution app run shared a display and is discarded;
  `dmesg` showed `WebKitWebProcess` crashes in the NVIDIA EGL library under Xvfb on this machine, so harness runs force Mesa EGL.

## 3. Environment

One machine, id `i5-9600kf-xvfb-software`: Linux Mint 22.3, Intel i5-9600KF, 32 GiB, Xvfb 1280x720x24, software GL, X11 backend, DMABUF renderer off, WebKitGTK 2.52.6, Tauri 2.12.7.

## 4. Results (rule outcomes)

| Rule | Outcome |
|---|---|
| Rust RSS growth | **failed** in the app run (above 2 MiB over the hour; steady, linear in wall time, not per item) |
| WebKit RSS growth | passed |
| Recorded file | passed: grew linearly, exact expected size, every injected gap present as a gap record |
| Gaps in the frontend | passed: every injected gap arrived once, identical in the `start_roll` result, the recording and the frontend markers, at the right absolute index |
| Sequence numbers | passed: none skipped |
| CPU | informative; backend far below the 2 % limit in the headless run, not recorded in the acceptance run |
| Ring buffer | passed: filled once and stayed at capacity; the frontend window stayed bounded |

Attribution runs:

- Headless backend (poller, ring, recorder, simulator): Rust RSS flat for the whole run, backend CPU negligible, recording exact, gaps present. The backend does not leak.
- App with the Channel send disabled: the Rust-side growth did not disappear and is of the same order per unit time as with the Channel on. The Channel send is therefore not shown to be the cause.
  WebKit RSS kept growing in this shorter run (UI idle redraw), not settled.
- Accelerated (60x) runs deliver the same number of items in about a minute with almost no growth, so the growth follows wall time, not the number of items.

## 5. Findings

1. Constant memory by construction works: the ring stays at capacity, the frontend window is bounded, the file grows linearly with an exact size.
2. Gaps travel as data and survive every hop; the chunk after a gap starts at `after + missed`, so the time axis stays continuous.
3. The Rust RSS growth is real, linear in wall time, small, and located outside our backend. The attribution runs point to the UI process under software rendering (or the harness), not to the poller,
   ring, recorder, simulator or Channel send. It is a slow leak worth tracking over days of runtime, not a blocker for a roll story.
4. The 5 minute exclusion is too short for a process that is still allocating and a ring that fills later; the baseline must be taken after the ring is full. The verdict does not change.
5. Natural overrun and CPU of the acceptance run were not part of that run.

## 6. Decision

**The pattern (poller thread, ring buffer with absolute sample indices, ordered binary items with sequence numbers on a Tauri `Channel`, gap items carrying `after` and an estimated `missed`, bounded
recorder queue that writes gap records) is accepted for the roll story (#27), with one condition on the Rust RSS rule (human decision: option A).**

Reasons: every functional rule passed with zero loss; the WebKit rule passed; the Rust rule failed with a slow, steady, small growth that the attribution runs place outside our backend; rejecting the pattern for
that would be disproportionate, and accepting it silently would break ADR 0008 point 7. The rule is not changed to fit the result.

### Condition (binding on #27)

1. The 2 MiB per hour Rust RSS rule stays as written in PRD 8.3, measured after the ring is full.
2. The definition of done of #27 repeats the one-hour real-time soak with the product frontend, and passes if Rust RSS growth is within the rule, or the growth has a documented cause outside our code (UI
   process, graphics stack).
3. Attribution done (section 4): the backend is flat and cheap, the Channel send is not the cause. What remains for #27: run the soak on a real display session (the human's, or a second machine with a GPU path), not
   only Xvfb software rendering; and one run with a blank page or no canvas drawing, to confirm the growth belongs to the WebView UI process and not our frontend code. If the growth is still above the rule there, the
   story documents the cause or fixes the frontend before acceptance.

## 7. Changes requested and open points

Changes requested:

1. Gap item in the product stream: written as [ADR 0009](../../adr/0009-roll-stream-gap-item.md). The roll story's requirements must say the size is an estimate (PRD 5.2) and the marker is drawn as an uncertain band.
2. PRD 8.3 "Memory in roll recording": baseline after the ring is full, Rust rule 2 MiB per hour: applied in `docs/prd.md`.
3. No change to the driver or to ADR 0007: `poll_roll` plus `Overrun` was enough.

Open points:

- A Channel-on run with Mesa EGL and a longer no-Channel run would close the caveats in section 2 (developer work, no human needed).
- Roll with the real device is verified nowhere in the driver (`poll_roll` timing, overrun detection by elapsed time); first M3 hardware check.

## 8. How a gap marker is placed on screen

A gap item carries `after` (absolute index of the last sample before the gap, exact) and `missed` (samples lost, estimate). The frontend keeps the window in
absolute sample indices: `start = max(0, head - windowSamples)`.

```
x(index)      = (index - start) / windowSamples          // 0..1 inside the view, outside it when scrolled off
marker.x      = x(after)                                 // left edge of the band
marker.width  = missed / windowSamples                   // right edge x(after + missed)
```

In pixels, `x_px = (index - start) / samplesPerPixel`, band width `missed / samplesPerPixel` (one missed poll period is a small fraction of a 5 s window).
Draw a hatched, translucent band, and no line segment between the last sample before and the first after the gap (break the trace). The head moves to
`after + missed` when the gap arrives, so the next chunk continues without a shift. Keep a gap while `after + missed > start`; a gap partly scrolled
out is clipped at the left edge. In the spike the band appears just inside the right edge when it arrives and scrolls left.
Because `missed` is an estimate, draw the right edge as soft (gradient or dashed edge) and show the sample count in the tooltip as "about".
