# ADR 0011: Frame wire format v1: scaled `f32` divisions, cropped to the 10 division window, latest-wins on a Tauri `Channel`

Status: accepted
Date: 2026-11-22
Issue: #8 (also #10); follow-up promised by the M0 spec (section 10) and ADR 0009 item 6 (roll format stays for #27)

## Context
Spike #2 showed that four channels of 4096 samples at 60 messages per second pass over a Tauri `Channel` as binary payloads with no loss and
latency far below the 100 ms target, in `u8` and in `f32` form ([report](../spikes/m0/2-frame-streaming.md)). Architecture 10 fixed little endian and
`f32` for sample payloads and left the layout to the first frame story. The graph component takes traces "already scaled to screen divisions"
(`frontend/src/ui/graph/graph-frame.ts`: values are divisions from the top edge, 0 to 8, spread evenly over the 10 screen divisions).
The record is 4096 samples, 16.4 divisions at 250 samples per division, but the screen shows 10 divisions (design 3.3); zoom and pan are a
later story (#32).

## Decision
1. **What is sent.** The backend crops the record to the visible window and sends exactly the points the screen shows: `N = round(10 x samples
   per division) + 1` samples per channel (at least 2), the first at the left edge and the last at the right edge, so the graph's rule `x = i / (N - 1)`
   is exact. The window is centred in the record (`window_start = (record_len - (N - 1)) / 2`, rounded down) until pan exists; the hardware
   trigger position is derived from the on-screen position (spec 3.2). A frame therefore carries about 2501 samples per channel at 250 samples
   per division and as few as 6 at 2 ns per division.
2. **Sample type.** `f32`, little endian, **divisions from the top edge** (`0` top, `8` bottom, may lie outside 0 to 8 when the signal is off
   screen; the graph clamps and marks it). Computed in Rust: `up = position_divs + sign x (code - zero_code) / counts_per_div` with `sign = -1` when
   inverted, `y = 4 - up`. No volts reach the frontend in a frame; scales and labels come from the instrument state.
3. **Layout** (header 48 bytes so every `f32` array stays 4-byte aligned; the header is read with a `DataView`):

   | Offset | Size | Field |
   |---|---|---|
   | 0 | 4 | magic `0x31465353` (`SSF1`) |
   | 4 | 4 | `u32` sequence (display stream, wraps) |
   | 8 | 8 | `f64` produced, microseconds since the Unix epoch, set by `display` when it encodes |
   | 16 | 8 | `f64` sent, same clock, written by the shell immediately before `Channel::send` |
   | 24 | 1 | channel mask (bit 0 = channel 1); present channels are stored in ascending order |
   | 25 | 1 | flags: bit 0 triggered, bit 1 auto (free-run frame after the auto sweep gave up) |
   | 26 | 2 | reserved, zero |
   | 28 | 4 | `u32` samples per channel `N` |
   | 32 | 8 | `f64` sample interval in seconds |
   | 40 | 4 | `i32` trigger sample index inside the sent window, `-1` when none or outside the window |
   | 44 | 4 | `u32` setup revision the frame was captured with |
   | 48 | 4 x N x channels | channel-major `f32` samples |

   A frame of four channels at 250 samples per division is about 40 KB, above the 1 KiB threshold, so Tauri takes the fetch path the spike measured.
4. **Stream.** `open_frame_stream` (command with a `Channel<Response>`) starts a forwarder in the shell that waits on the latest-wins subscription of
   the display event and sends each encoded frame; `close_frame_stream` and dropping the channel end it. One stream per window. The stream has no
   close item in M1 (frames are idempotent snapshots); the sequence number lets the frontend count replaced frames for diagnostics.
5. **Frontend.** `decodeFrame(buffer)` returns typed-array views without copying. The frame feed keeps only the newest frame and publishes it at
   most once per animation frame; it is the rAF loop, not the transport, that sets the redraw cadence.
6. **Golden vector.** One fixture file `backend/features/display/tests/fixtures/frame-v1.bin` is produced by a Rust test and decoded by a frontend test,
   so the two sides cannot drift.
7. **Versioning.** The magic carries the version (`SSF1`); an incompatible layout becomes `SSF2` and the decoder rejects unknown magics.

## Alternatives considered
- **Send the full 4096 record and let the graph crop.** Simple, but the crop depends on pan (a backend fact once #32 exists) and 64 KiB per frame is
  paid for 40 % invisible data. Rejected.
- **`u8` codes plus scale data, convert in the frontend.** Smaller (16 KiB), but puts volts and invert logic in TypeScript against P1, and the spike
  shows the `f32` size is not a problem.
- **JSON.** Rejected by the spike's own premise and by P1b cost.
- **Decimate to the pixel width now.** Not needed: the window holds at most about 2501 points per channel, fewer than the pixels of a 1080p screen
  and drawn in full by the graph, so a one-sample glitch is drawn (the DSP-3 intent). `libs/decimation` is created by the first story that exceeds
  the pixel count (zoom out, roll history).

## Consequences
- Pan and zoom (#32) change the window rule, not the format; interpolation (#13) changes how many points are sent, not the layout.
- The trigger marker position on screen is the horizontal setting, not something decoded from the frame; `trigger index` only confirms where the driver says
  the edge is (nominal; the hardware is within about 6 samples, PRD 5.2).
- The shell stamps the sent time by overwriting 8 bytes of an `Arc` copy; the codec test covers it.
- Persistence (#26) will draw the same frames; nothing here depends on the renderer.
