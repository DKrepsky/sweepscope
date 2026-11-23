# ADR 0013: Domain state reaches the frontend as revisioned slices; commands are few and typed; the backend steps and resets settings

Status: accepted
Date: 2026-11-22
Issue: #5, #7, #9, #10, #11, #12 (answers the open "state snapshot format" of architecture 9)

## Context
Architecture 4.5 says the shell aggregates module events into snapshots for the frontend and that after a reload the frontend asks for a snapshot and then
follows events. The format was left open ("whole snapshot vs per-slice"). M1 has three kinds of state with different update rates: connection (rare),
instrument setup (on user action), acquisition status (per frame outcome, deduplicated) and facts (at most 4 per second, Q-E1). The mock view model
(`app/main-window/types.ts`) already holds formatted strings; Q-E3 proposed that the backend send every selectable scale as `ScaleOption`.

## Decision
1. **Per-slice events and a combined snapshot.** Four slices, each a `Serialize` + ts-rs type with a monotonic `revision: u64` that starts at 1 per backend
   process: `connection` (device), `instrument`, `status`, `facts` (acquisition). Each change is one Tauri event carrying the whole slice
   (`connection_changed`, `instrument_changed`, `status_changed`, `facts_changed`). `get_snapshot` returns all four.
2. **No race at start.** The frontend subscribes to the four events first, then calls `get_snapshot`, and applies a payload to a slice only when its
   revision is greater than the revision already applied. Events and snapshot are therefore idempotent in any order. The shell is a forwarder with no
   logic: the aggregator holds nothing; each module owns its slice and its revision counter.
3. **Domain stores are read-only to the UI.** Four stores in `app/instrument` (`connection`, `instrument`, `status`, `facts`); everything else is a
   selector. The existing `MainWindowModel` becomes a derived view built by one function in `app/main-window`.
4. **Formatted values.** Every number the UI shows is a `FormattedValue { value: f64 (SI), text: String (the number part, for example "500"),
   unit: String (the display unit with prefix, for example "mV/div") }`, produced by `libs/engineering-format` (Q-E5). The frontend never formats a
   domain value. Numbers it needs for geometry (division positions as `f64`) travel next to the text.
5. **The backend steps and resets.** Stepping a setting and double-click reset are commands (`step_setting`, `reset_setting`) with a target, so the frontend never holds
   a scale table (the Q-E3 `ScaleOption` list is not needed by any M1 control: steppers show the current value and whether a step is possible). The state says
   whether a step is possible and why not (`Availability { enabled, reason }`), including the disabled roll time bases.
6. **Commands are few and patch-shaped.** One `set_channel` with an optional-field patch, one `set_trigger` with a patch, plus the value commands listed in the
   implementation spec 3.4. Every command returns `()` or a `KernelError`; the new state arrives as an event (unidirectional flow, architecture 5.5 rule 4).
   There are no optimistic updates; the drag ghost of design 8.6 is a presentation concern that ends when the slice with the confirmed value arrives
   (or after the 250 ms pending style).
7. **Streams are separate from commands in the client.** `BackendClient.openStream(kind, onMessage)` returns a handle with `close()`; `kind` is a literal from a
   generated `STREAMS` list so the contract test can compare names with the Tauri commands that open and close each stream (ADR 0003 mechanism extended with events
   and streams in the manifest).
8. **Settings are accepted when a device is ready.** While no device is ready (disconnected, connecting, error) every setup command is rejected with the reason and
   the matching controls are published as unavailable; the last setup stays in the state, read only, for display and for the restore. While stopped,
   settings are accepted and applied at once (the device needs no capture to be configured). This resolves the policy part of Q-E7.

## Alternatives considered
- **One growing snapshot event.** Simple, but a status flip would resend the setup, and facts at 4 Hz would resend everything.
- **Diff events (patches) per field.** Less data, more code and more ways to drift; the whole slice is under 3 KB.
- **No revisions, trust ordering.** Tauri events and a command reply travel different paths; a snapshot taken before an event can arrive after it.
- **Frontend-side stepping with a scale list (Q-E3 as written).** Puts tables and limit rules in TypeScript.
- **Generic `set(key, value)` command.** Untyped; defeats ts-rs.

## Consequences
- Q-E3 and the policy part of Q-E7 are answered; `docs/architecture.md` 11 is updated after approval.
- The frontend needs one revision guard, written once in `app/instrument`.
- Facts and status have their own event rates, so a fast status flip never causes the setup to be re-rendered.
- When roll mode (#27) adds a stream with a close item, `openStream` already has the handle shape it needs.
