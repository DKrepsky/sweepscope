# Spike #2: frame streaming and rendering

Issue #2. Spec: [m0-spike implementation spec](../../specs/m0-spike/implementation-spec.md) sections 2.5 and 6. Policy: [ADR 0008](../../adr/0008-spike-policy-and-measurement-reports.md).
Status: closed by the human's decision as done on one machine, with a reduced scenario set (section 7).
This report states conclusions and rule outcomes only. Measured figures and raw data are kept locally by the maintainer and are not committed (ADR 0008).

## 1. Question and decision rule

Can four channels of 4096 samples at 60 messages per second go over a Tauri `Channel` and be drawn at the display rate in WebKitGTK, with Canvas 2D and with WebGL2 including persistence-style accumulation?
Is WebGL2 worth building, and does ADR 0006 or PRD DSP-7 and DSP-9 change?

Rules fixed before measuring (spec section 6), per scenario after a 2 s warm-up:

| Rule | Pass |
|---|---|
| Frame rate | at least 58 fps on a 60 Hz display |
| Frame time | no animation frame interval over 33.3 ms (PRD 8.3); at most 0.1 % over 20 ms is marginal |
| Transport | no sequence gaps; IPC time p99 below 16 ms (informative here: it includes main-thread queueing) |
| Latency | end-to-end p99 plus one refresh interval below 100 ms (PRD 8.3) |

WebGL2 go rule: WebGL2 creates a context and meets the rules on every measured machine, and either Canvas 2D with persistence fails a rule on at least one machine, or Canvas 2D has a draw time p99 at least 1.5 times that of WebGL2 in the synchronised runs.
The DMABUF workaround is confirmed if `dmabuf-off` turns a failing default into a passing one, or shows no harm where default already passes.

## 2. Method

- Spike branch `spike/2-frame-streaming`, tip `2703284`. Code: `spikes/2-frame-streaming/` (standalone workspace, plain web assets). The harness is `tools/run-all.sh`.
- The backend sends 60 binary frames per second (four channels, 4096 samples each) on a Tauri `Channel`. The frontend draws the newest frame once per animation frame with Canvas 2D or WebGL2, with persistence, either free-running or with forced GPU completion (the "synchronised" runs, which give honest draw times at the cost of stalling the pipeline).
- Each scenario runs 60 s. The metrics are frames sent, received and coalesced, sequence gaps, late Rust sends, animation frame interval, draw time, IPC time and end-to-end latency, as defined in spec 2.5.
- Known limits: the run was made by the human in a normal desktop session on one machine; one run per scenario, no repeats; the end-to-end figure is measured to the first animation frame after the draw call, not to scan-out; IPC time includes main-thread queueing.

## 3. Environment

One machine: a desktop with a single NVIDIA GPU on X11, WebKitGTK default variant (no environment workaround set). Machine details are in the maintainer's local environment record.

## 4. Results (rule outcomes)

Scenarios run: Canvas 2D and WebGL2, each with persistence, free-running and synchronised (4 of the 8 planned scenarios).

| Rule | Outcome |
|---|---|
| Frame rate | passed in all four scenarios |
| Frame time (no interval over 33.3 ms) | **failed in three of four scenarios**, because of rare isolated long frames; the fourth (one WebGL2 run) had none |
| Transport | no dropped frames, no sequence gaps, no late Rust sends in any scenario; IPC time rule passed in the render scenarios |
| Latency | passed, well under the 100 ms target |
| WebGL2 against Canvas 2D | the 1.5 times ratio and the Canvas 2D draw time floor of the go rule were met in both the synchronised and the plain runs |
| WebGL2 context | created and usable on this machine |

## 5. Findings

1. The transport is not the bottleneck: 60 frames per second of this size go over a `Channel` with no loss, no gaps, and latency far below the target.
2. WebGL2 is clearly cheaper to draw than Canvas 2D with persistence on this machine, with the margin the go rule asked for.
3. The frame-time failures are rare, isolated long frames. They occurred with both renderers, and one WebGL2 run had none. They are not shown to be caused by the renderer or by the environment (no repeat was made, no DMABUF variant, no trace of what ran in those frames). They are the open risk of this spike.
4. The first clause of the go rule (WebGL2 meets every rule on every measured machine) is therefore not met in the strict sense: the frame-time rule failed in one of the two WebGL2 scenarios, and the same outliers hit Canvas 2D.

## 6. Decision

**Conditional go for WebGL2 as the persistence renderer; ADR 0006 is confirmed unchanged and PRD DSP-7 and DSP-9 are not changed.**

- ADR 0006 says Canvas 2D first behind `TraceRenderer`, WebGL2 and the renderer-choice item after this spike. The spike confirms that order: the transport holds, WebGL2 is worth adding for persistence, and Canvas 2D stays as the fallback (DSP-9).
- The condition is that the long-frame outliers are examined early in M1 and are not accepted as the renderer's normal behaviour: they touch DSP-7 ("without dropping frames") whichever renderer wins.
- No ADR is needed. ADR 0006 status stays `proposed` until the M1 renderer story accepts it.

## 7. Acceptance criteria not met

The human closed #2 as done on one machine. Stated plainly:

- **Criterion 1 (at least two machines, one with NVIDIA): not met.** One NVIDIA desktop only. Dropped frames and latency were recorded for that machine.
- **Criterion 3 (DMABUF workaround): not met.** Only the default variant was run; the `dmabuf-off` variant was not, so the documented workaround is neither confirmed nor corrected.
- Criterion 2 (frame time per renderer): met for one machine and four scenarios (no ipc-only, no compositing-off scenarios).
- Criterion 4 (end-to-end latency against 100 ms): met.
- Criterion 5 (go or no-go for WebGL2, ADR and PRD changes, committed): met by section 6, as a conditional go.

Not measured: a second machine, an integrated GPU, the `dmabuf-off` variant, the ipc-only scenarios, Wayland, compositing-off.

## 8. Risk left for M1 and recommendation

Risks:

- Rare isolated long frames: cause unknown, they violate the PRD frame-time target in most scenarios here.
- Untested GPU paths: integrated GPUs (the PRD 8.3 target class), Wayland, and the DMABUF workaround. A user with a blank or unstable view has no confirmed remedy.
- The frame-time result rests on one run per scenario on one machine.

Recommendation for M1:

1. Build the renderer on the `TraceRenderer` interface as ADR 0006 says: Canvas 2D first (it is the fallback and the baseline), then a WebGL2 renderer for persistence, with the renderer-choice status item and automatic fallback (DSP-9). Keep the `Channel` as the transport; the stream needs no redesign.
2. Early in M1, repeat the free-running and synchronised runs several times on this machine and look at what runs in the long frames (trace of the frame callback, garbage collection, the compositor) before the persistence work depends on them.
3. Run the H2-type checks (full 8 scenarios including `dmabuf-off`, compositing-off and ipc-only) as soon as a second machine exists, preferably an integrated-GPU laptop and, if possible, Wayland. Until then the product must not claim the DMABUF workaround as confirmed.
4. Do not mark DSP-7 as met on the evidence of this spike; re-measure with the real frame budget at the end of M1 (spec risk table).
