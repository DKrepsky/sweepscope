# ADR 0002: Design tokens: tokens.css is the single source, a generated TypeScript module mirrors it

Status: proposed
Date: 2026-11-21
Issue: none (requested directly by the human)

## Context
Architecture 5.2 said tokens are "defined once (a typed source file) and compiled to CSS custom properties". The approved
design (G2) instead ships `docs/design-prototype/tokens.css` as the canonical values, naming `--ss-<category>-<role>`,
dark theme only, with the theme-set mechanism kept (`:root[data-theme="dark"]`). Two consumers exist: CSS in
components, and TypeScript that cannot read CSS variables cheaply: the Canvas graph draws channel colours and
graticule colours, and the token tests compute contrast and colour-blind distances (AC-T2, AC-T3).

## Decision
`frontend/src/design-system/tokens.css` is the **only hand-edited source**. It is the prototype file wrapped in
`:root[data-theme="dark"]`. A zero-dependency Node script (`frontend/tools/gen-tokens.mjs`) reads it and writes
`frontend/src/design-system/tokens.generated.ts`: a typed, flat, readonly object of the **resolved literal values**,
keyed by the token name without prefix (`color-ch1` etc.), plus a `tokenNames` union type. The generated file is
committed; CI regenerates and fails on a diff. Components use `var(--ss-*)` only. TypeScript code (Canvas renderer,
tests) reads values through `tokens` and never writes a literal. A parity test compares the token set of
`docs/design-prototype/tokens.css` with the frontend file so design and code cannot drift silently. Adding a theme means
adding another `:root[data-theme="..."]` block; the generator emits one object per theme.

## Alternatives considered
- **Typed TS source compiled to CSS** (as architecture 5.2 first said). A second source of truth next to the approved
  `tokens.css`, a build step in the middle of the designer's workflow, and the human-approved file would become derived.
- **Style Dictionary or similar token tool.** Powerful (multi-platform output) but a large dependency tree for one flat
  file, one theme and two outputs (P15).
- **Read CSS variables at runtime with `getComputedStyle` for the canvas.** Works, but is untestable in jsdom, couples
  rendering to DOM timing, and cannot back the contrast tests.
- **CSS only.** Fails the canvas and the contrast tests.

## Consequences
- The designer edits one CSS file; the generator is about 60 lines; no runtime cost.
- Tokens that reference other tokens (`calc`, `var()`) are not supported by the generator: tokens stay flat literals
  (true today). The generator fails loudly on anything it cannot resolve.
- `docs/design-prototype/` stays as the approved visual reference; once the designer points `design.md` at the
  frontend file the parity test can be retired.
