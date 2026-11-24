# Memory: staff-engineer

Durable lessons only: decisions, conventions, pitfalls, user preferences. No task logs, no secrets. Rewrite, do not just append.

## Project facts
- Project: SweepScope, USB oscilloscope app (Tauri 2, Rust backend, Svelte 5 frontend); first device Hantek 6254BC.
- Docs: `docs/prd.md`, `docs/architecture.md` (draft 0.2), `docs/design.md` (dark only), ADRs 0001-0006 in `docs/adr/`;
  process: `.claude/team/workflow.md`. Scaffold spec: `docs/specs/scaffold-and-design-system/implementation-spec.md`.
- Dev machine: Rust 1.100, Node 24, npm 11; no pnpm. Driver repo uses edition 2021, rustfmt max_width 120.

## Decisions (see ADRs)
- One Cargo workspace (glob members), npm, Svelte+Vite (no SvelteKit), ts-rs for data types only, hand-written client.
- tokens.css is the single token source; generated tokens.generated.ts for canvas and tests.
- Architecture rules enforced by `tools/check-architecture.mjs` (cargo metadata), ESLint no-restricted-imports, Stylelint core rules.
- Layout tiers and 10:8 screen fit computed in TS (rem media queries ignore root font-size text scale).
- Graph: Canvas 2D behind TraceRenderer first; WebGL2 after the M0 spike.
- Text scale = presentation state, persisted via opaque UI-state blob in `settings` (arch 4.5 rule 3).

