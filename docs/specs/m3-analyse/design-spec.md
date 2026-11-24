# Design spec: M3 Analyse (#22 to #31)

| | |
|---|---|
| Status | `NEEDS_HUMAN_APPROVAL` (gate G2) |
| Scope | One spec for the milestone. Full design for #26, #29, #31 (Design: needed). UI touchpoints for #22, #23, #24, #25, #27, #28, #30 |
| Sources | `docs/prd.md` (5.2, 6, US-05 to US-16), `docs/design.md` (3.5, 4, 5, 8, 9, 10), `docs/architecture.md` 5, issues #22 to #31 |
| Wireframes | `prototypes/persistence.html`, `fft.html`, `xy.html` (full window, 1440 x 900), `touchpoints.html` (state sheet). Tokens from `docs/design-prototype/tokens.css` plus the PROPOSED block in `prototypes/m3.css` |
| Note | There is no M2 design spec in the repository (M2 had none); this spec follows `.claude/team/templates/design-spec.md`. All numbers in the wireframes are illustrative placeholders. This spec fixes presentation only; technical and hardware questions are in section 9. |

## 1. Goal and user journey

| Persona (PRD 3) | Situation | What they want | What can go wrong |
|---|---|---|---|
| Embedded developer (US-07) | A glitch appears once in a while on a periodic signal | Leave the glitch on screen long enough to see it, keep cursors and numbers | Glitch fades too fast; persistence hides cursors; persistence suggests a fast update rate the hardware cannot deliver |
| Student (US-15) | Look at harmonics of a square wave, noise on a supply | A spectrum with a clear axis, a window choice, one number for the peak | Reading a wrong bin spacing; no channel enabled; mistaking noise floor for signal |
| Student (US-16) | Compare two channels, Lissajous figure | Pick X and Y, see the figure with scales | A channel is off; X and Y are the same channel; losing Yt settings |
| Hobbyist (US-08) | Watch a slow drift for hours | A scrolling trace, honest gaps, optional recording | A silent splice; trigger controls that look broken; recording fills the disk |
| Hobbyist (US-02) | Unknown signal | One button, stable picture | Silent failure, dialogs, no explanation for a flat line |
| Student (US-05) | Reads a shape that is not real | A visible warning with a plain explanation | Warning that flickers or hides the sample rate |
| Embedded developer (US-10, US-11) | One-time event; stuck pulse | Arm once and see the state; trigger on pulse width | Unclear state while armed; invalid width silently ignored |

## 2. Workflow and states (summary)

Global rules reused from `docs/design.md`: controls never appear or vanish, unavailable controls are disabled with a tooltip that states why (principle 1, AC-I13); chips change only when the backend confirms; no modal dialogs during acquisition (10.3); native file pickers are allowed. Every new unavailable state below also shows its reason as **visible text**, not only a tooltip.

State matrix for the new views and features (rows = situation):

| Situation | Yt | XY | FFT | Persistence | Roll |
|---|---|---|---|---|---|
| Running, device ready | normal | normal | normal | accumulates | scrolls |
| Stopped by the user or Single done | held, full intensity | held | held | held (no new frames) | held |
| Disconnected or offline | frozen 40 % | frozen 40 % | frozen 40 % | held at 40 % | frozen 40 %, recording stops (see 7.2) |
| No device | graticule + note | graticule + note | graticule + note | button disabled | n/a |
| Needed channel off | trace missing | `ScreenNote` (4.3) | `ScreenNote` (3.5) | n/a | n/a |
| Unverified range | `?` badge as today | same on both axis tags | same on FFT axis tag | n/a | same |

## 3. FFT view (#29)

### 3.1 Layout (see `prototypes/fft.html`)

