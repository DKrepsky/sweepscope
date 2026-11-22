# Test plan: Project scaffold and design system (no issue, branch `feat/scaffold-and-design-system`)

Sources: `implementation-spec.md` (WP1 to WP11), `docs/design.md` (acceptance criteria AC-*, tiers 3.2, screen 3.3, graph contract 8),
`docs/architecture.md`, ADRs 0001, 0004, 0005, `docs/design-prototype/`.

## Scope and risks

This step ships no v1 feature and no device code. QA verifies the foundation: tooling, the Tauri shell and its security
configuration, the design tokens, the `ui` components, the component gallery, and the static main-window shell with its
responsive tiers and text scale. Interaction criteria that need the backend (AC-I1 to AC-I11) are out of scope.

Risks driving the plan:

1. jsdom cannot check layout, computed styles, focus traps, `<dialog>` top-layer behaviour, contrast or canvas pixels.
   The Playwright suite covers them in a real engine.
2. Responsive tiers depend on rem, text scale and `ResizeObserver`; the spec table has 28 rows, a sample is automated.
3. The scope screen must stay 10:8 and at least 512 x 410 px at 100 %; a collapse step must trigger before it shrinks.
4. HiDPI crispness (AC-G2) differs per scale factor.
5. The CSP and the capability manifest are the only security boundary of the webview.
6. The prototype PNG was rendered before the `border-control` token existed.
7. Known deviations reported by WP9 (see cases KD-1 to KD-7).

## Environment

- Linux, Chromium (Playwright build) and WebKit (Playwright build; may be unavailable if host libraries are missing, in
  which case the limit is reported), against the Vite dev server (`http://localhost:5173`, gallery at `/gallery.html`).
- No scope device and no simulator exist in this step: nothing here needs the 6254BC.
- Tauri IPC is stubbed in the page by `frontend/e2e/support/tauri-stub.ts` (`window.__TAURI_INTERNALS__.invoke`), which
  records calls and keeps the settings blob in `sessionStorage`; this is what lets a browser test change and persist the
  text scale. The real IPC is checked manually in the Tauri window (cases M-*).
- Tests live in `frontend/e2e/`, run with `npm --prefix frontend run test:e2e`. Names are `given ..., when ..., then ...`.
- Evidence (screenshots, logs) goes to `.claude/tmp/scaffold/` and is not committed.

## Test cases

Kinds: **E** automated e2e (Playwright), **A** automated command, **UI** manual visual check, **M** manual in the Tauri window.
Hardware needed is "no" for every case.

### Automated checks of the repository (A)

| ID | Given / when / then | Kind | Covers |
|---|---|---|---|
| A-1 | given the repo, when `npm run check` runs, then fmt, clippy `-D warnings`, eslint, stylelint, prettier, svelte-check, architecture and test-name checks exit 0 | A | WP1 |
| A-2 | given the repo, when `cargo test --workspace`, `vitest run` and `node --test tools` run, then all pass | A | WP1 to WP10 |
| A-3 | given the frontend, when `vite build` and `vite build --mode development` run, then both succeed and the production output has no `gallery.html` | A | WP10 |
| A-4 | given the committed generated files, when the Rust export test runs, then the working tree has no diff in `frontend/src/api/generated/` and `tokens.generated.ts` | A | ADR 0003, 0002 |

### Tokens, theme, contrast (E)

| ID | Given / when / then | Kind | Covers |
|---|---|---|---|
| T-1 | given the gallery in each browser, when computed styles of Button, StatusChip (each kind), Badge (each kind) are read, then colours, border style and sizes equal the token values from `tokens.generated.ts` | E | AC-T1 runtime, AC-BD1, AC-C1 |
| T-2 | given the gallery, when the Tokens section is read, then swatch count equals the colour token count and each swatch resolves to its token value | E | WP10 |
| T-3 | given the root element, then `data-theme="dark"` and `prefers-reduced-motion: reduce` makes motion tokens `0ms` | E | WP5 |
| T-4 | given the gallery, when axe-core runs (WCAG 2 A and AA, colour contrast included), then there are no violations | E | AC-T2 |
| T-5 | given the main window at 1440 x 900, when axe-core runs, then there are no violations | E | AC-T2 |
| T-6 | given the main window with the settings popover, persistence menu and the Accuracy popover open, when axe runs, then no violations | E | AC-T2 |
| T-7 | given the colour-blind simulation of `design-system/colour-blind.ts`, then pairwise channel distances meet the thresholds (vitest, re-run) and a simulated screenshot of the channel row and graph was inspected | A, UI | AC-T3 |

### Components in the gallery (E)

