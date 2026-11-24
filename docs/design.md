# SweepScope: Design System

| | |
|---|---|
| Status | Draft 0.2, `NEEDS_HUMAN_APPROVAL` (gate G2) |
| Date | 2026-11-21 |
| Related | [prd.md](prd.md), [architecture.md](architecture.md) section 5, prototype in [design-prototype/](design-prototype/) |
| Theme | **Dark only** |

SweepScope is an engineering tool, a bench instrument on a screen. It is not a web app. The interface is dense, precise
and functional, and follows the front panel of a real scope: **Run control, Vertical, Horizontal, Trigger**. There are no
cards, gradients, large radii, shadows on static panels or decorative whitespace.

## 0. Scope of this document

This document specifies **presentation and interaction**: where information appears, which component shows it, how it
looks in each state, how the user operates it. It does **not** decide what is true about the hardware or how a value is
computed. What is shown, and the facts behind it, come from [prd.md](prd.md) (section 5.2 for hardware limits, section 6
for requirement IDs) and from the backend. Where a design needs a technical decision, it is listed in section 14
("Questions for engineering / PM") and is not stated as fact.

**All numbers, texts and states in the prototype are illustrative placeholders.** They show layout and formatting
only. The prototype carries a visible note (`Illustrative numbers (placeholders)`) and the sheet says it too.

### 0.1 Decisions that touch other documents

| # | Decision | Effect |
|---|---|---|
| D1 | **Dark theme only.** One dark gray token set. No light theme. | Overrides PRD DSP-6. PRD DSP-6 should read "colour-blind-safe default channel colours; dark theme only"; PRD 8.4 accessibility bullet ("light and dark themes") and architecture 5.2 ("light, dark") need the same edit (PM). The token pipeline keeps the theme-set mechanism (`data-theme`), only `dark` exists. |

## 1. Principles

1. **Front-panel logic.** Controls are grouped as on a bench scope and stay where they are. No control appears or vanishes when state changes: unavailable controls are disabled and a tooltip says why.
2. **Real numbers, real units.** Every value has a unit and engineering notation (section 6).
3. **State is always visible.** Connection, acquisition state and the instrument facts of PRD 5.2 occupy fixed places. No silent failure.
4. **Honest hardware.** Limits from PRD 5.2 are shown through fixed presentation slots (section 4).
5. **One gesture per job**, each with a keyboard or field equivalent, a defined target, and a defined result when it cannot apply (section 10).
6. **Dense but legible.** 12 px base text, 24 px controls, 4 px grid, monospaced numerics, user text scaling.
7. **Fast feedback.** A control reacts on pointer-down; a dragged marker follows the pointer in the same frame as a ghost; the confirmed value is the backend's (architecture 5.5).
8. **Few patterns.** Every adjustable quantity is a toggle, segmented choice, stepper or number field.
9. **Components know nothing about the application** (architecture P7, P8): values, callbacks, snippets.

## 2. Users and journeys

Personas: PRD section 3. Journeys derive from US-01 to US-19.

