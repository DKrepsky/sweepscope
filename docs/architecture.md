# sweepscope: Architecture

| | |
|---|---|
| Status | Draft 0.2 (scaffold and design system; pending gate G3) |
| Date | 2026-11-21 |
| Related | [prd.md](prd.md) (requirements, user stories, hardware limits), [design.md](design.md), [adr/](adr/) |

This document describes how sweepscope is built. The PRD says what and why; this says how. Requirement IDs (DEV-1, ACQ-5, …)
refer to the PRD.

## 1. Principles

These are binding for people and for coding agents.

| # | Principle | Meaning in practice |
|---|---|---|
| P1 | **The Rust backend owns all domain processing and is the single source of truth for domain state.** | The frontend never computes measurements, FFT, volts or connection logic. It displays what the backend publishes and sends intents. State that exists only for presentation (cursors, panel layout) is frontend-owned, see P1b. |
| P1b | **Presentation-only state lives in the frontend; heavy computation always runs in Rust.** | State test: if the backend never needs the value, the frontend owns it. Computation test: anything beyond trivial arithmetic on data already delivered (FFT, statistics, measurements, interpolation, decimation) is done in Rust even when the state it serves is frontend-owned (section 4.5). |
| P2 | **Backend features are decoupled modules.** | A feature module never imports another feature module. |
| P3 | **Modules talk through events, request/response or streams only.** | Over an in-process bus defined in the shared kernel; no direct calls, no shared mutable state. |
| P4 | **Reusable code without feature logic lives in `libs/`.** | `libs/fft` computes an FFT; it knows nothing about channels or the scope. |
| P5 | **Shared application code lives in the shared kernel.** | Bus, ids, units, error types, base traits. |
| P6 | **The frontend follows a design system with predefined tokens.** | No raw colours, sizes or fonts in components. |
| P7 | **Reusable frontend components live in `ui/` and contain no application logic.** | Button, dropdown, checkbox, dialog, graph. |
| P8 | **UI components compose through slots.** | In Svelte 5 these are snippets: a button has a `left` snippet where an icon component goes. |
| P9 | **State management is a first-class concern; never duplicate state.** | Exactly one store per concept (for example one `connection` store); components read it, never copy it. Frontend-owned state is no exception: one `cursors` store. |
| P10 | **Pages and layouts only display.** | Logic and state live in a separate TypeScript-only layer. |
| P11 | **Tests are fundamental and written BDD-style (given, when, then).** | Behaviour names, one behaviour per test. |
| P12 | **Clean code, small functions and traits with meaningful names.** | Names carry the meaning. Comments are rare and explain why, not what. |
| P13 | **Libraries have a small interface and deep behaviour.** | Few public items; a lot done behind them. |
| P14 | **SOLID, dependency injection and KISS.** | Depend on traits and constructor-injected collaborators; pick the simplest design that works. |
| P15 | **External dependencies are used with care.** | Each one is justified, pinned, reviewed and wrapped behind our own interface when it leaks into the design. |

## 2. System overview

```
┌──────────────────────────────── Tauri application ────────────────────────────────┐
│                                                                                   │
│  Frontend (WebView, TypeScript)                  Rust backend                     │
│  ┌───────────────────────────┐                   ┌────────────────────────────┐   │
│  │ pages / layout (display)  │                   │ features/  (decoupled)     │   │
│  │            ▲              │                   │  device  acquisition       │   │
│  │ application layer         │   commands,       │  measurement  spectrum     │   │
│  │  (logic, stores, services)│ ────────────────▶ │  display  export  settings │   │
│  │            ▲              │ ◀──────────────── │            ▲   ▼           │   │
│  │ ui/ (design system)       │  state snapshots, │       shared kernel        │   │
│  └───────────────────────────┘  events, frames   │   (bus, ids, units, errors)│   │
│                                                   │            ▲               │   │
│                                                   │ libs/ (fft, interpolation, │   │
│                                                   │  statistics, decimation)   │   │
│                                                   └─────────────┬──────────────┘   │
└─────────────────────────────────────────────────────────────────┼──────────────────┘
                                                                  │ ScopeDevice trait
                                                    hantek6xx4-lib adapter │ simulator
```

Direction of dependencies: features → shared kernel and libs; shared kernel → libs only if needed; libs → nothing of
ours. The frontend application layer talks to the backend only through the generated typed API client.

## 3. Repository layout

