# ADR 0005: Responsive tiers and scope-screen sizing are computed in TypeScript, not with rem media queries

Status: accepted (G3 approved; amended after review and QA: minimum in rem, converted by `app`; amended again for D-L1 and the content floor 64 x 45 rem)
Date: 2026-11-21
Issue: none (requested directly by the human)

## Context
`design.md` 3.2 specifies tiers in rem so that the in-app text scale (100, 125, 150, 200 %) shifts the available width the
same way as OS DPI: available width in rem = window width / (16 x text scale). The text scale is applied by changing the
root font size. **Media queries in `em` or `rem` do not follow the root font size**: they resolve against the browser's
initial font size, so a text scale of 200 % would not move any breakpoint. Container queries resolve units against the
container and would work, but need a recent WebKitGTK, and the drawer and keyboard behaviour (`M` opens the drawer only
in W3) also need the tier in TypeScript. The scope screen must also be the largest 10:8 rectangle in its stage with a
minimum size (3.3), and the Graph needs its integer device-pixel size anyway for crisp lines (8.5).

## Decision
1. A pure function `layoutTierFor(widthPx, heightPx, rootFontPx)` in `app/layout` returns `{ width: 'w1'|'w2'|'w3'|'scroll', height: 'h1'|'h2'|'h3' }` using the thresholds of design 3.2, converting
   to rem with the **computed root font size**. A `layoutTier` store is fed by a `ViewportSource` (injected: a
   `ResizeObserver` adapter in production, a fake in tests) and re-evaluates when the text scale changes. The main-window
   shell sets `data-width-tier` and `data-height-tier` on its root; CSS selects on those attributes. Thresholds live only
   in the TypeScript module.
2. A pure function `fitScreen(stageWidthPx, stageHeightPx, minWidthPx, minHeightPx)` in `ui/graph` returns the 10:8
   rectangle (width limited: `min(stageW, 1.25 x stageH)`), clamped to the minimum. **Everything it takes and returns is CSS px**;
   it knows nothing about rem. The scope-screen minimum is **32 rem x 25.6 rem** (design 3.3), so it grows with the text scale
   (512 x 410 px at 100 %, 1024 x 820 px at 200 %). The **`app/layout` layer converts**: it holds `SCREEN_MINIMUM_REM` beside the tier
   thresholds and exports `screenMinimumPx(rootFontPx)` (rounded up to whole px), using the same computed root font size as
   `layoutTierFor`. `ScopeStage` passes that px minimum to `fitScreen`. `ui` never imports `app`, so the unit conversion stays out of
   `ui`. The ScopeScreen sets the result as `--ss-screen-width` and `--ss-screen-height`. No CSS container units, no `aspect-ratio` reliance.
   A stage smaller than the minimum does not shrink the screen: the stage scrolls as a safety net and the design's collapse order is
   the primary mechanism.
3. **Content floor 64 x 45 rem** (design 3.2, AC-L1 to AC-L3) and **Tauri window minimum 1024 x 720 logical px** (human
   decision D-L1; equals the floor at 100 % text scale, so at default text scale the window cannot be dragged into vertical
   scrolling). The earlier 1024 x 576 minimum and 64 x 36 rem floor are superseded. Width 1024 is tier W3; W1 begins at 90 rem.
   Below the floor in rem (width < 64 rem or height < 45 rem, which happens at text scales above 100 % on a 1024 x 720 window)
   the region between toolbar and status bar scrolls and the toolbar and status bar stay pinned (the `scroll` tier).
4. **Height tiers are a table of two columns, owned by `app/layout`**, not a single list: the height a layout needs depends on
   whether the dock is one row (W1, W2) or wrapped to two rows (W3, scroll). `layoutTierFor` returns
   `{ width: 'w1'|'w2'|'w3'|'scroll', height: 'h1'|'h2'|'h3'|'h4'|'scroll' }`. `h4` is collapse step 7 of design 3.2 (dock groups as
   tabs) and `height: 'scroll'` is below 45 rem. The thresholds are provisional until calibrated by measurement
   (implementation spec 11.10 holds the values, arithmetic and the recording rule). A calibrated threshold is the measured
   height the configuration needs, plus 0.25 rem, rounded up to 0.25 rem. If the measured need of the last configuration
   (`h4`) exceeds 45 rem, the floor height must rise (design 3.2 allows up to 48 rem = 768 px) and that is a new human decision;
   nobody lowers the screen minimum.

## Alternatives considered
- **CSS `@media (min-width: 90em)`**: simplest, wrong with text scale (see Context). Rejected.
- **Container queries on the app frame with rem thresholds**: no JavaScript for layout, but requires WebKitGTK 2.40 or
  newer, still needs the tier in TypeScript for the drawer, and the 10:8 fit needs container units too (`cqh`).
- **CSS `zoom` for text scale instead of root font size**: changes pixel snapping and breaks the crispness contract.

## Consequences
- The tier logic is a unit-testable pure function (the table in design 3.2 becomes a test table).
- CSS contains the layout per tier but not the thresholds.
- A flash of the wrong tier on first paint is avoided by evaluating synchronously before mounting.
- Window height below 45 rem is now a scroll case for height too (before: tier `h3`); tests and CSS that assumed `h3` at the bottom change.
- Q-E14 is answered: no dependency on rem media queries or container units in WebKitGTK.