## Lessons and conventions
- Media queries in em/rem do not follow root font-size changes: never rely on them for text-scale-aware layout.
- Disabled controls use aria-disabled (focusable, tooltip with reason), not the disabled attribute.
- No `ui` barrel file: keeps work packages disjoint. One-line stubs handed over between packages are the only shared files.
- Design doc has an inconsistency to watch: "min window 1280x720" vs "floor 1024x576"; implemented floor in Tauri.
- Do not create empty crates for unbuilt features; first story that needs a lib/feature creates it.
- Units: a pure `ui` function takes px only; the rem-to-px conversion happens in `app/layout` (knows the root font size). Minimum screen is 32 x 25.6 rem.
- Priority numbers: 0 is most important; "wrap at or above the threshold" means the less important items wrap. Write both directions in specs.
- Presentation state (text scale) is applied first, saved after; save failure = warning toast, never rollback. Error toasts do not auto-dismiss.
- Settings boundary validation: namespace regex, 64 KiB json, valid JSON, 64 entries; `.corrupt` keeps the first backup; temp file `{key}.{pid}.tmp`.
- Node 24: `node --test "tools/*.test.mjs"`; `npx --prefix` is unreliable, use `npm --prefix X exec --`; `@types/node` only in `e2e/tsconfig.json`.
- Playwright `test.fail('BUG-n: ...')` marks known bugs; the fixing developer deletes it in the fix commit. Interactive Tauri checks: Xvfb/Xephyr + window manager + isolated XDG_CONFIG_HOME, never the user's session.
- Spec section 11 holds review/QA rulings; keep appending rounds there instead of rewriting approved sections.
- M0 (spec docs/specs/m0-spike): ScopeDevice + DeviceCatalog live in feature crate `device` (ADR 0007, blocking, index-based setup, Clock::sleep for fake time); spikes live on unmerged `spike/` branches in standalone workspaces, only reports and raw results go to main (ADR 0008).
- Driver facts: capture() cannot be cancelled (ACQ-5 "one frame time or 250 ms" covers it); no measured frame cycle in the driver docs; roll overrun is inferred from elapsed time (95 % of 64 KiB span).
- Tauri Channel raw payloads under 1 KiB take a different path (eval) than larger ones (fetch); measure with frame-sized payloads. Write decision rules into the spec before a spike measures.
- Dev machine has an NVIDIA RTX 4060 Ti on X11 (WebKitGTK 2.52.6); a second machine for spikes must come from the human.
- Spike 4 lessons: compare memory runs only at equal wall time (an accelerated 60x run vs a real hour proved nothing); baseline for a ring-buffer RSS rule must be taken after the ring is full; tauri Rust-process RSS includes the whole UI process. Use `git tag -a -m` (tag signing config forces a message). Report worktree for docs: `.claude/worktrees/staff-spike4-report` (branch docs/spike-4-report).
- M1 planning (spec docs/specs/m1-live-view, ADRs 0010-0013): modules `device` (sessions, worker thread per opened device), `acquisition` (setup, run intent, facts; owns restore after replug), `display` (wire frame); cross-module data in `kernel::contracts` (7 messages). `Acquiring` is derived in the frontend (ready and running), never a device state. Bus handlers run on the caller thread: module state in one private Mutex, publish after unlock, never block on hardware.
- Driver facts for M1: `hantek6xx4-lib` 0.2.0 has DeviceMonitor; configure() is the only way to change any setting (over 100 ms, no partial update); no force trigger; capture() not cancellable and Normal sweep waits up to the timeout; ScopeInfo has no model or firmware (use family string, `driver_version()`); permission error is `Error::Usb(rusb::Error::Access)`; position span only about +-1.6 div on ranges 4, 7, 10 (50 mV, 500 mV, 5 V); record is 16.4 div but screen shows 10, so the backend maps screen trigger percent to record percent and crops the window (ADR 0011).
- Roll time bases (index 25+) cannot run as frame captures in M1 (hours); disabled with a reason until #27. 36 steps vs 25 reachable was escalated to the human as a G1 change.
- Dev machine lacks libusb-1.0-0-dev (pkg-config fails); the 6254BC was plugged in at planning time. CI needs the apt step in rust and shell jobs.
- Stub files: when two packages share a crate, the first package creates every stub (`mod` lines, empty files) so packages stay disjoint.
- Design gaps found while planning M1: no simulator selection control, no Settling chip possible, mock model has strings only (backend now sends FormattedValue and fractions).
- M2 planning (spec docs/specs/m2-measure, ADRs 0014-0016): `measurement` feature over pure `libs/measurements` (all 12 quantities for every enabled channel on the whole 16.4 div record; pinning is frontend state); interpolation lives in `display` (mode = domain state, automatic sin(x)/x below 250 samples/div, upsampled frame keeps ADR 0011 layout); backend opens file dialogs through kernel `FilePicker` (tauri-plugin-dialog, Rust only) and `write_atomic`; setups are acquisition state in pretty JSON, last-setup via `Storage`; screenshots composed in the frontend and sent as a raw invoke body. Q-E5: TS port of 3 quantities with shared golden.tsv; Q-E6: origin = left screen edge (needs PM confirm); Q-E8: clip = code 0 or 255, invalidates amplitude/edge quantities only.
- Pitfalls found: `StateSnapshot` lives in acquisition so new slices force it into the shell (ts-rs there); simulator calibrator was an ideal 5 V square, so rise time and 1.9 V criteria were untestable until changed; frequency within 0.01 % is limited by one-sample crossing quantisation unless the signal is locked to the sample clock; app layer may not import ui, so screenshot composer is injected from main.ts; mock sidebar statistics must be hidden when real numbers appear.
- Display precision: significant digits belong to the quantity enum in `libs/engineering-format` (`Digits`); `Hertz`/`Seconds` are shared with capture rate, sample interval and cursors, so a tighter AC gets its own variant (`Frequency`, `Period` at 4 digits) and its own Rust-only golden file. Check every formatted example in a spec against the AC digit count when writing the test strategy (the 1.000 kHz AC was specified but WP6 showed 1.00 kHz).
- R2 (frequency 0.01 %): current estimator (first to last rising crossing) worst 0.023 % at 200 us to 2 ms/div because 250 ns edges are shorter than a sample (800 ns at 200 us/div), so each crossing is uncertain by one sample. A common-slope least-squares fit through all rising and falling crossings (python scratch model, reproduces QA's 0.0225 %) gives worst 0.007 to 0.014 %; physical bound about sample interval / span. Reaching 0.01 % guaranteed at 100 to 200 us/div is not possible; at 500 us and up it is.
- M3 planning (spec docs/specs/m3-analyse, ADRs 0017-0020): roll = run mode of `acquisition` derived from `TimeBaseStep.streaming`; roll items (`SSL1`) carry absolute indices, consumers synthesise gaps from index jumps (no kernel ordered stream); `recording` own crate (`.ssroll`, removed file detected by inode compare because Linux writes to unlinked files succeed); `spectrum` over own radix-2 `libs/fft` (no rustfft), dBV from V rms, separate `SSS1` stream; math = acquisition state, computed once, `FrameCaptured.derived`, channel 5 in frames; XY frontend-only; auto-setup = pure `libs/autoset` over plain observations (LIB_ALLOWLIST is empty: libs may not depend on other libs) driven by acquisition, no intermediate frames; persistence = 8 age-bucket canvases at CSS resolution in `ui/graph`.
- Pitfalls: the 1-2-5 range table has 2->5 gaps, so "3 to 7 divisions" cannot hold for a 7 % amplitude band per decade (reported); driver trigger index is only good to about 20 bytes, so check edge position in the data; hardware height tiers: measure by cloning DOM rows in a throwaway Playwright spec (scratchpad, `main.left` height minus `section.stage` plus 445.6 px); an old calibration can be stale after UI changes; kernel messages are now nine.
- Packages that share a crate: first package pre-wires call sites to stub files (acquisition `domain/math.rs`, `module/derive.rs`, `autoset_run.rs`); lib packages create their own crates so they stay independent.
