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

## Status of the scaffold step (as of QA report)
- Open majors: graticule vanishes after redraw (Graph canvas width reset), screen minimum in px not rem, screen clipped below its
  minimum (1024x576). Minors listed in `docs/specs/scaffold-and-design-system/qa-report.md`. Retest = remove `test.fail` annotations
  (grep `BUG-` in `frontend/e2e`) and rerun the full suite.
- Window minimum is 1024 x 576 (ADR 0005), not the 1280 x 720 of design 3.2 text.
