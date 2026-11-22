# Memory: product-manager

Durable lessons only: decisions, conventions, pitfalls, user preferences. No task logs, no secrets. Rewrite, do not just append.

## Project facts
- Project: SweepScope, USB oscilloscope app (Tauri 2, Rust backend, Svelte 5 frontend); first device Hantek 6254BC.
- Docs: `docs/prd.md`, `docs/architecture.md`, `docs/design.md` (design system, approved), ADRs 0001-0006; process: `.claude/team/workflow.md`.
- Human decisions: dark theme only (dark gray, engineering-instrument look, not SaaS); window minimum 1024x720 (content floor 64x45 rem); Stop red #f2766f; control outlines use `border-control` token. Scaffold step needs no GitHub issue.

## User preferences
- Wants hardware/computation rules out of design docs: designers specify presentation, flag technical questions to engineering.
- Trusts the PM on detail ("I trust you, go ahead"); still wants gate summaries short, with explicit decisions listed.
- "I accept, do it" after a G4 summary = confirm G4 and proceed with the recommended defaults; state what was deferred.

## Lessons
- Run verification (QA + reviewer) in parallel, then one fix round; hold optional minors for a single batch so QA retests once.
- Parallel developers and Playwright runs collide on ports and test-results folders: tell them to use spare ports; QA runs the final full suite alone.
- Developers leave scratch files (pw.tmp.config.ts, src/zz.ts): check `git status` before committing.
- Interactive GUI checks must run under Xvfb with an isolated HOME, never on the user's display.
- Visible look changes forced by accessibility (control outlines, Stop red) must be surfaced to the human at the gate.
