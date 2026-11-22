# Implementation spec: Project scaffold and design system (no issue, requested by the human)

Status: G3 approved; implementation complete, pending QA and code review (see Changelog at the end)
Links: [design.md](../../design.md) (draft 0.2, G2 approved), [design-prototype/](../../design-prototype/), [prd.md](../../prd.md),
[architecture.md](../../architecture.md) (draft 0.2), ADRs [0001](../../adr/0001-project-foundation.md),
[0002](../../adr/0002-design-token-pipeline.md), [0003](../../adr/0003-typed-contract-with-ts-rs.md),
[0004](../../adr/0004-architecture-enforcement-tooling.md), [0005](../../adr/0005-layout-tiers-and-screen-sizing-in-code.md),
[0006](../../adr/0006-graph-renderer-canvas-first.md)

## 1. Scope

### In
1. Repository scaffold: Cargo workspace, Tauri 2 shell (composition root, IPC adapter, security config), shared kernel
   (bus with events and request/response, `Clock`, `Storage`), one feature crate (`settings`) as a walking skeleton,
   Svelte 5 + Vite + TypeScript frontend, tooling (fmt, lint, architecture checks, test-name check), CI.
2. Design system: tokens (dark), generated TypeScript tokens, base styles, icons, all `ui` components of design.md
   section 9 with the acceptance criteria AC-B1 to AC-T3 as tests, a component gallery page reproducing
   `components.html`, the main-window shell reproducing `index.html` with static mock data and the responsive tiers of
   design.md 3.2, and the text scale (100, 125, 150, 200 %) persisted through the backend.

### Out (explicit)
- Everything in PRD section 6 that touches a device: no `device`, `acquisition`, `display`, `measurement` crates, no
  `ScopeDevice`, no `hantek6xx4-lib` dependency, no libs crates (the first story that needs one creates it).
- Streams (kernel implements events and request/response only), state aggregator and snapshot, `Channel` frame transport.
- Interaction criteria that need backend behaviour: AC-I1 to AC-I11, keyboard shortcuts of design 10.4 except Tab order
  (AC-I12), gestures on the scope screen except the component-level ones (Marker drag AC-MK1, Stepper wheel AC-ST1).
- WebGL2 renderer, persistence rendering, decimation, interpolation modes beyond the visual contract on static data.
- XY, FFT, Roll views (the main window shows Yt only; the view tabs are inert), first-run hints, Settings dialog beyond
  the text-scale control, candidate stories C1 to C4.
- Packaging and release (the `bundle` CI job is defined but run manually).
- Light theme (removed by the human). The accuracy-note text is a placeholder (Q-E4).

All numbers and texts in the main-window mock are placeholders and carry the visible note
`Illustrative numbers (placeholders)`, as in the prototype.

### Scope notes and findings for the human
- **Window minimum and content floor** (resolved; was an inconsistency in design 3.2): human decision D-L1, Tauri window
  minimum 1024 x 720; content floor 64 x 45 rem (design 3.2, AC-L1 to AC-L3). 1280 x 720 is no longer a minimum; W1/W2 begin
  at 80 rem (ADR 0005, section 11.10).
- **Text scale is presentation state** (backend never needs it), persisted via the opaque blob of `settings`, in the
  app settings file as the PM asked. This is architecture 4.5 rule 3, not a new rule.
- **Graph on Canvas 2D** (ADR 0006) is a deliberate gap against PRD DSP-9 until the M0 spike.
- Deferred explicitly: Q-E5 cursor formatting, Q-E6, Q-E7 policy, Q-E8 details (architecture 11).

## 2. Design of the change

### 2.1 Modules touched
| Area | Crate or folder | Notes |
|---|---|---|
| Kernel | `backend/kernel` (`sweepscope-kernel`) | `Bus` (events, requests), `Clock`, `Storage` (`FileStorage`, `MemoryStorage`), `KernelError`, test helpers |
| Feature | `backend/features/settings` (`sweepscope-settings`) | UI-state blobs, one file `settings.json`, `version` field |
| Shell | `src-tauri` (`sweepscope-app`) | `main.rs`, `composition.rs`, `ipc/` |
| Frontend | `frontend/src/{design-system,ui,app,pages,api}` | as architecture 3 and 5 |
| Tools | `tools/`, `frontend/tools/` | checks, token generator |

### 2.2 Messages (the only backend contract in this scope)
| Kind | Name | Payload | Direction | Owner |
|---|---|---|---|---|
| Request | `LoadUiState` | `{ namespace: String }` returns `Option<UiStateEntry>` | frontend to `settings` | settings |
| Request | `SaveUiState` | `UiStateEntry` returns `()` | frontend to `settings` | settings |
| Data type | `UiStateEntry` | `{ namespace: String, version: u32, json: String }` | both | settings (ts-rs exported) |
| Event | none | | | |

Tauri commands: `load_ui_state(namespace) -> Option<UiStateEntry>`, `save_ui_state(entry) -> ()`. Errors are
`{ category: 'recoverable' | 'fatal', message }` (kernel `KernelError`, ts-rs exported). The namespace used now is
`"app.text-scale"` with `json = "{\"percent\":100}"`, `version = 1`.

State ownership: the `textScale` value is **frontend presentation state** (store in `app`); `settings` stores it without
interpreting it (architecture 4.5 rule 3). Layout tier and the mock main-window view model are frontend-only. There is no
domain state yet.

### 2.3 Deviations from architecture
None. The scaffold applies it. Architecture text changes made with this spec: layout (3), design system (5.2), client
types (5.8), CI and lint (8.2 to 8.4), decisions (9), open items and Q-E answers (10, 11).

## 3. Interfaces fixed up front

### 3.1 Kernel (Rust, `sweepscope-kernel`)
```rust
pub trait Event: Clone + Send + 'static {}
pub trait Request: Send + 'static { type Response: Send + 'static; }

pub struct Bus;                                   // cheap to clone (Arc inside)
impl Bus {
    pub fn new() -> Self;
    pub fn publish<E: Event>(&self, event: E);
    pub fn subscribe<E: Event>(&self) -> Subscription<E>;   // Subscription::try_next(), ::drain()
    pub fn handle<R: Request>(&self, handler: impl Fn(R) -> Result<R::Response, KernelError> + Send + Sync + 'static);
    pub fn request<R: Request>(&self, request: R) -> Result<R::Response, KernelError>;
}
// request with no handler registered returns KernelError::NoHandler(type name); one handler per request type.
// subscriber queue is bounded (256); overflow drops the oldest and the Subscription reports `lagged()` count.

pub trait Clock: Send + Sync { fn now(&self) -> std::time::Instant; }   // SystemClock, FakeClock (test helper)
pub trait Storage: Send + Sync {
    fn read(&self, key: &str) -> Result<Option<Vec<u8>>, KernelError>;
    fn write(&self, key: &str, bytes: &[u8]) -> Result<(), KernelError>;   // FileStorage writes via temp file + rename
}
pub enum ErrorCategory { Recoverable, DeviceLost, Fatal }
pub struct KernelError { pub category: ErrorCategory, pub message: String }
pub mod testing { pub struct EventRecorder<E>; pub struct FakeClock; pub struct MemoryStorage; }
```
Feature crate public surface: `SettingsModule::new(SettingsDeps { bus: Bus, storage: Arc<dyn Storage> })` registers its
handlers; types `LoadUiState`, `SaveUiState`, `UiStateEntry`. Nothing else.

### 3.2 Frontend api (`frontend/src/api`)
```ts
export interface BackendClient {
  send<C extends CommandName>(name: C, args: CommandArgs[C]): Promise<CommandResult[C]>;
  subscribe<E extends EventName>(name: E, listener: (payload: EventPayload[E]) => void): () => void; // returns unsubscribe
  openStream(kind: never): never;      // reserved; declared now so the interface does not change later
}
// generated/: UiStateEntry.ts, KernelError.ts, commands.ts (CommandName, CommandArgs, CommandResult, EventName, EventPayload)
// tauri-client.ts: createTauriClient(): BackendClient   (only file importing @tauri-apps/api)
// fake-client.ts:  createFakeClient(): BackendClient & { respond(name, handler), emit(name, payload), calls: Call[] }
```

### 3.3 App layer (`frontend/src/app`)
```ts
// text-scale
export type TextScalePercent = 100 | 125 | 150 | 200;
export const textScaleOptions: readonly TextScalePercent[];
export function createTextScaleStore(deps: { client: BackendClient; applyRootFontPercent: (p: number) => void }): {
  subscribe: Readable<TextScalePercent>['subscribe'];            // read side only
  load(): Promise<void>;                                         // LoadUiState -> default 100 on none/invalid
};
export function setTextScale(store, percent: TextScalePercent): Promise<'saved' | 'not-saved'>;   // use case: update store, then SaveUiState
// layout (ADR 0005)
export type WidthTier = 'w1' | 'w2' | 'w3' | 'scroll';   export type HeightTier = 'h1' | 'h2' | 'h3';
export function layoutTierFor(widthPx: number, heightPx: number, rootFontPx: number): { width: WidthTier; height: HeightTier };
export interface ViewportSource { subscribe(listener: (size: { width: number; height: number }) => void): () => void; rootFontPx(): number; }
export function createLayoutTierStore(source: ViewportSource, textScale: Readable<TextScalePercent>): Readable<{ width: WidthTier; height: HeightTier }>;
// main window view model (mock now, backend-fed later; strings are already formatted as the backend will send them)
export interface MainWindowViewModel { /* section 3.5 */ }
export const mockMainWindowViewModel: MainWindowViewModel;
```
Update note (changed in 11.4): `setTextScale` updates the store first and then saves, because the value is presentation state
that the backend never needs; the architecture 5.5 rule 3 (no optimistic update) covers domain state only. The store is the
source of what readers see. It resolves `'saved' | 'not-saved'` and never rejects.

