# Design spec: M4 Polish (#33, #34, #37, #50)

| | |
|---|---|
| Status | NEEDS_HUMAN_APPROVAL (G2). Revision 5 (hardware feedback: no focus ring on the step heading, `Continue` in the not-shorted step, per-channel tooltip rows; changes marked "rev 5"); revision 4 (Done table per range, Q10 resolved); revision 3 (copy, status bar layout, connection item); revision 2 supersedes the first G2 draft (task-panel wizard, Zero offset row, marker popovers), which is void after the M4 G1 re-approval |
| Scope | Full design: #33 calibration dialog and bottom-bar status tag, #37 application icon. UI touchpoints: #34 (status tooltip), #50 (removal of the UNVERIFIED and PARTIALLY VERIFIED tags). #38 (CLI) needs no design |
| Sources | `docs/prd.md` (5.2, VER-3, US-14, APP-4), `docs/design.md` (3, 4, 9, 10.3), issues #33, #34, #37, #50 as amended |
| Wireframes | `prototypes/calibration-dialog.html` (all dialog states), `status-tag.html` (tag in both states, tooltips, focus, no device, connection item states), `main-window.html` and `main-window-calibrated.html` (1440 x 900, markers removed, tag in place), `main-window-dialog.html` (dialog over the window, acquisition paused), `icon-preview.html`. Tokens from `docs/design-prototype/tokens.css`, PROPOSED rules in `prototypes/m4.css` |
| Note | Numbers in the wireframes are illustrative; the dialog copy is final wording for the PM to approve at G2 (tooltip text and numbers come from engineering). Technical and hardware questions are in section 9; this spec asserts none of them |
| Assets | `src-tauri/icons/` (unchanged since the icon revision: icon.svg, icon-small.svg, PNG 16 to 512, icon.ico, icon.icns) |

## 1. Goal and user journey (#33)

| Persona | Situation | Wants | What can go wrong |
|---|---|---|---|
| Test engineer (US-14) | Reads millivolts on a sensitive range | Know at a glance whether the device is calibrated, calibrate in a few guided clicks | Calibrating with an open input; a half-calibrated device; a stale or unclear status; the live trace changing under them while they handle probes |
| Hobbyist | Never calibrated | Plain instructions, one channel at a time, no way to break anything | Shorting a live circuit; not knowing whether anything was saved |

Journey: see `NO CALIBRATION` in the bottom bar -> click it -> read why and press Start -> for each channel: short the input, press Calibrate -> done screen with the values and accuracy summary -> `CALIBRATED`. Hover the tag at any time for the accuracy summary.

## 2. Status tag in the bottom bar

| State | Look | Text | Hover or focus tooltip |
|---|---|---|---|
| Not calibrated (device connected) | Amber outline, warning icon, solid border (no dashed look: that belonged to the removed Unverified tag) | `NO CALIBRATION` | Lead `No calibration`, caption `Typical zero error:`, one row per enabled channel (2.2), `Click to calibrate.` |
| Calibrated (all channels stored) | Green outline, check icon | `CALIBRATED` | Lead `Calibrated <date time>`, one row per enabled channel (2.2), `Click to calibrate again.` |
| No device or offline | Dimmed, disabled | `Calibration -` | Native title: "No device. Connect the scope to calibrate." |

- It is a button (`aria-label` "No calibration. Open calibration dialog" / "Calibrated. Open calibration dialog"). Click, Enter or Space opens the dialog (section 3). State is conveyed by text and icon, never by colour alone.
- Slot: the **right end** of the status bar, after Renderer, essential priority (never dropped by the status bar collapse; Renderer drops first). It **replaces** the status bar `Accuracy` button and popover (decision D4, question Q7); the tooltip carries the accuracy information.
- Tooltip: existing Tooltip component (500 ms delay, also on keyboard focus), multi-line, with a small table of offset rows (2.2). Text and numbers come from the backend; the wireframe values are placeholders.
- Wireframe: `prototypes/status-tag.html`, `main-window.html`, `main-window-calibrated.html`.
- **Known limit (QA C-6, accepted by the PM):** at text scale 150 to 200 % in a 1440 px window the tag, which is never dropped, makes the width fall in the existing scroll tier of `docs/design.md` 3.2; no new tier is added.

