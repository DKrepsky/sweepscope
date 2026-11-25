# Addendum: one executable for the window and the CLI

Approved by the human at G3 on 2026-11-25, with changes; this text is the approved result. `sweepscope` opens the window, `sweepscope capture --setup s.json --count 3 --out DIR` captures without it. Amends ADR 0022 and ADR 0024. Developer detail: [work-plan.md](work-plan.md) section 10.

## Dispatch
| First argument | Result |
|---|---|
| `capture`, `--help`, `-h`, `--version`, `-V` | CLI: runs and exits with its code. Mistakes after `capture` are usage errors (exit 2) |
| none, or anything else | The window |

The choice is made first thing in `main`, before `env_logger` and `tauri::Builder`. GTK and WebKitGTK are initialised only inside `Builder::run`, so CLI mode needs no display.

## Structure
- `cli` becomes a library only (Tauri free); the `sweepscope-cli` binary is removed. `src-tauri` depends on it. No new external dependency.
- The recognised words live in the library next to the argument parser, as a pure function.
- `--version` is new. There is one usage text, `sweepscope capture ...`.
- `cli-without-tauri` stays. `depends-on-cli` now means: only `src-tauri` may depend on `cli`.

## Accepted consequences and limitations
- A mistyped command (`sweepscope captur ...`, or an option before `capture`) opens the window; a script would then wait on it, or fail to open one without a display.
- The binary links WebKitGTK 4.1 and GTK 3, so they must be installed even to capture. There is no longer a build without them for headless machines.
- AppImage: arguments pass through (`./SweepScope_x_amd64.AppImage capture ...`). The runtime consumes its own `--appimage-*` options; relative paths should resolve against the caller's directory (checked on hardware); each start mounts a FUSE image, and `--appimage-extract-and-run` covers machines without FUSE.

## Tests
- The choice, including "anything else opens the window", is unit tested on the pure function. No test starts the window.
- An integration test runs the app binary with recognised commands only and no `DISPLAY` or `WAYLAND_DISPLAY`: exit codes 0, 2 and 10, help and version.
- Hardware checklist H-M4-6 uses `sweepscope capture ...` and adds an AppImage run.
