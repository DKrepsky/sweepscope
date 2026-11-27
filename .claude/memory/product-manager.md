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
- Human rule (2026-11-24): security checks in CI are never lowered or split off; the Code Reviewer guarantees all non-test CI checks pass (CI parity gate in its agent file, workflow step 9, review template). When a spawned reviewer reports, check the CI parity table is filled; a missing table means the review is incomplete.

## M3 lessons (2026-11)
- M3 = #22-#31 as one milestone (PRD section 9). Human accepted all defaults in bulk at G1/G2/G3 and again at G4 ("Confirm and push"); two AC readings (#22 AC1 range table, #26 8-bucket grading) were commented on the issues.
- Wave plan from the staff engineer works: merge B1 contracts first, then waves in parallel in worktrees. Merge conflicts: `.claude/memory/developer.md` (keep both), Cargo.toml/Cargo.lock (keep both sides), and shared test-support files (take the superset, then run vitest).
- Sandbox rule: the auto classifier refuses compound `cd` plus heredoc bus calls once; re-issue the exact call.
- BUG-1 class repeated (BUG-M3-1: main.ts left analysis wiring out); unit tests with stubs cannot see composition gaps. Real-window QA (Xvfb, no hardware) caught it; fix was to remove optional deps and inert fallbacks.
- Developer commit messages guess issue numbers (#17, #13): fix when preparing the PR. Developers' "tests that sleep" or wall-clock bounds are rejected (no-sleep rule).
- Visible side effects to surface at G4: dock thresholds rise, brand text hidden below 100 rem toolbar width, trigger phase chip dropped in roll, binary units (KiB).
- Usage limit can stop a subagent mid-run; resume it with SendMessage after reset instead of respawning.

## M4 lessons (2026-11)
- M4 = #33 #34 #37 #38 #50 (new: remove verification tags). Human changed scope mid-design: no UNVERIFIED/PARTIALLY VERIFIED tags anywhere; calibration is per device with a modal dialog (AC5 of #33 replaced); bottom bar shows CALIBRATED / NO CALIBRATION with accuracy tooltip; connection item restyled ("Ready Hantek 6254BC"). Edit issues and re-run G1 whenever the human changes an AC.
- Human wants the G3 document short and for a human reviewer: modules/libs with responsibilities, interfaces, workflows, added dependencies, critical decisions only; no files, tasks, branches, QA planning. Details go to work-plan.md. Domain logic gets its own module, not a lib.
- Human reviews icons and copy closely (sine curve, no placeholders, single phrases without dots, bold headline plus line below).
- Merge conflicts again only in .claude/memory/developer.md (keep both sides) and tauri-stub.ts (QA edits); commit QA artefacts before merging developer branches. Worktree agents commit with `git add -A` can sweep QA's untracked files.
- Usage limit stopped four agents at once; SendMessage resume worked.
- Reviewer found a real race QA missed (first frame uncalibrated); tests that pass "usually" need a deterministic same-wake-up test.
- Every new executable or command needs a "how do I get and run it" check before G4: the human tried `sweepscope-cli` and got "command not found". Human prefers one install with one executable (`sweepscope capture ...`), no extra flags.
- A permission-check refusal inside a subagent: do not retry or reroute; tell the human plainly what was refused. With their explicit authorization, run it openly as separate plain commands (worked for actionlint and the workspace tests).
- Package builds fail with "OS file watch limit reached" when VS Code holds the inotify watches: ask the human to raise `fs.inotify.max_user_watches` (done once, lost on reboot); no preload shims.