```
sweepscope/
├── Cargo.toml                    # workspace; members are globs (kernel, libs/*, features/*, src-tauri)
├── rust-toolchain.toml  rustfmt.toml  clippy.toml  deny.toml
├── package.json                  # root: Tauri CLI and task scripts (check, test, dev, build)
├── .github/workflows/ci.yml
├── docs/                         # prd, architecture, design, adr/, specs/, design-prototype/ (approved reference)
├── tools/                        # repository checks, no dependencies
│   ├── check-architecture.mjs    # crate rules from `cargo metadata` (+ tests, node --test)
│   └── check-test-names.mjs      # given_when_then naming
├── cli/                          # capture CLI library, second composition root, no Tauri, run by the app binary (ADR 0022)
├── src-tauri/                    # Tauri shell: composition root, window and IPC wiring only
│   ├── tauri.conf.json  build.rs  capabilities/
│   └── src/
│       ├── main.rs
│       ├── composition.rs        # builds modules, injects dependencies
│       └── ipc/                  # commands and channels, thin adapters onto the bus
├── backend/
│   ├── kernel/                   # shared kernel (crate `sweepscope-kernel`): bus, Clock, Storage, errors
│   ├── libs/                     # feature-free crates, created by the first story that needs one
│   │   ├── engineering-format/ measurements/ interpolation/   # created by M1 and M2
│   ├── fft/ math-channel/ autoset/                         # created by M3
│   └── (decimation ... created by its story)
│   └── features/
│       ├── settings/             # opaque UI-state blobs in settings.json (cursors, measurement pins)
│       ├── device/  acquisition/ # device sessions; setup, run state and setup files (setup JSON, last-setup)
│       ├── display/              # wire frame, interpolation mode (domain state)
│       ├── measurement/          # twelve measurements per channel
│       ├── export/               # CSV, raw SSR1 frame files
│       ├── spectrum/             # FFT frames on their own stream (ADR 0018)
│       ├── recording/            # roll data to .ssroll files (ADR 0017)
│       ├── calibration/          # zero calibration session, per-unit offsets (ADR 0021)
│       └── (channels, timebase-trigger ... created by their stories)
└── frontend/
    ├── package.json  vite.config.ts  svelte.config.js  tsconfig.json  eslint.config.js  .stylelintrc.json
    ├── index.html  gallery.html  # app entry; component gallery entry (not in production builds)
    ├── tools/gen-tokens.mjs
    ├── e2e/                      # Playwright (QA owned)
    └── src/
        ├── main.ts               # frontend composition root
        ├── design-system/        # tokens.css (single source), tokens.generated.ts, base.css, icons
        ├── ui/                   # reusable components, one folder each, no application logic
        ├── app/                  # application layer (TypeScript only): stores, services, use cases, view models
        ├── pages/                # display only: main-window, gallery
        └── api/                  # generated types, BackendClient interface, Tauri client, fake client
```

Each Rust folder is a crate in one Cargo workspace, so the compiler enforces the dependency rules: a feature crate lists
`sweepscope-kernel` and `libs` crates in its manifest and never another feature crate. A CI check fails the build if that
changes. Crates for features that are not built yet do not exist: no empty placeholders. Crate names are
`sweepscope-<folder>`.

## 4. Backend

### 4.1 Shared kernel (`backend/kernel`)

Holds only what every module needs and nothing feature specific:

- **Message bus** with two patterns (section 4.3): publish/subscribe events and request/response.
- **Identifiers and value types:** `ChannelId`, `TimeBase`, `VoltsPerDiv`, `Volts`, `Seconds`, `Hertz`, with unit-safe
  arithmetic.
- **Error type** and result alias; every error carries a user-presentable category.
- **Clock and scheduler traits** (for deterministic tests).
- **Base traits** used for injection: `Clock`, `Storage`.
- **Test helpers:** event recorder, fake clock.

The kernel has a small public interface and no dependency on Tauri or on `hantek6xx4-lib`.

### 4.2 Feature modules

A feature module is a crate that owns one capability end to end: its state, its rules, its commands and its events.

```
features/acquisition/
├── Cargo.toml
├── src/
│   ├── lib.rs            # public interface: `AcquisitionModule::new(deps)`, and the message types
│   ├── messages.rs       # events and requests this module publishes or answers
│   ├── state.rs          # the module's slice of application state
│   ├── run_control.rs    # small focused units
│   ├── frame_pipeline.rs
│   └── ...
└── tests/                # behaviour tests, BDD style
```

Rules:

1. **Public surface is tiny:** a constructor taking its dependencies, the message types, and nothing else.
2. **No feature depends on another feature.** If `measurement` needs frames, it subscribes to the `FrameCaptured` event
   defined in the kernel's contract list; it does not import `acquisition`.
3. **Cross-feature message types live in the kernel** (`kernel::contracts`), kept deliberately few and stable. A message
   used by exactly one module stays in that module.
4. **Each module owns its state slice** and is the only writer of it. Others see it through published
   events (`ConnectionChanged`) or by asking (`GetConnection` request).
5. **Hardware is behind a trait.** The `device` module defines `ScopeDevice` (capabilities as data, capture, configure,
   roll) and ships two implementations: `HantekDevice` (wraps `hantek6xx4-lib`, the only place that imports it) and
   `SimulatedDevice`. This satisfies DEV-5.

### 4.3 Communication between modules

Three patterns, all defined in the shared kernel:

| Pattern | Use for | Shape |
|---|---|---|
| **Event** (publish/subscribe, one to many, fire and forget) | Facts that happened: `DeviceConnected`, `TriggerArmed`, `SettingsChanged`, `ConnectionChanged` | Named past tense; immutable; carries what subscribers need |
| **Request/response** (one to one, typed) | Questions and commands that need an answer: `GetChannelSetup`, `StartAcquisition`, `ExportCsv` | Named imperative or question; replies with a typed result or error |
| **Stream** (one to many, ordered flow of items with a lifecycle) | Continuous sample data: scope frames, roll-mode chunks, recording to file | Opened by a request, carries sequence-numbered items, ends with a close or error item; has an explicit loss policy |

Stream policies. A stream declares one of two policies when it is opened:

