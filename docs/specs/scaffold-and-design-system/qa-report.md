# QA report: Project scaffold and design system (no issue, branch `feat/scaffold-and-design-system`)

Build or commit: `48165b9` (WP1 to WP10) plus the uncommitted design docs; QA changes on top (WP11).
Date: 2026-11-22
Test plan: [test-plan.md](test-plan.md). Evidence: `.claude/tmp/scaffold/` (not committed).

## Summary

**Verdict: fail (blocks G4 until BUG-6, BUG-1 and BUG-2 are fixed).** Everything else is pass or pass with minor issues.

- Tooling, Rust and unit tests, builds, CSP, token values, focus rings, Dialog trap, tiers, tab order and axe pass.
- Three major defects: the graph graticule vanishes after any redraw (BUG-6), the scope screen minimum is in px not rem (BUG-1),
  and the screen is clipped when the stage is smaller than its minimum, which includes the 1024 x 576 window minimum (BUG-2).
- Seven minor bugs and four cosmetic items are open (list below). No device is involved in this step.
- Playwright suite: 260 tests in Chromium and WebKit, 248 pass, 12 are skipped by design (snapshot tests run in Chromium only), 0 fail.
  Fifteen cases per browser are annotated `test.fail` with their bug id: they pass the run because they fail as expected, and flip to
  a red "unexpectedly passed" when the bug is fixed, which is the signal to delete the annotation.

## Results

### Automated checks of the repository

| Test ID | Result | Notes |
|---|---|---|
| A-1 `npm run check` | pass | fmt, clippy `-D warnings`, eslint, stylelint, prettier, svelte-check, architecture, test names; exit 0 (re-run after adding e2e files) |
| A-2 `npm test` | pass | Rust: all crates ok; vitest 36 files, 303 tests; `node --test tools` 22 pass |
| A-3 builds | pass | `vite build` ok and the production output has no `gallery.html`; `vite build --mode development` ok (gallery included) |
| A-4 generated files | pass | after `cargo test`, `git status` shows no diff in `src/api/generated` or `tokens.generated.ts` |

### Tokens, theme, contrast (`gallery.spec.ts`, `a11y.spec.ts`)

| Test ID | Result | Notes |
|---|---|---|
| T-1 computed styles = tokens | pass | Button (default, latched, hover, pressed, focus), StatusChip (11 kinds, dim 40 % same box), Badge (5 kinds, outline style, role note) |
| T-2 Tokens swatches | pass | count equals colour tokens; each swatch resolves to its token value |
| T-3 dark theme and reduced motion | pass | `data-theme="dark"`; motion tokens 0 ms under `prefers-reduced-motion` |
| T-4 axe on gallery | pass with exemptions | no WCAG 2.1 A/AA violations except BUG-9 and BUG-10 (tracked separately). Dim chips (40 % by design) and disabled controls are excluded as inactive components |
| T-5 axe main window | pass with exemptions | same as T-4; also at text scale 200 % |
| T-6 axe popovers, menu, drawer, dialog | pass | Settings popover, persistence menu, Accuracy popover, W3 drawer, open modal dialog |
| T-7 colour-blind (AC-T3) | partial | vitest `tokens`/contrast checks pass (distance thresholds); no visual simulator run on screenshots, see Not tested |
| AC-B1 focus ring | pass | forced focus and real Tab focus: 2 px solid `focus` colour; every Tab stop of the shell has it (markers use a 2 px drop-shadow ring by design) |

### Components (`gallery.spec.ts`)

