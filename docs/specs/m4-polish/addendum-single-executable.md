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
- AppImage: arguments pass through (`./SweepScope_x_amd64.AppImage capture ...`). The runtime consumes its own `--appimage-*` options; each start mounts a FUSE image, and `--appimage-extract-and-run` covers machines without FUSE.

## AppImage relative paths (decided 2026-11-25, after a test on the real runtime)
The AppImage moves the program into its own `usr` folder before it starts, so relative paths would break. In every start mode `PWD` still holds the caller's folder, because the AppImage start script (bash) sets it before that move. In CLI mode only, and only when the program really runs from inside an AppImage, it changes back to `PWD` before doing anything else. Relative `--setup`, `--out`, `--config-dir` and the default output folder then work, also with `--appimage-extract-and-run`. The installed `.deb` and the window are unaffected. If the folder cannot be entered, the run fails (exit 1) and writes nothing. `OWD` is not used: it is missing with extract-and-run.

## Tests
- The choice, including "anything else opens the window", is unit tested on the pure function. No test starts the window.
- An integration test runs the app binary with recognised commands only and no `DISPLAY` or `WAYLAND_DISPLAY`: exit codes 0, 2 and 10, help and version.
- Hardware checklist H-M4-6 uses `sweepscope capture ...` and adds AppImage runs with relative paths, normal and `--appimage-extract-and-run`.
- The return to the caller's folder is tested without an AppImage by pointing `APPDIR` at the build folder, and once on the built AppImage with the simulator.