| Region | In FFT |
|---|---|
| Toolbar | View segmented `Yt | XY | FFT` with the `SW` badge (exists). `Persist`, `Cursors` disabled, tooltips "Persistence is available in Yt only" and "Cursors are not available in FFT. Use the peak marker." |
| B Horizontal readout | `SPECTRUM 0 to 25 kHz | BIN 12.21 Hz | WINDOW Hann | SCALE 10 dB/div`. The record overview slot shows `Time domain overview not shown in FFT` (muted). Values come from the backend; the numbers are placeholders. |
| C Screen | Same 10 x 8 graticule. One spectrum trace per enabled channel in its channel colour (same width, draw order of 8.1). Tick labels sit **inside** the frame (`Graph` ticks, 3.4): frequency labels along the bottom edge every 2 divisions, level labels along the left edge every division, in `font-size-2xs` mono on `bg-overlay`. Top-right tag `FFT · Hann · dBV · bin 12.21 Hz` (self-documenting in screenshots). Gutter ground and trigger markers are not drawn (they belong to the time domain). |
| D1 Scale strip | Unchanged. Right side: `Peak marker follows CH<n>`. |
| D2 Horizontal group | Rows keep their places (the group does not change size): `Time/div` stepper (kept: it sets the sample rate, hence span and bin spacing, tooltip says so), `Scale` Segmented `dBV | Linear`, `Window` menu button (`Hann ▾`, with `SW` badge), and the readline `Fs 50 kS/s · bin 12.21 Hz (Fs / N)`. `Trig pos` and `Interp` are not applicable and are replaced in place. |
| D2 Vertical and Trigger groups | Unchanged (they still drive acquisition). |
| E Sidebar | The Cursors panel is replaced in place by the **Peak marker panel** (3.3). Measurements panel unchanged (time-domain numbers; question Q-M3-5). |

Window selector: a `Menu` button (reuse), items `Rectangular`, `Hann`, `Hamming`, `Blackman` as radio items (check mark, `role="menuitemradio"`). A Segmented of four words does not fit the Horizontal group column at W1 and W2 widths, so a menu is used. The button shows the short name (`Rect` for Rectangular).

Scale selector: Segmented `dBV | Linear`. The axis labels and the peak tag change unit with it (`dBV` or `V`; the meaning of the linear quantity, peak or RMS, is Q-M3-3).

### 3.2 Axes

- Frequency axis: `0` at the left edge, last label at the right edge (`25 kHz` in the wireframe); the bin spacing is always shown in three places: Horizontal readout (`BIN 12.21 Hz`), Horizontal group readline (`bin 12.21 Hz (Fs / N)`), screen tag. The value and the formula `Fs / N` come from the backend, the design shows both so the axis is self-explanatory (AC 3).
- Level axis: dBV `10 dB/div`, top label carries the unit (`+10 dBV`), others are bare numbers with the minus sign U+2212. Linear: labels in `V` with engineering prefixes. The reference level and the linear scale steps are Q-M3-2.
- The spectrum trace uses the decimated look of 8.3: a narrow peak is never thinner than one device pixel and keeps the channel colour.

### 3.3 Peak marker

Panel `PEAK MARKER` in the sidebar, header holds a Segmented `Off | Auto | Manual`.

| Mode | Behaviour (presentation) |
|---|---|
| Off | No marker; the panel shows `-` in every value. |
| Auto | Marker `▼` (token `color-cursor`) sits on the highest peak of the **selected channel** (selection by scale chip or `1` to `4`, existing). It moves when the spectrum changes. Tag next to it: `1.001 kHz   −6.9 dBV`. |
| Manual | Click on the spectrum places the marker at the clicked bin; dragging the marker or pressing Left and Right on it (focused) moves it bin by bin (`Shift` = 10 bins). Dragging or clicking while in Auto switches to Manual. |

Panel values: `Channel`, `Bin`, `Frequency`, `Level` (mono, right aligned, units), footnote `Frequency resolution is one bin (12.21 Hz).` Marker, tag and panel read the same numbers (one source). Marker is frontend presentation state; what peak is, and the numbers, come from the backend spectrum (P1).

Cannot apply: no spectrum (no channel, no device) shows `-` and the Segmented is disabled with the reason; Manual placement on frozen data works (like cursors).

### 3.4 Components

- `Graph` gains `ticksX[]` and `ticksY[]` (`{ at: 0..1, text }`), drawn inside the frame. Nothing about frequency is known to the component. Yt keeps its tickless look.
- `Menu` items gain `checked` (radio semantics).
- `Marker` is reused with a new `kind="peak"` look (down triangle) and the `tag` slot.

### 3.5 Empty and error states

| State | Presentation |
|---|---|
| No channel enabled | `ScreenNote` (warning): title `No spectrum`, text `Enable a channel to see its spectrum.` |
| Roll mode | `FFT` segment disabled, tooltip with the backend reason (pattern `FFT is not available in roll mode: it needs a full record.`) |
| Disconnected, offline | Spectrum at 40 %, existing banner; peak panel muted with `Frozen` or `Offline` badge |

## 4. XY view (#31)

### 4.1 Layout (see `prototypes/xy.html`)