| Policy | Behaviour | Used by |
|---|---|---|
| **Latest-wins** | A slow consumer sees only the newest item; older items are replaced | Triggered/auto frames to the display |
| **Ordered, gap-marked** | Items are delivered in order; if the consumer falls behind beyond the bounded queue, items are dropped and a `Gap { missed }` item is inserted, never a silent splice | Roll chunks, record-to-file |

Why a separate stream pattern (and not events): roll mode produces an append-only, ordered sequence where a lost or
reordered item corrupts the picture and loss must be visible (the driver itself cannot detect overruns from the pointer,
see PRD 5.2), whereas events are unordered facts with no lifecycle or loss policy. Frames at 60 per second also do not
belong in a general event channel.

Guidelines:

- Events describe the past, requests describe intent. Never use an event to demand work from a specific module.
- Handlers are small and return quickly. Long work (USB capture, FFT) runs on its own task or thread owned by its module.
- A module's inbox is bounded. Overflow policy is explicit per message kind (drop oldest for latest-wins streams, never
  drop for commands).
- The bus is injected. Tests pass an in-memory bus and assert on recorded events, replies and stream items.

#### Which transport carries what

| Boundary | Events | Request/response | Stream |
|---|---|---|---|
| **Module to module (in process)** | Our own small typed bus in the kernel | Our own typed request router in the kernel | Bounded in-process channel with the policies above |
| **Backend to frontend** | Tauri events | n/a | Tauri `Channel` with binary payloads |
| **Frontend to backend** | n/a | Tauri commands (`invoke`) | n/a (a stream is opened with a command that receives the Channel) |

Decision on the bus: the Tauri event system is **used at the frontend boundary and not for module-to-module traffic**.
It was evaluated first, as requested, and does not fit in process:

- it serialises every payload to JSON, also between two Rust listeners, which costs time for frames and loses types;
- event names are plain strings with untyped payloads;
- it offers no request/response, no bounded queues and no delivery policies;
- using it in feature crates would make them depend on `tauri`, which breaks the rule that only the shell knows Tauri
  and makes modules untestable without a running app.

So the kernel provides a small typed bus (a few hundred lines on std channels, no new dependency). The Tauri shell
subscribes to the bus and forwards selected events to the frontend as Tauri events, and forwards request/response
through Tauri commands. If a later Tauri release adds typed, in-process, non-serialising events the kernel bus can be
replaced behind its trait.

### 4.4 Libs (`backend/libs`)

Reusable, feature-agnostic crates: a function that computes an FFT goes into `libs/fft`, not into `spectrum`.

- Pure functions and small types over plain data (`&[f32]`, `Hertz`); no bus, no global state, no I/O.
- **Small interface, deep behaviour:** `libs/fft` exports a function such as `spectrum(samples, window, sample_rate)`
  and a `Window` enum; plan caching, scaling and bin maths stay private.
- Each lib has its own tests, runs without hardware or Tauri, and could be published on its own.
- A lib never depends on a feature or on the kernel's bus. It may depend on kernel value types if those are the shared
  vocabulary.
- Decimation, interpolation, statistics and formatting live here so measurement, display and export share one tested
  implementation.

### 4.5 State: who owns what

State has exactly one owner. Two kinds exist:

| Kind | Owner | Test | Examples |
|---|---|---|---|
| **Domain state** | Backend, in the module that owns the capability | The backend computes with it, persists it, or applies it to hardware | connection, channel setup, time base, trigger, acquisition status, measurement selection, math channels, persistence setting, interpolation mode (`display`; automatic below 250 samples per division), setups (`acquisition`) |
| **Presentation state** | Frontend, in one store in `app` | Only the view uses it | cursor positions, selected tab, panel sizes, which trace is highlighted, text scale, layout tier |

Rules:

1. A piece of state is never in both places. If the backend later needs a presentation value (for example the visible
   time window, to decimate to the screen), it is promoted to domain state and the frontend sends it as a command.
2. **Cursors are presentation state.** The frontend keeps their positions. Reading the value under a cursor is
   arithmetic on a trace the frontend already received, done by a selector in `app` (`cursorReadout`), not by the
   backend. The frontend may compute only trivial arithmetic over frontend-owned state and data it already holds (lookup
   of a sample, a difference, 1/Δt). Anything heavier is a backend request even if its inputs are frontend state: for
   example, a measurement between two cursors over a long record is a request carrying the cursor positions and
   answered by `measurement`; the cursor positions stay in the frontend.
3. Presentation state that must survive a restart or belong to a saved setup (cursor positions, layout) is handed to the
   `settings` feature as an opaque, versioned blob through a request; the backend stores it without interpreting it and
   returns it on load. The backend remains the only writer of files.
4. Screenshots and exports that include presentation items (cursors) are composed in the frontend; data-only exports
   are composed by the backend.

Backend mechanics:

- Domain state is split into per-module slices, each written only by its module.
- A **state aggregator** in the Tauri shell subscribes to every module's state-changed events and builds the snapshot
  that the frontend receives. It contains no business logic.
- Every domain change is published as an event first; the shell forwards it to the frontend. The frontend never
  mutates domain state locally. It sends a command and renders the resulting update.
