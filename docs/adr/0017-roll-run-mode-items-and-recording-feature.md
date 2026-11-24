# ADR 0017: Roll is a run mode of `acquisition`; roll items are ordered, indexed and gap-marked; recording is its own feature

Status: proposed
Date: 2026-11-24
Issue: #27, #28 (follows ADR 0009 and spike #4)

## Context
Spike #4 accepted the pattern "poller, ordered binary items with sequence numbers on a Tauri `Channel`, gap items with an exact position and an estimated size, bounded
recorder queue" and fixed it in ADR 0009, leaving the product layout to this milestone. M1 blocks the streaming time bases (500 ms/div and slower, 11 steps) in
`acquisition`. The driver offers `start_roll`, `poll_roll` and `Error::Overrun` (inferred from elapsed time, 95 % of the 64 KiB span), verified at 200 ms, 500 ms,
2 s and 20 s per division only. Recording to a file for hours needs a writer that never blocks the display and a file format a user can read (#28 AC4).
The bus has bounded subscriptions with drop-oldest and `lagged()`, but no ordered stream with a loss item; ADR 0009 item 4 wants a gap, never a silent splice.

## Decision
1. **Roll is a run mode of `acquisition`**, derived from the selected time base (`TimeBaseStep.streaming`), not a second setting. Steppers may enter and leave it. In roll
   `acquisition` publishes availability reasons (trigger, Single, Force, Auto-setup, views that need a record) and starts and stops the `device` worker's roll loop through the
   existing `SetCapturing`; the worker decides between frame and roll from the setup it applied. A change of any setup value, of an invert flag, or Run after Stop begins a new
   **roll session** (`u32`, counted by `acquisition`); the frontend clears its window on a new session.
2. **Absolute sample index.** The `device` worker counts samples per session. A chunk has `first_index`; an overrun becomes a gap item `{ after, missed }` with
   `missed = elapsed since the last good poll / sample interval` (an estimate, ADR 0009 item 3) and the counter advances by `missed`.
3. **Two kernel messages** (reviewed under ADR 0010 item 7, total nine): `RollPolled { setup_revision, item: RollPolledItem }` (`device` to `acquisition`) and
   `RollCaptured { session, item: RollItem, context: RollContext }` (`acquisition` to `display` and `recording`). `RollItem` is `Opened`, `Samples { first_index, frame }`, `Gap { after, missed }`
   or `Closed { reason }` (stopped, left roll, disconnected, setup changed). `acquisition` drops polls of an old setup revision, adds invert and probe, and publishes `Closed` itself when
   run intent, roll mode or the connection ends.
4. **Loss between modules is also a gap.** Consumers subscribe with the normal bounded subscription (capacity 256, about 50 s at 20 polls per second). A consumer that sees `first_index`
   greater than the index it expects synthesises `Gap { after: expected - 1, missed: difference }`. No special overflow path exists: a lagging recorder or display shows a gap like the device does.
5. **Display path (ADR 0009 items 1 and 6 made concrete).** `display` converts a `Samples` item to divisions (ADR 0011 formula, no crop, no interpolation) and encodes the **roll item** below on its own event
   `DisplayRollItem`; the shell forwards it on a second Tauri `Channel` (`open_roll_stream`, `close_roll_stream`), ordered, with a `u32` sequence per stream; a skipped number is a defect and is counted by the
   frontend. The frontend keeps only the visible window (10 divisions at 250 samples per division, 2500 samples per channel) in a `RollWindow`; gaps are drawn from `after` and `missed` as ADR 0009 item 5 and
   spike report section 8 describe. The backend keeps no history ring: bounded queues only (the display queue and the recorder queue), which is what makes memory constant.
   ```
   Roll item, little endian, header 32 bytes, then channel-major f32 divisions from the top edge (samples kind only)
   0  u32 magic 0x314C5353 ("SSL1")   4  u32 sequence     8 u8 kind (0 samples, 1 gap, 2 closed)   9 u8 channel mask (bit 0 = CH1)
   10 u8 close reason (0 stopped, 1 left roll, 2 disconnected, 3 setup changed; kind closed only)    11 u8 reserved
   12 u32 samples per channel (samples) | 0     16 u64 first_index (samples) or after (gap)
   24 u32 missed (gap) | 0                      28 u32 session
   ```
6. **`recording` is a new feature crate** (`sweepscope-recording`), not part of `export`: `export` writes one file at once and keeps the last frame; `recording` owns a long-lived writer thread, a queue, a clock-driven
   statistics slice and a different failure model. It subscribes to `RollCaptured`, asks `FilePicker` for the file, and writes **`.ssroll` v1** (documented in `docs/formats/roll-record-v1.md`, written by the package):
   header `SSRL` + `u16 version` + `u16 0`, then records `u8 kind, u32 length, payload`: `1 segment` (UTF-8 JSON: start time, sample interval, per channel number, volts per div, probe, zero code, counts per div, invert, first index),
   `2 samples` (`u64 first_index`, `u32 count`, channel-major `u8` codes of the segment's channels), `3 gap` (`u64 after`, `u64 missed`, estimate), `4 end` (`u8 reason`). A setup change starts a new segment in the same file.
   The writer flushes once per second; a truncated last record is ignorable, so a crash leaves a readable file. This is append-only streaming, so it is the documented exception to `write_atomic` (ADR 0016).
7. **Failure and removal.** A write or flush error stops the recording, publishes the reason and the size written, and never touches the display. A removed file is detected at each flush by comparing device and inode of the open
   file with the path (Linux; writing to an unlinked file succeeds silently). Recording ends on `Closed`, `Stop`, leaving roll or disconnect, without resuming.
8. **Slice.** `RecordingState { revision, active, file_name, duration, bytes_written, failure: Option<RecordingFailure { id, text, file_name, bytes }> }`, event `RecordingChanged` at most once per second, requests
   `StartRecording` (returns `Option<SavedFile>`, `None` = picker cancelled; rejected unless a roll session is open), `StopRecording`.

## Alternatives considered
- **Send the visible window as a normal frame at 20 Hz.** No frontend ring and rescaling for free, but 40 KB at 20 Hz is 400 times the byte rate the spike measured with 2 KB items; the one-hour memory condition of the spike would
  have to be proven again. Rejected to keep the measured pattern.
- **Roll as a separate setting.** Two things to keep consistent with the time base. Rejected.
- **A kernel `OrderedStream` with a loss item.** Right in principle (architecture 4.3) but a new kernel primitive for one consumer pair; the index discontinuity gives the same guarantee with no new primitive.
- **Recording inside `export`.** Mixes a one-shot exporter with a thread, a queue and a clock.
- **CSV recording.** About six times the size per hour and needs a gap convention; binary with a documented reader is smaller and exact.
- **Rely on write errors for a removed file.** On Linux a write to an unlinked file does not fail.

## Consequences
- Roll costs two kernel messages and one new crate; the frontend gets one window class and one stream.
- Gaps have one definition everywhere (device, display, recorder, file).
- A vertical change during roll loses the on-screen history (new session) but not the recording, which keeps all segments.
- The hardware has never been run in roll by the driver's tests beyond 22 s: the checklist items H-01 to H-04 are the first real evidence for `Overrun`.
- ADR 0009 item 6 is superseded by item 5 above; items 1 to 5 of ADR 0009 stand.