### 3.4 `ui` component contracts (props, snippets, events)
All components: Svelte 5 runes (`$props`), `Snippet` from `svelte`, tokens only, no import from `app`, `pages`, `api`,
`aria-disabled` instead of `disabled`, `disabledReason?: string` rendered through `Tooltip` where the control can be
disabled. Folder `ui/<kebab-name>/` with `<Name>.svelte`, `<Name>.test.ts`, `index.ts` (exports the component and its
prop types). **No barrel `ui/index.ts`.**

```ts
Button:      { size?: 'sm'|'md'|'lg'; pressed?: boolean; disabled?: boolean; disabledReason?: string; title?: string; onclick?: () => void;
               left?: Snippet; children?: Snippet; right?: Snippet; ariaLabel?: string }
RunButton:   { running: boolean; disabled?: boolean; disabledReason?: string; onclick: () => void; children?: Snippet }   // label = action
Segmented:   { value: string; options: { id: string; label?: string; disabled?: boolean; disabledReason?: string; badge?: string }[];
               onchange: (id: string) => void; ariaLabel: string; option?: Snippet<[{ id: string; active: boolean }]> }
Stepper:     { value: string; unit?: string; onstep: (dir: 1 | -1, coarse?: boolean) => void; onreset?: () => void; atMin?: boolean; atMax?: boolean;
               off?: boolean; valueText: string /* aria-valuetext */; ariaLabel: string; badge?: Snippet; unitSnippet?: Snippet }
NumberField: { value: string; commit: (text: string) => void; error?: string; disabled?: boolean; disabledReason?: string;
               ariaLabel: string; left?: Snippet; unit?: Snippet }
Badge:       { kind: 'warning'|'software'|'offline'|'error'|'experimental'; tooltip: string; children: Snippet }
StatusChip:  { kind: 'run'|'stop'|'armed'|'triggered'|'auto'|'disconnected'|'error'|'warning'|'info'|'offline'|'ok'; filled?: boolean; dim?: boolean;
               children: Snippet; left?: Snippet }
Panel:       { collapsed?: boolean; header?: Snippet; body: Snippet; actions?: Snippet }          // TaskPanel = Panel with actions
Readout:     { value: string; unit?: string; muted?: boolean; label?: Snippet; minWidthCh?: number }
Tooltip:     { text: string; delay?: number /* default 500 */; children: Snippet }                  // wraps its child, shows on hover and focus, Esc closes
MeasurementMatrix: { columns: { id: string; label: string; color: string /* token name */ }[];
               rows: { id: string; label: string; cells: Record<string, MeasurementCell> }[]; frozenLabel?: 'Frozen' | 'Offline';
               footnotes: { key: number; text: string }[]; cell?: Snippet<[MeasurementCell]> }
  type MeasurementCell = { state: 'valid'|'unverified'|'invalid'|'stale'; text: string; unit?: string; footnote?: number; tooltip?: string }
StatusBar:   { items: { id: string; priority: number; warning?: boolean; content: Snippet }[]; wrapPriorityAtLeast?: number }
Banner:      { kind: 'info'|'warning'|'error'; ondismiss?: () => void; icon?: Snippet; text: Snippet; actions?: Snippet }
Toast:       { kind: 'info'|'warning'|'error'; text: Snippet; timeoutMs?: number /* info 4000, warning 8000; ignored for error (no timeout, dismiss button always shown, 11.4) */; ondismiss: () => void }
BannerStack: { banners: { id: string; node: Snippet }[]; max?: number /* 2, newest first */ }
Dialog:      { open: boolean; modal: boolean; onclose: () => void; title: Snippet; body: Snippet; actions?: Snippet }
Menu:        { items: { id: string; label: string; disabled?: boolean; disabledReason?: string; badge?: string }[]; onselect: (id: string) => void;
               open: boolean; onclose: () => void; anchor: { x: number; y: number }; item?: Snippet<[{ id: string }]> }
Graph:       { frame: GraphFrame | null; state: 'live'|'frozen'|'empty'; size: { width: number; height: number } /* CSS px, from fitScreen */;
               overlay?: Snippet; gutterLeft?: Snippet; gutterRight?: Snippet; gutterTop?: Snippet }
  type GraphFrame = { channels: { id: 1|2|3|4; points: Float32Array /* divisions, y in -4..4, x spaced uniformly over 0..10 */; selected?: boolean }[];
                      math?: Float32Array; reference?: Float32Array }
  fitScreen(stage: { width; height }, rootFontPx = 16): { width: number; height: number }  // pure, ui/graph; CSS px in and out; as built (supersedes the 4-argument form of 11.1); the rem minimum 32 x 25.6 lives in ui/graph (see 11.11)
Marker:      { kind: 'ground'|'trigger-level'|'trigger-position'|'cursor-x'|'cursor-y'; axis?: 'x'|'y'; position: number /* 0..1 along its gutter */;
               colorToken: string; state?: 'idle'|'pending'|'disabled'; disabledReason?: string;
               ondrag?: (position: number) => void; oncommit?: (position: number) => void; ondelete?: () => void; tag?: Snippet }
Icon:        { name: IconName; size?: 'md' }      // IconName from ui/icon/icons.ts: the 20 names of design 7
```
Events and behaviour in the AC table of section 5. `Marker` emits `ondrag` for every pointer move (the ghost follows the pointer
in the same frame; the confirmed value is a prop from the outside) and `oncommit` on release.

### 3.5 Main-window view model (strings already formatted)
Fields are exactly what `index.html` shows: `toolbar` (running flag, chips `{kind,text}[]`, persistence label, view tab, tool
states), `horizontal` (time/div, trigger position, window, record overview fractions), `channels[4]` (number, enabled,
selected, voltsPerDiv, position, coupling, bw, probe, inv, unverified), `trigger` (type, source, slope, level, sweep),
`cursors`, `measurements` (a `MeasurementMatrix` props object), `status` (connection text, fs, record, capture,
renderer, accuracy placeholder), `graph` (`GraphFrame`, state), `banners`. The developer of WP9 defines the types and the
mock from `index.html`; the types are the future backend-fed contract and carry no mock-specific fields.

## 4. Work packages

Packages are numbered in dependency order; developer ids are suggestions (one person may take several in sequence).
Files are disjoint. Handover point: WP4 creates a one-line stub `frontend/src/pages/main-window/MainWindow.svelte`
and WP9 replaces it; no other shared file.