- Reconnect or window reload: the frontend asks for a full snapshot, then follows events. No other path exists.
- **Connection state** is one state machine in `device` (disconnected, settling, ready, acquiring, error). It uses
  `hantek6xx4-lib`'s `DeviceMonitor` with its 2 s settle time and never opens a device outside it (PRD 5.2).

### 4.6 Concurrency

- The Tauri shell runs the bus. Each module that does blocking work (device, acquisition) owns a dedicated thread; the
  rest run on an async runtime.
- No module holds a lock across a bus call. State slices are owned by their module's task, not shared behind a mutex.
- Shutdown is coordinated by a `Shutdown` event; USB resources are released in the device module.

### 4.7 Frame path

```
device (thread) ──▶ acquisition ──FrameCaptured──▶ display ──▶ ipc channel ──▶ frontend renderer
                                  │
                                  ├──▶ measurement (stats, cursors)
                                  └──▶ spectrum (FFT, on request or when its view is open)
```

- `display` builds the **view model**: volts-scaled traces, already decimated (`libs/decimation`) to the width the
  frontend reports, plus metadata (time base, volts/div, sample interval, trigger index).
- The shell sends it on a Tauri `Channel` as a binary payload with a sequence number (a latest-wins stream). Old frames
  are dropped when the frontend is behind.
- The renderer only draws. It receives the data it will paint, nothing to compute.
- On the 6254BC a record is at most 4096 samples per channel (16 KiB for four channels), fewer than the pixels of a wide
  screen. Decimation (`libs/decimation`) is therefore needed for roll history, deep zoom-out and future deeper devices,
  not for the normal frame; the first version may send the full record (KISS).

### 4.8 Roll mode path

```
device (thread) ──▶ acquisition ──RollChunk stream (ordered, gap-marked)──▶ display ──▶ Tauri Channel ──▶ frontend
                                                                       └──▶ export (record to file)
```

- Roll uses the **stream** pattern, not frames: `StartRoll` opens a stream, chunks carry a sequence number and the
  time of their first sample, and `acquisition` inserts a `Gap` item whenever the driver's elapsed-time estimate says
  samples were lost (PRD 5.2).
- The frontend appends chunks to a bounded visible window; the backend keeps the history. Only one roll stream per
  window exists, closed with the same request that stops roll mode.
- Data rate is low (about 1.2 k samples per second per channel at 200 ms/div, less at slower bases), so the stream is
  about ordering and loss visibility, not throughput.
- M3 (ADR 0017): roll is a run mode of `acquisition` derived from the time base. Items carry absolute sample indices; a consumer that sees an index jump inserts a gap, so a lagging consumer never splices silently. `display` encodes `SSL1` items on a second Channel, the frontend keeps only the visible window, and the backend keeps no history. `recording` writes `.ssroll` files from the same items. ADR 0009 item 6 is superseded by ADR 0017 item 5.

## 5. Frontend

### 5.1 Layers

```
pages/          display only: composition of ui components and application-layer values
app/            application layer, TypeScript only: stores, services, use cases; no markup
ui/             reusable components, no application logic (one folder per component, imported by path, no barrel file)
design-system/  tokens (dark theme set), base styles, icons
api/            generated types and the one backend client
```

Allowed imports: `pages` → `app`, `ui`; `app` → `api`; `ui` → `design-system`. `ui` never imports `app`, `pages` or
`api`. `pages` never import `api` directly. The `app` layer never imports `.svelte` files or the Svelte compiler runtime; its only
permitted Svelte import is `svelte/store` (plain TypeScript, no components), so it stays testable without a DOM (P10).
Lint rules enforce this.

### 5.2 Design system

- **Tokens** are the only source of visual values: colour (semantic: `bg-panel`, `text-muted`, `ch1`, `status-stop`),
  spacing scale, radius, typography, motion durations, z-index layers. Naming is `--ss-<category>-<role>[-<variant>]`
  (design.md 5.1).
- **Single source:** `frontend/src/design-system/tokens.css`, wrapped in `:root[data-theme="dark"]`. **Dark theme only**
  (PRD DSP-6); the theme-set mechanism stays (another theme would be another `data-theme` block), and the colour-blind-safe
  channel palette is part of the dark set. A zero-dependency script generates `tokens.generated.ts` (resolved values)
  for TypeScript consumers: the Canvas graph and the contrast and colour-blind tests. Both files are committed and CI
  diffs the generated one (ADR 0002).
- Components never contain a raw colour, pixel size or font. Lint rejects them (section 8.2). Strokes of one device pixel
  and the Graph's geometry are the only values computed in code, from tokens and the device pixel ratio.
- Waveform colours come from tokens too.
- **Text scale** (100, 125, 150, 200 %) is presentation state: the `textScale` store in `app` sets the root font size;
  every size is rem. It is persisted through the opaque UI-state blob of `settings` (section 4.5 rule 3); default 100 %.
- **Layout tiers** (design.md 3.2) are computed in TypeScript from the viewport in rem, not with media queries (ADR 0005). Content floor 64 x 45 rem; window minimum 1024 x 720 logical px; below the floor the region between toolbar and status bar scrolls.
- **Component gallery** (`frontend/gallery.html`, page `pages/gallery`) renders every `ui` component in every state and is
  the target of the layout and HiDPI tests; it is excluded from production builds.
