# sweepscope: Architecture

| | |
|---|---|
| Status | Draft 0.1 |
| Date | 2026-11-21 |
| Related | [prd.md](prd.md) (requirements, user stories, hardware limits) |

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
├── Cargo.toml                    # workspace
├── docs/
├── src-tauri/                    # Tauri shell: composition root, window and IPC wiring only
│   └── src/
│       ├── main.rs
│       ├── composition.rs        # builds modules, injects dependencies
│       └── ipc/                  # commands and channels, thin adapters onto the bus
├── backend/
│   ├── kernel/                   # shared kernel (crate `sweepscope-kernel`)
│   ├── libs/
│   │   ├── fft/                  # crate `sweepscope-fft`
│   │   ├── interpolation/        # sin(x)/x, linear
│   │   ├── decimation/           # min/max per pixel column
│   │   ├── statistics/           # running min/mean/max/sigma
│   │   └── engineering-format/   # 1.91 V, 500 µs formatting
│   └── features/
│       ├── device/               # discovery, connection state machine, ScopeDevice adapters
│       ├── acquisition/          # run/stop/single, sweep, roll, frame pipeline
│       ├── channels/             # vertical setup, probe factor, zero calibration
│       ├── timebase-trigger/     # horizontal and trigger setup
│       ├── measurement/          # cursors and automatic measurements
│       ├── spectrum/             # FFT view, window functions (uses libs/fft)
│       ├── math-channels/
│       ├── display/              # persistence, interpolation choice, view model
│       ├── export/               # CSV, binary, PNG data
│       └── settings/             # setups, session restore
└── frontend/
    └── src/
        ├── design-system/        # tokens (colour, spacing, type, motion), themes
        ├── ui/                   # reusable components, no application logic
        ├── app/                  # application layer (TypeScript only): stores, services, use cases
        ├── pages/                # display only: screens and layout
        └── api/                  # generated types and the single backend client