| Region | In XY |
|---|---|
| Toolbar | `XY` active. `Persist` and `Cursors` disabled with reasons (persistence Yt only; "Cursors are not available in XY. Return to Yt to measure."). |
| B readout | `XY | X CH1 500 mV/div | Y CH2 200 mV/div | TIME BASE 200 µs/div`; right: `Trigger markers hidden in XY; the Trigger group stays active`. |
| C Screen | Same 10 x 8 graticule, square divisions. One trace, **colour of the Y channel** (decision D4). Scale labels as two tags in the channel colours: top-left `Y  CH2  200 mV/div`, bottom-right `X  CH1  500 mV/div` (AC 1). Ground marker of Y in the left gutter (as in Yt); ground marker of X in the **bottom gutter** (up-pointing numbered marker, `Marker` with `axis="x"`). Trigger markers not drawn. |
| D1 Scale strip | The two chips carry a role tag `X` or `Y` after the scale (letter plus outline, so identity never relies on colour). |
| D2 Horizontal group | `Time/div` stays (it sets the sample rate and so how many points draw each cycle; **deviation from design.md 3.5 which replaced it**, decision D5). `Trig pos` row becomes `X axis` Segmented `1 2 3 4` (channel colours); `Interp` row becomes `Y axis` Segmented. Readline: `Fs 1.25 MS/s · straight segments`. |
| E Sidebar | Cursors panel body replaced by the note `Cursors are not available in XY. Return to Yt to measure time and voltage.` with a `XY` badge. Measurements unchanged. |

Selector rules: a channel that is off is a disabled segment (tooltip `Channel off`); the channel already used by the other axis is a disabled segment (tooltip `Already used as X` or `Already used as Y`). Defaults when entering XY for the first time: X = CH1, Y = CH2, or the first two enabled channels. The choice is presentation state (frontend), remembered while the app runs.

### 4.2 Return to Yt (AC 3)

Switching view changes only the View segmented, the screen content, and the rows listed above. Trigger, vertical, time base, cursors and persistence settings are never altered by XY, so returning to Yt shows them intact. `Interp` is not shown in XY but its value is kept (XY always draws straight segments, question Q-M3-6).

### 4.3 Cannot draw (AC 2)

`ScreenNote` (new, 7) centred on the graticule, warning style (dashed amber border), exact texts:

| Cause | Title | Text | Action |
|---|---|---|---|
| X or Y channel is off | `XY cannot draw` | `CH2 (Y) is off.` (names the channel and the axis) | `Enable CH2` button (same command as the channel toggle) |
| Fewer than two channels enabled | `XY needs two enabled channels` | `Only CH1 is on. Enable another channel.` | none |

The XY tab stays selectable so the user sees the reason. The scale strip chip of the missing channel is dimmed as usual.

## 5. Persistence (#26)

### 5.1 Controls (see `prototypes/persistence.html`)