### 2.2 Tooltip rows (rev 5)

The notes about gain, extreme vertical positions and resolution are removed. The body is one row per **enabled** channel, in channel order, at the range **currently selected** for that channel (the rows follow the scale strip while the tooltip is open). Disabled channels get no row.

| Part | Form |
|---|---|
| Row | `CH1 +0.66 mV at 2 mV/div`: three columns, channel name in its channel colour (mono, bold), offset right-aligned in mono, muted `at <range>` |
| Offset | engineering notation (`formatQuantity`), explicit `+` for positive, U+2212 for negative, `0 V` for zero |
| Calibrated | offset stored for that channel and range, rows only (no caption) |
| No calibration | caption `Typical zero error:` above the rows; rows in the same form with the typical zero error for those ranges, shown as signed values (for example `+0.66 mV at 2 mV/div`), uncorrected; from 50 mV/div the backend gives an upper bound instead of a typical value, in the form `up to 4.14 mV at 50 mV/div` (two counts), which is deliberately a different form |
| No channel enabled | muted line `No channel enabled` instead of rows |
| Unchanged | lead heading (and date when calibrated), click hint |

Numbers come from the backend (Q11). Wireframe: `prototypes/status-tag.html` (both states and variants).

### 2.1 Connection item (replaces the `CONNECTED` chip)

The connection status in the status bar changes from a `StatusChip` (filled or outlined box with the state word, plus a `Simulator` chip) to the plain item style of the prototype: **state dot, bold state word in `text-primary`, muted detail**. It applies everywhere the connection status is shown. The dot colour is secondary; the word carries the state.

| State | Dot | Word | Detail (from the backend) |
|---|---|---|---|
| Ready | `status-ok` | `Ready` | `Hantek 6254BC` and firmware, FPGA |
| Ready, simulator | `status-ok` | `Ready` | `Simulator` first, then the model (the `Simulator` chip is gone) |
| Connecting | `status-info` | `Connecting` | reason, for example "Waiting for the scope to settle" |
| No device | `status-disconnected` | `Disconnected` | "No scope found. Connect the scope" |
| Error | warning icon in `status-error` instead of a dot | `Error` | reason, for example "No permission to open the scope" |

Existing state words (`Ready`, `Connecting`, `Disconnected`, `Error`) and the tooltip stay. Wireframe: `prototypes/status-tag.html`, last section.

## 3. Calibration dialog (#33)

A modal Dialog (`modal`, native `showModal`). Wireframes: `prototypes/calibration-dialog.html`, `main-window-dialog.html`.

### 3.1 Structure (identical in every step so nothing jumps)

Header: title `Zero calibration`, right `Step n of 6`. Channel progress row (steps 2 to 6): `Channels` and four chips `1 2 3 4` in channel colours: done = filled with a check icon, current = ringed (`aria-current="step"`), to do = outline in muted colour; each has an accessible name ("Channel 2, current"). Body (fixed minimum height so footers stay put). Footer: secondary left of the primary, primary right. One primary action per step.

### 3.2 Steps

