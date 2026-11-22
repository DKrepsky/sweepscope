# ADR 0003: Backend/frontend contract types generated with ts-rs; the client is hand-written

Status: proposed
Date: 2026-11-21
Issue: none (requested directly by the human)

## Context
Architecture 5.8 requires command, event and snapshot types generated from Rust and committed, with one `BackendClient`
interface the application layer depends on (fake in tests). The open item was "specta or ts-rs", to evaluate in M0.
Our transport rules (4.3) are deliberately narrow: frontend to backend is Tauri `invoke`, backend to frontend is Tauri
events and binary `Channel`s, and the frame payload is a hand-defined binary layout, which no generator helps with.

## Decision
Use **ts-rs** (derive `TS`, MIT) on the Rust types that cross the boundary. Types are exported by a test-time export into
`frontend/src/api/generated/*.ts` (committed; CI regenerates and diffs). **Only data types are generated.** The command
and event names, and the `BackendClient` implementation over Tauri (`api/tauri-client.ts`), are written by hand and
covered by contract tests against the fake. `tauri-specta` is not used.

## Alternatives considered
- **tauri-specta / specta.** Also generates typed command wrappers, which looks attractive, but it ties the generated
  client to Tauri, would put Tauri-aware attributes on our command functions (only `src-tauri/ipc` may know Tauri,
  which is fine) and, at the time of writing, its Tauri 2 line is still a release candidate. A generated client also
  fights the `BackendClient` interface that tests need to fake. Revisit when it is stable and if hand-written wrappers
  become a maintenance cost.
- **Hand-written duplicate types.** Drift risk; no.
- **JSON Schema or protobuf.** A schema language and runtime for one in-process contract; no.

## Consequences
- Small dependency (derive plus export test), no codegen at app build time.
- Command names appear in two places (Rust `#[tauri::command]`, TypeScript client). A contract test enumerates
  the command list on both sides from one generated manifest (`api/generated/commands.ts`, produced by the same export
  test from a Rust constant list) so a rename fails CI.
- Wire quirks to remember: ts-rs maps `u64`/`i64` to `bigint`; we use `f64`, `u32` and `i32` on the wire (architecture
  open item 7 is closed with `f64` for values and times, `f32` for sample payloads).
