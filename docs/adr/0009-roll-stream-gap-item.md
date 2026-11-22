# ADR 0009: Roll stream carries gaps as data items with an exact position and an estimated size

Status: accepted
Date: 2026-11-22
Issue: #4 (spike), #27 (roll story)

## Context
At 500 ms/div and slower the device is polled continuously (PRD 5.1). The driver reports an overrun only by elapsed time, never by pointer (PRD 5.2): when the
host is late by more than about 95 % of the 64 KiB buffer span, data is lost and the loss size is unknown. The user must see where data is missing (ACQ-3, DAT-5).
Spike #4 ([report](../spikes/m0/4-roll-streaming.md)) streamed a one-hour roll with injected gaps; every gap arrived identical in the stream,
the recording and the frontend markers, and no sequence number was skipped.

## Decision
1. A roll stream is an ordered sequence of items on a Tauri `Channel`. Each item has a kind (samples, gap, close) and a `u32` sequence number; a skipped number
   means a lost item and is a defect.
2. Samples are addressed by an **absolute sample index** per channel (`t = index x sample interval`). The index counts delivered plus estimated missed samples,
   so the time axis stays continuous across a gap.
3. A **gap item** carries `after` (index of the last sample before the gap, exact) and `missed` (samples per channel lost, an **estimate** from elapsed time).
   The next samples item starts at `after + missed`.
4. The backend writes the same gap into the recording as a gap record, and when the recorder queue is full it drops the oldest item and writes a gap record, never
   a silent splice.
5. The frontend places a marker at `x = (after - windowStart) / samplesPerPixel` with width `missed / samplesPerPixel`, breaks the trace across it, and shows the
   size as approximate. It keeps a gap while `after + missed` is inside the window.
6. Item layout (little endian, header 32 bytes, then `u8` codes channel-major for samples) is the one measured in the spike; the product wire format is fixed
   in the roll story's implementation spec and keeps these fields.

## Alternatives considered
- **Mark gaps by a flag on the next samples item.** Fewer item kinds, but the position and size need extra fields on every item and a gap with no following
  data (end of recording) is not representable.
- **Fill the gap with zeros or interpolation.** Invents data; rejected (ACQ-3).
- **Report the exact lost count.** Not possible with this driver; claiming it would be false.

## Consequences
- The UI must treat `missed` as approximate (soft edge, "about"); exports and measurements must not use samples inside a gap.
- Sequence numbers make loss between Rust and WebView detectable at negligible cost (a few KB per second).
- Binary layout is little-endian and `Channel`-based; the large-payload path of Tauri applies (items over 1 KiB), which the spike measured as sufficient.
- The Rust RSS growth found by the spike is a condition on the roll story, not on this format (see report, Decision).