```

Each Rust folder is a crate in one Cargo workspace, so the compiler enforces the dependency rules: a feature crate lists
`sweepscope-kernel` and `libs` crates in its manifest and never another feature crate. A CI check fails the build if that
changes.

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
| **Domain state** | Backend, in the module that owns the capability | The backend computes with it, persists it, or applies it to hardware | connection, channel setup, time base, trigger, acquisition status, measurement selection, math channels, persistence setting |
| **Presentation state** | Frontend, in one store in `app` | Only the view uses it | cursor positions, selected tab, panel sizes, which trace is highlighted, theme |

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

## 5. Frontend

### 5.1 Layers

```
pages/          display only: composition of ui components and application-layer values
app/            application layer, TypeScript only: stores, services, use cases; no markup
ui/             reusable components, no application logic
design-system/  tokens and themes
api/            generated types and the one backend client
```

Allowed imports: `pages` → `app`, `ui`; `app` → `api`; `ui` → `design-system`. `ui` never imports `app`, `pages` or
`api`. `pages` never import `api` directly. The `app` layer never imports `.svelte` files or the Svelte compiler runtime; its only
permitted Svelte import is `svelte/store` (plain TypeScript, no components), so it stays testable without a DOM (P10).
Lint rules enforce this.

### 5.2 Design system

- **Tokens** are the only source of visual values: colour (semantic: `surface`, `text-muted`, `channel-1`, `danger`),
  spacing scale, radius, typography, elevation, motion durations, z-index layers.
- Tokens are defined once (a typed source file) and compiled to CSS custom properties. Themes (light, dark, plus a
  colour-blind-safe channel palette) are token sets, not stylesheets. Components never contain a raw colour, pixel size
  or font.
- Waveform colours come from tokens too, so the graph follows the theme.
- A lint rule rejects raw colour and size literals in `ui/` and `pages/`.
- A component gallery page renders every `ui` component in every theme (used for review and visual regression).

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

- `ui/graph` provides a graticule and a trace renderer: WebGL2 with a Canvas 2D fallback (PRD DSP-9).
- It draws the already-scaled data it is given. Persistence accumulation is a rendering technique of the graph
  (it composites successive frames); what to persist and for how long comes from backend state.
- Graph interaction emits intents. Dragging the trigger marker becomes a backend command; dragging a cursor updates the
  frontend `cursors` store.

### 5.8 Backend client and typed contract

- Command, event and snapshot types are generated from the Rust definitions (for example with specta or ts-rs) into
  `frontend/src/api`. The generated files are committed and checked in CI so they cannot drift.
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
- `ui` components: component tests for behaviour and accessibility, plus visual snapshots in the gallery.
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

| Rule | Check |
|---|---|
| Feature crates do not depend on other feature crates | Script reading `cargo metadata` |
| Libs depend on no feature and not on the bus | Same script |
| Only `device` imports `hantek6xx4-lib`; only `src-tauri/ipc` imports Tauri | Same script |
| `ui` imports only `design-system`; `pages` do not import `api`; `app` imports no markup | ESLint import rules |
| No raw colour or size literals in `ui/` and `pages/` | Lint rule |
| Generated API types are up to date | Regenerate and diff |
| Tests follow naming convention (`given_…_when_…_then_…`) | Lint or script on test names |

## 9. Decisions and open items

| Item | Status |
|---|---|
| Tauri 2 shell | Decided |
| Rust workspace with kernel, libs and feature crates | Decided |
| Svelte 5 with Vite as reactive framework (slots as snippets) | Decided. Plain Svelte + Vite rather than SvelteKit (no routing needs; fewer dependencies); revisit if multi-window routing grows |
| Frontend stores built on `svelte/store` | Proposed; the only Svelte import the `app` layer may use |
| Event bus | Tauri events at the frontend boundary; own small typed bus in the kernel for module traffic (section 4.3) |
| Communication patterns: event, request/response, stream | Decided; streams are latest-wins or ordered with gap markers |
| Cursors and other presentation state in the frontend | Decided; persisted through an opaque blob in `settings` |
| Type generation tool (specta, ts-rs) | Open; evaluate in M0 |
| Where decimation to pixel width runs | Backend (`display`) when needed; the first version may send the full 4096-sample record |
| State snapshot format | Open: whole snapshot vs per-slice snapshots; start with per-slice |
| How many cross-feature messages go in `kernel::contracts` | Keep a short list; review whenever one is added |
| Items not yet defined | See section 10 |

Tension to watch: P1 (backend does all domain processing) means features that look like display logic (persistence
decay, interpolation choice, zoom) need an explicit owner. Rule of thumb: if the backend never needs the value, it is
presentation state in the frontend; otherwise it is domain state and the graph only applies it. Each new such feature
says which it is.

## 10. Left to the implementing agents

These are implementation details to be decided during M0/M1 and recorded in the code or in short decision notes:

1. **Error handling and user notification.** One error taxonomy in the kernel (recoverable, device lost, fatal) and one
   frontend path that turns errors into messages (toast, banner, dialog). Who decides what is shown?
2. **Logging and diagnostics.** Structured logging in Rust, log file location, and a "copy diagnostics" action
   (versions, device info, recent log) for bug reports.
3. **Settings and file formats.** Versioning and migration of setup and capture files, plus where they are stored
   (XDG paths on Linux).
4. **Offline mode as a device.** A `FileDevice` implementing `ScopeDevice` for saved captures (DAT-4) would reuse all
   modules unchanged.
5. **Tauri security.** Capability/permission allowlist limited to our commands, strict CSP, no remote content.
6. **Multi-window.** The zoom window (DSP-8) gives each window its own presentation state but the same domain snapshot;
   which stores are per window and which are per application?
7. **Numeric conventions.** `f32` or `f64` for samples and times across the Rust kernel and the wire format; byte order
   and layout of the binary frame payload.
8. **Repository and release workflow.** `sweepscope` is not yet a git repository; the driver repo uses protected `main`,
   pull requests, pinned actions and release-plz. Adopt the same here, with the CI checks of section 8.2.
9. **Hardware test checklist.** The manual pre-release list (plug/unplug stress, calibrator reading) needs to be written.