| ID | Given / when / then | Kind | Covers |
|---|---|---|---|
| G-1 | given the gallery, then a section exists for every component of design 9 (list asserted) | E | WP10 |
| G-2 | given a Button reached by Tab, when focus-visible, then the outline is 2 px solid `focus` colour | E | AC-B1 |
| G-3 | given the forced hover, pressed and focus states, then each has a distinct background / outline per token | E | AC-B1 |
| G-4 | given a disabled Button with a reason, when clicked and when Enter and Space are pressed, then no action happens and, on focus, the tooltip shows the reason | E | AC-B1, AC-I13 |
| G-5 | given the Segmented group, when ArrowRight and Enter are used, then focus moves and exactly one item has `aria-pressed=true` | E | AC-S1 |
| G-6 | given a Stepper, when the wheel turns and keys are pressed in a real browser, then the page does not scroll and the value text changes (or the callback log changes) once per notch | E | AC-ST1 |
| G-7 | given a NumberField with an error, then the layout box has the same size as the valid field | E | AC-N1 |
| G-8 | given the gallery Dialog, when opened with the button, then it is a modal `<dialog>`, focus moves inside, Tab and Shift+Tab stay inside (real focus trap), the page behind is inert, Escape closes and focus returns to the opener | E | AC-D1 (jsdom could not verify) |
| G-9 | given the gallery Menu, when opened, then ArrowDown/ArrowUp move focus, a disabled item is focusable and Enter on it does nothing, Escape closes, focus returns to the trigger | E | WP7 (jsdom could not verify) |
| G-10 | given a Tooltip host, when hovered 500 ms and when focused, then the tooltip shows; Escape hides it; and the host element is linked to the tooltip by `aria-describedby` | E | AC-TT1; known item KD-2 |
| G-11 | given the gallery, when every state of StatusChip (active and dim) is measured, then dim keeps the same box and has opacity 0.4 | E | AC-C1 |
| G-12 | given the gallery, then the channel row enabled and disabled have identical height and child count | E | AC-CH1 |
| G-13 | given the gallery banners, then a stack of three shows two, newest first; error uses `role=alert`, info and warning `role=status` | E | AC-BN1 |
| G-14 | given the gallery, every state of each component is visible in a full-page screenshot, inspected against `components.html` | UI | design 9 |

### Graph (E)

| ID | Given / when / then | Kind | Covers |
|---|---|---|---|
| H-1 | given the gallery graph at `deviceScaleFactor` 1, 1.25, 1.5, 2, when the graticule canvas is read back, then the canvas backing size equals CSS size x dpr (rounded) and major lines are one device pixel wide (pixel scan of a row and a column: lit run length is 1 device px, the neighbours are background) | E | AC-G2 |
| H-2 | given the same four scale factors, then a screenshot of the graph (Chromium) matches its stored snapshot | E | AC-G2 |
| H-3 | given the graph, when `deviceScaleFactor` changes (new context at another factor), then the canvas resizes accordingly | E | AC-G2 |
| H-4 | given the graph, then it has `role=img` and an `aria-label` | E | AC-G1 |
| H-5 | given the Marker in the gallery, when dragged with the mouse, then it moves along its axis only and stays inside the clamp | E | AC-MK1 (pointer capture, not testable in jsdom) |

### Main window, tiers, screen size (E)

