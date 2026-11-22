# ADR 0004: Architecture rules enforced by small repository scripts and lint configuration

Status: proposed
Date: 2026-11-21
Issue: none (requested directly by the human)

## Context
Architecture 8.2 lists the rules checked in CI. They must run locally in seconds, give precise messages to coding
agents, and avoid heavyweight tools (P15). Rust boundaries are visible in manifests; frontend boundaries are visible in
import specifiers; "no raw literals" is visible in CSS; test naming is visible in test titles.

## Decision
| Rule | Mechanism |
|---|---|
| Crate dependency rules (feature to feature, libs to anything of ours, `hantek6xx4-lib` only in `device`, `tauri` only in `src-tauri`, no `tauri::` use outside `src-tauri/src/ipc` and `main.rs`) | `tools/check-architecture.mjs`: Node, no dependencies, reads `cargo metadata --format-version 1` plus a text scan of `src-tauri/src`. Rules are data in the script; unit tests (`node --test`) run it against fixture metadata for every rule. |
| Frontend layer imports | ESLint flat config, built-in `no-restricted-imports` with `patterns` per folder override. No plugin. |
| No raw colour, size or font literals in `ui/` and `pages/` | Stylelint (core rules only: `color-no-hex`, `function-disallowed-list` for colour functions, `unit-disallowed-list: [px]`, `declaration-property-value-disallowed-list` for `font-family`) with `postcss-html` to read `.svelte` `<style>` blocks. TypeScript literals: ESLint `no-restricted-syntax` on hex colour strings in `ui/` and `pages/`. |
| Generated files up to date (tokens, API types) | CI regenerates and runs `git diff --exit-code`. |
| Test names `given ... when ... then ...` | `tools/check-test-names.mjs`: regex over `#[test] fn` names (`given_*_when_*_then_*`) and over `it('...')` titles in `*.test.ts`. |
| Complexity and size | clippy `too_many_lines` (40) and `cognitive_complexity` (15); ESLint `max-lines-per-function` (40, skipping blank and comment lines) and `complexity` (10). |

All run through `npm run check` (root script) and the same commands in CI.

## Alternatives considered
- **dependency-cruiser / eslint-plugin-boundaries / cargo-deny bans.** More expressive but more configuration and
  dependencies; our rules are few and flat. `cargo deny` is still used for licences and advisories, not for layering.
- **Custom Rust lint (dylint).** Overkill.
- **Review only.** Violates the human's goal of machine-enforced rules.

## Consequences
- Rules live in two small scripts and two config files that developers must keep in step with `architecture.md`; a rule
  change is an ADR or an architecture edit plus the script.
- Stylelint covers CSS in components and pages; the `design-system/` folder is exempt because it defines the literals.
- Inline `style="..."` attributes and Svelte `style:` directives in `ui/` and `pages/` must use `var(--ss-*)`; the
  dynamic geometry the Graph needs (pixel sizes) is applied from TypeScript through CSS custom properties set on the
  element, which is allowed because the property name is a token-style name (`--ss-screen-width`), not a literal.

## Amendment (scaffold review)
`check-architecture` also enforces `upward-dependency` (a lower crate layer never depends on a higher one) and
`unknown-crate-location` (every workspace crate lives under `backend/kernel`, `backend/libs`, `backend/features` or `src-tauri`).
Gaps to close, with fixtures that assert the rule id: colour functions beyond hex, `each`/`fails`/`todo` test forms,
block comments in the `tauri::` scan, the full layer-pair import matrix (implementation spec 11.3).