| WP | Owner | Files and crates owned | Depends on | Done when |
|---|---|---|---|---|
| WP1 Repo tooling and CI | developer-1 | `Cargo.toml` (root), `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `deny.toml`, `package.json` (root), `.editorconfig`, `.gitignore` additions, `README.md`, `tools/**`, `.github/workflows/ci.yml` | none (start first) | `node --test tools` green; `check-architecture` and `check-test-names` run on the repo; CI workflow lints with `actionlint`; root scripts exist |
| WP2 Kernel and settings | developer-2 | `backend/kernel/**`, `backend/features/settings/**` | WP1 manifest | `cargo test -p sweepscope-kernel -p sweepscope-settings` green; clippy clean; ts-rs types exported |
| WP3 Tauri shell | developer-3 | `src-tauri/**` (including `tauri.conf.json`, `build.rs`, `capabilities/`, icons) | WP1, WP2 | `cargo build -p sweepscope-app`; `npm run tauri dev` opens the window showing the frontend; commands registered; CSP and capability set |
| WP4 Frontend scaffold, api, text scale | developer-4 | `frontend/{package.json,package-lock.json,vite.config.ts,svelte.config.js,tsconfig*.json,eslint.config.js,.stylelintrc.json,.prettierrc,index.html,vitest.setup.ts}`, `frontend/src/{main.ts,api/**,app/text-scale/**,app/ui-state/**}`, stub `pages/main-window/MainWindow.svelte` | WP1 | `npm run check` and `npm test` green in `frontend/`; contract test vs the command manifest; lint rules proven by a failing fixture each |
| WP5 Design system | developer-5 | `frontend/src/design-system/**`, `frontend/tools/gen-tokens.mjs` | WP4 | tokens, generated TS, base styles, icons; token tests (AC-T1 to AC-T3) green; generation diff clean |
| WP6 UI set A (controls) | developer-6 | `frontend/src/ui/{button,run-button,segmented,stepper,number-field,badge,status-chip,readout,tooltip,icon}/**` | WP4, WP5 | AC-B1, B2, S1, ST1, N1, BD1, C1, TT1 tests green |
| WP7 UI set B (containers, feedback) | developer-7 | `frontend/src/ui/{panel,measurement-matrix,status-bar,banner,toast,dialog,menu}/**` | WP4, WP5, Tooltip contract (3.4) | AC-M1, SB1, BN1, D1 tests green |
| WP8 Graph and Marker | developer-8 | `frontend/src/ui/{graph,marker}/**` | WP4, WP5 | AC-G1, G2 (unit part), MK1 tests green |
| WP9 Main window shell | developer-9 | `frontend/src/pages/main-window/**`, `frontend/src/app/{layout,main-window,dialog-guard}/**` | WP4 to WP8 | shell reproduces `index.html`; tiers switch; AC-CH1, AC-I12, tier tests green |
| WP10 Component gallery | developer-10 | `frontend/gallery.html`, `frontend/src/gallery-main.ts`, `frontend/src/pages/gallery/**` | WP6 to WP8 | gallery shows every component state of `components.html` |
| WP11 Browser tests and QA plan | qa | `frontend/e2e/**`, `frontend/playwright.config.ts`, `docs/specs/scaffold-and-design-system/{test-plan,qa-report}.md` | WP9, WP10 | contrast, tier, HiDPI, visual and tab-order checks pass in Chromium and WebKit |

Order: WP1 first (hours); then WP2, WP4, WP5 in parallel; then WP3, WP6, WP7, WP8 in parallel; then WP9 and WP10; then WP11
(QA writes the test plan from this spec while WP2 to WP8 are built).

### WP1 Repo tooling and CI
**Tasks**
- Root `Cargo.toml`: `[workspace] resolver = "2"`, `members = ["backend/kernel", "backend/libs/*", "backend/features/*", "src-tauri"]`
  (verify that an unmatched glob is accepted; otherwise list `libs` entries only when they exist), `[workspace.package]`
  (edition 2021, MIT, version 0.1.0), `[workspace.lints]` (clippy `too_many_lines`, `cognitive_complexity`, rust `unsafe_code = forbid` for non-shell crates via per-crate opt-in).
- `rust-toolchain.toml` (stable, pinned minor), `rustfmt.toml` (`max_width = 120`), `clippy.toml`
  (`cognitive-complexity-threshold = 15`, `too-many-lines-threshold = 40`), `deny.toml` (licences MIT, Apache-2.0,
  BSD, ISC, Unicode, MPL-2.0 for Tauri's tree; advisories deny; sources crates.io only).
- Root `package.json`: scripts `dev` (tauri dev), `build`, `check` (runs all of: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, frontend `lint`, `lint:css`, `format:check`, `typecheck`, `node tools/check-architecture.mjs`, `node tools/check-test-names.mjs`), `test` (`cargo test --workspace`, frontend `vitest run`, `node --test tools`), devDependency `@tauri-apps/cli`.
- `tools/check-architecture.mjs` (dependency-free) with the rules of ADR 0004 as a data table; exits non-zero with
  `rule-id: crate X depends on Y` messages. `tools/check-test-names.mjs` for Rust and TS names. `tools/*.test.mjs` with
  fixtures (one violating and one passing metadata sample per rule).
- `.github/workflows/ci.yml` with the jobs of architecture 8.4 (`bundle` as `workflow_dispatch`), actions pinned by
  SHA, least-privilege `permissions: contents: read`, `concurrency` cancel in progress, caching for cargo and npm.
- `README.md`: what it is, prerequisites (Rust, Node 24, WebKitGTK dev packages for Ubuntu 24.04 and Fedora), the five
  commands. Keep it to one screen.
- `.gitignore`: add `frontend/test-results/`, `frontend/playwright-report/`, `src-tauri/gen/`.

**Acceptance checks**: `node --test tools` passes; running both check scripts on the repo (once WP2 to WP3 land) exits 0;
adding a dependency from a feature crate to another feature in a scratch branch makes `check-architecture` fail with a
named rule; `actionlint` passes.

**Tests (given/when/then)**
- given metadata where `settings` depends on `device`, when check-architecture runs, then it fails with rule `feature-to-feature`.
- given metadata where a lib depends on `sweepscope-kernel` bus types (any feature or the kernel crate), when it runs, then it fails with `lib-depends-on-ours` (a lib may depend on the kernel only if it is on an allowlist in the script, empty today).
- given `tauri` in a feature crate, when it runs, then it fails with `tauri-outside-shell`.
- given `use tauri::` in `src-tauri/src/composition.rs`, when it runs, then it fails with `tauri-outside-ipc`.
- given `fn it_works()` marked `#[test]`, when check-test-names runs, then it fails and prints the file and name; given `given_x_when_y_then_z`, then it passes.
- given `it('works', ...)` in a `*.test.ts`, then it fails; given a `given ..., when ..., then ...` title, then it passes.

**Out of scope**: release-plz, signing, dependabot, packaging.

### WP2 Kernel and settings
**Tasks**: implement section 3.1. `Bus` on std types only (`std::sync::{Mutex, mpsc}` keyed by `TypeId`; no async, no error-derive crate: plain `impl Display`). `settings` handlers: `LoadUiState`
reads `settings.json` through `Storage`, returns the entry for the namespace; `SaveUiState` read-modify-write of the
whole file with `{"version":1,"ui":{namespace:{version,json}}}`; corrupt or unreadable file returns a recoverable
error and never panics; unknown future `version` is preserved and not overwritten (returns recoverable error).
`#[derive(TS)]` + `#[ts(export, export_to = "../../../frontend/src/api/generated/")]` on `UiStateEntry` and `KernelError`;
a test in `settings` and `kernel` writes them; plus a `COMMANDS` manifest constant consumed by the shell (WP3) for the
`commands.ts` generation (generation itself is WP3).
`#![forbid(unsafe_code)]`.
**Files**: `backend/kernel/{Cargo.toml,src/**,tests/**}`, `backend/features/settings/{Cargo.toml,src/**,tests/**}`.
**Acceptance checks**: crates compile with clippy `-D warnings`; no dependency on `tauri`, no feature crate imported
by `settings`; `settings` depends only on `sweepscope-kernel`, `serde`, `serde_json`, `ts-rs`.

**Tests**
- Kernel: given a subscriber, when an event is published, then the subscriber receives it once and in order.
- given two subscribers of the same event type, when published, then both receive it; given a different event type, then neither.
- given a full subscriber queue, when more events arrive, then the oldest are dropped and `lagged()` equals the dropped count.
- given a registered handler, when a request is sent, then the typed response returns; given no handler, then `NoHandler` error naming the type.
- given two handlers for one request type, when the second registers, then it replaces nothing and returns an error (one handler per request).
- given a `FakeClock`, when advanced, then `now()` reflects it. Given `FileStorage` in a temp dir, when a write is interrupted after the temp file, then the old file stays intact; given a missing key, then `read` returns `None`.
- Settings: given no file, when `LoadUiState` is requested, then `None`; given a saved entry, when loaded in a new module instance on the same storage, then equal; given two namespaces, when one is saved, then the other is untouched; given corrupt JSON, when loaded, then a recoverable error and the file is not deleted; given `version: 2` in the file, when saving, then a recoverable error and the file is unchanged.
**Out of scope**: streams, bounded request queues, async runtime, `libs`.

### WP3 Tauri shell
**Tasks**: `main.rs` (builds the Tauri app, calls composition), `composition.rs` (creates `Bus`, `FileStorage` on the
app config dir, `SettingsModule`, injects), `ipc/commands.rs` (`load_ui_state`, `save_ui_state` forwarding to
`bus.request`; error mapped to the ts-rs `KernelError`), `ipc/mod.rs`. `tauri.conf.json`: product name `SweepScope`,
identifier `dev.sweepscope.app` (placeholder, human to confirm), window minimum 1024 x 720 (D-L1), initial 1440 x 900,
`frontendDist ../frontend/dist`, `devUrl http://localhost:5173`, `beforeDevCommand`/`beforeBuildCommand` through
`npm --prefix frontend`, CSP `default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; connect-src ipc: http://ipc.localhost` (tighten if the dev server allows), `withGlobalTauri false`.
`build.rs` with an app manifest listing exactly the two commands; `capabilities/main.json` for window `main` allowing only
those. `commands.ts` generation: an export test writes `frontend/src/api/generated/commands.ts` from the `COMMANDS`
manifest and the types. Placeholder icon set generated from a simple SVG (human provides final art later).
**Files**: `src-tauri/**`. **Dependencies**: `tauri`, `tauri-build`, `serde`, `ts-rs`.
**Acceptance checks**: `cargo test -p sweepscope-app` green; `grep tauri::` only in `ipc/` and `main.rs`
(check-architecture); with the frontend dev server running, `npm run dev` opens a window that can call both commands
(manual check, see test plan); a command not in the manifest is refused by the webview.
**Tests**
- given a composed app with `MemoryStorage`, when the `load_ui_state` handler is called for an unknown namespace, then it returns `None`.
- given a saved entry, when loaded through the ipc handlers, then it round-trips.
- given a handler error, when mapped for the webview, then it serialises as `{category, message}` and the message has no file paths.
- given the generated `commands.ts` and the registered command list, when compared, then they are equal (guards ADR 0003 drift).
- given `tauri.conf.json`, when parsed, then min width is 1024, min height 720, CSP has no `unsafe-eval` and no remote origin.
**Out of scope**: state aggregator, events to the webview, `Channel`s, plugins (dialogs, fs), updater, bundling.

### WP4 Frontend scaffold, api, text scale
**Tasks**: `frontend/` as in ADR 0001 with the dependencies of section 7; `tsconfig` strict (+ `noUncheckedIndexedAccess`,
`verbatimModuleSyntax`); `vite.config.ts` (port 5173 strict, `build.target` for the system WebKitGTK `safari15`,
gallery entry only when `mode !== 'production'`); ESLint flat config with the folder import rules (section 5.1 of
architecture) and the complexity limits; Stylelint config per ADR 0004 (ignoring `design-system/`); Prettier; Vitest
(jsdom, `@testing-library/jest-dom`, setup file) ; `api/` per 3.2; `app/ui-state` (typed `loadUiState`, `saveUiState`
services over the client); `app/text-scale` per 3.3; `main.ts` composition root: create client, text scale store, apply
the root font size, load the text scale **before** mounting, mount `pages/main-window/MainWindow.svelte` with the
mock view model and the stores as props. `index.html` sets `data-theme="dark"` and `lang="en"`.
**Acceptance checks**: `npm run lint`, `lint:css`, `format:check`, `typecheck`, `test` all green; for each
boundary rule a fixture file (kept under `frontend/tools/lint-fixtures/`, excluded from normal lint) proves the rule fails:
`ui` importing `app`, `pages` importing `api`, `app` importing a `.svelte` file or `svelte` (other than `svelte/store`),
raw `#fff` and `12px` in a `.svelte` style block in `ui/`.
**Tests**
- given no stored entry, when `textScale.load()` runs, then the store value is 100 and the root font size is 100 %.
- given a stored `{"percent":150}`, when loaded, then the store is 150 and `applyRootFontPercent(150)` was called once.
- given stored garbage or an unsupported percent (130), when loaded, then the store is 100 (no throw).
- given the store at 100, when `setTextScale(125)` runs, then the store is 125 at once and `SaveUiState` is sent with the entry.
- given the backend rejects the save, when `setTextScale(125)` runs, then the store is 125 (applied for the session), the root font size is applied, and the promise resolves `'not-saved'` (never rejects); given a successful save it resolves `'saved'` (ruling 11.4).
- given the fake client, when `subscribe` is unsubscribed, then the listener is no longer called.
- given the generated command list, when compared with the keys the Tauri client maps, then they are equal.
**Out of scope**: real domain stores, error toast path, e2e.

### WP5 Design system
**Tasks**: `tokens.css` = `docs/design-prototype/tokens.css` wrapped in `:root[data-theme="dark"]` (keep the
`prefers-reduced-motion` block, move `color-scheme: dark` inside); `tools/gen-tokens.mjs` generating
`tokens.generated.ts` (ADR 0002; fails on a non-literal value); `base.css` (reset, body, `.num` mono tabular figures
class as utility `ss-num`, caps class, focus ring, scrollbar colours, root font size handling: `html{font-size:100%}`
and the text-scale override set by JS); `icons/` as `icons.ts` holding the 20 path definitions copied from the
prototype symbols (consumed by WP6 `Icon`); `index.ts` exporting `tokens`, `tokenNames`. A pure TS helper module
`contrast.ts` (WCAG ratio) and `colour-blind.ts` (Machado matrices, CIELAB distance) used only by tests.
**Acceptance checks**: `node frontend/tools/gen-tokens.mjs` is idempotent; regeneration diff clean; every token in
design.md 5.2 to 5.8 exists; AC-T1 lint passes on the whole `ui/` and `pages/` tree (once present).
**Tests**
- given `docs/design-prototype/tokens.css` and the frontend `tokens.css`, when their token names and values are compared, then they are identical (parity; removable later).
- given the generated module, when compared with a fresh generation, then equal.
- given a token file containing `var(--x)` as a value, when the generator runs, then it exits non-zero naming the token.
- AC-T2: given primary, secondary and muted text on `bg-panel`, `bg-app` and `bg-inset`, when contrast is computed, then each is at least 4.5; given border, focus and accent on `bg-panel`, then at least 3; given each channel colour on `bg-scope`, then at least 6.7.
- AC-T3: given the four channel colours under deuteranopia, protanopia and tritanopia matrices, when pairwise CIELAB distance is computed, then every pair is at least 20 and the minimum under normal vision is at least 34 (documented design-side value; the test states the measured minima so the designer can compare).
- given `prefers-reduced-motion`, then the motion tokens are 0 ms (parse test of the CSS block).
**Out of scope**: light theme, per-theme runtime switching UI.

### WP6 UI set A: controls
Components and contracts in 3.4, behaviour from design 9 and the prototype CSS (`prototype.css` classes become scoped Svelte
styles using `var(--ss-*)`; do not copy `px` values or literals: Stylelint rejects them; the one-device-pixel border is `var(--ss-border-width)`).
Common rule: disabled means `aria-disabled="true"`, no click, no Enter or Space, still focusable, tooltip shows `disabledReason`.
**Tests (names carry the AC id)**
- AC-B1: given an enabled Button, when pointerdown occurs, then `data-pressed` is set in the same tick, and `onclick` fires once on release (click).
- AC-B1: given Tab focus, when the Button receives focus-visible, then the element has the focus-ring class (Playwright asserts the computed 2 px outline in WP11).
- AC-B1: given a disabled Button with a reason, when clicked, Enter pressed or Space pressed, then `onclick` is not called; when focused, then the tooltip text equals the reason.
- AC-B2: given `running`, then the label is `Stop` with the stop class and glyph; given not running, `Run` with the run class; given click, then only `onclick` fires and the label does not change by itself.
- AC-B2: given focus outside any text field, when Space is pressed on the RunButton, then it activates once.
- AC-S1: given 3 options, when focus is in the group and ArrowRight is pressed, then focus moves to the next enabled option; Enter or Space selects it and exactly one item has `aria-pressed="true"`.
- AC-S1: given a disabled option, when selected by Enter, then `onchange` is not called; when focused, then its reason shows.
- AC-ST1: given a Stepper, when the wheel turns one notch up, or ArrowUp is pressed, then `onstep(1)` fires once; down gives `onstep(-1)`; PageUp and PageDown fire `onstep(dir, true)`.
- AC-ST1: given `atMax`, when the wheel turns up, then `onstep` is not called and the increase button has `aria-disabled="true"`.
- AC-ST1: given a double click on the value, then `onreset` fires; the element exposes `aria-valuetext` equal to `valueText`.
- AC-N1: given editing text, when Enter is pressed then `commit(text)` fires once; when Escape is pressed then the input shows the original value and `commit` is not called.
- AC-N1: given `error`, then the field has `aria-invalid="true"`, the message is rendered in a slot of fixed reserved height, and the layout box does not change size between valid and invalid.
- AC-BD1: given each kind, then the text label is present, `role="note"`-equivalent tooltip text is set, and the kind class differs (warning has the `dashed` modifier).
- AC-C1: given each kind, then text, a glyph element and the kind class are present; given `dim`, then the dim class applies and the element keeps the same box (class only); given the kind changes, then the live region text updates (`aria-live="polite"`).
- AC-TT1: given hover for 500 ms (fake timers), then the tooltip shows; given focus, then it shows immediately; given Escape, then it hides; its text includes the unit and range passed by the caller.
- given each of the 20 icon names, then `Icon` renders an `svg` with `aria-hidden="true"`; given a name not in the set, then TypeScript rejects it (type test).
**Acceptance checks**: Stylelint and ESLint clean; each component has a `ui/<name>/index.ts`; no import outside `ui/`, `design-system/`, `svelte`.
**Out of scope**: application wiring of Space as a global shortcut (feature story); the component only reacts when it has focus.

### WP7 UI set B: containers and feedback
**Tests**
- AC-M1: given a cell `invalid` with footnote 2, then it renders muted `-` followed by `[2]`, the footnote text is listed under the table, and the cell tooltip carries the reason.
- AC-M1: given `unverified`, then the value is followed by `?` and the cell has the `unverified` class (amber left edge).
- AC-M1: given a column for a disabled channel is not passed, then no header or cell for it exists; given `frozenLabel`, then the header shows the label and value cells use the muted class.
- AC-M1: table uses `th` with `scope`.
- AC-SB1: given items with priorities and `wrapPriorityAtLeast`, when rendered at the compact configuration, then items whose priority number is at or above `wrapPriorityAtLeast` wrap to a second row (priority 0 is the most important; a higher number is less important) and the two never-collapse items (connection, effective sample rate: priority 0 and 1, always below any threshold of 2 or more) are always present in the DOM and visible.
- AC-SB1: given an item with `warning`, then it has the warning class and an icon.
- AC-BN1: given an error Banner, then it stays rendered until `ondismiss` is called and shows its actions; given three banners in `BannerStack`, then two are rendered, newest first.
- AC-BN1: given a Toast with `timeoutMs`, when fake time passes, then `ondismiss` is called once; the Toast never receives focus (`tabindex` absent, no `autofocus`).
- AC-BN1: errors use `role="alert"` (assertive), info and warning `role="status"`.
- AC-D1: given a modal Dialog, when `open` is true, then it uses the native `dialog` element in modal mode, traps focus, and Escape calls `onclose`. The "refuse while acquiring" rule is not a Dialog responsibility: it is `app/dialog-guard` in WP9.
- given Panel `collapsed`, then the body is not rendered and the header remains and has `aria-expanded="false"`.
- given Menu open with a disabled item, when ArrowDown moves over it, then it is focusable, `onselect` is not called on Enter, its reason shows; Escape calls `onclose`.
**Out of scope**: modal policy during acquisition (WP9), toasts queue policy beyond the stack of two.

### WP8 Graph and Marker
**Tasks**: `fitScreen`; `Graph.svelte` hosting two canvases; `Canvas2dRenderer` per ADR 0006 implementing design 8.1
(draw order, widths, dash patterns, frozen 40 % opacity), 8.2 (sin(x)/x and linear are the caller's data; dots mode
is a prop `dots?: boolean` drawing 3 CSS px squares), 8.4 (clamp and off-screen triangle), 8.5 (device-pixel snapping at
fractional ratios: line positions at `(floor(x * dpr) + 0.5) / dpr`, widths `max(1 device px, cssWidth * dpr)`);
redraw on `devicePixelRatio` change (`matchMedia('(resolution: ...)')` listener), `role="img"` with an `aria-label`
summary passed as a prop `summary: string`; `Marker.svelte` drag with pointer capture, 8 px hit margin (via padding
token-derived), `Alt+ArrowUp/Down` nudging when focused, `Delete` calls `ondelete`, ghost vs pending styles.
**Tests**
- fitScreen: given a stage 1000 x 500, then width 625 and height 500; given 500 x 1000, then 500 x 400; given a stage below the minimum, then the minimum it was given. `fitScreen` is px in, px out and knows nothing about rem. The minimum is 32 x 25.6 rem; the `app` layer converts it (section 11.1): given root 16 px the minimum is 512 x 410 px, given root 32 px (200 %) it is 1024 x 820 px. Table from design 3.3.
- AC-G1: given a recorded canvas context (a fake 2D context recording calls), when a frame with channels 1 to 4 and a reference is drawn, then the strokeStyle sequence is reference, math, CH4, CH3, CH2, CH1, (selected on top), and each channel uses its token value.
- AC-G1: given state `empty`, then only graticule calls are made; given `frozen`, then traces are drawn with `globalAlpha` 0.4.
- AC-G1: given a point above the top border, then the trace is drawn clamped along the border and a triangle path in the channel colour is added.
- AC-G2 (unit): given dpr 1, 1.25, 1.5, 2, when the graticule is drawn, then every major-line coordinate multiplied by dpr is an integer plus 0.5, and every stroke width multiplied by dpr is at least 1. (Visual snapshots per scale: WP11.)
- AC-MK1: given a draggable Marker, when the pointer moves while captured, then `ondrag` is called for each move with the clamped position; on release `oncommit` is called once with the last position.
- AC-MK1: given `disabled` with a reason, when pointerdown occurs, then no drag starts, and the tooltip equals the reason.
- given `pending`, then the tag shows `...` and the dashed outline class is present.
- given a focused Marker, when Alt+ArrowUp is pressed, then `ondrag` and `oncommit` are called with a step toward 1; Delete calls `ondelete`.
**Out of scope**: WebGL2, persistence, decimation, interpolation algorithms, gutter content positioning (WP9).

### WP9 Main window shell (`pages/main-window`, `app/layout`, `app/main-window`, `app/dialog-guard`)
**Tasks**: types and mock of 3.5 from `index.html`; `layoutTierFor` and `createLayoutTierStore` per ADR 0005 with the
`ResizeObserver` adapter in `app/layout/viewport-source.ts` (state and API of 11.1; notifications of 11.4 in `app/notifications/**`, also owned by WP9) (the only place that touches the DOM in `app`, injected from
`main.ts`; **allowed exception**: this file uses DOM globals, not Svelte), `data-width-tier`/`data-height-tier`
on `MainWindow`; regions A to F of design 3.1 composed from `ui` components: Toolbar (Run/Stop, Single, Force, Autoset,
chips, view tabs (inert), persistence menu, tools, settings menu with the text-scale Segmented wired to
`setTextScale`), Horizontal readout with record overview, ScopeStage with `fitScreen` and Graph (static frame),
gutter markers, ScaleStrip, FrontPanelDock (Vertical with the channel row of 9.1, Horizontal, Trigger of 9.2: Video
disabled with `Exp` badge and reason), Sidebar (Cursors panel, Measurements panel), StatusBar items with the slots of
design 4 (`Accuracy` item opens a Popover built from Panel; text is a placeholder). W2: toolbar secondary tools lose text.
W3: sidebar becomes a drawer toggled by a toolbar button (key `M` is out of scope: no global shortcuts), dock wraps, status
items wrap. H2, H3, H4 (dock tabs) and the height `scroll` per design 3.2 with the thresholds of 11.10. `scroll` tier: content keeps its floor size in a scroll container.
`app/dialog-guard`: `modalBlockedReason(acquiring: boolean): string | null` returns `"Stop acquisition first"` when
acquiring. Tab order as design 10.4. Tooltips with unit and range for every control (APP-2). The visible note
`Illustrative numbers (placeholders)`.
Default mock state: running with chip `Run`; the Run button click toggles only a local mock flag **in the mock view
model factory** (`createMockMainWindow()` returns a model with a `toggleRun()` for the shell demo); this is stated in the
file as mock-only and is removed when the real backend arrives.
**Acceptance checks**: visual comparison to `docs/design-prototype/main-window.png` at 1440 x 900 (QA, WP11); every
control in the dock is operable by keyboard (AC-I12).
**Tests**
- Tier table (design 3.2 examples): given window width and text scale, when `layoutTierFor` is called with `rootFontPx = 16 * scale`, then the tier equals the table row, for all 7 x 4 rows (3840/100 -> w1; 2560/200 -> w2; 2048/200 -> w3; 1920/200 -> scroll; 1536/125 -> w3; 1280/100 -> w2; 1280/125 -> w3; 960/100 -> scroll ...). Boundaries: 90 rem exactly -> w1; 89.99 -> w2; 80 -> w2; 79.99 -> w3; 64 -> w3; 63.99 -> scroll. Height: the boundaries of the 11.10 table, one test per threshold (at, and 0.01 rem below), both columns; 44.99 rem -> scroll.
- given the viewport source emits a new size, when the store is read, then it reflects the tier; given the text scale store changes from 100 to 200 with the same pixel size, then the tier is re-evaluated (the ADR 0005 reason).
- given tier w3, when the toolbar drawer button is activated, then the sidebar drawer is shown and Escape closes it; given tier w1, then the drawer button is hidden.
- AC-SB1: given tier w3 (rendered with a fake viewport), then connection and effective sample rate items are present in the status bar.
- AC-CH1: given a channel row enabled and then disabled in the model, when rendered, then the row element has the same bounding classes and child count, controls have `aria-disabled`, and the number toggle has the outline-only class.
- AC-I12: given the rendered shell, when Tab is pressed from the start repeatedly, then focus visits regions in order toolbar, scope screen, scale strip, Vertical, Horizontal, Trigger, sidebar, status bar (asserted by `data-region` of each focused element's ancestor).
- AC-D1 (guard only): given `acquiring = true`, then `modalBlockedReason` returns `Stop acquisition first`; given false, `null`.
- given the text-scale Segmented, when 150 is selected, then `SaveUiState` is sent through the fake client and `applyRootFontPercent(150)` is called.
- given the mock model, then every channel colour on screen uses a channel token (no inline colours: Stylelint).
**Out of scope**: all gestures of design 10.2 on the screen, keyboard shortcuts, XY/FFT/Roll, backend events.

### WP10 Component gallery
**Tasks**: `gallery.html` (not in production build), `gallery-main.ts`, `pages/gallery/Gallery.svelte` with sections mirroring
`components.html`: tokens swatches (from `tokens.generated.ts`, so the sheet cannot drift), buttons and states, run
control, segmented, steppers, number fields, badges, status chips (all kinds, active and dim), measurement cell states,
channel row (enabled and disabled), banners, toast, task panel, tooltip (forced visible example), gap and disconnected
overlays on a Graph, status bar items normal and warning, dialog (open button), menu. States forced by attributes
(`data-force-state`) or props so a screenshot shows hover, pressed and focus without a pointer. A theme attribute selector
for future themes (only `dark`).
**Acceptance**: every component of design 9 appears with every state listed in its table; page lints clean.
**Tests**
- given the gallery, when mounted, then every component name of design 9 has a section with `data-component="<name>"` (list asserted).
- given the gallery, when mounted, then it contains no raw colour literal in rendered inline styles.
- given the gallery section `Tokens`, then the number of swatches equals the number of colour tokens in `tokens.generated.ts`.
**Out of scope**: Playwright snapshots (WP11).

### WP11 Browser tests and QA plan (qa)
QA writes `test-plan.md` first (while WP2 to WP8 are built), then `frontend/e2e/**` with Playwright (Chromium and WebKit
projects) against the Vite dev server: gallery and main window. Checks: computed styles equal token values for
Button, StatusChip, Badge (AC-T1 at runtime); focus ring 2 px and `focus` colour (AC-B1); contrast with axe-core
(AC-T2; dependency `@axe-core/playwright`); tiers at the viewport sizes of the design 3.2 example table with
`textScale` set through the settings control (AC-SB1, W1 to W3, scroll floor, H1 to H3); the scope screen is 10:8 and
never below 512 x 410 px at 100 %; HiDPI snapshots of the graticule at `deviceScaleFactor` 1, 1.25, 1.5, 2 (AC-G2);
visual snapshot of the main window at 1440 x 900 against the prototype; 4-region Tab order. A manual checklist for the
Tauri window (minimum size, text scale persists across restart, CSP blocks remote content) and the colour-blind
simulator check (AC-T3 visual confirmation). Bugs go to the owning developer as `bug` messages.

## 5. Acceptance criteria coverage
| Criterion | Where tested |
|---|---|
| AC-B1, B2, S1, ST1, N1, BD1, C1, TT1 | WP6 component tests; computed-style parts in WP11 |
| AC-M1, SB1, BN1, D1 | WP7; AC-SB1 tier part in WP9 and WP11; AC-D1 guard in WP9 |
| AC-G1, G2, MK1 | WP8; AC-G2 visual in WP11 |
| AC-CH1 | WP9 |
| AC-T1 | Stylelint and ESLint (WP4 config, whole tree), runtime in WP11 |
| AC-T2, AC-T3 | WP5 pure tests; WP11 axe and visual |
| AC-TT1 | WP6 (every control that can be disabled shows its reason: WP6, WP7, WP9) |
| AC-I12 | WP9 |
| AC-I1 to AC-I11, AC-I13 | Not in scope except AC-I13 (Segmented disabled option, WP6) and AC-I8 stepper reset (WP6); the rest need the backend |

## 6. Test strategy
- BDD names everywhere (`given_..._when_..._then_...` in Rust, `given ..., when ..., then ...` in `it()` titles), enforced by
  `check-test-names`. One behaviour per test.
- Rust: module tests through the public message interface with `MemoryStorage` and the in-memory bus; no private state.
- Frontend `app`: unit tests with the fake `BackendClient` and a fake `ViewportSource`; fake timers for tooltip and toast
  (no real sleeping).
- Components: Testing Library with `user-event` pointer sequences; jsdom cannot compute layout, so layout, computed styles,
  contrast and HiDPI are Playwright (WP11) and must not be faked in jsdom.
- Canvas: tests use a recording fake 2D context; pixel snapshots only in Playwright.
- Hardware: none in this scope.

## 7. Dependencies (justified)
| Dependency | Where | Why not write it | Licence | Notes |
|---|---|---|---|---|
| `tauri`, `tauri-build`, `@tauri-apps/cli`, `@tauri-apps/api` | shell | The decided app shell | MIT / Apache-2.0 | Tauri 2 line, pinned |
| `serde`, `serde_json` | kernel/settings/shell | Serialisation of the settings file and IPC | MIT / Apache-2.0 | ubiquitous |
| `ts-rs` | settings, kernel, shell (export tests) | Type generation (ADR 0003) | MIT | small derive crate |
| `svelte` (5), `vite`, `@sveltejs/vite-plugin-svelte`, `typescript`, `svelte-check` | frontend | Decided stack | MIT | |
| `vitest`, `jsdom`, `@testing-library/svelte`, `@testing-library/jest-dom`, `@testing-library/user-event` | frontend dev | Component and unit tests without a browser | MIT | |
| `eslint`, `typescript-eslint`, `eslint-plugin-svelte`, `stylelint`, `postcss-html`, `prettier`, `prettier-plugin-svelte` | frontend dev | Lint and format; boundaries use built-in rules only | MIT | no boundaries plugin |
| `@playwright/test`, `@axe-core/playwright` | frontend dev (QA) | Real engine for layout, HiDPI, contrast | Apache-2.0 / MPL-2.0 | browsers downloaded in CI only; WebKit approximates WebKitGTK, not identical |
| `cargo-deny`, `cargo-audit`, `actionlint` | CI | Licence and advisory gates | MIT/Apache-2.0 | CI-only |
No icon font, no UI kit, no CSS framework, no state library, no popper library, no font downloads (design 5.6 and 7).
Versions: latest stable at implementation, pinned exactly through the lockfiles; each PR states any addition (architecture 8.1).

## 8. Risks and hardware limits
- No hardware is touched. Mock content must never be mistaken for measurement: the placeholder note is always visible and
  the status-bar renderer and accuracy items say so.
- **WebKitGTK versus Chromium:** development and most CI use Chromium; the target is WebKitGTK. The Playwright WebKit
  project and the manual Tauri checklist (WP11) mitigate; any `:focus-visible`, `dialog`, or `ResizeObserver` quirk is
  a bug against the component. Target minimum WebKitGTK 2.40 (Ubuntu 22.04 and newer with updates); `dialog` and
  `:focus-visible` need it.
- **Text scale and fractional layout:** 125 % and 150 % produce fractional pixels; the Graph snaps in code (WP8).
- **Tier flicker** when the text scale changes: the store re-evaluates synchronously from the same source; covered by a test.
- **Hand-written command list drift** (ADR 0003): guarded by the manifest test.
- **Scope of WP9 is the largest package**; if it slips, split Toolbar and StatusBar items into a second package after the
  shell skeleton lands (files already separate under `pages/main-window/`).
- **Canvas vs WebGL2:** performance on 4 x 4096 samples at 60 fps is unmeasured; ADR 0006 keeps the swap cheap.

## 9. Open items deferred (not blocking G3; carried to the first feature stories)
Q-E4 text (PM), Q-E5 cursor formatting, Q-E6 cursor origin, Q-E7 settings policy, Q-E8 clipping details, WebGL2 and
persistence, streams, state aggregator and snapshot, first-run hints, application identifier and final icon (human).

## 10. Rollout and migration
Greenfield. Land as one feature branch `feat/scaffold-and-design-system` in package order (WP1 first); every package
leaves `npm run check` and CI green. Specs, ADRs and code are committed together after G4. The human merges via pull request.

## 11. Rulings after technical review and QA (round 1)
Rulings are binding; the owner of each work package changes code and tests accordingly. Where a ruling touches files of a
package already merged, the package owner named below makes the change (this is the only reason to reopen a package).

### 11.1 Minimum screen size and fitScreen (ADR 0005 updated; review major 2, BUG-1, BUG-2) - developer-8, developer-9
- The minimum is **32 rem x 25.6 rem** (design 3.3). One unit of rem is the computed root font size, so the px minimum is
  `32 x rootFontPx` by `25.6 x rootFontPx` (512 x 410 at 100 %, 1024 x 820 at 200 %).
- **`app/layout` converts.** It exports `SCREEN_MINIMUM_REM = { width: 32, height: 25.6 }` next to the tier thresholds (the only
  place with layout numbers in rem) and `screenMinimumPx(rootFontPx): { width: number; height: number }` (pure, unit tested at
  16, 20, 24, 32). The layout store exposes `rootFontPx` with the tiers and updates it on text-scale change.
- **`fitScreen` stays in px** (stage, minimum and result are all CSS px); `ui/graph` never imports `app`. `ScopeStage` (pages) calls
  `fitScreen(stage.width, stage.height, min.width, min.height)` with `min = screenMinimumPx(rootFontPx)`. `screenMinimumPx` rounds up (`Math.ceil`). Rounding of the result: width and height are floored to whole CSS px by `fitScreen` so the Graph gets integer sizes at 100 %.
- Tests: given root 16 px the minimum is 512 x 410 (rounded up from 409.6 to whole px: `Math.ceil`); given root 32 px it is 1024 x 820;
  given a 1000 x 500 stage at root 16, then 625 x 500; given the same stage at root 32, then the minimum 1024 x 820 (stage is smaller).
  Playwright W-7 must pass at 125, 150 and 200 % (QA removes the BUG-1 `test.fail`).
- **When the stage is smaller than the minimum** the screen keeps the minimum and never shrinks (design 3.3), and the stage is
  `overflow: auto` as a safety net (so nothing is silently clipped). The design's collapse order (design 3.2) is the primary
  mechanism and developer-9 must implement the steps the design lists for H2 and H3 until 1024 x 720 at 100 % (the floor, D-L1) shows the whole
  screen (thresholds and arithmetic in 11.10). If the arithmetic shows that the screen cannot reach 25.6 rem at h3 even after all collapse steps, developer-9
  reports the numbers to the staff-engineer and the designer (needs a design decision: lower h3 minimum or scrolling dock). Do not
  invent a smaller minimum.
- Also fix in the same package as a note, not a ruling: the `ultrawide` sidebar growth, key `M`, `Unverified` footnote line and
  status bar alignment (design deviations of KD-3 and KD-5) go to the PM and designer to accept or schedule; developers do not
  change them now. The Toolbar W3 label loss (KD-3): restore the design behaviour (only secondary tool labels hide at W2, the run
  buttons and brand keep their text at W3) unless the width arithmetic at 960 to 1280 px shows it does not fit, in which case
  developer-9 reports it to the designer instead of deviating silently.

### 11.2 Settings and storage (review WP2 minors) - developer-2
- **Validation at the `settings` boundary**, same validator for `LoadUiState` and `SaveUiState`, error is `KernelError::recoverable`
  with a fixed message (never echoes the payload or a path):
  - `namespace`: matches `^[a-z][a-z0-9.-]{0,63}$` (the existing `app.text-scale` fits); otherwise "invalid ui state namespace".
  - `version`: at least 1.
  - `json`: at most 65 536 bytes (64 KiB) and valid JSON (`serde_json::from_str::<serde::de::IgnoredAny>`; any JSON value is
    accepted, structure is the frontend's business); otherwise "ui state is too large" or "ui state is not valid JSON".
  - at most 64 namespaces in the file; a new namespace beyond that is refused ("too many ui state entries"); updating an existing one is allowed.
  - Validation happens before the file is read so a rejected save cannot touch storage.
- **`settings.json.corrupt`: keep the first backup, never overwrite it.** The first corrupt content is the evidence closest to the
  cause; later corruption is usually derived from it. If `settings.json.corrupt` already exists, the new corrupt bytes are not
  saved again and the save proceeds (the user's current action is not blocked). No timestamps (no `Clock` dependency in storage).
- **Temp file name**: `{key}.{process id}.tmp` appended to the full key (so `a.json` and `a.txt` cannot collide, as
  `with_extension` made them), `File::create` truncates a leftover from a crash, the temp file is removed when the rename
  fails, and `FileStorage` holds one `Mutex` that serialises writes inside the process. Cross-process locking is out of scope
  (single instance is enforced by the shell later; risk noted).
- Tests: given a namespace `Bad Name`, a 65 537-byte json, `{`, and a 65th namespace, when saved, then each returns the recoverable error
  and the file bytes are unchanged; given an existing `.corrupt` and a new corrupt file, when saved, then the backup is unchanged and the
  save succeeds; given keys `a.json` and `a.txt` written in turn, then neither temp name collides (assert the staged names differ);
  given a failing rename, then the temp file is gone.
- Review nit WP3: remove unused `serde` and `ts-rs` from `src-tauri/Cargo.toml` if nothing uses them (developer-3; `cargo test` and clippy stay green).

### 11.3 Tooling gaps (review WP1 minors) - developer-1 (files: `tools/**`; ESLint and Stylelint config changes are made by developer-4, who owns them, on developer-1's request)
developer-1 closes these, each with a failing fixture proven by rule id (assert the rule id and the count, never only "some error"):
1. **Raw colours**: besides hex, flag `rgb(`, `rgba(`, `hsl(`, `hsla(`, `hwb(`, `oklch(`, `oklab(`, `color(` and named CSS colours in
   `ui/` and `pages/`, in `<style>`, in `style="..."` and in SVG attributes (`fill`, `stroke`) of the markup, and in `.ts`. `design-system/` is exempt.
2. **Test names** (`check-test-names`): `it.each`, `test.each`, `describe.each` are forbidden (message: loop over a table and call `it` with a
   given/when/then title); `it.fails`, `it.todo`, `it.skip`, `it.only`, `it.concurrent`, `it.skipIf`, `it.runIf`, `test.fixme` titles are
   checked like `it`. `test.fail(` is an annotation, not a test: it is not checked as a title but its description must match
   `^BUG-\d+: ` (see 11.7). Note `fails` and `fail` differ; do not match `test.fail`.
3. **`tauri::` scan**: strip Rust line comments, block comments (including nested `/* /* */ */`) and string literals before scanning, so a
   comment neither hides nor fakes a violation. Fixtures: `use tauri::` after a `/* ... */` on the same line (must fail), `// use tauri::`
   (must pass), `"tauri::"` in a string (must pass).
