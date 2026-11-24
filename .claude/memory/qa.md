# Memory: qa

Durable lessons only: decisions, conventions, pitfalls, user preferences. No task logs, no secrets. Rewrite, do not just append.

## Project facts
- Project: SweepScope, USB oscilloscope app (Tauri 2, Rust backend, Svelte 5 frontend); first device Hantek 6254BC.
- Docs: `docs/prd.md`, `docs/architecture.md`, `docs/design.md`; process: `.claude/team/workflow.md`.
- Test names must be `given ..., when ..., then ...` (TS titles need the word `when`), enforced by `tools/check-test-names.mjs`
  for e2e files too. ESLint limits (function length, complexity) are off for `e2e/**`.

## How to run things
- E2E: `npm --prefix frontend run test:e2e` (Playwright, Chromium and WebKit). Two web servers start from the config: Vite dev on
  5173 (main window `/`, gallery `/gallery.html`) and a production preview on 4173 (CSP tests). Browsers:
  `npx playwright install chromium webkit` (OS is "unsupported", fallback ubuntu24.04 build is used).
- WebKit on this host needs libavif16, libyuv0, libdav1d etc. Without sudo: `apt-get download` the debs, `dpkg -x`, copy the
  `.so` files into `~/.cache/ms-playwright/webkit-*/minibrowser-wpe/sys/lib` (the wrapper resets LD_LIBRARY_PATH).
- Browser tests stub Tauri IPC with `e2e/support/tauri-stub.ts` (`window.__TAURI_INTERNALS__`); settings persist in
  `sessionStorage`, so a reload equals a restart. Without the stub the text scale cannot change in a browser.
- Tauri window: `npm run dev` at repo root builds and opens it (about 1 minute first time). Settings file:
  `~/.config/dev.sweepscope.app/settings.json` (`{"version":1,"ui":{"app.text-scale":{"version":1,"json":"{\"percent\":150}"}}}`);
  delete it after a test. Capture only the app window with `import -window <id>` (find id with `wmctrl -lG`).
- Kill what you started by PID (`pgrep -x sweepscope-app`, the vite PID from `ss -ltnp | grep 5173`); `pkill -f` patterns can kill
  your own shell and blanket `pkill node` is refused.

## Pitfalls and lessons
- NEVER inject input (XTEST, xdotool) on the user's real display without checking `import -window root` first: it can be a
  fullscreen game (one stray click already happened). Use Xephyr or Xvfb for GUI clicks, or skip and report the limit. Delete any
  root screenshots at once (private content).
- Playwright treats `aria-disabled="true"` buttons as not actionable: use `click({ force: true })` to test that they ignore clicks.
- Native `<dialog>` modal: Tab can leave to the browser UI (activeElement becomes body); assert it never reaches the page behind.
- Transitions (80 ms on buttons) make computed colours read mid-transition: wait 200 ms before comparing to tokens.
- Use `test.fail(cond, 'BUG-n ...')` for known open bugs so the suite stays green and flips red when the bug is fixed; remove the
  annotation in the retest.
- Graph graticule scans: background is opaque `bg-scope` (16,18,20); count runs of non-background pixels; the graticule is only
  redrawn on a size change, so force a redraw by resizing the viewport before reading it.
- axe: dim chips (40 % opacity) and `aria-disabled` controls are flagged but are inactive components; the e2e excludes them.
- Tests that compare to the screen minimum must measure against the stage (`[data-region="scope"]`), not only the screen box: the
  stage clips with overflow hidden.

## Xvfb GUI checks without sudo (worked, retest round 2)
- `apt-get download xvfb openbox libobrender32v5 libobt2v5 libstartup-notification0 libimlib2t64 xdotool libxdo3`, `dpkg -x` into a scratch dir, set
  `LD_LIBRARY_PATH` to its `usr/lib/x86_64-linux-gnu`. Start `Xvfb :77`, then `openbox` under `dbus-run-session`, with isolated `HOME`,
  `XDG_CONFIG_HOME`, `XDG_RUNTIME_DIR`, `GDK_BACKEND=x11`, `WEBKIT_DISABLE_DMABUF_RENDERER=1`, `WEBKIT_DISABLE_COMPOSITING_MODE=1`.
