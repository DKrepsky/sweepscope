# ADR 0008: Spike code lives on an unmerged branch; only the report and the raw results are committed to `main`

Status: accepted
Date: 2026-11-22
Issue: #2, #4 (applies to every later spike)

## Context
Stories #2 and #4 say "spike code is not merged into the product". The architecture rules (crate layering, lint, no raw
literals, CI) are written for product code and would only slow throwaway measurement code. At the same time the
numbers must be reproducible by someone else, and a decision must be traceable to a commit and to data. `main` is
protected and changes go through pull requests (workflow section 6).

## Decision
1. **Spike code location.** A spike lives in `spikes/<issue>-<slug>/` on a branch named `spike/<issue>-<slug>`. The
   directory is a standalone Cargo workspace (its own `[workspace]` table, its own `Cargo.lock`), outside the root
   workspace, with no npm package (plain ES modules, Tauri `withGlobalTauri`). It may depend on product crates by path
   (for example `sweepscope-device`) but no product code depends on it. `tools/check-architecture.mjs`,
   `cargo deny` and CI never see it.
2. **Not merged.** The spike branch is never merged. When the report is final, the last spike commit is tagged
   `spike-<issue>-final` and the tag is pushed so the code stays reachable. Product code that comes out of a spike is
   rewritten under the normal rules in its own story (the spike teaches, it does not donate).
3. **What reaches `main`.** A pull request from a separate branch `docs/spike-<issue>-report` carries only
   `docs/spikes/m0/` (report, raw results, the run instructions).
4. **Report format.** `docs/spikes/m0/<issue>-<slug>.md` with these sections, in order: Question and decision rule
   (copied from the implementation spec, fixed before measuring); Method (spike tag and commit, exact commands,
   scenarios, what each metric means, known measurement limits); Environments (one table row per machine, see below);
   Results (one table per metric, medians and percentiles, never only averages); Findings; Decision (go or no-go
   against the rule, with reasons); Changes requested (to ADRs, PRD requirements, the driver); Open points.
5. **Where measurements are recorded.** Raw data in `docs/spikes/m0/results/<issue>/<machine-id>/`: one JSON per run
   (summary statistics and the run configuration), plus gzipped CSV per-sample series when under 200 KB, plus
   `environment.json` and a screenshot of any failure. `machine-id` is a slug of CPU/GPU class and display server
   (`nvidia-rtx4060ti-x11`), never a host or user name. The environment capture script strips host name, user name,
   home paths and serial numbers. The report tables are generated from the JSON by a script kept on the spike branch,
   so they can be regenerated.
6. **Environment record** per machine: distribution and kernel, CPU, GPU and driver version, display server (X11 or
   Wayland), display refresh rate and scale factor, WebKitGTK version, Tauri version, relevant environment variables.
7. **Decision rules are written before the first measurement** in the implementation spec, so the result cannot be
   bent afterwards. A deviation is written into the report with its reason.
8. **Ownership.** Developers build the spike and fill the data tables. The staff engineer writes Decision and Changes
   requested; the human approves them (they may change an ADR or PRD requirement).

## Alternatives considered
- **Merge `spikes/` into `main`, excluded from CI.** Easy to find, but the repository then carries code that is
  not product, its dependencies need auditing, and "not merged into the product" becomes a promise instead of a fact.
- **Keep spike code only locally.** Not reproducible; fails the traceability need.
- **Put the report inside the spike branch only.** The decision would be invisible on `main`.

## Consequences
- Two branches and two commits sets per spike (code and docs); the tag costs nothing.
- Spike code is not covered by the quality gates, which is accepted; it is not shipped. Logic that produces numbers
  (percentiles, ring buffer, payload codec) still gets small unit tests so the numbers can be trusted.
- The workflow's branch naming (`feat/`, `fix/`) gains `spike/` and `docs/spike-`; `.claude/team/workflow.md` section 6
  needs one added line after approval.

## Amendment (2026-11-22, #2 harness review)
Item 5 is changed: a run stores its summary statistics (counts, mean, median, percentiles, maximum) and, for every timing metric, a
histogram in 1 ms buckets (0 to 99 ms plus overflow) in the JSON. Per-sample CSV series are not required. Reason: the statistics and the
histogram reproduce every table of the report and reveal stalls, at a few kilobytes per run, and the harness measures at 1 ms clock
resolution so per-sample values add little. Time series that are time series by nature (the #4 `memory.csv`) are still stored.
