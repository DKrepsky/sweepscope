# Memory: code-reviewer

Durable lessons only: decisions, conventions, pitfalls, user preferences. No task logs, no secrets. Rewrite, do not just append.

## Project facts
- Project: SweepScope, USB oscilloscope app (Tauri 2, Rust backend, Svelte 5 frontend); first device Hantek 6254BC.
- Docs: `docs/prd.md`, `docs/architecture.md`; process: `.claude/team/workflow.md`.

## Lessons
(none yet)
- Check that a11y ids (`$props.id()`) are actually referenced (aria-describedby/labelledby).
- Check px-vs-rem constants where spec says "caller converts"; verify callers.
- Lint fixture tests: assert the specific rule/message, not a rule family; look for missing fixtures per layer pair.
- check-test-names regex skips it.each/it.fails; spec text can contradict design (verify against design.md).
- Run `npm run check`, `npm test`, `npm --prefix frontend audit` as the baseline.
- Spread-forwarding props (`{...rest}`): check which own attributes and handlers silently override forwarded ones and that the type omits them.
- Check numbers duplicated across layers (rem constants in ui and app); specs may record an accepted duplicate (spec 11.x), then ask for the ADR to be amended.
- Derived-store lists that slice for display (notifications) can grow unbounded; check the source list.
- Playwright: grep for leftover annotations and `waitForTimeout` after test.fail removal.
- Re-review baseline also includes `git status` for untracked scratch files and `git ls-files` for png/log outside e2e snapshots.
- Rust crates: check `pub mod` against the spec's re-export list; check `verified`/unverified flags against PRD tables; flag real-time sleeps and wall-clock bounds in tests (use a FakeClock hook); check devices opened before an unplug stay dead after replug.
- Simulators/fakes: check handle generation after replug, initial Attached on subscribe, finite-value validation, and that test-only helpers are cfg(test).
- Tauri 2 custom capabilities: check that core:event:allow-listen/unlisten are present when the frontend uses `listen`; app-only permission lists silently block events.
- Rust threaded modules: check Drop joins, weak handler state, locks never held across bus requests, latest-wins via subscribe_latest; flag real sleeps in tests.
- Help commands shown in banners must work outside the repo root (relative paths).
- Shell commands built from embedded text (printf '%s' '...'): check a test forbids a single quote in the embedded file.
- Re-review: run `npm run check` (prettier) again; fix commits often break format:check.
- ts-rs `export_to` resolves from `<crate>/bindings`: depth differs per crate (kernel needs 3 `..`, feature crates 4). Check `ls ../frontend` next to the repo and `.claude/worktrees/frontend` after cargo test; contract tests read the repo copy and pass vacuously.
- File writers: check multi-file rollback does not delete pre-existing user data; staging files use create_new.
- Hand-rendered TS (StateSnapshot.ts): ask for type-name checks, not only field names.
- Cargo `[env]` config is found by cwd, not --manifest-path: test a crate from outside the repo and check for stray `<crate>/bindings` (untracked, not ignored).
- Rollback code: `fs::read(..).ok()` conflates NotFound with other errors; check before restoring/deleting user files.
- Popup placement with bind:this measurement: check stale position state on reopen.

## CI parity (human rule, 2026-11-24)
- The human requires: the reviewer guarantees every non-test CI check passes (fmt, clippy, cargo deny, cargo audit, frontend lint/format/typecheck/build/npm audit, architecture and tools checks). Security checks are never lowered, ignored or skipped.
- M2 PR #48 went red on `cargo deny` (RUSTSEC-2026-0594, `cookie` via tauri) because cargo-deny/cargo-audit were not installed locally and nobody ran them. Install them (`cargo install --locked cargo-deny cargo-audit`) and run them on every review, including the last one before G4: advisories appear without code changes.
- Read `.github/workflows/ci.yml` at the start of each review; the CI parity table in the review template is mandatory.
- Installed CI tools live in ~/.cargo/bin (cargo-deny, cargo-audit); actionlint 1.7.12 pinned (sha in ci.yml) goes to the scratch folder. `gh pr checks` may show the run of an older head: compare `gh pr view --json headRefOid` with local HEAD (a lockfile fix not yet pushed explains a red job).
- Background shell jobs that chain many npm steps report "completed" early and logs may lag; rerun the missing steps in the foreground before filling the CI table.
- Roll/worker loops: check that end conditions are edge-triggered (generation counter), not level-read per poll, so Stop then Run inside one poll does not continue the old session.
- Multi-source consumer loops (try_next on one subscription, timed wait on another) add latency and idle wake-ups; check POLL constants.
- Check `git status` for untracked QA scratch files (playwright.qa.config.ts) at the end of every review.
- Hand-maintained e2e stub command lists: check a test compares them with src-tauri/capabilities/main.json.
- Worker session counters (bumped on state change) are the accepted fix for Stop-then-Run races; check a test restarts inside one poll.
- Held-state republish patterns (rederive on edit while stopped): check a revision guard that bumps synchronously, equality guard for idempotence, and that no consumer feeds back; downstream consumers are latest-wins and idempotent.