| ID | Given / when / then | Kind | Covers |
|---|---|---|---|
| W-1 | given the 12 sampled rows of the design 3.2 table (at least 3840/100, 2560/200, 2048/150, 1920/150, 1536/125, 1280/100, 1280/125, 1280/150, 960/100), when the text scale is set through the Settings control and the viewport resized, then `data-width-tier` equals the table value | E | AC-SB1, 3.2 |
| W-2 | given heights 900, 800 (50 rem), 700, 576, when loaded at 100 %, then `data-height-tier` is h1, h2, h2, h3 | E | 3.2 |
| W-3 | given tier w1 / w2 / w3, then toolbar tool labels are visible at w1 only (Cursors, Math, Ref, Save), the drawer button exists at w3 only, the sidebar is in the flow at w1 and w2 | E | 3.2, KD-3 |
| W-4 | given tier w3, when the drawer button is clicked, then the sidebar shows over the screen without modal behaviour and Escape closes it | E | 3.2 |
| W-5 | given every tested tier, then the connection and sample rate items are visible in the status bar | E | AC-SB1 |
| W-6 | given tier w3, then the status bar wraps record, capture, renderer and accuracy to a second row and does not clip horizontally | E | AC-SB1, KD-1 |
| W-7 | given viewports from 1024 x 576 up to 3840 x 2160 at 100 %, then the scope screen aspect is 10:8 (within 1 px) and width >= 512 and height >= 410 | E | 3.3 |
| W-8 | given 1024 x 576 and 1280 x 720 at 100 %, then the scope screen is at least 512 x 410 (H3 at 576 px is a documented risk, KD-4) | E | 3.3 |
| W-9 | given the viewport below the floor (900 x 500), then the content keeps its floor size and the page scrolls, controls are not squeezed (no overlap) | E | 3.2 scroll |
| W-10 | given the text scale changed 100 to 200 at a fixed viewport, then the tier is re-evaluated and root font size changes | E | ADR 0005 |
| W-11 | given text scale 150 chosen, when the page reloads against the stub, then the scale is 150 and `save_ui_state` was invoked with namespace `app.text-scale` | E | WP3, WP9 |
| W-12 | given a Tab press from the page start, then focus visits the regions toolbar, scope, scale strip, vertical, horizontal, trigger, sidebar, status in this order | E | AC-I12, 10.4 |
| W-13 | given the main window, then each dock control has an accessible name, and the Video trigger option is disabled with reason on focus | E | AC-I12, AC-I13 |
| W-14 | given the Run button, when clicked, then the label toggles between Run and Stop (mock flag) and the status chip does not change by itself | E | AC-B2 |
| W-15 | given the main window and the note text, then `Illustrative numbers (placeholders)` is visible | E | WP9 |
| W-16 | given tiers h2 and h3, then the record overview is hidden (h2) and only the selected channel row with channel tabs shows (h3) | E | 3.2 |
| W-17 | given the settings popover, then text scale options 100, 125, 150, 200 exist and the current one is pressed; Escape closes and returns focus | E | WP9 |
| W-18 | given the main window, then no console errors or page errors occurred during load and interaction | E | quality |
| V-1 | given the main window at 1440 x 900, when a screenshot is compared with `docs/design-prototype/main-window.png` visually, then layout, colours and texts match except documented differences | UI | WP9, known item 4 |
| V-2 | given the main window at each tier (w1, w2, w3, scroll; h2, h3) and text scale 100 and 200, then screenshots are inspected for clipping, overlap and spacing | UI | 3.2 |
| V-3 | given a stored Playwright snapshot of the main window at 1440 x 900 (Chromium), then it matches (regression guard for later steps) | E | WP11 |

### Known items to confirm (E or UI)

| ID | Item | Kind |
|---|---|---|
| KD-1 | StatusBar `wrapPriorityAtLeast`: items with a priority number at or above the threshold wrap (0 is the most important). Spec text fixed (resolved); the unit test is the reference | E, doc |
| KD-2 | Tooltip is not linked with `aria-describedby` (see G-10) | E |
| KD-3 | Toolbar labels hidden at w3 for Single, Force, Autoset and the brand (design says only the secondary tools lose text at W2) | E, UI |
| KD-4 | H3 at 1024 x 576: the scope screen is clipped below its 410 px minimum | E |
| KD-5 | Status bar items left-aligned vs prototype; missing `Unverified` footnote line; ultrawide sidebar growth and key `M` not implemented | UI |
| KD-6 | `main-window.png` shows control outlines from before the `border-control` token | UI |
| KD-7 | Design docs and prototype files are uncommitted in the working tree (must not be committed by QA) | A |

### Tauri shell (manual, display available)

| ID | Given / when / then | Kind | Covers |
|---|---|---|---|
| M-1 | given `npm run dev`, when the app starts, then a window titled SweepScope opens at 1440 x 900 showing the main window | M | WP4 |
| M-2 | given the window, when the text scale is set to 150 % and the app is restarted, then it starts at 150 % (settings file written through `save_ui_state`) | M | WP2, WP3, WP9 |
| M-3 | given the devtools console of the running app, when `invoke` of a command outside the manifest is attempted, then it is refused; when the two allowed commands are invoked, then they work | M | ADR 0001, WP4 |
| M-4 | given the CSP in `tauri.conf.json`, when the page tries to load a remote script or image, then it is blocked and a violation is logged (checked in the browser with the same CSP header, and in the Tauri window where possible) | M, E | WP4 |
| M-5 | given the window minimum in `tauri.conf.json`, then it matches the design 3.2 minimum or floor; drag below it and observe | M | WP4, 3.2 |
| M-6 | given a corrupt `settings.json`, when the app starts, then the window opens at 100 % without crashing | M | WP2 |

### Retest round 2 (after the fix rounds and the human decisions: minimum 1024 x 720, Stop `#f2766f`, W3 labels hidden, error Toast persists)