| Test ID | Result | Notes |
|---|---|---|
| G-1 section per component | pass | all 18 design 9 components present |
| G-4 disabled Button | pass | click, Enter, Space ignored; reason shows on focus |
| G-5 Segmented | pass | arrows and Enter, exactly one pressed; Video disabled with reason |
| G-6 Stepper | pass | wheel does not scroll the page; `aria-valuetext`; at-max button `aria-disabled` |
| G-7 NumberField | pass | `aria-invalid`, same box as valid, Escape restores |
| G-8 Dialog (real browser) | pass | modal top layer (`:modal`), focus enters on open, focus never reaches the page behind with Tab and Shift+Tab, click outside does not dismiss, Escape closes and returns focus to the opener. The gallery dialog has one focusable control, so the multi-control cycle is not exercised |
| G-9 Menu (real browser) | pass with bug | arrows move focus; disabled item focusable and inert; Escape closes; Enter selects. BUG-5: focus is not returned to the opener |
| G-10 Tooltip | pass with bug | 500 ms delay, focus shows at once, Escape hides. BUG-4: no `aria-describedby` |
| G-11 StatusChip dim | pass | see T-1 |
| G-12 channel row enabled/disabled | pass (main window) | row 4 keeps its box; see C-1 |
| G-13 banners and toast | pass | stack of three shows two newest first; alert and status roles; toast never focused and self-dismisses |
| G-14 gallery visual inspection | pass with cosmetic | all states visible and match `components.html` closely; see C-3 |

### Graph (`graph.spec.ts`)

| Test ID | Result | Notes |
|---|---|---|
| H-1 crisp graticule at scale 1, 1.25 | pass | canvas at device resolution; every major and frame line is exactly 1 device pixel in a grid token colour |
| H-1 at scale 1.5, 2 | **fail** | BUG-8: lines are 2 device px wide centred on a pixel centre, spread over 3 px |
| H-2 snapshots per scale | pass | Chromium snapshots at 1, 1.25, 1.5, 2 stored (they record the current rendering, including BUG-8) |
| H-4 `role=img` and summary | pass | five gallery graphs, `aria-label` present |
| H-5 Marker drag | pass | follows the pointer on its axis only, clamped to the track; disabled Marker does not move |
| Graticule redraw | **fail** | BUG-6: graticule layer blank after the first redraw |

### Main window (`tiers.spec.ts`, `main-window.spec.ts`, `visual.spec.ts`, `csp.spec.ts`)

| Test ID | Result | Notes |
|---|---|---|
| W-1 width tiers | pass | 18 rows of the design 3.2 table (3840 to 960 px, text 100 to 200 %), through the stored text scale; both browsers |
| W-2 height tiers | pass | 900, 896, 895, 720 (h1/h1/h2/h2), 719 and 576 (h3) |
| W-3 per-tier layout | pass with deviation | tool labels at w1 only; w2 secondary tools icon-only; drawer button at w3 only. KD-3 below |
| W-4 drawer | pass | opens over the screen, Escape closes |
| W-5 connection and sample rate | pass | visible at 3840/100, 1440/100, 1280/100, 1280/125, 960/100, 1536/200 |
| W-6 status bar wrap at w3 | pass | rest wraps to a second row, nothing clips horizontally. KD-1 below |
| W-7 screen 10:8 and minimum | **fail** above 100 % | pass at 100 % for 3840x2160 to 1280x720 (10:8, at least 512 x 410, inside the stage). BUG-1 at 125, 150, 200 % |
| W-8 floor 1024 x 576 | **fail** | BUG-2 |
| W-9 below the floor | pass | content keeps 1024 x 576, viewport scrolls |
| W-10 re-evaluation on scale change | pass | 100 to 200 % in the Settings control changes the tier and writes `save_ui_state` |
| W-11 persistence through reload | pass | `load_ui_state` first, 150 % restored (browser, IPC stubbed) |
| W-12 tab order (AC-I12) | pass | toolbar, scope, scale strip, vertical, horizontal, trigger, sidebar, status |
| W-13 names and disabled reasons | pass with bug | every control has a name; Video disabled with reason on focus. BUG-10 (label in name) |
| W-14 Run button | pass | label toggles Stop/Run. The mock also swaps the chips with the flag (mock-only, stated in the code) |
| W-15 placeholder note | pass | visible |
| W-16 h2, h3 | pass | overview hidden at h2; channel tabs and one row at h3 |
| W-17 Settings popover | pass with bug | four options, current pressed, Escape closes when focus is inside. BUG-3 |
| W-18 console errors | pass | none during load, popover, resize; failing backend falls back to 100 % |
| Security (browser) | pass | a command outside the manifest is rejected by the stubbed bridge (see M-3 for the real one) |
| V-1 compare with `main-window.png` | pass with differences | layout, regions, texts, colours match at 1440 x 900. Differences: control outlines (KD-6), status bar left-aligned, missing Unverified footnote line, graph mock trace shapes, "Illustrative numbers" note style, channel row column alignment (C-1) |
| V-2 tiers visual | pass with bugs | w2, w3, scroll, h2, h3 inspected; clipping per BUG-2 |
| V-3 Chromium snapshots | pass | 7 main-window sizes and 17 gallery sections stored, stable over three runs. Snapshots are Linux and system-font dependent; regenerate with `--update-snapshots` on a font change |