| Step | Body | Footer | Notes |
|---|---|---|---|
| 1 Why and Start | `Why calibrate`, one sentence, then three bullets: you short each of the 4 inputs in turn; acquisition is paused while this dialog is open; nothing is stored until you finish, and the factory calibration is never changed | `Cancel`, `Start` | Reopened from `CALIBRATED`: a note with bold "This device is calibrated" and, on its own line below, "Last calibrated at <date time>" |
| 2 to 5 Channel n | Heading `Channel n: short the input`, three numbered instructions, probe picture, note "An open input is not zero: it picks up the neighbouring channels." | `Cancel`, `Calibrate` | Repeats for CH1 to CH4. A channel that is off still gets its step: the dialog never asks the user to enable it (Q2) |
| Measuring | Heading `Channel n: measuring`, status line "Measuring CHn, keep the input shorted and still" and a progress bar with percent. The bar covers the sweep over all voltage ranges of that channel; no range names, no range selection UI (the app sweeps ranges itself, Q1) | hint "Cancel stores nothing.", `Cancel`, disabled `Measuring...` | Progress is determinate when the backend reports it, otherwise an indeterminate bar without a percent |
| Not shorted | Error note (`role="alert"`): "The CHn input does not look shorted. Nothing was stored for this channel." plus the measured numbers and "Check the connection between probe tip and ground clip, then try again." Below the steps list a warning note (rev 5): `Continue keeps the measured offsets for CHn, use it only if the input is shorted` | `Cancel`, `Continue`, `Try again` (primary) | Stays on the same channel until the user chooses. `Try again` measures again. `Continue` accepts the measured offsets for this channel (the user vouches the input is shorted) and moves to the next channel step, or to Done after CH4; the channel chip becomes done. Still no `Skip`: a channel is never left without offsets. Offsets are still written only at Done, `Cancel` stores nothing |
| 6 Done | Success note: bold "Calibration completed" and below it, on its own line, "Offsets applied per channel and range and kept after restart, the device is now CALIBRATED"; the **offset table per voltage range and channel** (3.4), volts only. No accuracy summary here (it lives in the status tag tooltip). For each channel accepted with `Continue` a quiet line under the table (rev 5): `CHn accepted although the input did not look shorted` (muted, `font-size-xs`, no icon, no tag, no colour; none when no channel was accepted) | `Done` | The status tag changes to `CALIBRATED` when this step appears |

### 3.4 Done table: offset per voltage range and channel (Q10 resolved)

The calibration samples every voltage range of every channel, so the Done step shows all of them: rows = the 12 voltage ranges (`2 mV/div` to `10 V/div`, ascending), columns = `CH1` to `CH4` (channel-coloured number chips as column headers), cells = offset in volts with the app's engineering formatting (`formatQuantity`: three significant digits, SI prefix, U+2212 minus, for example `-41.2 µV`, `1.52 mV`). Wireframe: `prototypes/calibration-dialog.html`, step 6.

- **Fit:** the table sits in a scroll region with a maximum height of about 11.5 rem (about seven rows at 100 % text scale) so the dialog does not grow with the number of ranges; the header, note and footer stay fixed. At 200 % text scale the region shrinks in rem terms only through the dialog's own max-height (`min(100% - 2rem, ...)`), the table scrolls and the header row stays visible.
- **Sticky header** (`position: sticky`, opaque `bg-raised`); the range column is a row header (`th scope="row"`).
- **Keyboard:** the scroll region is focusable (`tabindex="0"`, `role="region"`, name "Stored offsets per voltage range and channel, scrollable"), arrow keys, Page Up/Down, Home and End scroll it; visible focus ring. Tab order: scroll region, `Done`.
- A channel or range with no value (backend reports none) shows muted `-`; offsets of zero show `0 V`.
- The same table is **not** shown when the dialog is reopened (step 1 shows only the date); the status tag tooltip keeps the compact largest-per-channel summary (approved, unchanged).

### 3.3 Behaviour rules

- **Cancel at any step stores nothing.** Offsets are written only when the last channel finished (the Done step). `Cancel` and `Esc` close the dialog; on the Done step `Esc` and `Done` both just close (already stored).
- **Acquisition is paused** while the dialog is open. The toolbar state chip reads `Paused (calibration)` (stop colour) behind the scrim; the previous run state is restored on close if acquisition was running (decision D3, Q3). While measuring, `Cancel` is enabled and aborts the measurement (nothing stored).
- **Device lost** while open: the body is replaced by an error note "Device disconnected. Calibration cancelled. Nothing was stored." with one button `Close` (`role="alert"`). No device: the tag is disabled and cannot open the dialog.
- **Keyboard and focus:** native modal dialog gives the focus trap and `Esc` = Cancel (inert background, scrim `rgb(8 9 10 / .66)` token-proposed `color-scrim`). On open and on every step change focus moves to the step heading (`tabindex="-1"`), never to `Calibrate`, so Enter cannot start a measurement before the input is shorted. **No visible ring on it (rev 5):** a heading is not interactive, so focus is programmatic only (for screen readers) and the dialog styles `[tabindex="-1"]:focus` and `:focus-visible` with `outline: none`. The same rule covers every element that only receives programmatic focus (the focus-target `Note` in the dialog, the drawer region, the popover container). Audit: the rings that stay are on elements the user operates with the keyboard (buttons, the status tag, the scrollable offset table, which scrolls with arrow keys, and the scope screen, which takes arrow-key shortcuts). Tab order: body links (none), `Cancel`, primary. On close focus returns to the status tag. Step changes and results are announced (`role="status"`; errors `role="alert"`). Dialog has `aria-labelledby` (title) and `aria-describedby` (body).
- Errors stay inside the dialog; no toast.

