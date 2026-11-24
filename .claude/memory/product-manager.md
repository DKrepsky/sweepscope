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

## M1 lessons (2026-11)
- Stories #5-#12 were approved as one milestone (one spec, one G3). Human accepts recommended defaults in bulk; D1 (roll steps disabled, #10 AC1 = 25 steps) amended an AC.
- Developers' worktrees start on old branches: always say "create branch from feat/m1-live-view"; merges conflict only in .claude/memory/developer.md (keep both sides).
- Tests that stub Tauri hide missing capabilities (BUG-1: event listen). Real-window QA is needed for IPC changes.
- QA opened the real scope by accident: always set SWEEPSCOPE_NO_HARDWARE=1 and hide USB for automated app runs; tell the human when it happens.
- Visual baselines are regenerated on CI after push; QA cannot do it before.
- M1 closed (2026-11-23): human accepted after a basic hardware run (calibration signal seen), declined the 40-item hardware checklist. Keep future hardware checklists short (about 10 items, critical path only). Human removed visual snapshot tests as too costly: never propose screenshot-comparison tests again.

## M2 lessons (2026-11)
- M2 = #13 #14 #15 #18 #19 #20 as one milestone (PRD section 9). Human approves in bulk and dislikes questions I can answer from the PRD, issues or code: look first, ask only for decisions that change an AC or touch things outside the repo.
- Check spec values against issue ACs before accepting a developer report: "1.000 kHz" needed four digits but the formatter had three; spec said all twelve pins by default but I accepted five. Re-read spec section 1.3 before ruling on a developer's question.
- Verify claims yourself: the reviewer's ts-rs "depth" finding was wrong; the real cause was the export dir resolved per running crate (fixed by TS_RS_EXPORT_DIR in .cargo/config.toml). Run `cargo test --workspace` and check `git status` after any such fix.
- Agent reports can be empty ("placeholder"): check the branch commit and re-run tests before merging.
- Merges conflict only in .claude/memory/developer.md (keep both sides); commit spec edits before spawning worktrees or they will not see them.
- Never let agents delete outside the repo; ask the human first (stray folder at ../frontend was deleted with approval).
- Accuracy AC changes (#15 AC2: 0.01 % at >=500 us/div, 0.02 % faster) were decided by the human after a staff engineer analysis; run an assessment before asking.