- **Disabled controls** use `aria-disabled="true"`, not the `disabled` attribute, so they stay focusable and can show the
  tooltip that states why (design.md principle 1 and AC-I13).

### 5.3 The `ui` folder

Reusable, application-agnostic components: button, icon button, dropdown, checkbox, number field, dialog, tooltip, tabs,
panel, toolbar, graph canvas, and so on.

- They receive values and callbacks as props and emit events. They hold only transient presentational state (is the
  dropdown open).
- **Slots for composition (Svelte 5 snippets).** Components take snippets instead of configuration flags. Example:
  `Button` takes `left`, `children` and `right` snippets rendered with `{@render left?.()}`; an icon component passed as
  `left` gives an icon button; a spinner as `right` gives a loading button. `Dialog` takes `title`, `body` and `actions`.
  `Panel` takes `header` and `body`.
- No imports from `app`, no knowledge of the scope, channels or measurements. The `graph` component draws traces handed
  to it; where they come from is not its concern.
- A component stays small: one responsibility, a handful of props, accessibility built in (roles, focus, keyboard).

### 5.4 Application layer (`app`)

TypeScript only, no markup, no framework components.

- **Stores** hold state. They are `svelte/store` primitives (`readable`, `writable`, `derived`) wrapped so components get
  only the read side; Svelte components consume them with the `$store` syntax. **Services** perform actions (send a command, subscribe to an event stream). **Use cases** are
  small functions that coordinate them (`connectScope`, `setVoltsPerDiv`, `exportCsv`).
- Dependencies are injected (the backend client, clock, storage) so everything runs in tests without Tauri.
- Naming says what a thing does: `connectionStore`, `selectActiveChannel`, `toggleChannel`.

### 5.5 State management

State management is designed first and implemented deliberately.

| Kind of state | Where it lives | Example |
|---|---|---|
| **Domain state** (backend truth) | One read-only store per concept, filled from backend snapshots and events | `connection`, `channelSetup`, `acquisitionStatus`, `measurements` |
| **Presentation state** (frontend-owned, section 4.5) | Store in `app`, scoped to the concern | cursors, selected tab, panel sizes, which cursor is dragged |
| **Transient state** | Inside a `ui` component | dropdown open |

Rules:

1. **One store per concept, one owner.** There is exactly one `connection` store. No component keeps its own
   "isConnected"; it reads the store or a derived selector. This is the rule that prevents a header showing "connected"
   while a panel shows "disconnected".
2. **Derive, don't copy.** Anything computable from a store is a selector (`isAcquiring = status === 'running'`), not a
   second stored value.
3. **Domain stores are read-only to the UI.** (Presentation stores are writable through use cases only, never directly from markup.)
    They change only through backend events. User actions send commands;
   optimistic updates are not allowed unless a rule is written down for that command.
4. **Unidirectional flow:** UI event → use case → backend command → backend state event → store → UI.
5. **Stores are tested in isolation** with a fake backend client.

### 5.6 Pages and layout

- Pages and layout components describe structure only: arrange `ui` components, bind to store values and callbacks
  provided by the application layer.
- A page contains no conditionals about domain rules (no "if connected and not triggered then …"). That logic is a
  selector or use case in `app`, named for its meaning.

### 5.7 Waveform rendering

- `ui/graph` provides a graticule and a trace renderer behind a `TraceRenderer` interface. The first implementation is
  Canvas 2D (ADR 0006); the WebGL2 renderer with Canvas fallback (PRD DSP-9) is added after the M0 spike.
- It draws the already-scaled data it is given. Persistence accumulation is a rendering technique of the graph
  (it composites successive frames); what to persist and for how long comes from backend state.
- Graph interaction emits intents. Dragging the trigger marker becomes a backend command; dragging a cursor updates the
  frontend `cursors` store.

### 5.8 Backend client and typed contract

- Command, event and snapshot data types are generated from the Rust definitions with **ts-rs** into
  `frontend/src/api/generated`. The generated files are committed and checked in CI so they cannot drift. The client
  itself is hand-written; a contract test compares the command names on both sides (ADR 0003).
- `api` exposes one client interface (`BackendClient`) with `send(command)`, `subscribe(event)` and `openStream(kind)`. The
  application layer depends on that interface; tests use a fake.

## 6. Testing

Tests are written in a **given / when / then** style. Names state behaviour, not implementation.

### 6.1 Rust

```rust
#[test]
fn given_a_connected_scope_when_it_is_unplugged_then_connection_becomes_disconnected() {
    let mut scope = SimulatedDevice::connected();
    let mut events = EventRecorder::attached_to(&module_bus);

    scope.unplug();

    assert_eq!(events.last::<ConnectionChanged>(), ConnectionChanged::Disconnected);
}
```

- Feature modules are tested through their public message interface with an in-memory bus, a fake clock and
  `SimulatedDevice`. No test reaches into private state.