4. **Layer-pair matrix**: a table-driven test covers every ordered pair of the layers `pages`, `app`, `ui`, `design-system`, `api` plus the
   `svelte` and `svelte/store` imports (architecture 5.1): for each pair a virtual file in the `from` layer imports a file of the `to`
   layer through `ESLint.lintText` with `filePath`, and the test asserts allowed or `no-restricted-imports`. Missing today: every pair
   not already fixtured.
5. Spec text of WP1 acceptance check is `node --test "tools/*.test.mjs"` (Node 24 finding), and `backend/libs/.gitkeep` stays
   (confirmed, keep the glob member; remove `.gitkeep` when the first lib lands).
6. Architecture-check extra rules (`upward-dependency`, `unknown-crate-location`) are accepted; document them in ADR 0004 (staff-engineer, done below).
7. CI `browser` job (11.6) and the `test.fail` description check (11.7) belong to developer-1 as well.

### 11.4 Text-scale save failure, notifications and Toast rules - developer-4, developer-7, developer-9
- Failure to **save** the text scale does not roll the UI back. The scale applies for the session and the user sees a Toast of kind
  `warning`: "Text size changed but could not be saved. It resets when the app restarts." Failure to **load** falls back to 100 % silently
  (already tested; the first start has no file) and does not toast.
