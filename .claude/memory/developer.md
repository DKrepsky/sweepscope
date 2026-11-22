# Memory: developer

Durable lessons only: decisions, conventions, pitfalls, user preferences. No task logs, no secrets. Rewrite, do not just append.

## Project facts
- Project: SweepScope, USB oscilloscope app (Tauri 2, Rust backend, Svelte 5 frontend); first device Hantek 6254BC.
- Docs: `docs/prd.md`, `docs/architecture.md`; process: `.claude/team/workflow.md`.
- Repo tooling (WP1): root `Cargo.toml` (globs), toolchain pinned `1.100`, `tools/check-architecture.mjs` (`cargo metadata --no-deps`), `tools/check-test-names.mjs`, CI in `.github/workflows/ci.yml` (actions pinned by SHA; look up SHAs with `gh api repos/<r>/commits/<tag> --jq .sha`).

## Verified tooling facts
- Cargo workspace glob member with zero matches (missing or empty dir) is a hard error; one file inside (`backend/libs/.gitkeep`) makes it valid. Members that do not exist yet (kernel, src-tauri) also break every cargo command until created.
- `node --test tools` fails on Node 24 (directory is run as a module); use `node --test "tools/*.test.mjs"`.
- `cargo metadata --no-deps` works offline and lists direct deps (name, rename, path) of members: enough for layering rules.
- Cargo `[workspace.lints]` cannot be combined with extra per-crate lints, so `forbid(unsafe_code)` is a `#![forbid(unsafe_code)]` attribute per crate.
- ts-rs latest is 12.x (13 does not exist); tauri crate 2.12.x, `@tauri-apps/cli` 2.12.6; `deny.toml` verified against that tree with cargo-deny 0.22.
- Scratch binaries (actionlint, cargo-deny) can be downloaded from GitHub releases with `gh release download` into the scratchpad; not installed system-wide.

## Lessons
- `git add` with one non-existent path aborts the whole add; check `git show --stat` after committing.
- Verify crates before `src-tauri` exists: scratch workspace in the scratchpad with a symlink to the repo's `backend/`, a Cargo.toml copy without `src-tauri`, and copies of rust-toolchain/rustfmt/clippy toml; run cargo there (generated ts files still land in the repo via the symlink).
- ts-rs 12: `export_to` is relative to `TS_RS_EXPORT_DIR` (default `<crate>/bindings`), so the up-count depends on crate depth (kernel 3, features/* 4). `export_all` takes `&Config` (`Config::from_env()`); `#[ts(export)]` already generates a test that writes the file. `ts-rs` pulls `thiserror`.
- Kernel API facts: `Bus::handle` returns `Result<(), KernelError>` (duplicate refused); `SettingsModule::new` returns `Result`; `CommandDescriptor` lives in kernel, `settings::COMMANDS` is the manifest.

## Frontend facts (WP4)
- `frontend/` pinned latest: svelte 5.59, vite 8, vitest 5, eslint 10, stylelint 18, **typescript 6.0.3** (TS 7 breaks svelte-check/typescript-eslint peers).
- svelte-check needs `src/env.d.ts` with `/// <reference types="svelte" />` and a `<script lang="ts">` block in a component to resolve `*.svelte` imports.
- Vitest jsdom breaks `new URL('..', import.meta.url)`; tests that need file paths use `// @vitest-environment node`. No `@types/node` installed: avoid `node:` imports in TS tests.
- Lint boundary rules are proven by `frontend/tools/lint-rules.test.ts` against `tools/lint-fixtures/src/<layer>/` (ESLint Node API with `cwd` = fixtures dir, Stylelint with `configFile`).
- `src/api/generated/*` is committed output of the Rust ts-rs export: never overwrite by hand (I once clobbered WP2's files; restored). `commands.ts` is a placeholder until WP3.
- Check `git status` for files tracked by other WPs before writing into shared dirs.
