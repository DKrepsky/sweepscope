# Memory: staff-engineer

Durable lessons only: decisions, conventions, pitfalls, user preferences. No task logs, no secrets. Rewrite, do not just append.

## Project facts
- Project: SweepScope, USB oscilloscope app (Tauri 2, Rust backend, Svelte 5 frontend); first device Hantek 6254BC.
- Docs: `docs/prd.md`, `docs/architecture.md` (draft 0.2), `docs/design.md` (dark only), ADRs 0001-0006 in `docs/adr/`;
  process: `.claude/team/workflow.md`. Scaffold spec: `docs/specs/scaffold-and-design-system/implementation-spec.md`.
- Dev machine: Rust 1.100, Node 24, npm 11; no pnpm. Driver repo uses edition 2021, rustfmt max_width 120.

## Decisions (see ADRs)
- One Cargo workspace (glob members), npm, Svelte+Vite (no SvelteKit), ts-rs for data types only, hand-written client.
- tokens.css is the single token source; generated tokens.generated.ts for canvas and tests.
- Architecture rules enforced by `tools/check-architecture.mjs` (cargo metadata), ESLint no-restricted-imports, Stylelint core rules.
- Layout tiers and 10:8 screen fit computed in TS (rem media queries ignore root font-size text scale).
- Graph: Canvas 2D behind TraceRenderer first; WebGL2 after the M0 spike.
- Text scale = presentation state, persisted via opaque UI-state blob in `settings` (arch 4.5 rule 3).

## Lessons and conventions
- Media queries in em/rem do not follow root font-size changes: never rely on them for text-scale-aware layout.
- Disabled controls use aria-disabled (focusable, tooltip with reason), not the disabled attribute.
- No `ui` barrel file: keeps work packages disjoint. One-line stubs handed over between packages are the only shared files.
- Design doc has an inconsistency to watch: "min window 1280x720" vs "floor 1024x576"; implemented floor in Tauri.
- Do not create empty crates for unbuilt features; first story that needs a lib/feature creates it.
- Units: a pure `ui` function takes px only; the rem-to-px conversion happens in `app/layout` (knows the root font size). Minimum screen is 32 x 25.6 rem.
- Priority numbers: 0 is most important; "wrap at or above the threshold" means the less important items wrap. Write both directions in specs.
- Presentation state (text scale) is applied first, saved after; save failure = warning toast, never rollback. Error toasts do not auto-dismiss.
- Settings boundary validation: namespace regex, 64 KiB json, valid JSON, 64 entries; `.corrupt` keeps the first backup; temp file `{key}.{pid}.tmp`.
- Node 24: `node --test "tools/*.test.mjs"`; `npx --prefix` is unreliable, use `npm --prefix X exec --`; `@types/node` only in `e2e/tsconfig.json`.
- Playwright `test.fail('BUG-n: ...')` marks known bugs; the fixing developer deletes it in the fix commit. Interactive Tauri checks: Xvfb/Xephyr + window manager + isolated XDG_CONFIG_HOME, never the user's session.
- Spec section 11 holds review/QA rulings; keep appending rounds there instead of rewriting approved sections.