- `setTextScale` resolves `'saved' | 'not-saved'` and never rejects (3.3 updated). developer-4 also fixes the unhandled rejection of
  `main.ts:31` accordingly (nothing to catch; `main.ts` passes a `notify` function to the caller).
- `app/notifications` (developer-9, new folder): `createNotifications(): { subscribe; notify(n: { kind: 'info'|'warning'|'error'; text: string }): void; dismiss(id): void }`,
  at most 2 shown, newest first (the Toast stack of two already in the design). `MainWindow` renders the stack with `Toast` and
  wires `setTextScale`'s `'not-saved'` result to `notify`. No other feature adds a path to notifications now.
- **Toast auto-dismiss rules** (developer-7): `info` 4 000 ms; `warning` 8 000 ms; `error` never auto-dismisses and always shows a dismiss
  button (WCAG 2.2.1 timing; design 10.2 routes recoverable errors to Banner, so an error Toast is rare). `timeoutMs` may only shorten or
  lengthen `info` and `warning`; for `error` it is ignored. The timer pauses while the pointer is over the Toast or focus is inside it
  (a Toast never takes focus by itself). Tests: given `kind: 'error'`, when 60 s pass (fake time), then `ondismiss` is not called; given `warning`
  and 7.9 s then 8 s, then `ondismiss` once; given hover, then the timer does not run. AC-BN1 "a Toast disappears on its own" holds
  for info and warning; designer is informed of the error-toast exception (message sent, not blocking).