## 4. Removal of UNVERIFIED and PARTIALLY VERIFIED tags (#50)

Rule: no tag, badge, marker, tooltip or popover about verification anywhere. Ranges and features that were flagged behave as before (no blocking). Nothing replaces them in place: accuracy information lives in the status tag tooltip (section 2) and the accuracy page.

### 4.1 Places in the shipped UI (frontend/src) and what replaces them

| # | Place | Today | After |
|---|---|---|---|
| 1 | Channel row (`ChannelRow.svelte`) | `?` Badge at row end | removed; row end is empty |
| 2 | Scale strip (`ScaleStrip.svelte`) | `Unverified` Badge; word in the chip's accessible name | removed from chip and name |
| 3 | Horizontal group (`HorizontalGroup.svelte`) | `Unverified` Badge for unverified time bases | removed |
| 4 | Horizontal readout (`HorizontalReadout.svelte`) | `Unverified` Badge | removed |
| 5 | Trigger group (`TriggerGroup.svelte`) | `?` Badge when pulse width is outside the verified range | removed |
| 6 | Measurement matrix (`MeasurementMatrix.svelte`, `types.ts`) | cell state `unverified`: `?` after the value, amber left edge, aria-label "unverified range" | removed; cell states are valid, invalid, stale. Value text no longer gets ` ?` (also in `select-measurements.ts` and `compose-screenshot.ts` for screenshots) |
| 7 | Matrix footnote | `? Unverified range ...` line | removed |
| 8 | Dropdown items | `Unverified` Badge inside a Select/Menu item (gallery `ControlSections`) | removed; the `badge` snippet stays available for other uses |
| 9 | Gallery pages (`IndicatorSections`, `ControlSections`, `mock-data`) | Unverified badge and cell state demos | removed |
| 10 | Data model (`ChannelState.unverified`, `ChannelMeasurements`, horizontal `unverified`, pulse `verified`) | carries the marker text or flag | engineering decides whether to remove the fields (Q6) |
| 11 | Status bar `Accuracy` button and popover (`MainStatusBar.svelte`) | opens the accuracy note | replaced by the calibration tag at the right end (D4) |
| 15 | Connection status (`MainStatusBar.svelte` `connection` snippet, `Simulator` chip, `StatusChip` kinds map, tests of the chip) | state `StatusChip` plus `Simulator` chip | plain item: dot, bold state word, muted detail (section 2.1); not an #50 removal but listed so engineering updates code and tests together |
| 12 | Auto-setup toast text (M3 spec) | "unverified ranges keep their `?` badges" | sentence removed |
| 13 | FFT and XY axis tags (M3 spec) | `?` on axis tags for unverified range | removed |
| 14 | Tests of the markers | present | removed or updated (AC 4 of #50) |

Kept, not a verification tag (decision D5, question to the PM): the `Exp` badge on the disabled Video trigger. It marks an untested feature that is disabled, not a measured range; its tooltip says "not verified on this hardware", so the wording is changed to "Experimental: video triggers are untested" if kept. The alias warning, `SW` badge and gap markers are unaffected. The placeholder "accuracy note slots" for trigger position and cursor X (PRD 5.2, +-6 samples) are not verification tags and stay.

### 4.2 Places in `docs/design.md` to amend (after approval, by the designer)

| Section | Change |
|---|---|
| 2, J4 and J8 | delete "`?` badge ... `Unverified` in the scale strip" and "Unverified: `?`" |
| 3.1 region D1 | remove `Unverified` badge from the scale strip contents |
| 3.1 region F | `Accuracy note` becomes the calibration status tag at the right end; the connection item uses the dot, word, detail style (2.1) |
| 3.2 known limit | remove "the unverified-hardware banner" |
| 3.4 deferrals | delete the `Unverified` footnote line row |
| 4 (hardware information) | delete the "Unverified value" row; the "Accuracy note" row becomes "Calibration status: status bar tag, tooltip, dialog" |
| 5.5 | `color-status-warning` use "`Unverified`," removed from the list |
| 9 components | MeasurementMatrix states: valid, invalid, stale; remove the unverified note in 9.1 (`Badge ?`) and 9.2 (accuracy-note slot only for trigger position); add `CalibrationTag` and `CalibrationDialog` (section 6) |
| 9.3, AC-M1 | remove the unverified clauses |
| 12 | prototype list: `index.html` and `components.html` lose the unverified markers and the Accuracy placeholder (regenerated after approval) |
| 14 | Q-E3 and Q-E4 closed (void) |
| `docs/prd.md` | goal 3 sentence about "says so", 5.2 volts row consequence ("unverified range marker"), VER-3 (wizard becomes the dialog, per device), US-14 bullets, M4 roadmap row: PM amends |

## 5. Application icon (#37)

Unchanged from the revised icon draft. Files in `src-tauri/icons/` (`prototypes/icon-preview.html`): `icon.svg` (source, 1024 viewBox), `icon-small.svg` (16 to 48 px), `16x16`, `24x24`, `32x32`, `48x48`, `64x64`, `128x128`, `128x128@2x`, `256x256`, `icon.png` (512), `icon.ico`, `icon.icns`.

Design: a scope screen. Rounded square in `bg-scope` #101214 with a frame in `grid-frame` #59616a, 4 x 4 graticule in `grid-major` #2a3036, centre axes in `grid-axis` #3d454d, and one thick CH1 trace in `ch1` #f2e24a: exactly one period of a true sinusoid (four cubic Béziers derived from sin(), equal half-periods, peaks at the amplitude, zero crossings on the centre axis). Small sizes use the simplified art. Rejected: green sine (green means Run), several traces, letters. Colours are token values written into the SVG; regenerate if tokens change. Identifier `io.github.dkrepsky.sweepscope` is engineering's.

## 6. Design system impact

- **Tokens added:** `color-scrim` `rgb(8 9 10 / 0.66)` for the modal backdrop (dark theme only). Nothing else.
- **Components reused:** Dialog (modal), Button, Tooltip, StatusChip look, Badge (no verification kinds), Segmented unchanged.
- **Added:**
  - `ConnectionItem` (app level, status bar): props `state`, `detail`, `simulated`; replaces the StatusChip use for connection. Look in 2.1.
  - `CalibrationTag` (app level, in the status bar): props `state` (`none`, `calibrated`, `unavailable`), `onclick`; snippet `tooltip`. States: default, hover, focus-visible, disabled. Uses `status-warning`, `status-ok`, text-disabled.
  - `CalibrationDialog` (app level): steps as in 3.2; built from Dialog snippets `title`, `body`, `actions`.
  - `StepChips` (small, `ui/`): props `items[{label, state, color}]` with states done, current, todo, snippet `chip`; used for the channel progress row.
  - Icons: `close`, `ok`, `info` (16 px, same stroke).
- **Changed:** `MeasurementMatrix` loses cell state `unverified`; `Dialog` gains focus-to-heading on open (native behaviour kept); `Badge` kinds unchanged (`warning` is still used by alias).
- **Dialog and wizard parts** (probe picture, step texts) live in `app/`, not `ui/`.

## 7. Content (exact text)

| Where | Text |
|---|---|
| Tag | `NO CALIBRATION`, `CALIBRATED`, `Calibration -` |
| Tooltip, not calibrated | `No calibration`; `Typical zero error:`; rows `CHn +-<value> at <range>` per enabled channel; `Click to calibrate.` |
| Tooltip, calibrated | `Calibrated <date time>`; rows `CHn +0.66 mV at 2 mV/div`, `CHn 0 V at 1 V/div` per enabled channel; `Click to calibrate again.` |
| Tooltip, no channel enabled | `No channel enabled` instead of the rows |
| Title | `Zero calibration` |
| Step 1 | `Why calibrate`; "Calibration measures the zero error of each input while it is shorted and subtracts it, so small voltages read correctly."; bullets "You short each of the 4 inputs in turn", "Acquisition is paused while this dialog is open", "Nothing is stored until you finish, and the factory calibration is never changed"; `Cancel`, `Start` |
| Channel step | `Channel n: short the input`; 1 `Disconnect the probe from the circuit. Never short a live circuit.` 2 `Short the CHn input: touch the probe tip to its ground clip, or join the BNC centre pin to its shell.` 3 `Click Calibrate when ready, and keep the input still.`; note `An open input is not zero: it picks up the neighbouring channels.`; picture alt `Probe tip touching the probe ground clip` |
| Measuring | `Channel n: measuring`; `Measuring CHn, keep the input shorted and still`; `Cancel stores nothing.`; button `Measuring...` |
| Not shorted | `The CHn input does not look shorted. Nothing was stored for this channel.` + numbers from the backend + `Check the connection between probe tip and ground clip, then try again.`; note `Continue keeps the measured offsets for CHn, use it only if the input is shorted`; buttons `Cancel`, `Continue`, `Try again` (primary, right) |
| Done | bold `Calibration completed`; line below `Offsets applied per channel and range and kept after restart, the device is now CALIBRATED`; table header `Range`, `1`, `2`, `3`, `4`, rows `2 mV/div` to `10 V/div`; button `Done` |
| Done, accepted channel | `CHn accepted although the input did not look shorted` (one line per accepted channel, under the table) |
| Reopened note | bold `This device is calibrated`; line below `Last calibrated at <date time>` |
| Device lost | `Device disconnected. Calibration cancelled. Nothing was stored.`; `Close` |
| Toolbar | `Paused (calibration)` |

Numbers use engineering notation with U+2212 minus and the existing `formatQuantity` rules.

## 8. Accessibility

Modal with focus trap, inert background, `Esc` = Cancel (except the Done step: close), focus to the step heading on every step, return to the tag on close. Accessible names: dialog (title and body), channel chips ("Channel 2, done"), progress bar (`role="progressbar"`, `aria-valuenow`), tag (state and action). State never by colour alone (text plus icon: warning triangle, check). Contrast: text on `bg-raised` uses `text-primary` and `text-secondary` as in the existing Dialog; control borders `border-control`; tag text on `bg-inset`: `status-warning` #e0a030 and `status-ok` #4fd18b are the existing status colours already used on `bg-inset` chips (measured values in `docs/design.md` 5.5). Text scale 100 to 200 %: the dialog is `max-width: min(34rem, 100% - 2rem)` and its body scrolls; the footer stays pinned. Reduced motion: progress bar has no animation beyond width. Icon: single luminance-contrast hue, colour-blind safe.

## 9. Questions for engineering / PM

| # | Question | To |
|---|---|---|
| Q1 | Sweep of all ranges per channel (proposed default): feasible, duration per channel, can it report progress (percent), what is the unit of the "Largest offset" summary shown in the Done table and tooltip (largest absolute over ranges, or another summary)? | Staff |
| Q2 | Calibrating a channel that is switched off: does the backend enable it internally for the measurement, and is the user's channel setup restored? Do coupling, probe attenuation, bandwidth limit or inversion change the key? | Staff |
| Q3 | Pausing: does the dialog pause by stopping the acquisition and resume to the prior state on close (design default), including during Single? | Staff |
| Q4 | Not-shorted criteria, limits and the reason numbers the UI can show; are offsets committed atomically after the last channel (design assumes yes, so Cancel stores nothing)? | Staff |
| Q5 | Are offsets applied to trace, cursors, measurements, exports and saved raw frames, and recorded in the saved header? "Same unit": which identity (serial) is the key; what shows if a different unit connects (tag shows `NO CALIBRATION`)? | Staff |
| Q6 | Remove the data fields that carried the unverified marker (`unverified`, `verified`) or keep them unused? | Staff |
| Q7 | Status bar `Accuracy` button: replaced by the tag (design default). Does the PM want to keep a separate Accuracy item (the tag tooltip already holds the summary)? | PM |
| Q8 | Dialog copy in section 7 is proposed final wording for PM approval; tooltip texts and numbers (standard values, offsets, accuracy summary) come from engineering. | PM, Staff |
| Q9 | Is the `Exp` badge on the Video trigger in or out of #50? | PM |
| Q11 (rev 5) | Tooltip rows: can the backend give, for each enabled channel, the stored offset at its selected range, and the typical zero error per range when not calibrated (and a value for a range that was never measured)? Does `Continue` store exactly the measured offsets, and is the "accepted despite the warning" flag kept per channel for the Done step only (design: not persisted, not in the tooltip)? | Staff |
| Q10 | RESOLVED by the human: the Done step shows the offset per voltage range per channel (section 3.4). | Human |

## 10. Acceptance mapping

| Story | Criterion | Satisfied by |
|---|---|---|
| #33 | Bar shows `CALIBRATED` or `NO CALIBRATION` | Section 2 |
| #33 | Click on NO CALIBRATION opens a dialog explaining why, with Start | 3.2 step 1 |
| #33 | Start shows channel 1 with short-the-input instructions and a Calibrate button | 3.2 channel step, note on open input |
| #33 | Calibrate measures and moves to the next channel until all done | 3.2 measuring then next channel step, channel chips |
| #33 | Not shorted: says so, stores nothing, retry | 3.2 not-shorted step |
| #33 | Done shows values per channel and accuracy summary, status becomes CALIBRATED | 3.2 step 6 (offsets per channel; the accuracy summary is shown in the tag tooltip, section 2, by the human's G2 decision), section 2 |
| #33 | Applied per channel and range, persists | Done text; data behaviour Q5; no separate UI |
| #33 | Click on CALIBRATED reopens the dialog | 2 and 3.2 reopened note |
| #33 | Acquisition paused, cancel at any step stores nothing | 3.3 |
| #34 | Hover on the status gives the accuracy summary tooltip, standard values when not calibrated, offsets values when calibrated | Section 2 and 2.2 (per enabled channel at its selected range) |
| #34 | Page content (statuses, one-unit limitation) | Documentation, no design |
| #50 | No UNVERIFIED or PARTIALLY VERIFIED tag, badge or marker on any screen, setting or range selector | Section 4.1 (14 places) |
| #50 | Previously flagged ranges behave as before | Section 4 rule |
| #50 | PRD and design docs amended | Section 4.2 |
| #50 | Marker tests removed or updated | 4.1 row 14 |
| #37 | Identifier and icon replace placeholders | Section 5 |

## 11. Decisions for the human (proposed defaults)

| # | Decision | Default |
|---|---|---|
| D1 | Dialog shape | One modal dialog, 6 fixed steps, same frame every step, channel chips row |
| D2 | Not-shorted | Stay on the channel; `Cancel`, `Continue` (accept offsets), `Try again` (primary); no skip |
| D3 | Pause | Acquisition pauses while open, chip `Paused (calibration)`, previous run state restored on close |
| D4 | Accuracy item in the status bar | Replaced by the calibration tag (tooltip carries the summary) |
| D5 | `Exp` badge on Video trigger | Keep, reword the tooltip (not an UNVERIFIED tag) |
| D6 | Tag when no device | Dimmed `Calibration -`, disabled |
| D7 | Done table | Offset per voltage range and channel (12 rows x CH1 to CH4), volts only, scrolls inside the dialog body with sticky header, no divisions, no accuracy block (human answer to Q10) |
| D9 | Status bar layout | Calibration tag at the right end; connection shown as `Ready Hantek 6254BC` style item with the states of 2.1 |
| D8 | Icon | As shown in `icon-preview.html` (true sinusoid) |

| D10 (rev 5) | Focus on the step heading | Programmatic only, no visible ring on any non-interactive focus target; rings stay on operable controls |
| D11 (rev 5) | `Continue` copy and weight | Note `Continue keeps the measured offsets for CHn, use it only if the input is shorted`; order `Cancel`, `Continue`, `Try again`; `Try again` primary (the safe action); `Continue` secondary |
| D12 (rev 5) | Accepted channel on Done | Quiet muted line per channel under the table, no tag, no colour |
| D13 (rev 5) | Tooltip rows | Enabled channels only, signed offset at the selected range, no caption when calibrated, caption `Typical zero error:` and `+-` values when not |

Next after approval: handoff to `staff-engineer`.
