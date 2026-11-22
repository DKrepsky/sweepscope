# ADR 0006: The Graph is built on Canvas 2D behind a renderer interface; WebGL2 follows the M0 spike

Status: accepted (Canvas 2D is the M1 renderer; WebGL2 with persistence in M3, approved at G3 of M1)
Date: 2026-11-21
Issue: none (requested directly by the human)

## Context
PRD 8.2 proposes a custom WebGL2 renderer with a Canvas 2D fallback (DSP-9) and the M0 spike has not yet measured
WebGL against Canvas on the target (WebKitGTK, including NVIDIA). The design-system step needs a Graph that satisfies
AC-G1 and AC-G2 with static data: graticule, up to four traces, frozen state, clipping at the screen edge, crisp lines at
fractional device pixel ratios, draw order. A 4096-sample record is smaller than a wide screen, so Canvas 2D is not a
bottleneck for it. Persistence and 60 fps at scale are not part of this step.

## Decision
`ui/graph` exposes one `Graph` component and an internal `TraceRenderer` interface
(`draw(frame: GraphFrame, size: PixelSize): void`, `dispose()`). The first implementation is `Canvas2dRenderer`: the
graticule is drawn on a device-pixel-snapped layer cached per size, traces on a second layer. Colours come from
`tokens.generated.ts`. `GraphFrame` is plain data (already scaled to screen divisions, 0..10 by 0..8) as the backend
`display` module will produce it, so the backend never learns about the renderer. A WebGL2 renderer, persistence and the
renderer-choice status-bar item (DSP-9) are added behind the same interface in the story that follows the M0 spike.

## Alternatives considered
- **WebGL2 now.** Likely the final answer for persistence, but unmeasured on the target, and larger to test; delays the
  design system for something it does not need.
- **Third-party chart library.** PRD 4.3 notes none handles per-column decimation and persistence; wrapping one would
  leak its types into `ui` (P15).
- **SVG traces.** Simple and crisp, but 4 x 4096 points at 30 to 60 fps is the wrong tool.

## Consequences
- AC-G1 and AC-G2 are satisfied by Canvas 2D; PRD DSP-9 (WebGL2 with fallback) stays open and is called out as an
  explicit gap for the next story.
- The interface costs one small file now and removes a rewrite later.
- Marker ghosts and pending style (8.6) are DOM/SVG overlays above the canvas (`Marker`), not canvas drawing, so they
  work with any renderer.