| ID | Case | Kind | Source |
|---|---|---|---|
| L-1 | given each calibrated height threshold of `layout-calibration.md` (wide 756, 740 px at 1440 wide; narrow 944, 912, 804 px at 1024 wide; the same in rem at 125 % and 1280 wide) and one pixel below it, then the tier is the expected one and AC-L1 holds (screen 10:8, at least 32 x 25.6 rem, inside the stage, no content scroll, essentials visible) | E (`tiers.spec.ts`) | AC-L1, 11.10 |
| L-2 | given 1024 x 720 at 100 % then h4 with the whole screen; given 1024 x 719 then `scroll` and the screen keeps its minimum | E | D-L1, AC-L2 |
| L-3 | given the banners (unverified hardware, notifications), then they are not counted in the height thresholds (documented known limit, design 3.2); no automated case | doc | design 3.2 |
| TO-1 | given an info Toast, then it is shown at 3.4 s and gone by 4.8 s; given a warning Toast (failed text-size save), shown at 7.4 s and gone by 8.8 s; pointer over it keeps it past 8.8 s | E (`feedback.spec.ts`) | 11.4 |
| TO-2 | given an error Toast, then it never auto-dismisses and has a dismiss button | unit only (no UI producer of an error Toast exists in the shell) | 11.4 |
| MN-1 | given the persistence button in the real Toolbar, then `aria-haspopup="menu"` and `aria-expanded` follow open and close, Escape and Tab close it and focus returns to the button | E | BUG-5, review |
| TP-1 | given the disabled Video option in the real Trigger group, when focused, then it is described (`aria-describedby`) by the Tooltip and the link is removed when it closes | E | BUG-4 |
| S-1 | given snapshots that recorded old behaviour, then each diff is inspected and intended before `--update-snapshots`; scope-screen snapshots are taken at device resolution (`scale: 'device'`) so line width is visible | UI, E | BUG-2, BUG-6, BUG-8, C-3 |
| CB-1 | given the main window and the channel, status and math colours under protanopia, deuteranopia and tritanopia simulation (Machado 2009, severity 1.0), then channels stay distinguishable (pairwise CIE76 distance) and nothing relies on colour alone | UI | AC-T3 |
| M-2/3/5 | the Tauri window on Xvfb with openbox and an isolated HOME and XDG_CONFIG_HOME: change and restore the text scale across restart, a command outside the manifest is refused, the window cannot be resized below 1024 x 720 | M | see the QA report |

### Guidance: interactive checks of the Tauri window on a virtual display (Xvfb or Xephyr)
Never inject input into the user's live session. Run the app on its own display:
- Start `Xvfb :99 -screen 0 1600x1000x24 &` (headless), or `Xephyr :99 -screen 1600x1000` when someone wants to watch. Set `DISPLAY=:99`,
  `GDK_BACKEND=x11` and unset `WAYLAND_DISPLAY` for every command of the run (`xvfb-run -a` does this for one command).
- A window manager is needed to honour the window minimum and to resize (for example `openbox` on `:99`); without one the minimum
  of M-5 cannot be tested. WebKitGTK may need `WEBKIT_DISABLE_COMPOSITING_MODE=1` (or `WEBKIT_DISABLE_DMABUF_RENDERER=1`) in a virtual display.
- Isolate the settings: `XDG_CONFIG_HOME=$(mktemp -d)` so the user's `settings.json` is neither read nor written; for M-2 restart the
  app with the same `XDG_CONFIG_HOME` and read the file back. For M-6 write a corrupt file there first.
- Drive and observe with `xdotool` (only with `DISPLAY=:99` set explicitly) and `import -window root` or `xwd`; keep screenshots in
  `.claude/tmp/<issue>/`. Devtools for M-3 need a dev build (`npm run dev`).
- Cases M-2, M-3, M-5 and M-6 use this method; mark each result with the display and the window manager used. Not a CI job
  now (a later story may adopt `tauri-driver`).

### Guidance: known-bug annotations
Playwright `test.fail(cond, 'BUG-n: reason')` marks an open bug. The fixing developer removes it with the fix (spec 11.7); at G4 no blocker
or major bug may carry one. Snapshots recorded with a bug are regenerated by QA after the fix. CI snapshots are the arbiter (spec 11.6).

## Entry and exit criteria

Entry: WP1 to WP10 committed; `npm run check` green.
Exit: every case executed or its limit stated; no open bug of severity blocker or major; known deviations listed in the
report with their status. Verdict "pass with known issues" is allowed when only minor or cosmetic items remain and each has
a follow-up owner.