### CSP and production bundle (`csp.spec.ts`)

| Test ID | Result | Notes |
|---|---|---|
| M-4 CSP | pass | `tauri.conf.json` policy has no `unsafe-eval` and no remote origin. The production bundle served with that policy runs without violations; a remote script, remote image, inline script and remote fetch are all blocked (`script-src`, `img-src`, `connect-src` violations). Note `style-src 'unsafe-inline'` is allowed (Svelte inline styles) |
| Production bundle | pass | `/gallery.html` is not served; main window renders without console errors |

### Manual check of the Tauri window

| Test ID | Result | Notes |
|---|---|---|
| M-1 window opens | pass | `npm run dev` built and opened a window titled SweepScope at 1440 x 900 (WebKitGTK); the main window and graticule render |
| M-2 text scale persistence | partial | **Load half passed**: with a hand-written `settings.json` (150 %) in the app config directory, the window started at 150 % through the real `load_ui_state`. **Save half not exercised in the window** (see limits); saving is covered by Rust tests and by the stubbed e2e (W-10, W-11) |
| M-3 command outside the manifest | not done in the window | static review: `capabilities/main.json` lists only `allow-load-ui-state` and `allow-save-ui-state`; permissions folder has only those two. Needs devtools in the running app |
| M-5 window minimum | pass by config | `tauri.conf.json` has minWidth 1024, minHeight 576 (ADR 0005). Drag test not done |
| M-6 corrupt settings | not done in the window | covered by the Rust test `given_corrupt_json_when_loaded_then_a_recoverable_error_returns_and_the_file_stays` |

### Known items from the request

| Item | Result |
|---|---|
| (1) StatusBar wrap semantic | Behaviour matches the design (items with priority at or above the threshold wrap; connection and sample rate, priority 0 and 1, never do). The implementation spec sentence ("below the threshold wrap") reads the other way round because priority 0 is the most important. Spec text is ambiguous, code and its test are consistent with the design. At w3 all of record, capture, renderer and accuracy wrap together (design lists a progressive order). Minor, spec clarification |
| (2) Tooltip `aria-describedby` | Confirmed: BUG-4 |
| (3) WP9 deviations | Confirmed: toolbar Single, Force, Autoset and brand lose their text at w3 (design only removes secondary tool text at w2); H3 at 576 clips the screen (BUG-2); status bar items left-aligned (prototype right-aligns renderer and accuracy); the `Unverified` footnote line under the measurements table is missing; ultrawide sidebar stays 344 px at 3440 px width (design: grows to 28 rem); key `M` not implemented (out of scope per the spec, in design 3.2). Each needs a designer or PM decision to accept |
| (4) `main-window.png` outlines | Confirmed: the PNG predates `border-control`; app controls have brighter outlines. No tooling to regenerate it here without the prototype's renderer; the designer should regenerate it. Report only |
| (5) design docs uncommitted | Confirmed and respected: `docs/design.md`, `docs/design-prototype/`, `docs/adr/`, `docs/specs/implementation-spec.md` were not committed by QA |

## Bugs