- Libs are tested with known signals (a 1 kHz sine's FFT peaks at 1 kHz; sin(x)/x of a sampled sine reconstructs it).
- `hantek6xx4-lib`'s mock transport is used for the adapter; real-hardware checks are a manual, scripted checklist run
  before releases (plug/unplug stress, calibrator reading).
- Property tests for decimation (min and max of each column always preserved).

### 6.2 Frontend

```ts
describe('connection store', () => {
  it('given a connected scope, when the backend reports a disconnect, then every reader sees disconnected', () => {
    const backend = fakeBackend();
    const store = createConnectionStore(backend);
    backend.emit(connectionChanged('connected'));

    backend.emit(connectionChanged('disconnected'));

    expect(store.status()).toBe('disconnected');
  });
});
```

- `app` layer: unit tests with a fake `BackendClient`. This is the bulk of frontend tests.
- `ui` components: Vitest + Testing Library component tests for behaviour and accessibility (one test file per
  component, acceptance criteria of design.md 9.3 as test names), plus Playwright layout and pixel-scan tests on the gallery for
  what jsdom cannot see (computed styles, layout tiers, HiDPI crispness, contrast). There are no screenshot-comparison (visual snapshot) tests.
- Token tests are pure: contrast ratios (AC-T2) and colour-blind pair distances (AC-T3) are computed from
  `tokens.generated.ts`.
- Pages: a few smoke tests that wire the real application layer to the fake backend.
- End-to-end: the Tauri app against `SimulatedDevice` for the user stories (US-01 to US-19) that do not need hardware.

### 6.3 Test rules

- One behaviour per test; given, when and then visible in the name or in marked blocks.
- No sleeping on real time: inject the clock.
- A bug fix starts with a failing test.
- CI runs all of the above plus the dependency-rule checks (section 8).

## 7. Clean code rules

- **Small functions and traits.** A function does one thing and fits on a screen; a trait has one reason to change
  (interface segregation). Prefer several small traits over one large one.
- **Meaningful names.** `arm_for_single_shot()`, not `process()`; `ConnectionState::Settling`, not `State::S2`.
- **Few comments.** Code explains what; a comment is allowed for why (a hardware quirk, a protocol constant, a link to
  `protocol.md`). Long comment blocks are a smell: rename, extract a function or a type instead. Doc comments on the
  public items of libs and kernel are brief.
- **SOLID.** Single responsibility per module and function; open for extension through traits (a new device is a new
  `ScopeDevice`, not an edit in acquisition); depend on abstractions.
- **Dependency injection.** Collaborators are passed in through constructors or function parameters. The composition
  root (`src-tauri/composition.rs`, and one `main` file in the frontend) is the only place that builds concrete
  objects. No global singletons.
- **KISS.** The simplest design that satisfies the requirement. No framework or abstraction without a second use or a
  test that needs it.
- **Small, deep libraries.** A lib exports few items and hides its complexity (P13).
- **Tooling enforces taste:** `cargo fmt`, `clippy -D warnings`, ESLint, Prettier, TypeScript strict mode; maximum
  function length and complexity thresholds in lint.

## 8. Dependencies and enforcement

### 8.1 External dependencies

- Every new dependency needs a one-line justification in the pull request: what it does, why not write it, licence,
  maintenance status, size.
- Prefer the standard library and small, focused crates and packages. Avoid large frameworks where a short function does
  the job.
- Wrap a dependency behind our own interface when its types would leak across modules (the driver behind `ScopeDevice`;
  the Tauri API behind the `ipc` adapter and the frontend `BackendClient`).
- Pin versions; commit lockfiles; run `cargo deny`/`cargo audit` and `npm audit` in CI. Pin GitHub Actions as in the
  driver repo.
- The driver, `hantek6xx4-lib`, is used through its published API only (PRD 8.1).

### 8.2 Architecture rules checked in CI

Mechanisms are in ADR 0004. All run locally with `npm run check`.

| Rule | Check |
|---|---|
| Feature crates do not depend on other feature crates | `tools/check-architecture.mjs` over `cargo metadata` |
| Libs depend on no feature and not on the bus | Same script |
| Only `device` depends on `hantek6xx4-lib`; only `src-tauri` depends on `tauri`; `tauri::` only in `src-tauri/src/ipc` and `main.rs` | Same script |
| The `cli` library (second composition root, ADR 0022) may depend on kernel, libs and feature crates but never on `tauri`; only `src-tauri`, whose binary runs it, may depend on `cli` | Same script |
| `ui` imports only `design-system`; `pages` do not import `api`; `app` imports no markup | ESLint `no-restricted-imports` per folder |
| No raw colour, size or font literals in `ui/` and `pages/` | Stylelint core rules (CSS in `.svelte`) and an ESLint rule (TS) |
| Generated files up to date (tokens, API types, command manifest) | Regenerate and `git diff --exit-code` |
| Tests follow naming convention (`given_..._when_..._then_...`) | `tools/check-test-names.mjs` |
| Function size and complexity | clippy and ESLint thresholds (section 7) |

### 8.3 Lint, format and compiler settings

| Area | Setting |
|---|---|
| Rust | `rustfmt.toml` `max_width = 120` (as the driver); `clippy -D warnings`; workspace lints enable `too_many_lines` (40) and `cognitive_complexity` (15); `#![forbid(unsafe_code)]` in kernel, libs and features; edition 2021; toolchain pinned |
| TypeScript | `strict`, `noUncheckedIndexedAccess`, `verbatimModuleSyntax`; no `any` (ESLint `no-explicit-any`) |
| ESLint | flat config; typescript-eslint, eslint-plugin-svelte; `max-lines-per-function` 40, `complexity` 10 |
| Stylelint | core rules plus `postcss-html`; see ADR 0004 |
| Prettier | print width 120, single quotes, with the Svelte plugin; `.editorconfig` for the rest |
| Svelte | `svelte-check` with warnings as errors (accessibility warnings included) |

### 8.4 CI outline (GitHub Actions, actions pinned by commit SHA as in the driver repo)

| Job | Steps |
|---|---|
| `rust` | fmt check, clippy `-D warnings`, `cargo test` for all crates except `src-tauri`, `cargo deny check`, `cargo audit` |
| `shell` | install WebKitGTK dev packages, `cargo clippy` and `cargo test` for `src-tauri`, ts-rs export and diff |
| `frontend` | `npm ci`, ESLint, Stylelint, Prettier check, `svelte-check`, Vitest, token generation diff, `npm audit` |
| `architecture` | `node --test tools`, `node tools/check-architecture.mjs`, `node tools/check-test-names.mjs` |
| `browser` | Playwright on the gallery and main window (Chromium and WebKit), layout and bounding-box checks, HiDPI scales |
| `bundle` (manual and on tags) | `tauri build` to .deb and AppImage; release steps need human confirmation (workflow section 6) |

`main` is protected; every job is a required check. release-plz as in the driver repo is added by a later story.

## 9. Decisions and open items

| Item | Status |
|---|---|
| Tauri 2 shell | Decided |
| Rust workspace with kernel, libs and feature crates | Decided |
| Svelte 5 with Vite as reactive framework (slots as snippets) | Decided. Plain Svelte + Vite rather than SvelteKit (no routing needs; fewer dependencies); revisit if multi-window routing grows |
| Frontend stores built on `svelte/store` | Decided; the only Svelte import the `app` layer may use |
| Event bus | Tauri events at the frontend boundary; own small typed bus in the kernel for module traffic (section 4.3) |
| Communication patterns: event, request/response, stream | Decided; streams are latest-wins or ordered with gap markers |
| Cursors and other presentation state in the frontend | Decided; persisted through an opaque blob in `settings` |
| Type generation tool | Decided: ts-rs for data types, hand-written client (ADR 0003) |
| Where decimation to pixel width runs | Backend (`display`) when needed; the first version may send the full 4096-sample record |
| State snapshot format | Open: whole snapshot vs per-slice snapshots; start with per-slice |
| How many cross-feature messages go in `kernel::contracts` | Keep a short list; review whenever one is added |
| Project foundation, npm, test stack | Decided (ADR 0001) |
| Token pipeline: `tokens.css` source, generated TS | Decided (ADR 0002) |
| Architecture enforcement tooling | Decided (ADR 0004) |
| Layout tiers and screen fit computed in code | Decided (ADR 0005) |
| Graph renderer: Canvas 2D first behind an interface | Decided (ADR 0006); WebGL2 after M0 spike |
| Roll run mode, roll items, `recording` feature | Decided (ADR 0017) |
| `spectrum` feature, `libs/fft` without external dependency | Decided (ADR 0018) |
| Math as derived trace in `acquisition`, XY in the frontend | Decided (ADR 0019) |
| Auto-setup as `libs/autoset` driven by `acquisition` | Decided (ADR 0020) |
| Zero calibration as its own feature `calibration`; four new kernel messages (thirteen in total, ADR 0010 list) | Decided (ADR 0021) |
| Headless capture CLI as a second composition root in the top-level `cli/` library, run by the one `sweepscope` executable | Decided (ADR 0022) |
| Verification markers removed from the product; facts stay in documentation and an internal hint | Decided (ADR 0023) |
| Linux packaging with the Tauri bundler, udev rule in the .deb, identifier `io.github.dkrepsky.sweepscope` | Decided (ADR 0024) |
| Items not yet defined | See section 10 |

Tension to watch: P1 (backend does all domain processing) means features that look like display logic (persistence
decay, interpolation choice, zoom) need an explicit owner. Rule of thumb: if the backend never needs the value, it is
presentation state in the frontend; otherwise it is domain state and the graph only applies it. Each new such feature
says which it is.

## 10. Left to the implementing agents

Resolved by the scaffold: item 7 (numeric conventions: `f64` for values and times, `f32` for sample payloads, little
endian binary frames, layout fixed by the first frame story) and item 8 (the repository exists; workflow and CI in
section 8.4). Still open, to be decided in the story that first needs them:

1. **Error handling and user notification.** One error taxonomy in the kernel (recoverable, device lost, fatal) and one
   frontend path that turns errors into messages (design.md 10.3 fixes the presentation: Banner, Toast, inline).
2. **Logging and diagnostics.** Structured logging in Rust, log file location, and a "copy diagnostics" action.
3. **Settings and file formats.** The scaffold stores UI-state blobs in one `settings.json` with a `version` field in the
   Tauri app config directory (XDG on Linux). Setups are `acquisition` state, saved as pretty JSON with `format` and `version` (and `last-setup.json` through `Storage`); the
   exports are CSV and the raw `SSR1` frame file ([formats/frame-raw-v1.md](formats/frame-raw-v1.md)). All file writes go through
   the kernel `write_atomic`; migrations arrive with the first version 2.
4. **Offline mode as a device.** A `FileDevice` implementing `ScopeDevice` for saved captures (DAT-4).
5. **Tauri security.** The scaffold sets a strict CSP, an app manifest listing only our commands and one capability for the
   main window. Reviewed in M2 for the dialog plugin (`tauri-plugin-dialog`): used from Rust only through the kernel `FilePicker`, no dialog or file-system permission is granted to the frontend (ADR 0016).
   The CSP (`src-tauri/tauri.conf.json`, JSON has no comments) allows `style-src 'unsafe-inline'` on purpose: Svelte
   sets inline `style` attributes and `style:` directives at run time (scale, positions, colours from tokens), which a
   strict `style-src` blocks. Scripts stay `'self'` only, so inline styles cannot run code. Revisit if styles move to
   classes only.
6. **Multi-window.** Which stores are per window and which per application (DSP-8).
7. **Hardware test checklist.** The manual pre-release list.

## 11. Engineering answers to the design questions (design.md section 14)

| ID | Answer | Status |
|---|---|---|
| Q-E1 | The backend publishes `InstrumentFacts` as domain state (acquisition slice): effective sample rate and interval, record length in points per channel, capture rate in waveforms per second, dead time as a fraction of time. Each value travels as a number in SI base units plus the backend-formatted text (`FormattedValue { text, value, unit }`); one value and one unit per slot. Update rate at most 4 per second; changes highlight for one `motion-normal`. | Decided; implemented by the acquisition story |
| Q-E2 | The backend decides. Aliasing and similar warnings are backend state (`warnings` list with id, severity, channel, short text, message in the measurement slice; alias raised above Fs/5, cleared at 0.95 of it) that the UI only displays and that clears when the backend clears it. | Decided; implemented in M3 |
| Q-E3 | Void: the verification markers and `ScaleOption.verified` are removed (ADR 0023). | Void (M4) |
| Q-E4 | Void: the accuracy note is removed from the product; the M4 accuracy page replaces it (ADR 0023). | Void (M4) |
| Q-E5 | Formatting is one implementation in Rust (`libs/engineering-format`), 3 significant digits by default (1.91 V, 500 µs), prefix rules per quantity; the digit count belongs to the `Quantity` (measured frequency and period use 4: `Quantity::Frequency`, `Quantity::Period`). Domain values arrive pre-formatted. Cursor readouts are frontend state and are formatted in the browser by a small TypeScript port of three quantities (volts, seconds, hertz) checked against shared golden vectors (`golden.tsv`, read by the Rust and the TypeScript tests). A backend round trip per cursor move cannot meet the same-frame feedback of #14 AC1. Four-digit rows live in a Rust-only golden file. | Decided (M2, ADR 0014) |
| Q-E6 | Time cursors are relative to samples, never to the trigger (PRD 5.2). X readouts are Δ-based (ΔX and 1/ΔX). The absolute X1 and X2 values are the time from the left edge of the screen, that is the first sample of the displayed window, not the first sample of the record (the record starts 3.2 divisions left of the screen and pan would move it invisibly). Cursor positions are stored as fractions of the screen width. | Decided (M2 spec D1, approved at G3) |
| Q-E7 | Backend publishes availability per control, `Availability { enabled, reason }`, so the UI encodes no rule (P1). Default policy: settings are accepted while stopped and applied at the next arm; offline and disconnected accept presentation settings only; roll disables trigger and fast time bases. To be confirmed against the driver in the acquisition story. | Principle decided; policy open |
| Q-E8 | A channel is clipped when any sample code of the whole record is 0 or 255 (the converter rails). Clipping invalidates the amplitude-dependent quantities (Vmax, Vmin, Vpp, Vmean, Vrms, rise time, fall time) with reason `Clipped`; frequency, period, widths and duty cycle stay valid, because clipping leaves the mid-level crossings in place. Invalid values carry a reason code plus backend text (shown as `-` with a numbered footnote); the slice carries `clipped` per channel for the Clip badge. | Decided (M2, ADR 0014 item 4) |
| Q-E9 | Ground marker 0 div; steppers to their documented default (a value in the backend scale table marked default); pan centred. | Decided (PM) |
| Q-E10 | Wheel zoom anchors at the pointer position. | Decided (PM) |
| Q-E11 | Persistence presets: Off, 0.5 s, 1 s, 2 s, infinite (US-07). Setting in `display`; ghosts accumulate in 8 age buckets in `ui/graph`. | Decided (PM, M3) |
| Q-E12 | Default text scale 100 %, stored in the app settings file through the opaque UI-state blob (presentation state, rule 4.5.3). | Decided (PM, implemented in the scaffold) |
| Q-E13 | Canvas 2D first behind `TraceRenderer` (ADR 0006), device-pixel snapping in code, redraw on `devicePixelRatio` change. Marker ghosts are DOM overlays reconciled by the `app` layer: ghost on drag, `pending` prop after 250 ms, confirmed value from the store. WebGL2, persistence and budget measurement after the M0 spike. | Partly decided |
| Q-E14 | Window minimum 1024 x 720 logical px in `tauri.conf.json` (human decision D-L1; content floor 64 x 45 rem, design 3.2); tiers and screen fit computed in TypeScript, no rem media queries or container units (ADR 0005). | Decided |

Open items for the feature stories (explicit): Q-E7 policy, WebGL2 and
persistence, the stream pattern (kernel implements events and request/response first), the state aggregator and
snapshot (introduced by the first domain feature), first-run hints.