### 11.5 `@types/node` scope (QA's e2e addition) - developer-4 (config), qa (e2e tsconfig)
`@types/node` is kept as a devDependency but only `e2e/**` and `playwright.config.ts` may see it:
- `frontend/e2e/tsconfig.json` (qa) extends `../tsconfig.json`, sets `"types": ["node"]` and includes `**/*.ts` and `../playwright.config.ts`.
- `frontend/tsconfig.json` (developer-4): `"types": ["vite/client"]` (plus the vitest/jest-dom types the setup needs), no `@types/node`
  globals; `include` loses `e2e/**` and `playwright.config.ts`. `typecheck` runs both projects (`tsc -p . --noEmit && tsc -p e2e --noEmit`;
  svelte-check keeps the main one).
- ESLint (developer-4): `no-restricted-imports` on `node:*` and `fs`, `path`, `child_process` for `src/**` (a fixture proves it).
- `process` and `Buffer` in `src/**` **cannot be forbidden by tsconfig alone**: the Vite client typings pull `@types/node` into every
  project that uses them, so the globals still typecheck. The enforcement is the ESLint `no-restricted-globals` rule (`process`, `Buffer`)
  for `src/**` from developer-4, with a fixture. Given `src/**` uses `process` or `Buffer`, when `npm run lint` runs, then it fails.

