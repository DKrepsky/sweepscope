# ADR 0001: Project foundation: one Cargo workspace, Tauri 2 shell, Svelte 5 + Vite + TypeScript frontend, npm

Status: proposed
Date: 2026-11-21
Issue: none (requested directly by the human)

## Context
The PRD (8.2) and architecture fix Tauri 2, Rust and Svelte 5 with Vite. Nothing is built yet. Before any feature, we
need the shape of the repository, the build and package tooling, and the rule that makes the dependency rules
(architecture P2 to P7) enforceable by machines and not only by review. Constraints: Linux first (WebKitGTK), the
driver `hantek6xx4-lib` lives in another repository and uses edition 2021, `rustfmt max_width = 120`, pinned actions and
release-plz; the team works with coding agents that need fast, unambiguous feedback (lint, tests, architecture checks).

## Decision
1. **One Cargo workspace at the repository root** with a crate per kernel, lib and feature, plus `src-tauri` as the only
   crate that depends on `tauri`. Members are globs (`backend/kernel`, `backend/libs/*`, `backend/features/*`,
   `src-tauri`) so a new crate needs no edit of the root manifest. Edition 2021, resolver 2, toolchain pinned in
   `rust-toolchain.toml`, workspace-level `[workspace.lints]` and `[workspace.package]`.
2. **Frontend is one npm package in `frontend/`**: Svelte 5 (runes, snippets), Vite, TypeScript strict, no SvelteKit.
   Package manager is **npm** with a committed `package-lock.json` (already installed everywhere, no extra tool; pnpm is
   not on the development machine). A thin root `package.json` holds the Tauri CLI and the task scripts
   (`npm run check`, `npm run test`, `npm run dev`, `npm run build`) so one command runs everything.
3. **Frontend test stack:** Vitest with jsdom and Testing Library for units and components; Playwright (Chromium and
   WebKit) for the layout, HiDPI and visual checks that need a real engine (QA owned).
4. **Quality gates are code, not conventions:** `cargo fmt`, `clippy -D warnings` (plus `too_many_lines` and
   `cognitive_complexity`), ESLint with import-boundary rules, Stylelint for token-only styles, Prettier,
   `svelte-check`, and two small Node scripts for architecture and test-name checks (ADR 0004).
5. **Dependencies in the root manifest of each ecosystem are minimal** (section Dependencies of the implementation
   spec lists every one with its reason).

## Alternatives considered
- **SvelteKit instead of Svelte + Vite.** Adds routing, SSR concepts and an adapter for a single-window app. Rejected
  (already decided in architecture 9).
- **pnpm or npm workspaces with several frontend packages** (design-system as its own package). Stronger enforced
  boundaries but more config, slower feedback, and two lockfile concerns for one small team. The folder rules are
  enforced by ESLint instead. Revisit if the design system is reused by a second app.
- **One crate per layer instead of per feature.** Hides the "no feature imports another feature" rule from the compiler.
  Rejected; per-feature crates make it a manifest check.
- **Separate repositories for backend and frontend.** The typed contract and the design tokens change together with the
  code that uses them; one repository keeps commits atomic.
- **Cargo edition 2024.** Newer, but the driver is 2021 and mixed editions in one mental model add noise; no feature of
  2024 is needed.

## Consequences
- A new feature is a new folder with a `Cargo.toml`; the compiler and `tools/check-architecture.mjs` police the rules.
- CI needs the WebKitGTK development packages for `tauri build`; the default CI path (fmt, clippy, tests, lint,
  frontend tests) must not require them where possible (`src-tauri` is excluded from `cargo test --workspace` jobs
  that do not build the shell, see the spec).
- Two lockfiles (Cargo, npm) are audited in CI.
- If the frontend is later split, ESLint boundaries already name the layers, so the move is mechanical.