| # | Journey (stories) | Flow in the UI | Edge and failure states (presentation) |
|---|---|---|---|
| J1 | **First connection** (US-01, DEV-6, APP-3) | `Disconnected` chip and hint. After plug-in: `Settling`, then `Ready` with model and firmware in the status bar. A first-run hint points at the CAL terminal. | Permission denied: error Banner with the udev command, `Copy command`, `Retry`. Not found: `Disconnected` plus "Connect a scope or use the simulator". |
| J2 | **Quick look** (US-02) | `Autoset` button; screen settles; Toast reports the result. | No signal: Toast "No signal on any enabled channel". |
| J3 | **Unplug and replug** (US-03) | Screen freezes on the last frame (dimmed), `Disconnected` banner with last-frame time; controls disabled but readable; numbers muted with `Frozen`. After replug: `Settling`, setup restored. | Repeated events do not stack banners. Frozen frame can still be cursor-read and saved. |
| J4 | **Vertical setup** (US-04) | Channel rows (8.1). | Any range is usable; no verification markers (M4, #50). |
| J5 | **Sample rate awareness** (US-05) | Instrument facts always in the status bar and Horizontal group (slots, section 4). | Warning Banner and measurement badge when the backend raises an aliasing warning. |
| J6 | **Trigger a repeating signal** (US-09) | Drag the level marker; `Trig'd` chip. | Normal sweep without event: chip `Waiting for trigger`. Auto sweep: chip `Auto`. |
| J7 | **Single shot** (US-10) | `Single`, chip `Armed`, then `Trig'd` and `Stop`. | Stop during `Armed` cancels. No modals. |
| J8 | **Measure** (US-12, 13, 14) | Cursors (`C`), measurement matrix. | Invalid: `-` with footnote. Stopped or disconnected: muted and `Frozen`. |
| J9 | **Slow signal / roll** (US-08) | Chip `Roll`; moving time axis. | Gap band labelled `GAP`, never a splice. |
| J10 | **Zero calibration** (US-14) | Status bar `CalibrationTag`, click opens the modal `CalibrationDialog` (M4 spec 3): why, channel 1 to 4 (short the input, Calibrate), done with offsets per range and channel. | Not shorted: message in the dialog, nothing stored, `Try again`. Cancel at any step stores nothing. Acquisition is paused while the dialog is open. |
| J11 | **Export and resume** (US-17, 18) | `Save` menu; opened capture sets `Offline`. | Offline: acquisition controls disabled with tooltip "Offline: no device". |

## 3. Main window layout and responsive behaviour

### 3.1 Regions

```
+------------------------------------------------------------------------------------------+
| A  TOOLBAR  brand | RUN CONTROL [Stop][Single][Force][Autoset] chips | VIEW | tools | data |
+------------------------------------------------------------+-----------------------------+
| B  HORIZONTAL readout + record overview                    |                             |
|------------------------------------------------------------| E  SIDEBAR                  |
| C  SCOPE SCREEN  10 x 8 graticule                          |   Cursors panel             |
|    gutters: ground markers (left), trigger level (right),  |   Measurements panel        |
|    trigger position (top)                                  |   (matrix, footnotes, stats)|
|------------------------------------------------------------|                             |
| D1 CHANNEL SCALE STRIP                                     |                             |
| D2 FRONT PANEL DOCK  [VERTICAL][HORIZONTAL][TRIGGER]       |                             |
+------------------------------------------------------------+-----------------------------+
| F  STATUS BAR                                                                            |
+------------------------------------------------------------------------------------------+
```

| Region | Purpose | Contents | Rules |
|---|---|---|---|
| A Toolbar (44 px) | Run control, global tools | Run/Stop toggle, Single, Force, Autoset; acquisition chips; view tabs (Yt, XY, FFT); persistence; cursors, math, ref; save, screenshot, settings | Run/Stop is one toggle: label is the action a click performs, the chip beside it is the state. Order never changes. |
| B Horizontal readout (28 px) | Position in time | Time/div, trigger position, window size, record overview bar | Overview shows the record, visible window and trigger tick (HOR-2). |
| C Scope screen | The measurement | Graticule, traces, trigger guides, cursors, markers, overlays | Sizing rule in 3.3. |
| D1 Scale strip | Scales at a glance | One chip per channel: number, volts/div, coupling; `off` | Always visible. |
| D2 Dock | Controls | **Vertical** (4 channel rows), **Horizontal**, **Trigger** | Fixed row heights. |
| E Sidebar (344 px at 100 %) | Numbers | Cursors panel; Measurements panel (`Now`, `Stats`) | Collapses per 3.2. |
| F Status bar (24 px) | Instrument facts | Connection (model, firmware, FPGA, calibration); effective sample rate; record length; capture rate and dead time (slots from PRD 5.2); renderer; **calibration tag** at the right end (`CALIBRATED` / `NO CALIBRATION`, M4 spec 2). Connection is the dot, bold state word, muted detail item (M4 spec 2.1) | Read-only. Warning items turn amber with an icon. |

Out of scope for the window: generator controls (PRD 5.2: none on the BC), digital channels, deep memory.

### 3.2 Responsive behaviour

Layout is specified in **rem** and breakpoints in **rem** (`em` media queries), so the in-app text scale (100, 125, 150, 200 %)
and the OS DPI scale both shift the available width in rem the same way. Available width in rem =
window width in CSS px / (16 x text scale). 1 rem = 16 px at text scale 100 %.

**Content floor: 64 rem x 45 rem (1024 x 720 CSS px at 100 % text scale).** This is the smallest size at which the whole
layout, with the scope screen at its minimum (3.3), fits without clipping. Under the floor the content keeps the floor size
inside a scroll container: **vertical scroll applies to the region between the toolbar and the status bar, which stay
pinned**; horizontal scroll moves the whole content. Controls and the screen are never squeezed or clipped, they scroll.
Behaviour at the floor and above is a testable rule (AC-L1 to AC-L3).

**Window minimum (Tauri): 1024 x 720 (decided, D-L1).** It equals the content floor at 100 % text scale, so the window
can never be dragged into vertical scrolling at default text scale. The Staff Engineer sets ADR 0005 and `tauri.conf.json`
to 1024 x 720. The earlier 1280 x 720 and 1024 x 576 minimums are dropped: width 1024 is supported (tier W3). If the Staff
Engineer measures that W3 at the floor cannot meet AC-L1, the floor height rises (up to 48 rem = 768 px) and the Tauri
minimum follows through G2; the screen minimum never shrinks.

Width tiers:

| Tier | Available width | Layout |
|---|---|---|
| **W1 Full** | >= 90 rem | As the prototype: sidebar 21.5 rem, dock in one row of three groups, toolbar with labels. |
| **W2 Tight** | 80 to < 90 rem | Toolbar secondary tools (Cursors, Math, Ref, Save) lose their text and keep icons with tooltips. Everything else as W1. |
| **W3 Compact** | 64 to < 80 rem | Sidebar becomes a **drawer** over the right edge of the scope screen (toggle button in the toolbar and key `M`; not modal; `Esc` closes). Dock groups wrap: Vertical full width on the first row, Horizontal and Trigger side by side on the second. Status bar items beyond the priority ones wrap to a second line. **Toolbar at W3 (ratified, fits 1024 px):** the brand and the `RUN` and `VIEW` group captions are hidden, and `Single`, `Force`, `Autoset` lose their text and keep icon, `aria-label` (full name) and tooltip (name and key). The `Run`/`Stop` toggle keeps its text label (the action label is its meaning), as do the view tabs, chips and `Persist`. |
| **Scroll** | < 64 rem | W3 layout at its floor size inside a scroll container. |

Height tiers (window height in rem; thresholds are minimum heights, calibrated by measurement in `docs/specs/scaffold-and-design-system/layout-calibration.md`, spec 11.10). The table is keyed by width class: **wide** = W1, W2 (dock in one row), **narrow** = W3 and Scroll (dock in two rows, status bar may take two lines).

| Tier | wide (W1, W2) | narrow (W3, Scroll) | Layout |
|---|---|---|---|
| **H1 Full** | >= 47.25 rem | >= 59 rem | Prototype layout. |
| **H2 Tight** | 46.25 to < 47.25 rem | 57 to < 59 rem | Dock row height 22 px (`size-control-sm`), record overview hidden (the readout stays). |
| **H3 Compact** | not used (same need as H2) | 50.25 to < 57 rem | Vertical group shows only the selected channel's row with channel tabs 1 to 4; scale strip remains. |
| **H4 Tabs** | 45 to < 46.25 rem | 45 to < 50.25 rem | The three dock groups become tabs `Vertical | Horizontal | Trigger`, one group visible at a time, same order. The scale strip shows a **trigger summary** (source, slope, level, mode) so the trigger state stays visible; the Trigger tab is one click away. |
| **Scroll** | < 45 rem | < 45 rem | Content keeps the floor size inside a scroll container (above). |

These are the final thresholds (they replace the provisional 56 / 50 / 45 rem values). The measured values differ from the early estimates because the W3 dock is 20.9 rem high, not 16.25. Consequence: a 1366 x 768 laptop (48 rem) at W1 or W2 gets H1; at W3 it gets H4 (tabs), because two dock rows plus a two-line status bar do not fit above the 720 px floor.

**Known limit:** banners (for example the notifications stack) are not counted in the height thresholds. Near a threshold a visible banner can take height the table did not budget and cause slight scrolling of the content region, even though the window is above the threshold. Toolbar and status bar stay pinned; the screen keeps its minimum size.

**Known limit (M4, QA C-6, accepted):** the calibration tag sits at the right end of the status bar and is never dropped, so at text scale 150 to 200 % in a 1440 px window the available width falls in the existing scroll tier (below the content floor in rem) and the page scrolls horizontally; no new tier is added.

Collapse order when space runs out (first collapses first): 1 toolbar tool labels (W2: secondary tools; W3: brand, captions, Single/Force/Autoset text), 2 sidebar to drawer (width), 3 dock wraps (width), 4 record overview (height), 5 dock to compact rows (height), 6 status bar items by priority (lowest first: renderer, accuracy note, record length, capture rate), 7 height tier H4: the three dock groups become tabs `Vertical | Horizontal | Trigger` showing one group at a time (same tab order, Trigger one click away; the trigger summary is shown in the scale strip), 8 below the floor: scroll (see above). Steps 1 to 5 are driven by the **available stage size, never by window size alone**: they are re-evaluated when text scale, OS scale or window size change. The scope screen is **never** collapsed or clipped: it absorbs spare space, and a collapse step is triggered when the screen would drop below its minimum size (3.3).

**Never collapses or hides:** run control buttons and acquisition chips, the Trigger group (at H4 it is the `Trigger` tab plus the scale-strip summary), the scale strip, the scope screen with its markers, the connection state and the effective-sample-rate slot in the status bar (US-05 "at all times"; they wrap instead of dropping).

Examples of available width (rem) per OS scale and in-app text scale. Tier names as above.

| Window (CSS px) | Typical case | Text 100 % | 125 % | 150 % | 200 % |
|---|---|---|---|---|---|
| 3840 | 4K at OS 100 % | 240 W1 | 192 W1 | 160 W1 | 120 W1 |
| 2560 | 1440p at 100 %, 4K at 150 % | 160 W1 | 128 W1 | 107 W1 | 80 W2 |
| 2048 | 1440p at OS 125 % | 128 W1 | 102 W1 | 85 W2 | 64 W3 |
| 1920 | 1080p at 100 %, 4K at 200 % | 120 W1 | 96 W1 | 80 W2 | 60 Scroll |
| 1536 | 1080p at OS 125 % | 96 W1 | 77 W3 | 64 W3 | 48 Scroll |
| 1280 | 1080p at OS 150 % | 80 W2 | 64 W3 | 53 Scroll | 40 Scroll |
| 960 | 1080p at OS 200 % | 60 Scroll | Scroll | Scroll | Scroll |

Ultrawide (aspect wider than 16:9): the scope screen is height-limited and centred; spare width goes first to the sidebar (up to 28 rem), then to equal margins around the screen. Nothing is stretched.
4K at OS 100 %: controls keep their rem size and therefore look small; a first-run hint offers a larger text scale (default is a PM question, Q-E12).
Text scale and OS scale are independent: the OS scale changes device pixels per CSS pixel (crispness, 8.5), the text scale changes rem.

### 3.3 Scope screen sizing

- The graticule is **10 x 8 divisions with square divisions** (aspect 10:8, locked).
- Size = the largest 10:8 rectangle that fits the stage (the stage is the region minus the gutters: 2 rem left, 2 rem right, 1.25 rem top, 1 rem bottom). Width limited: `min(stage width, 1.25 x stage height)`.
- **Minimum size 32 rem x 25.6 rem** (512 x 410 px at 100 %; the minimum is in rem, so at 150 % text it is 768 x 614 px). There is no maximum; it grows to 4K.
- **Never clipped:** at every window size at or above the content floor (3.2) the whole screen, frame included, lies inside the stage. Below the floor the stage scrolls instead.
- Spare space on the long axis is empty stage background, centred.
- Gutter content (ground markers, trigger markers) always lies outside the graticule and never overlaps it.

### 3.4 Scope of this step (scaffold) and deferrals

| Item | Scaffold step | Decision |
|---|---|---|
| W3 toolbar labels hidden (3.2) | Required | Ratified (accepted by the human); implemented as is |
| AC-L1 to AC-L3, content floor, pinned bars | Required | BUG-2 and BUG-1 (rem minimum) |
| Hairline graticule at 1.5 and 2 (AC-G2) | Required | BUG-8: code fix, rule corrected in 8.5 |
| Stop contrast | Required | D-L2 (decided) |
| Status bar renderer right-aligned | Deferred | Left-aligned accepted until the status bar gets its real slots (first feature story touching it). The calibration tag is right-aligned (M4) |
| Sidebar grows to 28 rem on ultrawide | Deferred | Scaffold keeps 21.5 rem; do with the sidebar content story |
| Key `M` and the other shortcuts | Deferred | Arrive with the feature each one acts on |

### 3.5 Other views and empty state

- **XY** (M3, spec `specs/m3-analyse/design-spec.md` 4): same regions; one trace in the Y channel colour; scale tags `Y CH2 200 mV/div` and `X CH1 500 mV/div`. The Horizontal group keeps `Time/div` (it sets the sample rate); the `Trig pos` and `Interp` rows become `X axis` and `Y axis` selectors. Trigger markers are hidden, Persist and Cursors are disabled with reasons. A missing channel gives a `ScreenNote` with the cause.
- **FFT** (M3, spec 3): spectrum per enabled channel, tick labels inside the frame; the Horizontal group shows `Time/div`, `Scale` (`dBV | Linear`) and a `Window` menu (Rectangular, Hann, Hamming, Blackman) with the `SW` badge and the readline `Fs ... bin ... (Fs / N)`; the sidebar Cursors panel is replaced by the Peak marker panel (`Off | Auto | Manual`). Persist and Cursors are disabled with reasons.
- **Roll**: moving time axis; trigger controls disabled with a tooltip; gap markers (4).
- **No device**: graticule only with a centred note (state and next step). Controls are disabled, never hidden.

## 4. Presentation contract for hardware information

The facts (what limits exist, their values, when they apply) are in PRD 5.2 and come from the backend. The design only
fixes **where** and **how** each is shown. Values in the prototype are placeholders.

| Information (source) | Slot and component | Appearance | Wording pattern |
|---|---|---|---|
| Effective sample rate, sample interval, record length (PRD 5.2, US-05) | Status bar items; Horizontal group readline | `Readout` with unit; changes highlight for the duration of one `motion-normal` | `Fs <value> <unit>`, `Ts <value>`, `Record <n> pts/ch` |
| Capture rate and dead time (PRD 5.2) | Status bar item | Same | `Capture <n> /s  dead time <n> %` (exact quantities: Q-E1) |
| Warning that the displayed signal may be misread (aliasing, US-05) | `Badge warning` `Alias` on the status bar Fs item (amber, icon), the channel scale chip and the measurement column header; amber edge and footnote on that channel's Frequency and Period cells. No banner (M3 D6) | Dashed amber, clears with the backend state | Tooltip text in the M3 spec 6.2 |
| Calibration status and accuracy summary (VER-3, M4) | Status bar `CalibrationTag` (right end) with tooltip; modal `CalibrationDialog` on click. Cursor panel and trigger position keep their accuracy note lines | Tag: amber outline and warning icon `NO CALIBRATION`, green outline and check icon `CALIBRATED`, dimmed `Calibration -` without a device | Tooltip body: one row per enabled channel, `CH1 +0.66 mV at 2 mV/div` (typical zero error when not calibrated), M4 spec 2.2; wording in M4 spec 7; numbers from the backend |
| Software-computed feature (PRD open question 2) | `Badge software` `SW` next to the control or view | Solid blue outline | `SW` plus tooltip "Computed in software" |
| Roll data gap (PRD 5.2, US-08) | Scope screen overlay; status bar `Gaps n` item | Band in `color-gap-fill` between two dashed `status-warning` lines, label `GAP <duration>`, scrolls with the trace, no trace inside | `GAP` |
| Invalid measurement (MEA-4) | Measurement cell | Muted `-` followed by footnote key `[n]`; reasons listed under the table and in a tooltip | `[n] <reason from backend>` |
| Frozen data (disconnected, stopped by loss, offline) | Panel header; numeric cells | Muted values and `Frozen` or `Offline` badge | |
| Experimental feature (PRD 5.2 video trigger) | `Badge experimental` | Dotted outline, control disabled | `Exp` plus tooltip |

## 5. Tokens

### 5.1 Naming convention

`--ss-<category>-<role>[-<variant>][-<state>]`, lowercase kebab case. The typed source uses the same path with dots:

| Source path | CSS custom property |
|---|---|
| `color.bg.panel` | `--ss-color-bg-panel` |
| `color.status.run` | `--ss-color-status-run` |
| `color.ch1`, `color.ch1Tint` | `--ss-color-ch1`, `--ss-color-ch1-tint` |
| `font.size.md` | `--ss-font-size-md` |
| `space.4` | `--ss-space-4` |
| `size.control.md` | `--ss-size-control-md` |
| `motion.fast` | `--ss-motion-fast` |

Categories: `color`, `font`, `space`, `size`, `radius`, `border`, `trace`, `shadow`, `motion`, `z`. The prefix `ss` avoids clashes. Components reference tokens only (lint rule, architecture 5.2). Role names say what a value is for, not how it looks. State variants are suffixes (`-hover`, `-disabled`). **Canonical values: [design-prototype/tokens.css](design-prototype/tokens.css).** Theme sets live on `:root[data-theme="dark"]`.

### 5.2 Surface layers and text

| Token | Value | Use |
|---|---|---|
| `color-bg-scope` | `#101214` | Waveform screen only |
| `color-bg-app` | `#1a1c1f` | Window, status bar, strips |
| `color-bg-inset` | `#15171a` | Readout wells, fields, steppers |
| `color-bg-panel` | `#212428` | Panels, dock, sidebar, toolbar |
| `color-bg-raised` | `#2b2f34` | Buttons, segmented, dialogs |
| `color-bg-hover` / `-pressed` | `#343940` / `#3c4249` | Interaction |
| `color-bg-selected` | `#1e3350` | Selected, on, active |
| `color-bg-overlay` | `rgb(16 18 20 / 0.72)` | Tags over the screen |
| `color-border-subtle` / `color-border` / `color-border-strong` | `#2f3338` / `#3f454c` / `#59616a` | Decorative separators only (not needed to identify a control, WCAG 1.4.11 exempt): row dividers / panel and group outlines / dialog outline, graticule frame |
| `color-border-control` | `#7b858f` | Boundary of an interactive control that has no other visible shape cue: buttons, Segmented, Stepper, NumberField, select, toggle. At least 3:1 on `bg-panel`, `bg-inset`, `bg-raised` and `bg-hover` (3.99, 4.6, 3.55, 3.1). Focus ring uses `color-focus` |
| `color-focus` | `#8ec1ff` | 2 px focus ring |
| `color-text-primary` | `#e6e9ec` | 12.5:1 on panel |
| `color-text-secondary` | `#b4bbc2` | 7.8:1 |
| `color-text-muted` | `#8f979f` | 5.1:1 |
| `color-text-disabled` | `#5f666d` | Exempt; always paired with a disabled cue |
| `color-text-on-accent` | `#08111f` | Text on channel, accent, filled chips |
| `color-accent` / `-hover` | `#4c9aff` / `#6aaeff` | UI chrome only. **Never inside the scope screen.** |

### 5.3 Graticule

`color-grid-major #2a3036`, `color-grid-axis #3d454d`, `color-grid-tick #4b545d`, `color-grid-frame #59616a`. Look specified in 8.

### 5.4 Channel and trace colours (colour-blind safe)

| Token | Value | Contrast on `bg-scope` | Role |
|---|---|---|---|
| `color-ch1` | `#f2e24a` yellow | 14.1:1 | Channel 1 |
| `color-ch2` | `#2fd0d8` cyan | 10.0:1 | Channel 2 |
| `color-ch3` | `#b08cff` violet | 7.2:1 | Channel 3 |
| `color-ch4` | `#ff6a5c` coral | 6.68:1 | Channel 4 |
| `color-math` | `#ffffff` | 18.8:1 | Math: thin, solid, label `M` |
| `color-ref` | `#9aa6b2` | 7.6:1 | Reference: dashed, label `R` |
| `color-cursor` | `#c8cdd2` | 12:1 | Cursors: dashed, tags |
| `color-trigger` | `#e6e9ec` | 15:1 | Trigger markers and guides |
| `color-ch1-tint` ... `ch4-tint` | 14 % channel colour over `bg-panel` | | Row and chip backgrounds |

Validation: the four channel colours were chosen from a candidate set to maximise the minimum perceptual (CIELAB) distance between every pair under normal vision and simulated deuteranopia, protanopia and tritanopia; the minimum pairwise distance of the four is 34 (next best candidate set: 28); with math included 22, hence math is white and thin. This is a design-side check; QA verifies with a simulator (acceptance criterion AC-T3). **Colour never carries identity alone**: ground marker number, row number badge, colour-matched row border; math and reference also have a letter and a line style.

### 5.5 Status colours

Always with a text label and a glyph.

| Token | Value | Chip text | Glyph |
|---|---|---|---|
| `color-status-run` | `#4fd18b` | `Run`, `Ready` | filled dot |
| `color-status-stop` | `#f2766f` (D-L2, decided; was `#e5534b`) | `Stop` | filled square |
| `color-status-armed` | `#f5a524` | `Armed`, `Waiting for trigger`, `Settling`, `Connecting` | filled dot |
| `color-status-triggered` | `#8be0b0` | `Trig'd` (filled chip) | up triangle |
| `color-status-auto` | `#a2aeff` | `Auto` | none |
| `color-status-disconnected` | `#f0877f` | `Disconnected` | cross |
| `color-status-error` | `#ff6b6b` | `Error` | warning triangle |
| `color-status-warning` | `#e0a030` | gap, alias, `NO CALIBRATION`, warnings | dashed border or warning triangle |
| `color-status-info` | `#7fb0e0` | `SW` | none |
| `color-status-offline` | `#b0b8c4` | `Offline` | none |
| `color-status-record` | `#ff8fb1` (M3) | `Rec` | filled dot |
| `color-status-ok` | `#4fd18b` | calibration OK | dot |

### 5.6 Typography

| Token | Value | Use |
|---|---|---|
| `font-ui` | system-ui, Segoe UI, Noto Sans, DejaVu Sans | Labels, buttons, text |
| `font-mono` | JetBrains Mono, DejaVu Sans Mono, Liberation Mono | Every numeric readout with tabular figures; first available font, nothing is downloaded |
| `font-size-2xs` | 0.625 rem | Group titles, badges: small caps, letter spacing `0.06em` |
| `font-size-xs` | 0.6875 rem | Secondary text, footnotes, units |
| `font-size-sm` | 0.75 rem | Default control text |
| `font-size-md` | 0.8125 rem | Readouts, measurement values |
| `font-size-lg` | 1 rem | Promoted readouts |
| `font-size-xl` | 1.375 rem | Dialog titles |
| `font-weight-regular/medium/bold` | 400 / 500 / 650 | Bold only for headings and chips |

All sizes are rem. Numerics are right-aligned in cells so decimal points line up; the unit follows the value in `font-size-xs`, muted.

### 5.7 Spacing and size

4 px grid: `space-1..7` = 2, 4, 6, 8, 12, 16, 24 px. Controls: `size-control-sm` 20, `-md` 24, `-lg` 32 px. `size-toolbar` 44, `size-statusbar` 24, `size-sidebar` 344, `size-marker` 16, `size-icon` 16 px. Groups are separated by 1 px borders, not space. Minimum click target 24 x 24 px (20 px only for secondary clear and close).

### 5.8 Shape, elevation, motion, layers

Radius `radius-sm` 2 px, `radius-md` 3 px, nothing larger. Border 1 px, focus 2 px (only on elements the user operates; elements that receive programmatic focus only, such as a dialog step heading, show no ring). Static panels are flat; `shadow-popover` and `shadow-dialog` only on floating layers. `motion-fast` 80 ms, `motion-normal` 140 ms, none on the waveform; both 0 under `prefers-reduced-motion`. Layers `z-base 0`, `z-markers 10`, `z-overlay 20`, `z-popover 30`, `z-dialog 40`, `z-toast 50`.

## 6. Number and unit presentation

The formatter is backend work (`libs/engineering-format`); the frontend does not format domain values. The design fixes the look:

- Engineering notation with SI prefixes, no exponents: `1.91 V`, `500 µs`, `250 MS/s`.
- One narrow no-break space between value and unit. Minus is U+2212. Unavailable or invalid value is `-` (en dash U+2013), never `0`, `NaN` or blank.
- Trailing zeros are kept so widths are stable. Readout boxes have a `min-width` for the longest expected text so neighbours do not jump when the prefix changes.
- Scale settings are shown exactly as the step values of PRD 5.1.
- Number of significant digits per quantity: Q-E5. Decimal separator: dot in v1 (candidate story for locale support).

## 7. Iconography

16 px grid, 1.5 px stroke, square caps, `currentColor`, inline SVG symbols (no icon font, no network). Geometric and technical; fills only for transport glyphs and markers. v1 set: play, stop, single, force, autoset, cursors, math, reference, save, screenshot, settings, minus, plus, chevron, rising edge, falling edge, warning, info, close, disconnected. Primary actions always have a text label; icon-only buttons have `aria-label` and a tooltip.

## 8. Graph: visual contract

Appearance only. Implementation (performance, algorithms, WebGL or Canvas) is in section 14.

### 8.1 Strokes and looks

| Element | Look |
|---|---|
| Channel trace | Channel colour, `trace-width` 1.5 px, round joins; user option `trace-width-thick` 2.5 px |
| Math trace | `color-math`, 1 px, solid, label `M` at the ground marker |
| Reference | `color-ref`, 1 px, dashed 6/4, label `R` |
| Cursors | `color-cursor`, 1 px, dashed 6/4, tags in `bg-overlay` with 1 px border in cursor colour |
| Trigger guides | `color-trigger` at 45 % opacity, 1 px, dotted 3/5 |
| Graticule | 1 px, `grid-major` lines each division, `grid-axis` centre axes with 5 ticks per division (`grid-tick`), `grid-frame` border |
| Frozen frame | Traces at 40 % opacity; graticule unchanged |
| Persistence (DSP-5) | Live frame in full channel colour on top; ghosts fade by age in opacity only, hue unchanged, from `opacity-persist-newest` 0.85 to `-oldest` 0.30 over the decay (`Infinite`: `-hold` 0.60); overlaps brighten by alpha compositing, capped below the live trace; a one-sample glitch keeps its min/max span of 8.3. Layer sits above the graticule and below reference, math, traces, cursors |
| Draw order (bottom to top) | graticule, reference, math, CH4, CH3, CH2, CH1, selected channel on top, trigger guides, cursors, tags |

### 8.2 Interpolation modes (DSP-2) as appearance

- **sin(x)/x and linear**: one continuous line, no point markers; same width and colour.
- **Dots**: each sample is a 3 px square in the channel colour, no connecting line.
- Switching mode changes only the line between samples; stroke width, colour and position do not change.

### 8.3 Decimation appearance (DSP-3)

When there are more samples than pixel columns, each column is drawn as a vertical span from its lowest to its highest sample, joined to its neighbours without gaps. A single-sample glitch is visible at least 1 px wide and with the same colour and opacity as the rest of the trace. A decimated trace and an undecimated trace look the same in colour and width.

### 8.4 Clipping and off-screen

- A trace beyond the top or bottom of the graticule is clamped to the border and drawn as a 2 px line along it, with a small triangle in the channel colour at the edge pointing off-screen.
- A ground marker outside the screen sticks to the border with an arrow towards its true position.
- Signal clipping flagged by the backend (MEA-4) shows a `Clip` warning badge in the channel's scale chip. How it is detected: Q-E8.

### 8.5 HiDPI crispness and legibility

- All strokes are specified in CSS px; physical thickness is CSS px x device pixel ratio, and **never less than one device pixel**.
- Graticule, frame and axis lines are **hairlines: exactly one device pixel wide at every OS scale** (100, 125, 150, 200 %), on the device pixel grid (no blur), including fractional ratios. They are the exception to the CSS-px rule above: 1 CSS px would be 2 device px at ratio 2 and not crisp at 1.5. Their legibility comes from the grid tokens (5.3), not from thickness. Traces, cursors and markers stay in CSS px (x ratio, at least one device pixel) and are anti-aliased.
- Legibility minimums at every scale: text at least `font-size-2xs` in rem (never rendered below 10 CSS px at text scale 100 %), tags and readouts at least `font-size-xs`, markers `size-marker` with an 8 px hit margin, trace contrast per 5.4.
- Cursor and trigger tags never overlap each other: tags that would collide stack vertically in order X1, X2, delta, Y1, Y2.
- Changing monitor or OS scale redraws the graph without changing its logical size.

### 8.6 Feedback latency expectations (design targets, validated in section 14)

- A marker or cursor follows the pointer in the same frame (no perceptible lag between pointer and ghost).
- A pressed control shows its pressed state on pointer-down, in the same frame.
- A command-backed marker keeps its dashed ghost until the backend confirms; if no confirmation within 250 ms the marker shows a pending style (dashed outline plus `...` in its tag) until it is confirmed or fails (red tag with the reason).
- Capture-to-screen latency target: PRD 8.3 (under 100 ms).

## 9. Components

All in `ui/`, props and callbacks only, composition by snippets, no application knowledge. Live states: [design-prototype/components.html](design-prototype/components.html). Hard rules for every component: tokens only; no import from `app`; visible focus ring (`focus` colour, 2 px); keyboard operable; a disabled control keeps its place and has a tooltip stating why.

| Component | Props | Snippets | States |
|---|---|---|---|
| Button | `size`, `pressed`, `disabled`, `onclick` | `left`, `children`, `right` | default, hover, pressed, focus-visible, on, disabled |
| RunButton | `running`, `onclick` | `children` | Stop (red, while running), Run (green, while stopped), disabled |
| Segmented | `value`, `options[]`, `onchange` | `option` | item default, active, disabled item |
| Stepper | `value` (display string), `onstep(dir)`, `onreset` | `badge`, `unit` | normal, at limit, off, focus |
| NumberField | `value`, `commit(text)`, `error` | `left`, `unit` | idle, editing, invalid, disabled |
| Badge | `kind` (warning, software, offline, error, experimental) | `children` | static |
| StatusChip | `kind`, `filled`, `dim` | `children`, `left` | active, dim |
| Panel / TaskPanel | none | `header`, `body`, `actions` | normal, collapsed (TaskPanel is the non-modal pattern for future in-sidebar tasks; the zero calibration uses the modal `CalibrationDialog`, M4) |
| MeasurementMatrix | columns, rows | `cell`, `footnotes` | cell: valid, invalid, stale; channel off: column omitted |
| Readout | `value`, `unit`, `muted` | `label` | live, muted |
| Graph | `traces[]`, `view` | `overlay`, `gutterLeft`, `gutterRight`, `gutterTop` | live, frozen, empty |
| Marker / Cursor | `kind`/`axis`, `position`, `color` | `tag` | idle, hover, dragging (ghost), pending, disabled |
| StatusBar | none | `items` | normal; item normal or warning |
| Banner / Toast | `kind` | `icon`, `text`, `actions` | shown, dismissed |
| Dialog | `modal` | `title`, `body`, `actions` | open (refused while acquiring, see 10.3) |
| Menu | `items[]`, `onselect` | `item` | open, hover, disabled, with badge |
| Tooltip | `delay` 500 ms | `children` | on hover and on focus |

M4 additions (spec `specs/m4-polish/design-spec.md` 6): `CalibrationTag` (states none, calibrated, unavailable; snippet `tooltip`), `CalibrationDialog` (modal, 6 steps, focus to the step heading without a visible ring, Esc cancels; not-shorted step offers `Cancel`, `Continue`, `Try again`), `StepChips` (`ui/`, states done, current, todo), `ConnectionItem` (dot, bold state word, muted detail; replaces the connection StatusChip and the Simulator chip); token `color-scrim` `rgb(8 9 10 / 0.66)`; icons `close`, `ok`, `info`; `MeasurementMatrix` loses the `unverified` state. Removed: all UNVERIFIED and PARTIALLY VERIFIED tags, badges, markers and popovers, and the status bar `Accuracy` item.

M3 additions (spec `specs/m3-analyse/design-spec.md` 7): `ScreenNote` (snippets `title`, `children`, `actions`; kind info or warning); `Graph` props `ticksX[]`, `ticksY[]`; `Menu` item `checked` (radio); `Marker` kinds `peak` and `axis="x"`; `StatusChip` kind `record` and text `Done`; `Button` busy look; `MeasurementMatrix` snippet `columnHeader`; icons `record`, `pulse-positive`, `pulse-negative`, `peak`, `info`. Token additions: `opacity-persist-newest/oldest/hold`, `color-gap-fill`, `color-status-record` (to be added to `tokens.css` by the developer).

### 9.1 Channel row

`[number toggle] [Stepper volts/div] [NumberField position] [Segmented AC|DC] [ToggleButton BW] [ToggleButton probe] [ToggleButton Inv]`. 3 px left border in the channel colour; selected row on `ch-tint`. A disabled channel: number toggle outline only, other controls at 40 % opacity, still keyboard reachable.

### 9.2 Trigger group

Type (Edge, Pulse, Video disabled with `Exp`), source 1 to 4 (coloured, disabled when the channel is off), slope, level stepper, sweep (Auto, Normal), trigger-position accuracy-note slot. **Pulse (M3, spec 6.4)**: the slope Segmented becomes polarity (positive or negative pulse), a `Width` row is added with condition Segmented `> < = ≠` and a `NumberField` (invalid input: error border and message, applied value kept), and the accuracy note moves to the Type row tooltip. **Roll**: the group is disabled in place with a visible reason line.

### 9.3 Acceptance criteria per component

Format: **Given** a state, **when** an action, **then** an observable result. "Token" means the value comes from tokens.css (QA checks computed styles against token values).

| ID | Component | Criteria |
|---|---|---|
| AC-B1 | Button | Given an enabled Button, when the pointer is pressed on it, then the pressed background shows in the same frame and `onclick` fires on release. Given focus by Tab, then a 2 px `focus` ring is visible. Given disabled, then it ignores click and Enter and its tooltip states the reason. |
| AC-B2 | RunButton | Given the scope running, then the button reads `Stop` with the stop colour and glyph; given stopped, `Run` with the run colour. The StatusChip changes only when the backend state changes, not on click. Space activates it when no text field has focus. |
| AC-S1 | Segmented | Given 2 or more options, when Left or Right is pressed with focus inside, then focus moves to the neighbour and Enter or Space selects it; exactly one option has `aria-pressed=true`. A disabled option cannot be selected and shows its reason on hover and focus. |
| AC-ST1 | Stepper | Given a value, when the wheel turns over it, or Up or Down is pressed, then `onstep` fires once per notch or key in that direction; at the limit the corresponding button is disabled and the wheel does nothing. Given double-click on the value, then `onreset` fires. Every unit and value is announced via `aria-valuetext` (for example "500 millivolts per division"). |
| AC-N1 | NumberField | Given editing, when Enter is pressed, then `commit(text)` fires; Esc restores the shown value. Given `error`, then border uses `status-error` and the message is below or in the tooltip; the layout does not move. |
| AC-BD1 | Badge | Given kind warning, then dashed outline in `status-warning`; software, solid `status-info`; offline, `status-offline`; each has a text label (never colour only) and a tooltip. |
| AC-C1 | StatusChip | Given any chip, then it has text, a glyph and a colour from 5.5. Given an inactive kind, then it stays in place at 40 % opacity (no layout shift). State changes are announced in an `aria-live=polite` region. |
| AC-M1 | MeasurementMatrix | Given an invalid value, then the cell shows muted `-` plus `[n]`, and the reason for `[n]` appears under the table and in the tooltip. Given a disabled channel, then its column is absent. Given frozen data, then values use `text-muted` and the header shows `Frozen` or `Offline`. |
| AC-G1 | Graph | Given 1 to 4 channels, then each trace uses its channel token and the draw order of 8.1. Given the graph empty, then only the graticule shows. Given frozen, then traces are at 40 % opacity. Given a trace beyond the screen, then 8.4 applies. |
| AC-G2 | Graph, HiDPI | Given OS scale 100, 125, 150 and 200 %, then every graticule, frame and axis line is exactly one device pixel wide in a grid token colour (no neighbouring pixel tinted), and all other strokes are at least one device pixel (pixel scan per scale). |
| AC-L1 | Layout | Given any window of at least the content floor (3.2) at any text scale, then the scope screen is 10:8, at least 32 x 25.6 rem, fully inside the stage (no clipped edge), and the Run control, scale strip, Trigger group (at H4: the Trigger tab and the trigger summary in the scale strip), connection state and sample rate are visible without scrolling. |
| AC-L2 | Layout | Given a window smaller than the floor (for example 1280 x 900 at 200 % text, where the floor is 2048 x 1440 px), then the region between toolbar and status bar scrolls, toolbar and status bar stay pinned, and the screen keeps its minimum size. |
| AC-L2b | Layout, Settings | Given the scroll tier (window smaller than the floor, any text scale), then the text-scale control in Settings stays reachable (the user can scroll to it and operate it by keyboard), so a user who raised the text scale can always lower it again. (PM decision C-2.) |
| AC-L3 | Layout | Given the text scale or window size changes, then the collapse steps are re-evaluated without reload and AC-L1 or AC-L2 holds at the new size. |
| AC-MK1 | Marker, Cursor | Given a draggable marker, when the pointer drags it, then a ghost follows the pointer in the same frame and its tag shows the value with unit; on release the committed value shows when the backend confirms. Given disabled, then the cursor is the default pointer and the tooltip gives the reason. |
| AC-SB1 | StatusBar | Given a window at tier W3, then connection state and the effective-sample-rate slot remain visible (wrapping to a second line if needed). Given a warning item, then it uses `status-warning` with an icon. |
| AC-BN1 | Banner, Toast | Given an error banner, then it stays until dismissed or its cause clears, and shows an action. Given two banners, then at most 2 are visible, newest first. Error Toasts never auto-dismiss and always have a dismiss button; info Toasts dismiss after 4 s and warning Toasts after 8 s; the timer pauses on hover and on focus. A Toast never takes focus on its own. |
| AC-D1 | Dialog, TaskPanel | Given acquisition running, when a modal Dialog is requested by any action except the calibration dialog, then it does not open and the control shows "Stop acquisition first". A TaskPanel opens in the sidebar without blocking anything. Given the calibration status tag is clicked while acquisition runs, then the modal calibration dialog opens and acquisition is paused until it closes. |
| AC-TT1 | Tooltip | Given hover for 500 ms or keyboard focus, then the tooltip shows the control's unit and range (APP-2) and disappears on `Esc`. |
| AC-CH1 | Channel row | Given a disabled channel, then the row keeps its height and position and the row layout does not change when enabled. |
| AC-T1 | Tokens | Given any component's CSS, then it contains no raw colour, size or font literal (lint). |
| AC-T2 | Contrast | Given text on its background, then contrast is at least 4.5:1 (primary, secondary, muted). Given a control boundary (`color-border-control`) or the focus ring, then contrast is at least 3:1 against every background the control can sit on. Decorative separators (`border-subtle`, `border`, `border-strong`) are exempt (WCAG 1.4.11) and must not be the only cue that identifies a control. Channel traces are at least 6.5:1 on `bg-scope`. |
| AC-T3 | Colour blind | Given the four channels under deuteranopia, protanopia and tritanopia simulation, then each pair is distinguishable by colour, number badge and marker. |

## 10. Interactions

### 10.1 Run control

`Run/Stop` toggles. `Single` arms once and shows as on while armed; a second click cancels. `Force` is enabled while running or armed. `Autoset` triggers the backend setup (PRD ACQ-6). Chips change only when the backend confirms.

### 10.2 Gestures on the scope screen

Each gesture has a target, an effect, feedback, an equivalent, and a stated behaviour when it cannot apply. "Cannot apply" covers: no device, offline, disconnected, roll mode where noted, control disabled. Whether a setting is accepted while acquisition is stopped is Q-E7.

| Gesture | Target | Effect | Feedback | Equivalent | When it cannot apply |
|---|---|---|---|---|---|
| Drag trigger level marker | Right-gutter marker | Sets trigger level to the pointer position | Ghost follows the pointer; tag shows level with unit; pending then confirmed (8.6) | Level stepper; `Alt+Up/Down` on the focused marker | Offline, disconnected, roll: marker disabled (default pointer, tooltip "Trigger not available: <reason>"), no drag starts |
| Drag trigger position marker | Top-gutter marker | Sets horizontal trigger position within 0 to 100 % | Same | Trig pos stepper | Same |
| Drag ground marker | Left-gutter marker of a channel | Sets that channel's vertical position and selects the channel | Same | Position field; `Alt+Up/Down` | Disabled channel: marker not drawn. Offline: allowed only if engineering confirms (Q-E7), otherwise disabled with reason |
| Wheel | Scope screen | One notch = one step of Time/div | Stepper value updates; at the limit the stepper flashes its limit style once | Time/div stepper; `Left/Right` | Time/div disabled (offline, roll limits): no change, status hint "Time base fixed: <reason>". Pointer outside the screen: ignored. The zoom anchor is Q-E10 |
| Shift + wheel | Scope screen; over a ground marker, that channel | One notch = one step of volts/div | Stepper updates | Volts/div stepper; `Up/Down` | No enabled channel selected: no change plus hint "Select an enabled channel". Disabled control: as above |
| Drag empty screen area | Scope screen | Pans the visible window within the record | Overview window moves in the same frame | Drag the overview window; `Alt+Left/Right` | Record not larger than the window: nothing moves, default pointer. A drag that starts on a cursor line, marker or tag moves that object instead |
| Drag cursor line or tag | Cursor | Moves that cursor (frontend state) | Ghost not needed: the cursor follows the pointer; readouts update in the same frame | Arrow keys with the cursor focused (`Shift` = 10 steps) | No data (empty graph): cursors cannot be added or dragged; tooltip "No waveform". On frozen or offline data they work |
| Double-click empty screen | Scope screen | Resets the pan to its default position | Window and overview jump | `Home` | Nothing to reset (already default): no effect |
| Double-click ground marker | Marker | Resets that channel's position to 0 div (fixed default) | As drag | | Disabled: as drag |
| Double-click a stepper value | Stepper | Resets to the fixed default of that quantity (values in Q-E9) | Value updates | | Disabled stepper: no effect |
| Double-click a cursor | Cursor | Removes it | Cursor disappears; readout row shows `-` | Focused cursor: `Delete` | |
| Right-click | Scope screen | Menu: `Set trigger level here` (the pointer's vertical position), `Add X cursor here`, `Add Y cursor here` | Menu at pointer | `Shift+F10` | Menu items that cannot apply are shown disabled with a reason |
| Click scale chip | Chip | Selects the channel as target for cursors and keyboard | Chip outline and row tint | `1` to `4` | Disabled channel: chip not selectable (tooltip "Channel off") |

Removed from this version: "double-click the trigger marker sets 50 % of amplitude" (the amplitude source is undefined and unreliable on noisy signals). Candidate for a later story, PM decides (C1).

Zoom is always in instrument steps; there is no free zoom.

### 10.3 Modality and errors

- **No modal dialog while acquisition runs, with one exception.** Needs during acquisition use TaskPanel, Menu, Banner or Toast. A modal action stays available but the app never stops acquisition by itself: it asks the user to Stop first (the control is disabled, tooltip "Stop acquisition first"). Native file pickers are allowed. **Exception (M4, approved): the zero `CalibrationDialog` is modal, opens while acquisition runs and pauses it itself** (toolbar chip `Paused (calibration)`), restoring the previous run state on close; the user cannot lose data because the dialog is the only way to change the stored offsets and Cancel stores nothing.
- Error presentation: recoverable and device-lost errors use Banner with an action; informational results use Toast; fatal errors use a full-width Banner with `Copy diagnostics`. The taxonomy is architecture section 10.1. A command error shows next to its control (red border and message) and never moves the layout.

### 10.4 Keyboard

| Key | Action | When it cannot apply |
|---|---|---|
| `Space` | Run/Stop | No device: no effect, hint in the status bar |
| `S` / `F` / `A` | Single / Force / Autoset | Disabled controls: no effect, tooltip reason shown |
| `C` | Toggle cursors | |
| `M` | Focus the measurements panel (opens the drawer in W3) | Deferred to the first Cursors/Measurements story (3.4); the drawer toggle button exists now |
| `1` to `4` | Select channel; `Shift+1..4` enable or disable | Disabled channel cannot be selected |
| `Up/Down` (screen focused) | Volts/div of the selected channel | As Shift + wheel |
| `Left/Right` (screen focused) | Time/div, left is faster | As wheel |
| `Alt+Up/Down` | Vertical position of the selected channel, or trigger level when its marker is focused | |
| `Alt+Left/Right` | Pan | |
| `Home` | Reset pan | |
| `Ctrl+S`, `Ctrl+E`, `Ctrl+P` | Save setup, export CSV, screenshot | Needs data for export and screenshot; else disabled with reason |
| `Esc` | Cancel drag, close popover, banner or drawer | |
| `F1` or `?` | Shortcut list | |

Tab order follows the front panel: toolbar, scope screen, scale strip, Vertical, Horizontal, Trigger, sidebar, status bar. Shortcuts are inactive while a text field has focus. Undo of Autoset is a candidate story (C2).

### 10.5 Acceptance criteria for interactions

| ID | Criteria |
|---|---|
| AC-I1 | Given a connected device and an enabled trigger, when the user drags the level marker, then the ghost marker is under the pointer in every frame, the tag shows the level with unit, and on release the marker shows the backend-confirmed value. |
| AC-I2 | Given no device, offline, or roll mode, when the user hovers the trigger markers, then the pointer is the default pointer, the tooltip names the reason, and a drag changes nothing. |
| AC-I3 | Given the pointer over the screen, when the wheel turns one notch, then Time/div changes by one step in the wheel direction; at the first or last step it does not change and the limit style shows once. |
| AC-I4 | Given the pointer over the screen and an enabled selected channel, when Shift+wheel turns one notch, then that channel's volts/div changes by one step; given no enabled selected channel, then nothing changes and the hint "Select an enabled channel" shows. |
| AC-I5 | Given a record larger than the window, when the user drags an empty screen area, then the window pans and the overview moves in the same frame; given a record not larger than the window, then nothing moves. |
| AC-I6 | Given a drag that starts on a cursor, marker or tag, then only that object moves and the window does not pan. |
| AC-I7 | Given a non-default pan, when the user double-clicks the empty screen or presses Home, then the pan returns to its default. Given a default pan, then nothing happens. |
| AC-I8 | Given a stepper at a non-default value, when its value is double-clicked, then it resets to the fixed default of Q-E9; the double-click on the trigger marker does nothing. |
| AC-I9 | Given cursors on the screen, when a cursor is double-clicked or focused and Delete is pressed, then it disappears and its readout shows `-`. |
| AC-I10 | Given the graph is empty, when the user presses `C` or right-clicks, then cursor items are disabled with the reason "No waveform". |
| AC-I11 | Given acquisition running, when the user triggers a modal-only action other than calibration, then no dialog opens and the control shows "Stop acquisition first". Calibration is the approved exception (10.3). |
| AC-I12 | Given the keyboard only, when the user presses Tab from the toolbar, then focus visits regions in the order of 10.4 and every control in section 9 can be operated without a pointer. |
| AC-I13 | Given a Segmented choice with a disabled option (for example Video), when it receives focus, then the tooltip shows why it is unavailable. |

## 11. Accessibility

- **Contrast**: AC-T2. Traces at least 6.5:1 on the screen (lowest is ch4, 6.68).
- **Colour blindness**: AC-T3; status and identity always have text, glyph or shape.
- **Scalable text**: all sizes in rem. Settings offer 100, 125, 150, 200 % (3.2 shows the layout effect).
- **Keyboard**: everything operable by Tab, arrows and Enter; steppers Up/Down, `PageUp/PageDown` for coarse; Segmented Left/Right; markers focusable.
- **Screen readers**: controls have names and values with units in `aria-valuetext`. The graph is `role="img"` with a generated summary (channels, scales, trigger). Chips and the status bar are `aria-live="polite"`; errors `assertive`. Tables use real `th` with `scope`.
- **Motion**: none on the waveform; transitions off under `prefers-reduced-motion`.
- **Pointer**: 8 px hit margin on markers; 24 px minimum controls.

## 12. Prototype

Files in `docs/design-prototype/` (static, no network, no JavaScript). **All numbers and texts are illustrative placeholders.**

| File | Content |
|---|---|
| `index.html` | Main window at 1440 x 900: 10 x 8 graticule, three traces, trigger markers, cursors, Run control, Vertical, Horizontal and Trigger groups, cursors and measurements panels (including `-` with footnoted reasons), status bar with the slots of section 4 |
| `components.html` | State sheet of tokens and components, status chips, measurement cell states, gap and disconnected overlays, banners, tooltip. M4 adds `specs/m4-polish/prototypes/` (calibration dialog, status tag, connection item); `index.html` and `components.html` still show the removed unverified markers and Accuracy item until regenerated |
| `tokens.css` | Tokens of section 5 |
| `prototype.css` | Component styles using tokens only |
| `main-window.png` | Rendering of `index.html` |

View: open `docs/design-prototype/index.html` in Chromium or Firefox at least 1280 px wide. The prototype shows the W1/H1 layout only; the tiers of 3.2 are specified here, not drawn. It uses CSS container units for the screen sizing; this is a prototype technique, production sizing is an engineering question (14).

## 13. Traceability

| Requirement | Satisfied by |
|---|---|
| DSP-1 | 3.1 C, D1; 3.3; 8 |
| DSP-2, DSP-3 | 8.2, 8.3 (appearance); engineering in 14 |
| DSP-5 | 8.1 persistence; toolbar menu |
| DSP-6 | D1 dark only; 5.4 |
| DSP-9 | status bar renderer item; engineering in 14 |
| ACQ-1, 2, 4 | Region A, chips, Trigger group |
| ACQ-3 | 3.5 roll; section 4 gap |
| VER-1, 2 | 9.1 |
| HOR-1, 2 | Horizontal group, overview |
| TRG-1, 2 | 9.2, 10.2 |
| MEA-1 | cursors, 10.2 |
| MEA-2, 3, 4 | matrix, section 4, AC-M1 |
| DEV-1 to 4 | status bar, J1, J3 |
| US-05, US-14 | section 4 slots |
| APP-1, 2 | 10.4, AC-TT1 |
| PRD 8.4 | sections 1, 10 |

## 14. Questions for engineering / PM

For the Staff Engineer to validate against the driver's `docs/protocol.md`, `prd.md` and `architecture.md`. The design does not assume the answers.

| ID | Question | Owner |
|---|---|---|
| Q-E1 | Which instrument facts does the backend publish for the status slots (effective sample rate, interval, record length, capture rate, dead time) and in what units and update rate? Design needs one value and one unit per slot. | Staff |
| Q-E2 | Who decides when an aliasing warning is raised (PRD US-05 states a rule)? Is the warning a backend state the UI only displays? | Staff |
| Q-E3 | Void (M4 #50: verification markers removed) | n/a |
| Q-E4 | Void (M4): the accuracy summary is the calibration tag tooltip, wording in M4 spec 7 | n/a |
| Q-E5 | Number of significant digits per quantity, and the unit prefix rules (design only fixes the look). | Staff |
| Q-E6 | Origin of cursor X values: PRD 5.2 says time cursors are relative to samples, not the trigger. The prototype shows the window's left edge as origin. Confirm what the backend and PRD intend. | Staff, PM |
| Q-E7 | Which settings are accepted while acquisition is stopped, offline, or in roll mode (affects which controls are disabled and what the markers do)? | Staff |
| Q-E8 | How is signal clipping detected and reported (for the `Clip` badge, MEA-4)? Full list of invalid-measurement reasons: backend strings or codes? | Staff |
| Q-E9 | Fixed default values for the reset gestures (volts/div, time/div, trigger position). | PM |
| Q-E10 | Anchor point for time zoom with the wheel (centre, trigger, pointer). | PM, Staff |
| Q-E11 | Persistence presets and decay options (DSP-5). | Closed: Off, 0.5 s, 1 s, 2 s, Infinite (architecture Q-E11) |
| Q-E12 | Default text scale at first run on high-DPI screens; where the text scale is stored (presentation state in `app`, persisted via settings blob). | PM, Staff |
| Q-E13 | Rendering implementation, for `architecture.md` section 5.7: performance budget (PRD 8.3 gives 60 fps and 33 ms), decimation and interpolation algorithms, WebGL2 and Canvas 2D parity with the visual contract of section 8, device-pixel snapping and multi-monitor scale changes, and how marker ghosts are reconciled with backend confirmation (including the 250 ms pending style). | Staff |
| Q-E14 | Window minimum enforced in Tauri config, rem media queries support in the target WebKitGTK, and whether CSS container units may be used or the screen is sized in code. | Staff |

### Candidate future stories (PM decides)

- C1: Trigger level "set to 50 % of the signal" (needs a defined amplitude source and noise handling).
- C2: Undo of the last Autoset or setup recall.
- C3: Locale-aware decimal separator.
- C4: High-contrast token set (`data-theme="dark-hc"`).

## 15. Decisions for the human (D-L1, D-L2, Q7 decided; the rest open)

| # | Question | Recommended default |
|---|---|---|
| Q1 | Confirm dark only (D1) and amend PRD DSP-6, 8.4 and architecture 5.2. | Yes |
| Q2 | Channel palette: yellow, cyan, violet, coral (colour-blind optimised) vs. classic yellow, green, blue, red. | Optimised palette |
| Q3 | Run/Stop as one toggle labelled with the action plus a separate state chip. | Yes |
| Q4 | Front-panel dock under the screen, measurements in a right sidebar (alternative: left control rail). | Dock |
| Q5 | `SW` badge on software-computed features. | Yes |
| Q6 | Fixed 10 x 8 graticule with square divisions, locked 10:8 sizing, pan over the record (PRD open question 3). | Yes |
| Q7 | Responsive tiers of 3.2: content floor 64 x 45 rem with scroll below (supersedes the 1280 x 720 minimum and 1024 x 576 floor; window minimum 1024 x 720, D-L1), sidebar to drawer at W3, W3 toolbar labels hidden. | **Decided: yes** |
| Q8 | App never stops acquisition on its own for a dialog. | Yes |
| Q9 | Text scale options 100, 125, 150, 200 %. | Yes |
| Q10 | Which candidate stories (C1 to C4) to add to the backlog. | PM decides |
| D-L1 | Tauri window minimum (3.2): 1024 x 720 (recommended) or keep 1024 x 576 with vertical scroll below 720 | **Decided: 1024 x 720** |
| D-L2 | Stop colour: `color-status-stop` `#e5534b` to `#f2766f` (visible change, lighter red). Measured on `bg-raised`: 4.87:1 (was 3.64); `bg-panel` 5.64; on `bg-hover` 4.21 and `bg-pressed` 3.67 (transient button states; hover/pressed of the Stop button keep the red border as the main cue). Alternative (rejected): Stop text in `color-text-primary` | **Decided: `#f2766f`** |
