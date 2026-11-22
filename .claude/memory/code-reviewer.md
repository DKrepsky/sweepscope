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
