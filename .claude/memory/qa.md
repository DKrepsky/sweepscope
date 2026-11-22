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
- `toHaveScreenshot` defaults to `scale: 'css'`: HiDPI snapshots need `{ scale: 'device' }` or they cannot show device-pixel changes.
- The gallery snapshot test stops at the first mismatch: fix one section, rerun, others may be hidden behind it.
- Playwright `page.clock` does not work with the Svelte timers here (elements never appear); measure Toast timing with real timers and a margin.
- `addInitScript` order matters: a wrapper of `__TAURI_INTERNALS__.invoke` must be added after `stubTauri`, and `openMain` adds the stub again, so
  for failing saves call `stubTauri`, then the wrapper, then `page.goto` yourself.
- Layout thresholds: tests in `tiers.spec.ts` for each calibrated height at, and 1 px below; re-run them whenever dock or toolbar content changes.

## Status of the scaffold step
- Retest round 2 verdict PASS: BUG-1..10 and C-2, C-3 closed; open cosmetic C-1 (channel row alignment), C-4 (Marker aria-valuenow, unverified), C-5
  (Settings popover labels wrap in WebKitGTK). Window minimum is 1024 x 720 (content floor 64 x 45 rem), Stop is #f2766f.
- Known limit: banners are not counted in the height thresholds.

## Visual snapshots and CI (PR #1)
- Baselines in `frontend/e2e/__screenshots__/chromium/` must come from the CI runner (ubuntu-24.04): a local host and even the official
  `mcr.microsoft.com/playwright:v1.65.1-noble` container render differently from the runner (the container differed from CI more than the
  old local baselines did). Fonts are not bundled, so wrapping can differ (StatusChip row wrapped differently).
- How to regenerate: temporarily change the browser job's Run Playwright step to
  `npm --prefix frontend run test:e2e -- --project=chromium e2e/visual.spec.ts --update-snapshots all --retries=0` plus an
  `upload-artifact` of `frontend/e2e/__screenshots__/chromium/visual.spec.ts/`, push, `gh run download <run> -n snapshots`, copy the images,
  revert the workflow in the next commit. (`npm --prefix frontend exec` does not find the config; use `run test:e2e`.) Pushing a new
  branch for this was denied by the permission classifier: use the PR branch only.
- The gallery snapshot test stops at the first mismatch, so a failed run shows only one gallery section; regenerate all at once.
- CI renders are deterministic (two runs gave 0 differing pixels). Graph scope-screen snapshots passed on CI unchanged.
- Working tree may hold another agent's uncommitted edits: use a `git worktree add --detach` in the scratchpad and push `HEAD:<branch>`.
- Gallery crops use `maxDiffPixelRatio: 0.01` (global 0.002 stays for full-window shots): Dialog text differed by 518 px between two CI runs.

## Backend-only stories (M0 #3, device crate)
- No UI: verify by `cargo test -p sweepscope-device` (repeat 20x for flakiness), `npm run check` at the REPO ROOT (frontend has no `check` script), and inspection
  against PRD 5.1 verification markers and the driver docs (`/home/kr/workspace/hantek6xx4-lib/docs/protocol.md`, `crates/*/src/hotplug.rs`).
- Check simulator tables against PRD 5.1 (time base verified indexes 0-22, 24, 25, 27, 30; ranges 50 mV to 1 V) and that events match the real monitor
  (it replays already connected scopes). Test plan and report: `docs/specs/m0-spike/`.
- Retest pattern: after a fix round run `cargo test --workspace`, loop `cargo test -p sweepscope-device` 20x (filter `^test result` for non-"0 failed"), `npm run check` at root. Device crate has 110 tests at 30ef39c. The main tree is /home/kr/workspace/sweepscope (agent worktrees may be on other branches).