### 11.6 CI Playwright install (developer-1)
- `npx --prefix frontend playwright install ...` is not reliable (`npx` has no `--prefix` semantics); use
  `npm --prefix frontend exec -- playwright install --with-deps chromium webkit`.
- Keep the browser cache for speed, but run `install --with-deps` **always**: the cache holds browsers, not the apt packages WebKit needs,
  and `install` is a no-op for cached browsers. Pin the runner to `ubuntu-24.04` (not `ubuntu-latest`) so fonts and libraries, and therefore
  snapshots, do not change under us.
- Snapshots (Chromium only, Linux) are system-font dependent. The first CI run is the arbiter: if a snapshot differs from the local ones,
  QA regenerates the snapshots from the CI artifact (the report folder is already uploaded on failure), not locally. Note in
  `test-plan.md`. A `workflow_dispatch` input `update-snapshots` is not added now (KISS).
- The job must run `npm --prefix frontend run build` first only if the e2e preview server needs `dist` (the QA config starts a preview on 4173);
  developer-1 confirms with QA's `playwright.config.ts` `webServer` entries.

### 11.7 Known-bug annotations in e2e (`test.fail`) - qa, developers, developer-1
QA's convention is accepted: `test.fail(<condition>, 'BUG-n: <reason>')` inside the test, one annotation per bug, so the suite is
green while the bug is open and turns red ("unexpectedly passed") on the fix. Rules:
- The description starts with `BUG-<n>: ` (checked by `check-test-names`, 11.3.2). One bug id per annotation; no `test.skip` or `test.fixme` for known bugs.
- **Retire**: the developer who fixes the bug deletes the matching `test.fail` line **in the same commit** as the fix (allowed exception
  to "QA owns `e2e/**`"); the red run is the trigger; QA retests and then closes the bug in the QA report. Snapshot tests that recorded the
  buggy rendering (BUG-8) are regenerated by QA after the fix, not by the developer.
- A `test.fail` for a bug that is not open in the QA report is an error (QA checks this at every report). At G4 no blocker or major bug may
  carry a `test.fail`; minor ones may, each with a follow-up owner in the report.

### 11.8 Other review minors (assigned, no new design)
- developer-6 (Tooltip, review major 1 and BUG-4): `aria-describedby` on the trigger referencing the tooltip id whenever the tooltip is rendered or visible (also for focus); BUG-9 token to designer.
- developer-7: Menu returns focus to the opener on close and on selection, handles Tab (closes and lets focus move on), sets `aria-haspopup="menu"` and `aria-expanded` on the trigger (BUG-5).
- developer-8: BUG-6, BUG-8 (graticule at 1.5 and 2: AC-G2 stays the target, one crisp device pixel per line; the 2-device-pixel width came from `max(1, cssWidth * dpr)` rounding up, so snap the width to an integer number of device pixels and the position to the matching pixel or pixel centre; if that cannot hold at some scale, report to the staff-engineer to relax the criterion, do not accept blur silently), BUG-7 (reason through the Tooltip on focus), C-4.
- developer-9: BUG-3 (popover Escape at document level while open, move focus in on open, restore on close), BUG-10 (accessible name contains the visible label), unhandled save rejection (11.4).
- developer-10: C-3 (double warning icon in the gallery StatusBar).
- C-1, C-2, C-4 cosmetics and observation C-2 (user at 150 % on 1440 px must scroll to find Settings): PM turns into issues; designer decides C-2.

### 11.9 AC-T2 thresholds (developer-5 question)
- `border` (1.61:1) and `border-strong` (2.48:1) are decorative separators, not the boundary of a control; the 3:1 non-text requirement applies to
  `border-control` (the token added for control boundaries). developer-5 changes the test to assert `border-control` >= 3 on `bg-panel` and removes the `it.fails`
  for `border`; if `border-control` is below 3, report to the designer.
- Channel colours: the threshold is 6.65 (one-decimal rounding of design 5.4, ch4 = 6.676). Accepted, the test stays.