- Debug app (`cargo build -p sweepscope-app`) loads `devUrl` 5173, so start `npm --prefix frontend run dev` first. Right click, Inspect Element
  opens docked devtools; type into the console prompt with `xdotool type` one command at a time (a second command typed at once is lost).
  `window.__TAURI_INTERNALS__.invoke(...)` tests the capability. Window minimum: `xdotool windowsize <id> W H` is clamped by size hints.
- Stop everything by PID afterwards (app, openbox, Xvfb, vite) and delete the temp HOME.

## Pitfalls added in round 2
- Playwright `page.clock` does not work with the Svelte timers here (elements never appear); measure Toast timing with real timers and a margin.
- `addInitScript` order matters: a wrapper of `__TAURI_INTERNALS__.invoke` must be added after `stubTauri`, and `openMain` adds the stub again, so
  for failing saves call `stubTauri`, then the wrapper, then `page.goto` yourself.
- Layout thresholds: tests in `tiers.spec.ts` for each calibrated height at, and 1 px below; re-run them whenever dock or toolbar content changes.

## Status of the scaffold step
- Retest round 2 verdict PASS: BUG-1..10 and C-2, C-3 closed; open cosmetic C-1 (channel row alignment), C-4 (Marker aria-valuenow, unverified), C-5
  (Settings popover labels wrap in WebKitGTK). Window minimum is 1024 x 720 (content floor 64 x 45 rem), Stop is #f2766f.
- Known limit: banners are not counted in the height thresholds.

## CI notes (PR #1)
- Working tree may hold another agent's uncommitted edits: use a `git worktree add --detach` in the scratchpad and push `HEAD:<branch>`.

## Backend-only stories (M0 #3, device crate)
- No UI: verify by `cargo test -p sweepscope-device` (repeat 20x for flakiness), `npm run check` at the REPO ROOT (frontend has no `check` script), and inspection
  against PRD 5.1 verification markers and the driver docs (`/home/kr/workspace/hantek6xx4-lib/docs/protocol.md`, `crates/*/src/hotplug.rs`).
- Check simulator tables against PRD 5.1 (time base verified indexes 0-22, 24, 25, 27, 30; ranges 50 mV to 1 V) and that events match the real monitor
  (it replays already connected scopes). Test plan and report: `docs/specs/m0-spike/`.
- Retest pattern: after a fix round run `cargo test --workspace`, loop `cargo test -p sweepscope-device` 20x (filter `^test result` for non-"0 failed"), `npm run check` at root. Device crate has 110 tests at 30ef39c. The main tree is /home/kr/workspace/sweepscope (agent worktrees may be on other branches).

## Spike harness review (M0 #2)
- Review a spike by `git archive <commit> | tar -x -C <scratch>` (clean copy, no build artefacts), then follow the README literally. Xvfb needs an unused display
  number (check `/tmp/.X11-unix`), isolated HOME/XDG dirs, `GDK_BACKEND=x11`, `unset WAYLAND_DISPLAY`. Full release build took about 70 s.
- Harness has no duration override: cut `web/scenarios.json` (warmupS, durationS) in the scratch copy for 4 s runs. Test failure paths with `--binary /bin/false`
  and a copy of run-all.sh with a short `run_timeout_s`. Software-rendered numbers are invalid; never keep them.
- Screenshots of Xvfb windows are safe; real desktops are not (see root-screenshot rule above).

