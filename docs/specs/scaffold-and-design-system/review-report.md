# Review report: scaffold and design system

Verdict: changes requested (0 blockers, 2 majors, 12 minors, 1 nit). All local checks pass: `npm run check`, `npm test` (303 frontend tests, Rust, 22 tool tests), `npm audit` 0 vulnerabilities.

## Majors
1. Tooltip.svelte:25,57 (WP6): tooltip id is never referenced; no aria-describedby on the trigger.
2. fit-screen.ts:6-14 and ScopeStage.svelte:23 (WP8/WP9): minimum screen is 512x410 px, design and spec require 32x25.6 rem with the caller converting; nobody converts, so it is wrong at 125 to 200 % text.


## Minors and nits
- WP9: run-button labels and brand hidden at W3, not in design (MainWindow.svelte:57); main.ts:31 unhandled save rejection; H3 clipping not verifiable without e2e.
- WP7: Menu focus return and Tab handling, aria-haspopup (Menu.svelte); error toasts auto-dismiss (Toast.svelte:14).
- WP2: no boundary validation in settings save; second `.corrupt` overwrites first; temp file name collisions (storage.rs:34).
- WP1: lint fixture assertions weak and incomplete; hex rule misses rgb()/markup; test-name check misses it.each/it.fails; tauri:: scan ignores block comments; CI browser job.
- Spec: AC-SB1 text says "below the threshold", code and design wrap at or above (staff-engineer).
- nit WP3: unused serde/ts-rs in src-tauri.

## Verified fine
Crate layering and the extra rules (upward-dependency, unknown-crate-location) with tests; tauri only in main.rs and ipc/; CSP without unsafe-eval or remote origins; capability grants exactly two commands (test); errors without paths; atomic write, corrupt bytes kept in settings.json.corrupt, newer-version file untouched; ts-rs files and commands.ts with CI diff check; no raw colours or px in ui/pages (stylelint, only dynamic inline px for size/position); function size and complexity lint on; dependencies pinned in npm, Cargo.lock with --locked; typescript 6.0.3 works with the toolchain; spec "missing footnote": the placeholder note exists (HorizontalReadout.svelte:32). Dialog uses native modal (focus trap and restore by the platform).

## Re-review (after commits 77f0f0d to 558ad9b)

Verdict: approved. 0 blockers, 0 majors, 6 minors (sent to developer-9, optional). Checks: `npm run check` green, `npm test` green (399 frontend tests, Rust, tool tests), `npm --prefix frontend audit` 0 vulnerabilities. e2e not re-run by the reviewer.

### Earlier findings
- Tooltip aria-describedby: closed (Tooltip.svelte links and unlinks the first child, keeps other tokens; unit and e2e test, test.fail removed).
- fit-screen rem: closed (rem minimum, ScopeStage clamps with `screenMinimumPx`). The minimum 32 x 25.6 rem now exists in `ui/graph` and `app/layout`; accepted by spec 11.11. ADR 0005 text still shows the px-only 4-argument `fitScreen`; staff-engineer to amend.
- WP9 deviations: closed (run labels kept until W3 per 11.1, unhandled save rejection fixed via `chooseTextScale`, H1 to H4 collapse calibrated in layout-calibration.md).
- Menu (focus return, Tab, aria-haspopup via `menuTriggerAttributes`), Toast (error persists with dismiss button, per-kind timeouts, pause on hover and focus): closed.
- Settings: namespace/version/size/count validation on load and save, first `.corrupt` kept, pid temp names, rename-failure cleanup, FileStorage no longer Clone (Mutex): closed, tests cover each limit at the boundary.
- Tooling: lint fixtures per layer pair with rule-specific assertions, rgb/hsl and markup colour rules, it.each and test.fail checks, block-comment tauri scan, CI browsers installed on every run and ubuntu-24.04 pinned: closed.
- Shell deps: serde and ts-rs dropped, serde_json dev-only: closed.

### New minors (message to developer-9)
1. gallery.spec.ts:337 leftover `observation` annotation.
2. text-scale.ts: `persist`, `setTextScale`, `chooseTextScale` redundant layering.
3. notifications.ts: hidden notifications are never dropped; unbounded and they resurface.
4. Button.svelte: forwarded `aria-label` and pointer handlers silently overridden.
5. Marker.svelte: native `title` plus the new focus tooltip.
6. MainWindow.svelte: gutter 2.25 rem and 45 rem floor duplicated outside `app/layout`.

### Hygiene
No scratch files, bus messages or secrets in the tree; the only binaries are the intended e2e snapshots. Untracked docs (ADRs, design, specs) are not yet committed, as expected.