### 11.10 Window minimum 1024 x 720, content floor 64 x 45 rem, height tiers and calibration - developer-9 (with qa)
Human decision D-L1: Tauri window minimum **1024 x 720**; designer ruling: content floor **64 x 45 rem**, AC-L1 to AC-L3 (design 3.2).
Supersedes 1024 x 576 and 64 x 36 rem wherever they appear. Files: `app/layout/**` and the shell CSS (developer-9);
`src-tauri/tauri.conf.json` already says 720 (developer-2 keeps the test at 720).

**Interface change (`app/layout/layout-tier.ts`)**: `HeightTier = 'h1' | 'h2' | 'h3' | 'h4' | 'scroll'`. `h4` is design collapse step 7
(dock groups as tabs `Vertical | Horizontal | Trigger`); `scroll` is window height under 45 rem. `layoutTierFor(widthPx, heightPx,
rootFontPx)` keeps its signature. The shell scrolls when either axis is `scroll`. The designer is asked to name `h4` in design 3.2
(step 7 is a collapse step there, not a tier; behaviour is identical). The height table is keyed by width class: `wide` = w1, w2;
`narrow` = w3, scroll.

**Final thresholds (calibrated by measurement, `layout-calibration.md`; window height in rem, thresholds are minimum heights)**

| Height tier | wide (W1, W2): dock in one row | narrow (W3, scroll): dock in two rows |
|---|---|---|
| h1 | >= 47.25 | >= 59 |
| h2 | >= 46.25 | >= 57 |
| h3 | not used (same need as h2) | >= 50.25 |
| h4 | >= 45 | >= 45 |
| scroll | < 45 | < 45 |

**The thresholds above are measured values** (`layout-calibration.md`, 2026-11-22, Chromium and WebKit identical, rule measured + 0.25 rem
rounded up to 0.25 rem, h4 raised to the content floor), no longer estimates. The table equals `HEIGHT_TIER_REM` in
`frontend/src/app/layout/layout-tier.ts` (wide 47.25 / 46.25 / 45; narrow 59 / 57 / 50.25 / 45). The measured h4 need is 42.1 rem, so the
floor stays 45 rem (1024 x 720) and escalation (a) did not trigger. Escalation (b) triggered: four estimates were off by more than 1.5 rem
(wide h1 and h2, narrow h1 and h2) because the W3 dock is 20.9 rem high, not 16.25; acknowledged, the calibration is accepted and the
arithmetic below is **kept only as the historical record, not as normative values**. AC-L1 was verified in 2 px steps from 45 to 62 rem at
widths 1024 and 1440 in both engines without failures.

**Arithmetic (historical, superseded by the measured table above; all rem; px / 16; estimates from the CSS and the design)**
- Fixed stack: toolbar 44 px = 2.75; horizontal readout 1.75; scale strip 6 + 24 + 6 px = 2.25; scope stage = screen minimum 25.6 +
  gutters 1.25 + 1.0 = 27.85. Sum F = 2.75 + 1.75 + 2.25 + 27.85 = **34.60**.
- Status bar: 1.5 (one line, 24 px). At W3 non-priority items wrap, budget 3.0 (two lines) until step 6 hides them.
- Dock group chrome (padding 6 + 8, header and gap about 18 px) = 32 px = 2.0. Vertical group: 4 rows x (24 + 4 px) = 7.0, so h1 = 9.0;
  h2 (row 20 px) = 4 x 24 px = 6.0, so 8.0; h3 (channel tabs 20 + gap 6 + one row 28 = 54 px) = 3.375, so 5.375.
  Horizontal and Trigger row (3 control rows plus 2 gaps of 6): h1 3 x 24 + 12 = 84 px = 5.25, so 7.25; h2, h3 (20 px rows) = 72 px = 4.5, so 6.5.
- Hiding the record overview frees 0 rem of height: as built it shares the readout row (readout 1.75 rem minimum, overview 0.75 rem inline).
  Keep step 4 for width and spec fidelity, but do not count on it.
- **wide**, one row, status 1.5: h1 = 34.60 + 1.5 + 9.0 = 45.1, threshold 45.1 + 0.5 margin, up to 46 (whole rem). h2 = 34.60 + 1.5 + 8.0 = 44.1, so h2
  holds at the whole floor (45) and h1 starts at 46. h3 and h4 are never needed.
- **narrow**, two rows, status 3.0: h1 = 34.60 + 3.0 + (9.0 + 7.25) = 53.85, + 0.5 margin = 54.35, up to 54.5. h2 = 34.60 + 3.0 + (8.0 + 6.5) = 52.1, + 0.5 =
  52.6, up to 53. h3 = 34.60 + 3.0 + (5.375 + 6.5) = 49.475, + 0.5 = 49.975, up to 50.
- **narrow, h4**: dock tabs 24 px = 1.5 plus the tallest group body (Vertical at h3, 5.375) = 6.875; status 1.5 (step 6 applied): 34.60 + 1.5 + 6.875 =
  42.975 <= 45. Slack 2.0 rem, so AC-L1 holds at the floor with the screen at its 25.6 rem minimum, and the floor stays 45 rem (720 px) if the
  measurements agree.
- Consequence to know about: a 1366 x 768 laptop (48 rem) at W1 gets h1 (wide column); at W3 a 48 rem window is h4 (tabs), because two dock
  rows plus a two-line status bar do not fit. This is the cost of the floor at 720 px.
- Collapse order calculations, in the order the design lists them (a step triggers when the stage would give the screen less than 32 x 25.6 rem):
  1 toolbar labels (width, frees 0 height); 2 sidebar to drawer at < 80 rem (frees 21.5 rem width, no height); 3 dock wraps at < 80 rem (costs +7.25 rem height, which is why the narrow column is higher);
  4 overview (0 rem); 5 compact rows h2 (-1.0 Vertical, -0.75 second row, -0.0 status) then h3 (-2.625 more); 6 status items by priority (-1.5 at narrow);
  7 tabs (-5.0 from h3: 11.875 -> 6.875); 8 scroll below 64 x 45.
  Width check at the floor: 64 rem minus sidebar drawer (0) minus gutters 4 = 60 rem stage width >= 32 rem minimum.
  At W1 with the sidebar: 80 - 21.5 = 58.5 rem, minus gutters 4 = 54.5 >= 32. Width never forces extra collapse; height does.

**Calibration by measurement and how it is recorded**
1. developer-9 builds the shell with the table above in `layout-tier.ts` (data only, exported as `HEIGHT_TIER_REM` next to `SCREEN_MINIMUM_REM`).
2. developer-9 measures with a throwaway Playwright script (kept out of the repo, or in the scratchpad), Chromium and WebKit, 100 % text scale:
   for each (width class, tier) force the tier with a viewport at width 1024 (narrow) and 1440 (wide), step the window height down in 0.25 rem
   (4 px) steps, and read the needed height as the smallest height at which `getBoundingClientRect` of the scope screen is >= 512 x 410 px, lies
   inside the stage, and the Run button, scale strip, Trigger group, connection and sample-rate items are visible without scrolling (AC-L1).
   The larger of the two engines is the measured value.
3. developer-9 records the results in a new file `docs/specs/scaffold-and-design-system/layout-calibration.md`: one row per (column, tier) with
   estimate, measured need (px and rem), rule applied (measured + 0.25 rem, rounded up to 0.25 rem), final threshold, engine, date, and
   the commit of the measurement. The table in `layout-tier.ts` and the table of this section are updated to the final numbers in the same commit
   (the ADR 0005 text is not changed by calibration; the spec table and the calibration file are the record).
4. QA (WP11) tests AC-L1 at each final threshold and 1 px below it, in both columns, and at the floor (1024 x 720), reading the numbers from the
   calibration file by hand into the test table (no parsing of docs).
5. Escalation, to the staff-engineer on the bus the same day: (a) the measured `h4` need exceeds 45 rem (then the floor height rises, design 3.2
   allows up to 48 rem; needs the human since D-L1 is 720); (b) any estimate is off by more than 1.5 rem (re-run the arithmetic); (c) the
   screen would drop below 32 x 25.6 rem at any size >= the floor. Do not shrink the screen minimum.

### 11.11 Duplicate screen minimum (decision)
`ui/graph` `fitScreen(stage, rootFontPx)` keeps its own rem minimum (`MIN_SCREEN_REM` 32 x 25.6, via `minScreen`), and `ScopeStage` also
computes `screenMinimumPx(rootFontPx)` from `app/layout` to size the scroll floor. The number 32 x 25.6 therefore exists twice (`ui/graph`
and `app/layout`). **Accepted for now**: `ui` must not import `app`, the two functions are pure and covered by tests at 16 and 32 px, and
the value is fixed by design 3.3. **Follow-up (first feature story that touches the stage)**: remove the duplication by having
`app/layout` import the constant from `ui/graph` (allowed direction) or `ScopeStage` pass the minimum in, and add a test that both agree.
No behaviour change; not a blocker for QA or review.

## Changelog
- 2026-11-22: scaffold and design system implementation complete (WP1 to WP11 and fix rounds 1 to 3), pending QA sign-off and final code
  review. Height tiers calibrated by measurement (11.10); 11.4 Toast defaults corrected in 3.4; 11.5 enforcement reworded to ESLint;
  11.11 added.
- Open for the first feature stories: Q-E4 text (PM), Q-E5 cursor formatting, Q-E6 cursor origin, Q-E7 settings policy, Q-E8 clipping
  details, WebGL2 performance on 4 x 4096 samples and persistence, streams, state aggregator and snapshot, first-run hints, application
  identifier and final icon (human), removal of the duplicate screen minimum (11.11).
- Design cosmetics: C-1 open (PM turns into an issue), C-2 done, C-3 assigned to developer-10 (11.8), C-4 assigned to developer-8 (11.8), completion not confirmed to me.
