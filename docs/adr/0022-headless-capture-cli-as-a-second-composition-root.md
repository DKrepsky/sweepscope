# ADR 0022: The headless capture CLI is a second composition root in its own crate, without Tauri

Status: Accepted
Date: 2026-11-24
Issue: #38

## Context
#38 wants N frames captured from a setup file without the UI, with distinct exit codes. The shell links Tauri and WebKitGTK, which a headless machine should not need; composition happens only in composition roots. Setup loading and export go through `FilePicker`, and export keeps only the newest frame.

## Decision
1. New crate `cli` (library `sweepscope_cli`; the application binary is its entry point, see item 7), classed as a shell crate by the architecture check, with one new rule: it never depends on `tauri*`.
2. Its own small composition: bus, `acquisition`, `device` (hardware, or simulator with a flag), no `calibration` module (no session is needed): it reads the offsets file through calibration's `OffsetFile` and publishes `ZeroOffsetsChanged` itself before running, so stored offsets apply without a thread. No display, measurement, spectrum or recording.
3. Reuse through existing ports: a `FilePicker` returning the path given on the command line for `LoadSetup`; frames from an ordered `FrameCaptured` subscription encoded by a new pure `export` function (so every frame is written).
4. Storage is a read-only view of the app config folder: stored offsets apply, `last-setup` and settings are never overwritten.
5. One subcommand `capture` with `--setup`, `--count`, `--out`, `--format`, `--timeout`, `--device-wait`, `--simulator`; no overwrite of existing files; arguments parsed by hand (no `clap`).
6. Exit codes: 0 success, 1 failure, 2 usage, 10 no device, 11 timeout, 12 permission denied.

## Alternatives considered
A second binary in the shell (links Tauri); a shared composition crate (wait for a third caller); `clap` (new dependency for one subcommand); writing through to the app storage (a script would overwrite the user's last setup).

## Amendment 2026-11-25: one executable (addendum-single-executable.md, human decision)
7. The application binary runs the CLI. When its first argument is `capture`, `--help`, `-h`, `--version` or `-V`, `src-tauri` calls `sweepscope_cli::run` before `env_logger` and `tauri::Builder` and exits with its code. Otherwise (no argument or anything else) it opens the window. The recognised words are a pure function in the library. GTK and WebKitGTK are initialised only inside `Builder::run`, so CLI mode needs no display.
8. The `sweepscope-cli` binary is removed; the crate stays a Tauri-free library and `src-tauri` depends on it. Rule `depends-on-cli` now reads: only `src-tauri` may depend on `cli`. `--version` is added.

Accepted consequences: a mistyped command opens the window; the CLI needs the WebKitGTK and GTK libraries installed, which gives up the context's aim that a headless machine should not need them. This replaces the rejected alternative of putting the CLI on the GUI binary: the CLI code stays in its own crate, only the entry point is shared.

## Consequences
One new workspace member and one architecture rule. The CLI inherits acquisition behaviour and is tested against the simulator.