| Bug | Severity | Owner | Status |
|---|---|---|---|
| BUG-6 Graph graticule disappears after any redraw (Stop click, gallery) | major | developer-8 | open |
| BUG-1 Scope screen minimum is 512 x 410 px, not 32 x 25.6 rem; below minimum at text scale above 100 % | major | developer-8 (fit-screen), developer-9 (ScopeStage) | open |
| BUG-2 Screen clipped when the stage is smaller than the minimum (1024 x 576, 1280 x 900 at 125 %); no size-driven collapse | major | developer-9 | open |
| BUG-8 Graticule not one crisp device pixel at scale 1.5 and 2 (AC-G2) | minor | developer-8 (maybe spec) | open |
| BUG-3 Popovers (Settings, Accuracy) do not close on Escape unless focus is inside, focus not moved or restored | minor | developer-9 | open |
| BUG-4 Tooltip lacks `aria-describedby` | minor | developer-6 | open |
| BUG-5 Menu does not restore focus to its opener | minor | developer-7 | open |
| BUG-7 Disabled Marker reason is a native `title`, not shown on focus | minor | developer-8 | open |
| BUG-9 Stop text contrast 3.63:1 (status-stop on bg-raised, 12 px) | minor | developer-6, designer for the token | open |
| BUG-10 Label in name (WCAG 2.5.3): BW, Inv and channel scale buttons | minor | developer-9 | open |
| C-1 Channel row columns do not align across rows (row 1 POS field wider); disabled row 4 AC/DC border brighter than its other controls | cosmetic | developer-9 | open, not sent as a bug message, follow-up issue via PM |
| C-2 Window at 150 % on 1440 px is in the scroll tier and the Settings button is off screen to the right (by design, but a user who set 150 % must scroll to change it back) | observation | designer | open |
| C-3 Gallery StatusBar warning item shows two warning icons | cosmetic | developer-10 | sent |
| C-4 Marker `aria-valuenow` stays at the committed value during a drag | cosmetic | developer-8 | open, not sent |

Messages sent on the bus: six `bug` messages, to developer-6, developer-7, developer-8 (two), developer-9 and developer-10.
Cosmetic items C-1, C-2, C-4 are for the PM to turn into GitHub issues if wanted.

## Not tested and why

- **Interactive Tauri GUI** (clicking the Settings control, restart persistence of a changed value, devtools invoke refusal,
  dragging the window below its minimum): the only display is the user's live session with a fullscreen game on top. One
  synthetic click reached that window by mistake, so no further input was injected. Rendering and the load path were verified
  by a window-only screenshot. A virtual display (Xvfb or a nested Xephyr) is needed to complete M-2, M-3, M-5.
- **Colour-blind simulator on screenshots** (AC-T3 visual confirmation): not run; only the numeric vitest check exists.
- **WebKit snapshots**: Playwright's WebKit build runs here only after copying the missing libraries `libavif`, `libyuv`,
  `libdav1d` etc. into `~/.cache/ms-playwright/webkit-*/minibrowser-wpe/sys/lib` (no sudo); CI needs
  `npx playwright install --with-deps`. Snapshots are stored for Chromium only. The Playwright WebKit is not WebKitGTK as in the
  Tauri window, which was only checked manually by one screenshot.
- **Modal Dialog blocked while acquiring** (AC-D1 UI half, AC-I11): the shell has no control that requests a modal, only the
  `modalBlockedReason` guard (unit tested).
- **Real HiDPI displays** (OS scale 125, 150, 200 %): emulated with Playwright `deviceScaleFactor`; no multi-monitor scale change test.
- **Light theme, device, protocol, acquisition**: out of scope for this step; no hardware needed.
- Playwright test H-3 (scale change while open) and the drag-below-minimum Tauri check were not written.

## Screenshots and logs
Kept in `.claude/tmp/scaffold/` (not committed): `main-1440x900.png`, `main-1024x576.png`, `main-w3-h1-1280x900-125.png`,
`main-w1-h3-1440x640.png`, `graph-live-1.png`, `cmp-vertical.png`, `gallery-montage-0.png`, `gallery-montage-1.png`,
`tauri-window.png`, `tauri-150.png`, `tauri-dev.log`. Committed Playwright snapshots live in `frontend/e2e/__screenshots__/`.

## How to run
`npm --prefix frontend run test:e2e` (starts the dev server on 5173 and a production preview on 4173; first time
`npx playwright install chromium webkit`). Run one project with `--project=chromium`.
