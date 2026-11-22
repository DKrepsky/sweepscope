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

## M0 lessons (2026-11)
- Human decisions: spike results (raw data and measured figures) are never committed, reports hold rule outcomes only (ADR 0008). Human accepts "you merge it" for docs PRs when told to; default stays that the human merges. Shorter runs preferred (5 min cap for manual measurements).
- Human has one Linux machine only; do not plan two-machine criteria. Close spikes as "done on one machine" with the gap stated in the report.
- Check PR state before pushing to a branch: PR #43 merged while late commits were still pushed to it; late docs need a new PR.
- Never repeat a subagent's comparison unchecked: I mis-stated the Channel as the cause of the Rust memory growth; the staff engineer found windows of different length. Verify attributions before relaying.
- Xvfb is not installed and sudo is unavailable: extract the .deb to the scratchpad; NVIDIA EGL segfaults under Xvfb, force Mesa.
- Parallel developers on one branch: give each its own worktree and branch, merge afterwards.