## Spike harness checks (#2, spikes/2-frame-streaming)
- Xvfb run: isolated HOME breaks rustup: also export RUSTUP_HOME=/home/kr/.rustup CARGO_HOME=/home/kr/.cargo; no CARGO_TARGET_DIR (run-all hardcodes target/release). Use Xvfb from /tmp/claude-1000/xv/root with `:94`-style unused display (X0 is the user's, others may run too).
- Background jobs ignore SIGINT: to test Ctrl+C use python Popen(start_new_session, SIGINT default) and os.killpg. `timeout` puts the app in its own group (run-all did not stop on Ctrl+C at e34f3a9).
- Never `pkill -f` patterns (kills own shell, exit 144); use pgrep and kill by PID.
- Synthetic summarize checks: clone a real JSON, drop quickRun, set configuredDurationS 60, vary drawMs.mean / ipcMs.p99 (4 ms floor, 1.5 ratio, ipc-only rule).

## M1 live view lessons (real Tauri app + simulator)
- The Tauri app ALWAYS composes `HantekCatalog`: `SWEEPSCOPE_SIMULATOR=1` does not stop it opening a plugged-in 6254BC (happened once, killed at once). Run the app
  only as `unshare -rm ./run-app.sh ...` where run-app.sh does `mount -t tmpfs none /dev/bus/usb && mount -t tmpfs none /sys/bus/usb/devices` first. Check `lsusb | grep 04b5`.
- Xvfb recipe: debs already extracted in `/tmp/claude-1000/xv/root`; env also needs `__EGL_VENDOR_LIBRARY_FILENAMES=/usr/share/glvnd/egl_vendor.d/50_mesa.json`
  and `RUSTUP_HOME/CARGO_HOME`. The window shows black for ~15 s after start and clicks sent too early are lost: `xdotool windowmap/windowmove` the "SweepScope" window,
  wait, then click. Capture with `import -window root` (DISPLAY must be the Xvfb). F5 does not reload; right click, Inspect Element docks devtools, console prompt at the bottom.
- The vite dev server log prints webview console errors (`[vite] (client) ...`): grep it for capability errors (`not allowed`). Browser stubs hide capability gaps
  (BUG-1: `core:event:allow-listen` missing); always smoke the real window once.
- Drive the backend from devtools: `window.__TAURI_INTERNALS__.invoke('use_simulator',{enabled:false|true})` = unplug/replug; 20-cycle loop and `/proc/<pid>/task` count work.
- To test a capability fix without editing the repo: `git archive HEAD | tar -x` into scratch, patch, `CARGO_TARGET_DIR=<scratch> cargo build -p sweepscope-app` (~70 s with warm deps cache), run that binary.
- e2e stub now has `emit(page, event, payload)` and `sendFrame(page, bytes)` (support/tauri-stub.ts) and `wireFrame` (support/frames.ts); wait for `open_frame_stream` in the IPC log before the first frame.
  Pixel counting of traces: count bright pixels (max channel > 80) against an empty baseline; frozen traces are dimmed.

## M1 retest lessons
- Use `SWEEPSCOPE_NO_HARDWARE=1` (log line `hardware discovery is off`) together with the `unshare -rm` USB-hiding recipe: both guards, every real-app run. Scripts live in the scratchpad (env.sh, run-app.sh); `import -window root -crop WxH+X+Y` then `convert -scale` to zoom (import has no -scale).
- Window at Xvfb 1440x900 sits at 0,0: Run button (188,21), V/div `+` (177,747) and `-` (59,747), TIME/DIV `+` (738,745); devtools console prompt about (400,886) after Inspect Element. One devtools command at a time; a 20-cycle async loop in one line works (`/proc/<pid>/task` stayed 32).
- Do not wait with `pgrep -f "playwright test"` in an until-loop: the waiter matches itself. Full Playwright run takes about 4 min and is moved to the background by the harness (exit code shown is of the wrapped pipeline, read the log).
- Dry check of a shell install command: rebuild it from the source constants, `bash -n`/`sh -n`, run only the `printf` part into a scratch file and `cmp` with the shipped file; `udevadm verify <file>`. Never run the sudo part.
- At 100 ms/div the simulator's 1 kHz sine aliases (2.5 kS/s) and fills the screen: not a bug. Check the Horizontal dock badge against the Trigger section at 1440x900 (BUG-5).

## No visual snapshots
- Screenshot-comparison tests (`toHaveScreenshot`) were removed by decision of the human (cost over value). Do not add baselines or regenerate them; check looks with layout, bounding-box, token and pixel-scan assertions, and manual UI checks.

## M2 Measure lessons (real window + e2e)
- Xvfb recipe needs no sudo: `apt-get download` + `dpkg -x` into the scratchpad (xvfb, openbox, xdotool, libxdo3, libobrender32v5, libobt2v5, libstartup-notification0, libimlib2t64, x11-xkb-utils, xserver-common, libxfont2, libfontenc1, xkb-data); start `Xvfb :77 -xkbdir /usr/share/X11/xkb`. App via `unshare -rm run-app.sh` (tmpfs over USB dirs, `SWEEPSCOPE_NO_HARDWARE=1`, isolated HOME). `unshare -rm` root bypasses file permissions: test read-only errors against `/sys/...`, not a chmod 555 folder.
- Native GTK dialogs: openbox does not focus them; `xdotool windowfocus <id>` (search `--name "Save File"`), then `ctrl+a`, type the absolute path, Return. Keys typed before focus go to the main window (shortcuts fire!). The File menu stays open behind the dialog.
- Press `c` only after clicking an empty spot of the page; after devtools the webview has no focus. Real frequency numerics: devtools console loop over `invoke("get_snapshot")` reading `measurements.channels[0].measurements[0].value.value`.
- The simulator default signal is a sine (the calibrator is not selectable from the app). Sine+noise gives frequency error of +-0.04 % (R2); calibrator shape sweep is in `backend/libs/measurements/tests/qa_calibrator_frequency.rs`.
- e2e stub: `stubTauri(page, settings, { commands: { export_frame: { result|error } } })` scripts command replies; set_interpolation is acknowledged. Interpolation and tool buttons are `button[aria-pressed]`, not radios. Stub time base is not a real number (browser time cursors show odd units).
- Spare ports: copy `playwright.config.ts` to an untracked `playwright.qa.config.ts` (ports 5273/4273, report dirs in /tmp) and run with `E2E_PREVIEW_PORT=4273`; delete it afterwards. Prettier: run from `frontend/` (`npx prettier --write`), not `npx --prefix`.
- Check the tree after `cargo test --workspace`: at 6361535 it leaves `backend/frontend/` and rewrites generated TS imports (BUG-15). Restore with `rm -rf backend/frontend && git checkout -- frontend/src/api/generated`.
- Other agents merge into the checked-out branch while QA runs: note `git log -1` at start and rerun suites at the end.

## M2 retest lessons
- Debug app on a spare dev port: `TAURI_CONFIG='{"build":{"devUrl":"http://localhost:5373"}}' cargo build -p sweepscope-app` and `npx vite --port 5373 --strictPort` (check with `strings -a target/debug/sweepscope-app | grep localhost:5`). Rebuild without the override afterwards if the default is needed.
- Drive the backend from devtools: `invoke("step_setting",{target:{kind:"timeBase"},direction:1})` steps the time base; guard `v&&typeof v.value==="number"` (null value when invalid kills the loop silently). Measurements read as `get_snapshot().measurements.channels[0].measurements[0].value`.
- Native dialogs: never send keys before the dialog exists. Use `xdotool search --onlyvisible --name "Save File"`, `windowfocus`, then type; stray keys reach the main window (shortcuts open more dialogs; cancel at (957,798)). Overwrite prompt: Replace at (696,464).
- Simulator sine+noise frequency error in the real window stays above the calibrator bounds (200 us +-0.029 %, 500 us +-0.02 %, 1 ms +-0.013 %); use the library sweep (`cargo test -p measurements --test qa_calibrator_frequency`) for the AC.
- Package name of the measurement lib is `measurements`. Clean-tree check: `cargo test --workspace` then `git status --short` (ignore `.claude/memory`).