- The existing toolbar button `Persist: <value> ▾` opens a `Menu` with radio items `Off`, `0.5 s`, `1 s`, `2 s`, `Infinite` (exact options of the AC) and, separated, `Clear persistence` (decision D3: not in the AC; useful with `Infinite`; remove it if not approved).
- Button label always shows the value (`Persist: Off`, `Persist: 1 s`, `Persist: Infinite`); when not Off it uses the `on` look (selected background, accent border). Tooltip: `Persistence: overlays successive frames. Its update rate is the capture rate shown in the status bar.` (no "fast persistence" claim, PRD 5.2).
- The setting is a backend value (architecture 5.7: what and how long comes from backend state); the button shows the confirmed value.
- Disabled (reason in tooltip) in XY, FFT and roll: `Persistence is available in Yt only` (question Q-M3-7).
- Screen tag top-right `PERSIST 1 s` (muted) while on, so screenshots document themselves. Horizontal readout shows `PERSISTENCE 1 s` and `UPDATE 18.4 /s` (placeholder; the backend's frame rate).
- Scale strip right side: `Persistence builds from successive frames; the update rate is in the status bar.` (muted, shown only while on, uses the existing free space).

### 5.2 Colour grading (AC 1)

Grading is **by age, in opacity only, hue unchanged** (design.md 8.1; decision D2, alternatives in 9).

| Rule | Value |
|---|---|
| Live (newest) frame | Full channel colour, `trace-width`, always on top of the ghosts |
| Ghost opacity, newest ghost | `persist-opacity-newest` 0.85 (never brighter than the live trace) |
| Ghost opacity, at expiry | `persist-opacity-oldest` 0.30, then removed. Linear in age over the chosen decay (0.5, 1, 2 s). |
| `Infinite` | Nothing expires; ghosts keep `persist-opacity-hold` 0.60 |
| Overlap | Where many ghosts overlap the pixel becomes brighter by normal alpha compositing, capped below the live trace; this gives the analog "dwell" grading without a second colour scale |
| A single glitch | Drawn as a ghost with the same min/max span rule as 8.3: at least one device pixel wide, same hue as its channel, never dropped for being rare |
| Measured contrast on `bg-scope` (ghost colour composited over the screen background) | At 0.85: ch1 10.34, ch2 7.39, ch3 5.50, ch4 5.09. At 0.60: 5.64, 4.24, 3.31, 3.08 (all at least 3:1). At 0.30 (expiring frame, intentionally dim): 2.28, 1.93, 1.71, 1.62. A ghost is therefore at least 3:1 for most of its life only with `Infinite` or in the first part of a decay; the dim end is a design choice (decision D2), QA verifies the final renderer (Q-M3-8). |

Draw order: graticule, **persistence layer**, reference, math, live traces (CH4 to CH1), trigger guides, cursors, tags. The persistence layer is under everything that is interactive.

### 5.3 Cursors and measurements still work (AC 3)

Cursors, trigger guides, tags and the Measurements panel are drawn and computed as without persistence, on the **live frame only** (the numbers never include ghosts). Cursor readouts are unchanged. Wireframe shows two cursors and the delta tag over the persisted view.

### 5.4 Narrow screens and glitch (AC 4)

At W3 and at the screen minimum (32 x 25.6 rem) the record has more samples than pixel columns; the min/max span of 8.3 applies to every frame including ghosts, so a one-sample glitch is at least 1 device pixel wide and keeps its hue and opacity grade.

## 6. Touchpoints of the other stories (see `prototypes/touchpoints.html`)

### 6.1 Auto-setup (#22, sheet A)

- Press `Autoset` (toolbar or `A`): the button shows a spinner on the right (Button `right` snippet), `aria-busy="true"`, same size; chip `Settling` (existing amber). No dialog (AC 4). `Run`, `Single`, `Force` are disabled with reason `Auto-setup is running` for its duration (Q-M3-9).
- Success: info Toast (4 s) `Auto-setup done. CH1 500 mV/div, 200 µs/div.` (values from the backend; unverified ranges keep their `?` badges).
- No signal: **persistent** screen tag `No signal` (top centre, `bg-overlay`) plus a flat line at the ground level, and an info Toast `Auto-setup: no signal on any enabled channel.` The tag stays until the backend reports a signal or the next Auto-setup (AC 2).
- Failure: warning Toast (8 s, pauses on hover) `Auto-setup could not find a stable setting. Previous setup kept.` with a `Retry` action; the controls show the previous values (AC 3).
- Disabled: no device, offline, no enabled channel (tooltip `Enable a channel first`).

### 6.2 Sample rate and aliasing (#23, sheet B)

- Status bar `Fs` and `Record` items exist; they highlight for `motion-normal` when a channel is toggled (design.md 4). `Fs` is never hidden at any tier.
- Alias warning (backend state, per channel; Q-E2): shown with the `Badge warning` (dashed, text `Alias`) in **four persistent places**, no Banner (decision D6):
  1. Status bar `Fs` item turns amber with the warning icon and `Alias CH1`.
  2. The channel's scale chip: `Alias` badge.
  3. Measurement column header: `Alias` badge; Frequency and Period cells of that channel get an amber left edge and a footnote key `[n]` whose text is the backend message.
  4. All badges share one tooltip.
- Clearing: all four disappear together when the backend clears the state (AC 3). Hysteresis, if any, is the backend's (Q-M3-10).
- Tooltip (exact, plain language): `Possible aliasing on CH1. Its frequency (62.0 MHz) is more than one fifth of the sample rate (250 MS/s), so the display may show a wrong shape or frequency. Try enabling fewer channels, or a faster time base if that raises the sample rate.` Values and channel are filled from the backend; the last sentence needs engineering confirmation (Q-M3-10).

### 6.3 Single shot (#24, sheet C)

| Step | Single button | Chips | Screen |
|---|---|---|---|
| Idle, stopped | off | `Stop` | last frame |
| Armed | on (second click cancels) | `Waiting for trigger` (amber) | trigger markers visible at the chosen position |
| Event, frame arriving | on | `Trig'd` filled | pre-trigger part drawn left of the trigger position marker |
| Done | off | `Stop` and `Done` (kind triggered, outline) | frame held at **full intensity** until Run or Single; cursors and measurements work |
| Cancel (Stop or Single again) | off | `Stop` | previous frame stays |

`Done` is a new chip text on the existing `triggered` kind. `Waiting for trigger` covers `Armed` when no event has arrived (exact AC wording). The trigger position (10, 50, 90 % are values of the existing `Trig pos` stepper) is shown by the top-gutter marker and the readout `TRIG POS 30 %`; no sub-sample claim is made in the UI.

### 6.4 Pulse-width trigger (#25, sheet D)

Trigger group with `Type = Pulse` (rows in place; the Source row's slope Segmented becomes **polarity**):

| Row | Controls |
|---|---|
| Type | `Edge | Pulse | Video` (Video stays disabled with `Exp` badge and tooltip `Video trigger is untested on this hardware (experimental)`, AC 4) |
| Source | channel Segmented; polarity Segmented with icons, `aria-label` `Positive pulse` and `Negative pulse` |
| Level | existing stepper (Q-M3-11: confirm the level applies to pulse) |
| Width | condition Segmented `>  <  =  ≠` (`aria-label`, tooltip `Longer than`, `Shorter than`, `Equal to`, `Not equal to`), then a `NumberField` `WIDTH 500 µs` |
| Sweep | `Auto | Normal` |

The Pulse layout has one more row than Edge: the accuracy-note readline of the Trigger group moves into the tooltip of the Type row while Pulse is selected (it stays in the status bar `Accuracy` item). The Staff Engineer verifies the dock height does not change (the Vertical group sets it); otherwise the note moves to a tooltip in both modes (Q-M3-12).

Width field: accepts text such as `500 us`, `0.5 ms` (parsing by the backend `commit(text)`), shows engineering notation after commit. **Invalid** (AC 3): border `status-error`, inline message `Invalid width. Enter a time such as 500 µs. Applied: 500 µs.` (below the group readline slot, layout does not move), the **applied trigger stays unchanged**; `Esc` or blur restores the shown value and clears the error. Outside the verified region: `?` badge and amber left edge (Q-E3 flag from the backend). H4 scale strip trigger summary: `CH1 pulse + > 500 µs, 900 mV, Auto`.

### 6.5 Math channel (#30, sheet E)

- **Row M** in the Vertical group, a fixed fifth row (always present, never added or removed; off state like a disabled channel). Controls left to right: round-square toggle `M` (white, like a channel number toggle), operation menu button (`A+B`, `A−B`, `A×B`, `Invert A`), source A menu (`A 1 ▾`, channel colour), source B menu (`B 2 ▾`, disabled and `-` for Invert), volts stepper (`1 V/div`, unit `V²/div` for `A×B`), `POS` field, `SW` badge, and one status badge slot (`Mixed` or `Unavailable`).
- The toolbar `Math` button is a toggle mirroring the row's `M` toggle (same command, like Cursors).
- Trace: `color-math`, 1 px, label `M` on its ground marker in the left gutter (draggable like a channel).
- Scale strip: chip `M  A−B  1 V/div` in white outline; `A×B` shows `V²/div`.
- Different source scales (AC 4): the chip and the row show a `Mixed` warning badge, tooltip `CH1 500 mV/div, CH2 20 mV/div. The math trace has its own scale, 1 V/div.` The math scale and unit are always written in the row and the chip.
- Source disabled (AC 3): the row dims in place (as a disabled channel), the badge shows `Unavailable`, and the chip reads `M  A−B  unavailable: CH2 is off` (dashed outline) so the reason is visible without hovering; the trace is not drawn.
- Measurements: the matrix gets an `M` column (white header) that behaves like a channel column; column omitted while unavailable (existing AC-M1). Cursors snap to the selected channel as today; `M` is selectable through its chip.
- Only one math channel in v1 (decision D8).

### 6.6 Roll mode, gaps and recording (#27 and #28, sheet F)

- Roll: chip `Roll` (info colour, existing), graticule fixed, the trace enters at the right edge and scrolls left; tag `now` at the right edge. Horizontal readout: `ROLL  1 s/div | WINDOW 10 s`.
- **Gap overlay** (design.md 4, unchanged look): translucent amber band (`color-gap-fill`, the 14 % already used in the component sheet) between two dashed `status-warning` lines, no trace inside, label `GAP 1.2 s` (duration when the backend reports it, otherwise `GAP`). The band scrolls with the trace; its label sticks to the visible part at the left edge. An `aria-live="polite"` announcement `Data gap of 1.2 s` is made. The status bar gets a `Gaps 2` item (amber, warning icon) while roll has gaps; it clears when roll restarts (optional, decision D9).
- **Trigger controls in roll** (AC 3): the whole Trigger group is disabled in place (`aria-disabled`, focusable), plus a **visible reason line** in the group: `Trigger is not available in roll mode (500 ms/div and slower). Use a faster time base to trigger.` (backend `Availability.reason`, wording pattern). Trigger markers are disabled per 10.2. `Single` and `Force` are disabled with the same reason. Status chips `Trig'd` and `Auto` stay dimmed.
- **Record** (#28): toolbar Data group, before `Save`: toggle `Record` with the record dot icon. Always present. Outside roll it is disabled, tooltip `Recording needs roll mode (500 ms/div or slower)`. In roll and off: enabled. At W3 it keeps icon, `aria-label` and tooltip like `Single`.
  - Click starts the **native file chooser** (allowed while acquiring, 10.3), title `Record roll data to file`. Cancel changes nothing. Confirm starts recording; the button label becomes `Stop rec` in the `status-record` colour (action label pattern of Run/Stop) and a chip `Rec 01:23:45 · 12.4 MB` appears in the Horizontal readout in the slot of the record overview (hidden in roll). Tooltip of the chip: file name and path.
  - Failure (disk full, file removed) (AC 3): recording stops, the chip and button return to off, an **error Banner** stays until dismissed: `Recording stopped: the disk is full. 1.2 GB written to roll-2026-11-24.ssroll. The display is still running.` with `Choose new file` and `Dismiss` (reason text from the backend). The roll display never stops because of recording.
  - Format and gap marking in the file are an engineering and documentation matter (AC 4); the UI offers the file extension chosen by engineering (Q-M3-13).

## 7. Design system impact

### 7.1 Tokens added (dark only)

| Token | Value | Purpose |
|---|---|---|
| `persist-opacity-newest` | 0.85 | Opacity of the newest ghost frame |
| `persist-opacity-oldest` | 0.30 | Opacity of a ghost at expiry |
| `persist-opacity-hold` | 0.60 | Ghost opacity with `Infinite` |
| `color-gap-fill` | `rgb(224 160 48 / 0.14)` | Roll gap band (the existing sheet value made a token) |
| `color-status-record` | `#ff8fb1` | Recording button and chip. 7.28:1 on `bg-panel`, 8.39:1 on `bg-inset` (measured). Pink on purpose, distinct from the coral Stop red; always paired with a dot glyph and the text `Rec`. |

The first three need the `persist` category (or `opacity`); I propose category `opacity`: `opacity-persist-newest`, `-oldest`, `-hold` in the token file (decision for engineering, naming only). No other new colours: spectrum and XY use channel colours; the peak marker uses `color-cursor`; math uses `color-math`.

### 7.2 Components

| Component | Change | Props and slots | States | Accessibility |
|---|---|---|---|---|
| `ScreenNote` (new, `ui/`) | Centred note over the `Graph` `overlay` snippet | `kind` (`info`, `warning`); snippets `title`, `children`, `actions` | shown | `role="status"`, text names the cause and the next step; actions are Buttons |
| `Graph` | Tick labels inside the frame | `ticksX[]`, `ticksY[]` of `{ at, text }` | as today | labels are in the generated `role="img"` summary, not individually announced |
| `Menu` | Radio items | item `checked`, `role="menuitemradio"` | open, hover, checked, disabled | arrow keys, Enter, Esc, as today |
| `Marker` | New kinds | `kind="peak"` (down triangle), `axis="x"` (bottom gutter ground marker) | idle, hover, dragging | focusable; arrow keys move it |
| `StatusChip` | New kind and text | `kind="record"`; text `Done` on kind `triggered` | active, dim | glyph (dot) plus text |
| `Button` | Busy look | uses `right` snippet with a spinner; `aria-busy` | busy | under `prefers-reduced-motion` the spinner is a static `...` |
| `MeasurementMatrix` | Column header badge | snippet `columnHeader` | as today | `th scope="col"` keeps its text |
| `Icon` | New icons | `record`, `pulse-positive`, `pulse-negative`, `peak`, `info` | n/a | `aria-hidden` unless alone |

Reused unchanged: Segmented, Stepper, NumberField, Badge (warning, software, experimental, offline), StatusBar, Banner, Toast, Tooltip, Panel, Readout.

### 7.3 Amendments to `docs/design.md` to apply after approval (Designer)

3.5 (XY row set, FFT row set, Peak marker panel); 4 (alias badges instead of Banner, `Alias` and `Gaps` items, record chip); 5.5 (`status-record`); 8.1 (persistence numbers); 9 (new components); 9.2 (Pulse rows); 14 (Q-E11 closed).

## 8. Accessibility

- Focus order stays toolbar, screen, scale strip, Vertical, Horizontal, Trigger, sidebar, status bar. New controls sit in existing groups.
- Every new Segmented segment has an `aria-label` (`Positive pulse`, `Longer than`, channel numbers with colour names are in the label). Disabled items stay focusable with a tooltip reason (AC-I13).
- Colour never alone: `X` and `Y` role letters; `M` letter plus white outline; `Alias`, `Mixed`, `Unavailable`, `Rec` are text badges with glyphs; peak marker has a numeric tag.
- Contrast: all text uses existing text tokens (AC-T2). `status-record` measured above. Persistence ghosts are data, not UI chrome: the dimmest ghost is intentionally low contrast (5.2), the live trace keeps 6.5:1.
- `aria-valuetext` for the window menu (`Window: Hann`), Width field (`500 microseconds`), math scale (`1 volt per division`).
- Live regions: acquisition chips and status bar `polite` (existing); gap and alias announcements `polite`; recording failure `assertive` (error Banner).
- No animation on the waveform; spinner and transitions off under reduced motion.
- Scalable text: all sizes in rem; the FFT tick labels and the new tags use `font-size-2xs` or larger (minimum rule 8.5).

## 9. Acceptance mapping

| Story | Criterion | Satisfied by |
|---|---|---|
| #22 | 2 to 6 periods, 3 to 7 divisions | Backend behaviour; UI shows result Toast (6.1) |
| #22 | No signal: flat line and `No signal` note | Flat line plus persistent tag `No signal` (6.1, sheet A4) |
| #22 | Failure says so, previous setup kept | Warning Toast with `Retry` (6.1) |
| #22 | No modal | Button busy state, Toasts only (6.1) |
| #23 | Status bar shows Fs and record, updates | Existing items with change highlight (6.2) |
| #23 | Warning above a fifth of Fs | `Alias` badge in four places (6.2) |
| #23 | Warning goes away | All four clear together (6.2) |
| #23 | Tooltip in plain language | Exact text (6.2) |
| #24 | Arms once, stops after one frame, says done | Chip sequence `Waiting for trigger`, `Trig'd`, `Stop` and `Done` (6.3) |
| #24 | Position 10, 50, 90 %, pre-trigger shown | Trig pos marker and readout (6.3); accuracy is engineering |
| #24 | `Waiting for trigger`, Stop cancels | Chip text and cancel row (6.3) |
| #24 | Frame stays until Run or Single | `Done` state, held full intensity (6.3) |
| #25 | Polarity, condition, width with units | Pulse rows (6.4) |
| #25 | Triggers only on matching pulses | Backend; UI sets the three values (6.4) |
| #25 | Invalid width: error, previous stays | Error state and message (6.4) |
| #25 | Video hidden or experimental | Disabled with `Exp` (6.4) |
| #26 | Accumulate, colour grading, fade | 5.2 |
| #26 | Options off, 0.5, 1, 2 s, infinite | Menu items (5.1) |
| #26 | Cursors and measurements in same view | 5.3 |
| #26 | One-sample glitch on narrow screen | 5.4 |
| #27 | Scrolls at 500 ms/div or slower | 6.6 roll description |
| #27 | Gap marker, not a silent splice | Gap band `GAP` (6.6) |
| #27 | Trigger controls say why unavailable | Disabled group plus visible reason line (6.6) |
| #27 | Constant memory for an hour | Engineering; no UI claim |
| #28 | Record streams to chosen file | Record toggle, file chooser, chip (6.6) |
| #28 | Memory does not grow | Engineering |
| #28 | Disk full or file removed: stops, says why, display keeps running | Error Banner (6.6) |
| #28 | File readable, format documented, gaps marked | Engineering and docs (Q-M3-13) |
| #29 | dBV or linear, four windows | Scale Segmented, Window menu (3.1) |
| #29 | Peak marker reads frequency and level | Peak panel and tag (3.3) |
| #29 | Bin spacing = Fs / N on the axis | Three places show it with the formula (3.2) |
| #29 | 1 kHz square: odd harmonics | Wireframe demonstration; backend result |
| #30 | Own trace, scale and position | Row M, chip, ground marker (6.5) |
| #30 | Measurements like a channel | `M` column (6.5) |
| #30 | Source disabled says why | Unavailable badge and chip text (6.5) |
| #30 | Scale and unit explicit | Row, chip, `Mixed` badge (6.5) |
| #31 | X against Y with the same scale labels | Two scale tags, strip roles (4.1) |
| #31 | Channel disabled: view says why | `ScreenNote` (4.3) |
| #31 | Back to Yt, settings intact | 4.2 |

No acceptance criterion needs to change. Two are only partly a design matter: #22 criterion 1 and #24 criterion 2 (backend accuracy).

## 10. Decisions for the human (G2)

| ID | Decision | Recommendation |
|---|---|---|
| D1 | Approve FFT layout: Window as menu, Scale as Segmented, peak marker in a sidebar panel replacing Cursors, FFT of all enabled channels in their colours (alternative: one source channel) | Approve as drawn |
| D2 | Persistence grading by age in opacity, hue unchanged (rejected: blue-to-red heat map, which breaks channel identity and colour-blind safety; rejected: brightness by hit density only, because a rare glitch would be the dimmest thing on screen) | Approve |
| D3 | Add `Clear persistence` to the menu (not in the AC) | Approve; or remove |
| D4 | XY trace takes the Y channel colour | Approve |
| D5 | XY keeps `Time/div` (changes design.md 3.5, where it was replaced) and replaces `Trig pos` and `Interp` rows by the X and Y selectors | Approve |
| D6 | Aliasing shown as badges in four places, no Banner (changes design.md 4: banner would cost height near tier limits and flicker near the threshold) | Approve |
| D7 | Math row M is a fixed fifth row in the Vertical group; the dock grows by one row height and the height tiers need re-measurement (alternatives: a sidebar panel, or only a tab at tiers H3 and H4) | Approve the row; the Staff Engineer re-measures tiers |
| D8 | One math channel in v1 | Approve |
| D9 | `Gaps n` status bar item in roll (not in the AC) | Approve; or remove |
| D10 | New tokens in 7.1, notably the pink `status-record` | Approve |
| D11 | Persistence, cursors and FFT limited as listed (persistence Yt only; no cursors in XY and FFT) | Approve; PM to confirm whether persistence in XY is wanted (Lissajous) |

## 11. Questions for engineering and PM

| ID | Question | Owner |
|---|---|---|
| Q-M3-1 | FFT source: all enabled channels or the math channel too? Cost of four spectra per frame. | Staff |
| Q-M3-2 | FFT vertical mapping: reference level, linear scale steps, dB per division fixed at 10? | Staff |
| Q-M3-3 | dBV reference (RMS) and linear quantity (peak or RMS); window scaling; does auto peak skip the DC bin? Frequency of a peak is the bin centre or interpolated? | Staff |
| Q-M3-4 | Does Time/div set Fs in FFT and XY exactly as in Yt (design assumes yes)? | Staff |
| Q-M3-5 | Measurements panel in FFT and XY: keep time-domain numbers (assumed) or hide? | PM |
| Q-M3-6 | XY draws straight segments only; sin(x)/x or dots wanted? Does channel position move the figure horizontally for X (design draws an X ground marker)? | PM, Staff |
| Q-M3-7 | Persistence in XY and roll: allowed? Does persistence reset on a scale or time base change? | PM, Staff |
| Q-M3-8 | Measure ghost contrast at the final opacities; accumulation cap. | Staff, QA |
| Q-M3-9 | Auto-setup duration, which controls lock, cancel possible? Available in roll? | Staff |
| Q-M3-10 | Alias: per channel or global state, hysteresis, correct hint wording about time base and channels. | Staff |
| Q-M3-11 | Does the pulse trigger use the Level setting? Semantics and tolerance of `=` and `≠`; width range for the error message. | Staff |
| Q-M3-12 | Pulse layout needs one more row than Edge: confirm dock height; where the accuracy note goes. | Staff |
| Q-M3-13 | Recording: file extension and name pattern, behaviour when roll stops while recording (design assumes recording stops and a Toast says so), duration and size source for the chip. | Staff, PM |
| Q-M3-14 | Math: scale steps, default operation and sources (design placeholder: `A−B`, CH1, CH2), whether the unit for `A×B` is `V²`. Row M and height tiers (D7). | Staff, PM |
| Q-M3-15 | Held frame after Single done and after Stop: confirm the existing look is full intensity (design assumes it; 40 % is for disconnected). | Staff |
