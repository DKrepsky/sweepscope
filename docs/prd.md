# sweepscope: Product Requirements Document

| | |
|---|---|
| Status | Draft 0.1 |
| Date | 2026-11-21 |
| First supported device | Hantek 6254BC (4 channels, 250 MHz, 1 GS/s, USB `04b5:6cde`) |
| Hardware layer | `hantek6xx4-lib` 0.2.0 (separate repo, published on crates.io) |
| Primary platform | Linux (Windows and macOS are stretch goals) |

## 1. Summary

sweepscope is a desktop application that turns a USB oscilloscope into a usable bench instrument: live waveforms,
trigger control, cursors, automatic measurements, FFT and data export, with the responsiveness and trustworthy numbers
people expect from a real scope. The first target is the Hantek 6254BC. The design must keep the device layer
replaceable so that other scopes can be added later without rewriting the application.

The vendor software for this class of device is Windows-only, dated and closed. sweepscope is a modern, open, cross-platform alternative that is honest about what the hardware can and cannot do.

## 2. Goals and non-goals

### 2.1 Goals

1. A user can connect a 6254BC, press one button and see a stable, correctly scaled waveform within a minute.
2. Controls follow the front panel of a bench scope (vertical, horizontal, trigger, run/stop) so that existing scope knowledge transfers.
3. Numbers on screen (volts, time, frequency) are correct and carry a stated accuracy; where the hardware layer has not verified a value, the application says so.
4. The application survives the hardware's quirks (re-enumeration, unplug, stalls) without crashing or hanging.
5. Device support is a plug-in boundary (a trait), not a hard dependency throughout the code.

### 2.2 Non-goals (version 1)

- Remote or LAN control, SCPI server.
- Logic analyzer, serial-bus decoding, mixed-signal features. The 6254BC has no digital inputs and the driver does not implement bus triggers.
- Signal generator control. The 6254BC has no generator output.
- Firmware update, EEPROM or FPGA flash writes, or calibration writing. The driver deliberately never writes these.
- Replacing a lab-grade instrument: the 8-bit ADC and 64 KiB of memory set the limit of what can be claimed.

## 3. Users and use cases

| Persona | Needs |
|---|---|
| **Hobbyist / maker** (primary) | Look at a signal quickly, see rise time, frequency, glitches. Does not want to read a manual. |
| **Embedded developer** | Trigger on a specific event (pulse width, edge), compare channels, save screenshots and captures for bug reports. |
| **Student / educator** | Understand what the controls do; clear units, readable display, measurement and cursor tools, XY mode for Lissajous figures. |
| **Test / automation user** (secondary) | Capture repeatedly and export data (CSV and binary) without a mouse. |

## 4. Research findings

### 4.1 What competing scope software offers

Surveyed: PicoScope, Digilent WaveForms, TiePie Multi Channel, NI scope panels, OpenHantek6022, sigrok/PulseView.

| Area | Typical feature set |
|---|---|
| Display | Yt and XY modes, graticule with 8 to 10 divisions, persistence (analog-like intensity grading), zoom and pan, multiple views |
| Acquisition | Run/stop/single, auto/normal/single trigger sweep, roll mode at slow time bases, hold-off, deep memory with a zoomable overview |
| Triggers | Edge, pulse width, video; advanced ones (window, runt, serial bus) on higher-end products |
| Measurements | Frequency, period, Vpp, Vrms, mean, min/max, rise and fall time, duty cycle, with statistics (min/mean/max over many frames) |
| Analysis | Cursors (single, delta, XY phase), FFT with selectable window, math channels (A+B, A−B, A×B, …), reference waveforms, mask testing on high-end |
| Data | CSV and binary export, screenshots, save/restore of setups |
| Conveniences | Auto-setup, undo of last setup, probe attenuation factors, dark and light themes |

Observations that shape this PRD:

- Persistence and math are the strongest differentiators of PC scopes; both are cheap to add once a frame pipeline exists.
- WaveForms' persistence lives in a separate window without cursors, which reviewers criticised. Features should work in
  the main view, not in side windows with reduced capability.
- Decoding and mask testing depend on data volume and hardware support the 6254BC does not provide; excluded from v1.
- The OpenHantek6022 line shows that scope applications live or die by trigger and display behaviour, and that a
  maintained, packaged (Flatpak) application with a clear hardware boundary survives better than a one-off tool.

### 4.2 Measurement-theory considerations

- A scope needs enough samples per signal period. With sin(x)/x interpolation, sample rate ≥ 2.5× the highest frequency
  component is enough; with linear interpolation about 10× is needed. The display must therefore offer sin(x)/x
  interpolation, and linear/dots as options.
- Bandwidth rule of thumb: scope bandwidth 3 to 5× the signal bandwidth for digital signals.
- Waveform capture rate (waveforms per second) is a separate quantity from sample rate. For this device it is bounded by
  USB readback, see section 5.
- Display decimation must preserve glitches: when more samples than pixel columns are drawn, draw min/max per column,
  never plain subsampling.

### 4.3 Implementation research

- GPU-friendly approach used by other real-time plotting projects: upload sample buffers and compute min/max per pixel
  column in a shader. Drawing every sample as geometry needs decimation for large traces.
- **Framework decision: Tauri** (Rust backend, web frontend). Consequences, from Tauri's documentation and issue
  discussions:
  - Streaming data to the frontend should use Tauri `Channel`s with binary payloads, not repeated `invoke` calls or JSON.
    Large channel payloads are fetched through a custom path instead of sent inline. Published throughput figures are
    informal and platform dependent (one report: about 5 ms on macOS versus about 200 ms on Windows for 10 MB), so
    throughput must be measured on Linux.
  - On Linux Tauri uses WebKitGTK. Tauri's own Linux graphics page documents rendering problems on some GPU/driver
    combinations (most often NVIDIA, with the DMABUF renderer as a common cause) and recommends a non-WebGL fallback on
    Linux. The waveform renderer therefore needs a Canvas 2D fallback besides WebGL, and the app must offer a
    documented workaround (`WEBKIT_DISABLE_DMABUF_RENDERER`).
  - The data volume is small (4 channels × 4096 samples × 8 bit = 16 KiB per frame), so IPC should not be the limit at
    the tens of frames per second this hardware can deliver; this is an assumption to be validated in the M0 spike.

## 5. Hardware capabilities and limits (6254BC)

Source: `hantek6xx4-lib` `docs/protocol.md` (sections 1, 6, 7, 9, 10), the manufacturer datasheet, and measurements on
one unit. Statuses mirror the driver's verification table. The application must not promise more than is stated here.

### 5.1 Capabilities

| Item | Value | Status |
|---|---|---|
| Channels | 4, independently enabled | Verified (3-channel: data layout only) |
| Bandwidth | 250 MHz (analog), optional bandwidth limit | Vendor spec; limit relay verified |
| ADC | 8-bit, HMCAD1511 | Hardware |
| Sample rate | 1 GS/s with 1 channel, 500 MS/s with 2, 250 MS/s with 3 or 4 enabled | Verified (rise-time measurements) |
| Vertical | 12 ranges, 2 mV to 10 V per division, AC/DC coupling, 1 MΩ ‖ 25 pF | Verified |
| Time base | 36 steps, 2 ns to 1000 s per division, 1-2-5, 250 samples per division | Indexes 0-22, 24, 25, 27, 30 measured |
| Triggers | Edge (level, slope), pulse width (>, <, =, ≠), auto and normal sweep, horizontal position | Edge, pulse verified |
| Streaming | Scan/roll for time bases of 500 ms/div and slower | Verified |
| Volts | Conversion with factory calibration, zero and scale model | Verified 50 mV to 1 V/div on one unit |
| Hotplug | Connect/disconnect events with 2 s settle time | Verified |

### 5.2 Limits that drive product decisions

| Limit | Consequence for the application |
|---|---|
| **Capture memory 64 KiB, shared by enabled channels.** Documented, verified window is 4096 samples per channel; other lengths are untested in the driver. | No deep-memory features in v1. The record is about 16 divisions at 250 samples/div (10 on screen), so pan/zoom over the extra record is possible. Record length is fixed, not user-selectable, until the driver verifies more. |
| **Capture is frame based, not streaming**, at time bases faster than 500 ms/div. Each frame needs arm, wait, state poll (about 20 ms recommended) and USB readback. | Waveform update rate is modest (order of tens per second at best, not thousands). Do not offer "fast persistence" claims; persistence builds from successive frames. The UI must show dead time honestly. Measure and publish the real update rate. |
| **Sample rate falls as channels are enabled** (1 / 2 / 4 ns interval). | The UI shows the effective sample rate and warns when it is too low for the displayed signal (aliasing risk, 250 MS/s with 250 MHz bandwidth). |
| **Time bases 2 ns to 500 ns share the ADC interval**, so samples per division fall below 250 there. | Display uses sin(x)/x interpolation by default at fast time bases. |
| **Trigger position accuracy about ±6 samples**, depends on channel count and time base. | Do not display trigger-relative time to sub-sample claims; document the uncertainty. Time cursors are relative to samples, not the trigger. |
| **Volts accuracy: up to 2 to 3 counts (about 5 % of a division) at extreme vertical positions**, ranges below 50 mV and above 1 V/div unchecked, one unit measured. | Show an accuracy note in the measurement panel and "unverified range" marker for ranges outside 50 mV to 1 V/div. Provide a user zero-offset calibration using a shorted input. |
| **An open input is not zero** (picks up neighbouring channels). | Zero-calibration wizard instructs to short the input. Never use an open input as reference. |
| **Video trigger untested**, serial triggers, peak detect and alternate trigger not implemented. | Video trigger hidden or labelled experimental. |
| **Re-enumeration**: after plug-in the device disappears about 1.2 s later and returns with a new address; opening in that window can fail or wedge the device until replug. | Use `DeviceMonitor` and never open devices by hand outside it. Connection state machine (section 7.1). |
| **Roll streaming cannot detect lost data from the pointer**; the driver infers overruns from elapsed time. | Roll view must show a data-gap marker when the driver reports an overrun. |
| **Some control requests stall** (`0xEA`, `0xE5`, `0xED`, `0xF4`), harmless. | Application never issues raw requests; goes through the driver API. |
| **Linux only tested.** udev rule grants the `plugdev` group. | Install docs and package must ship the udev rule. First-run check shows a clear message when permission is denied. |
| **No signal generator on the BC.** | No generator UI; the 1 kHz calibrator on the probe terminal is the only built-in reference signal. |

## 6. Functional requirements

Priority: **M** must (v1.0), **S** should (v1.x), **C** could (later).

### 6.1 Device management

| ID | Requirement | Pri |
|---|---|---|
| DEV-1 | Detect connected scopes and show connection state (disconnected, connecting, ready, acquiring, error). | M |
| DEV-2 | Handle plug and unplug at any moment, including mid-capture and during the power-up re-enumeration, without crash or hang. | M |
| DEV-3 | Reconnect automatically after replug and restore the previous setup. | S |
| DEV-4 | Show device identification (model, firmware, FPGA version, calibration status). | M |
| DEV-5 | Device layer behind a trait so another model can be added without touching UI code. A simulated device for demos and tests ships in v1. | M |
| DEV-6 | Guided help for missing USB permissions (udev rule). | S |

### 6.2 Acquisition control

| ID | Requirement | Pri |
|---|---|---|
| ACQ-1 | Run/Stop and Single. | M |
| ACQ-2 | Trigger sweep auto and normal; force trigger button. | M |
| ACQ-3 | Roll mode at 500 ms/div and slower, with the time axis moving. | M |
| ACQ-4 | Show acquisition status: armed, triggered, auto-triggered, waiting, capture rate. | M |
| ACQ-5 | Acquisition runs on its own thread; the UI never blocks on USB. Stop takes effect within one frame time or 250 ms, whichever is larger. | M |
| ACQ-6 | Auto-setup: pick vertical range and time base from a short capture. | S |
| ACQ-7 | Acquisition counter and averaging over N frames (reduces noise on an 8-bit ADC). | S |
| ACQ-8 | Hi-res/box-car filtering for slow time bases. | C |

### 6.3 Vertical, horizontal, trigger

| ID | Requirement | Pri |
|---|---|---|
| VER-1 | Per channel: enable, volts/div (12 ranges), position, AC/DC, bandwidth limit, invert, colour, label. | M |
| VER-2 | Probe attenuation factor (1×, 10×, 100×, custom) applied to scale and labels. | M |
| VER-3 | Zero-offset calibration wizard (short the input) stored per range and channel. | S |
| HOR-1 | Time/div (36 steps) and horizontal trigger position (percent) with trigger marker. | M |
| HOR-2 | Zoom and pan inside the record, with a record overview bar. | S |
| TRG-1 | Edge trigger: source, level (draggable marker), slope. | M |
| TRG-2 | Pulse-width trigger: polarity, condition (>, <, =, ≠), width in time units. | S |
| TRG-3 | Video trigger, enabled only after hardware verification. | C |
| TRG-4 | Trigger hold-off (software, if the hardware cannot do it). | C |

### 6.4 Display

| ID | Requirement | Pri |
|---|---|---|
| DSP-1 | Yt display with 10 × 8 graticule, scale labels per channel, readable at 1080p and HiDPI. | M |
| DSP-2 | Interpolation: sin(x)/x, linear, dots (sin(x)/x default at fast time bases). | M |
| DSP-3 | Min/max decimation so no glitch is lost when zoomed out. | M |
| DSP-4 | XY mode (channel pairs). | S |
| DSP-5 | Persistence with adjustable decay and colour grading. | S |
| DSP-6 | Dark and light themes; colour-blind-safe default channel colours. | S |
| DSP-7 | Waveform display redraws at the display rate (target 60 fps) without dropping frames while acquisition is running on a typical laptop, in the Tauri WebView. | M |
| DSP-8 | Separate zoom window (second view of the same data, a second Tauri window). | C |
| DSP-9 | Renderer falls back from WebGL to Canvas 2D when WebGL is unavailable or unstable (WebKitGTK issues). | M |

### 6.5 Measurements and analysis

| ID | Requirement | Pri |
|---|---|---|
| MEA-1 | Cursors: two vertical (time), two horizontal (volts), delta and 1/Δt readout. | M |
| MEA-2 | Automatic measurements per channel: Vpp, Vmax, Vmin, Vmean, Vrms, frequency, period, positive/negative width, duty cycle, rise time, fall time. | M |
| MEA-3 | Statistics for each measurement (current, min, mean, max, sigma over N frames). | S |
| MEA-4 | Measurement validity: measurement shows a dash and reason when the signal does not fit (not enough periods, clipped). | M |
| MEA-5 | FFT with window choice (rectangular, Hann, Hamming, Blackman), dBV or linear, peak marker. | S |
| MEA-6 | Math channels: A+B, A−B, A×B, invert; later user formulas. | S |
| MEA-7 | Reference waveforms (store, show, clear). | S |
| MEA-8 | Phase between channels (XY and time based). | C |
| MEA-9 | Mask / limit testing. | C |

### 6.6 Data and settings

| ID | Requirement | Pri |
|---|---|---|
| DAT-1 | Export the current frame to CSV (volts, time, selected channels) and raw binary. | M |
| DAT-2 | Screenshot to PNG with the graticule and measurements. | M |
| DAT-3 | Save and load setups (file) plus automatic restore at start. | M |
| DAT-4 | Save and load waveform captures and view them offline without hardware. | S |
| DAT-5 | Record roll-mode data to a file continuously. | S |
| DAT-6 | Export to common formats (Sigrok session, WAV for audio-range signals). | C |

### 6.7 Application

| ID | Requirement | Pri |
|---|---|---|
| APP-1 | Keyboard shortcuts for run/stop, single, auto-setup, cursors, screenshot. | S |
| APP-2 | Tooltips with units and ranges on every control; first-run hint overlay. | S |
| APP-3 | Built-in calibrator demo ("connect probe to the 1 kHz CAL terminal") to prove the setup works. | S |
| APP-4 | Packaging with the Tauri bundler (AppImage and .deb first, Flatpak later), with the udev rule and WebKitGTK dependency handled. | S |
| APP-5 | CLI/headless capture mode, scripts that reuse the same acquisition engine. | C |
| APP-6 | SCPI-style remote control. | C |

## 7. User stories

Format: *As a [persona], I want [capability] so that [benefit]*, with acceptance criteria. IDs are traced to the
requirements in section 6.

### 7.1 Getting started

**US-01 First connection** (DEV-1, DEV-4, DEV-6, APP-3)
As a hobbyist, I want to plug in my scope and see it recognised so that I know it works before touching a probe.
- Within 5 s after the scope settles, the status shows "Ready" with model and firmware.
- If USB permissions are missing, the message names the udev rule and the command to install it.
- A "connect probe to CAL" hint shows a 1 kHz square wave measured as about 1 kHz.

**US-02 Quick look** (ACQ-6, ACQ-1)
As a hobbyist, I want to press Auto-setup so that an unknown signal shows up stable on screen without tuning controls.
- After Auto-setup a periodic signal from 10 Hz to 10 MHz shows 2 to 6 periods with 3 to 7 vertical divisions of amplitude.
- A signal-less input shows a flat line at zero and a "no signal" note.

**US-03 Unplug and replug** (DEV-2, DEV-3)
As any user, I want the app to survive me unplugging the scope so that I do not lose my setup or have to restart.
- Unplug while running: within 1 s the display freezes on the last frame with "Disconnected" and no crash.
- Replug: after the settle time, the app reconnects and restores the previous setup without user action.
- Repeat 20 times in a stress test without a hang.

### 7.2 Looking at signals

**US-04 Set vertical scale** (VER-1, VER-2)
As an embedded developer, I want to set volts/div, position, coupling and probe factor per channel so that the screen
reads in real signal units with a 10× probe.
- A 10× probe setting multiplies the displayed scale and labels by 10.
- Each channel has its own colour, label and enable switch; disabled channels do not cost sample rate.

**US-05 Understand the sample rate** (Section 5.2)
As a student, I want the app to show the effective sample rate and warn me about aliasing so that I do not misread a
signal.
- The status bar shows sample rate and record length at all times; they update when channels are enabled or disabled.
- If the displayed signal frequency exceeds a fifth of the sample rate (per section 4.2) a visible warning appears.

**US-06 Smooth at fast time bases** (DSP-2)
As an embedded developer, I want sin(x)/x interpolation so that a fast edge looks like an edge and not like a staircase.
- At 2 ns/div to 100 ns/div the default is sin(x)/x; the user can switch to linear or dots.
- A 1 kHz calibrator edge is displayed with a visible rise time consistent with the 240 to 256 ns measured by the driver.

**US-07 Catch a glitch** (DSP-3, DSP-5)
As an embedded developer, I want persistence so that an occasional glitch stays visible among normal cycles.
- With persistence on, successive frames accumulate with an adjustable decay (off, 0.5 s, 1 s, 2 s, infinite).
- Min/max decimation shows a one-sample glitch when the whole record is drawn on a narrower screen.

**US-08 Slow signals** (ACQ-3, DAT-5)
As a hobbyist, I want a rolling display at slow time bases so that I can watch a temperature sensor or battery voltage drift.
- At 500 ms/div and slower the trace scrolls continuously.
- If the host falls behind and data is lost, a visible gap marker is drawn rather than a silent splice.
- Optional record-to-file continues for hours without growing memory.

### 7.3 Triggering

**US-09 Edge trigger** (TRG-1, ACQ-2, ACQ-4)
As a hobbyist, I want to drag the trigger level marker so that a repeating signal holds still.
- The trigger marker shows the level and slope; status shows Armed, Triggered, or Auto.
- In normal sweep without a trigger event the last frame stays and status says "Waiting for trigger".
- Force-trigger captures one frame immediately.

**US-10 Single shot** (ACQ-1, HOR-1)
As an embedded developer, I want a single-shot capture so that I can catch a one-time event such as a power-up.
- Arms once, stops after one trigger, and shows the pre-trigger part according to the horizontal position (10 %, 50 %, 90 %).
- The trigger edge appears within ±8 samples of the selected position (the driver measured about ±6).

**US-11 Trigger on a pulse width** (TRG-2)
As an embedded developer, I want to trigger on a pulse longer or shorter than a time so that I can find a runt or stuck signal.
- Selecting pulse trigger offers polarity, condition (>, <, =, ≠) and width with units.
- Pulses of 500 µs verified with the hardware: the app triggers only on matching pulses.

### 7.4 Measuring

**US-12 Cursors** (MEA-1)
As a student, I want to place cursors so that I can read time, voltage and frequency differences directly.
- Two vertical and two horizontal cursors, draggable with the mouse, readout of values, Δ and 1/Δt.
- Values follow the channel selected and update when scale changes.

**US-13 Automatic measurements** (MEA-2, MEA-3, MEA-4)
As an embedded developer, I want a measurement panel so that I get frequency, Vpp, rise time and duty cycle without a pencil.
- At least eight measurements can be pinned per channel.
- The 1 kHz calibrator reads 1.000 kHz (±0.01 %) and about 1.9 V peak-to-peak, matching the driver's notes.
- If the measurement cannot be made (fewer than one period, clipped signal), it shows "–" and a reason.
- Statistics (min/mean/max) over the last N frames can be shown.

**US-14 Trust the numbers** (Section 5.2, VER-3)
As a test engineer, I want to know the accuracy limits so that I do not report a value the hardware cannot support.
- Measurements display an uncertainty hint for ranges outside the verified region (50 mV to 1 V/div).
- A zero-calibration wizard shows how to short the input and stores the offset per channel and range.
- The documentation contains an "accuracy" page derived from the driver's verification table.

**US-15 Spectrum** (MEA-5)
As a student, I want an FFT view so that I can see harmonics of a square wave and noise on a supply rail.
- FFT shows dBV or linear scale, a selectable window, and a peak marker with frequency and level.
- Frequency axis labelled with the correct bin spacing (sample rate / record length).
- A 1 kHz square wave shows odd harmonics at the expected frequencies.

**US-16 Compare channels** (MEA-6, DSP-4)
As a student, I want math channels and XY mode so that I can see A−B, A×B and Lissajous figures.
- Math traces behave like channels (scale, position, measurements).
- XY mode plots channel X versus channel Y with the same scale labels.

### 7.5 Data

**US-17 Export** (DAT-1, DAT-2)
As an embedded developer, I want to export what I see so that I can attach it to a bug report or analyse it elsewhere.
- CSV contains time and volts per selected channel and metadata (time base, volts/div, probe, sample interval, timestamp).
- PNG screenshot includes the graticule, scales and measurements.

**US-18 Resume work** (DAT-3, DAT-4)
As any user, I want to save and reload setups and captures so that I can pick up where I left off or inspect a capture without the scope.
- The last setup restores on start.
- A saved capture opens in offline mode with the same cursors and measurements, and a clear "offline" indicator.

**US-19 Automation** (APP-5)
As a test user, I want to capture from a script so that I can repeat a measurement overnight.
- A CLI command captures N frames with a given setup and writes files; exit codes distinguish no device, timeout and success.

## 8. Non-functional requirements and design considerations

### 8.1 Architecture

The architecture, design rules, repository layout, state management and testing approach are in
[architecture.md](architecture.md). In short: a Rust backend organised as decoupled feature modules is the source of
truth and does all processing; a TypeScript frontend built on a design system only displays and sends intents; the
hardware sits behind a `ScopeDevice` trait with the `hantek6xx4-lib` adapter and a simulator.

### 8.2 Technology (proposed, to confirm before implementation)

| Choice | Proposal | Reason |
|---|---|---|
| Language | Rust | Same language as the driver; no FFI boundary; safe threading. |
| App shell | Tauri 2 (decided) | Rust backend shares the language with the driver; small bundles; web UI makes layout, theming and accessibility easier. |
| Frontend | TypeScript with Svelte 5 and Vite (decided) | Light runtime so most of the frame budget goes to drawing the waveform; snippets provide slot-style composition. |
| Waveform rendering | Custom WebGL2 renderer (min/max per pixel column) with Canvas 2D fallback; UI widgets in DOM/SVG | Existing charting libraries rarely handle persistence and per-column decimation at scope update rates; a small custom renderer is expected. A canvas-based library may be used for the FFT and secondary plots. |
| Concurrency | OS threads and channels (crossbeam or std) | Blocking USB belongs on its own thread. |
| Persistence | TOML or JSON for setups, CSV/binary for data | Human-readable and diffable. |
| Packaging | Tauri bundler: AppImage and .deb first; Flatpak later | Native to Tauri; the WebKitGTK runtime comes from the system or the Flatpak runtime. |

A short spike (milestone M0) validates, on the real Linux target (including an NVIDIA machine): Channel throughput with
4 channels × 4096 samples at 60 messages per second, WebGL versus Canvas 2D rendering at 60 fps with persistence, and
end-to-end latency from driver return to pixels, and the roll-mode stream.

### 8.3 Performance and quality targets

| Target | Value |
|---|---|
| UI frame rate | 60 fps while acquiring, no frame over 33 ms on a 2020-era laptop with an integrated GPU, in the WebKitGTK WebView |
| Capture-to-screen latency | Under 100 ms for a frame once the driver returns it |
| Stop latency | Under 250 ms |
| Idle CPU (stopped) | Under 2 % of one core |
| Memory in roll recording | Constant (ring buffer in Rust, data streamed to disk; the frontend keeps only the visible window). Measured after the ring is full: Rust RSS grows at most 2 MiB per hour, WebKit process at most 10 MiB per hour (spike #4 baseline; the Rust rule is a condition on the roll story until its cause is attributed) |
| Start-up to ready scope | Under 5 s after the 2 s settle time |
| Crash-free | No panic on unplug, malformed data or driver errors; every driver `Error` maps to a UI state |
| Test coverage | Core logic (measurements, FFT, interpolation, state machine) unit tested; UI smoke-tested with the simulator; hardware checklist run before releases |

### 8.4 Usability principles

- Front-panel mental model: Vertical, Horizontal, Trigger, Run control, in the same grouping as bench scopes.
- Numbers always have units; time and voltage use engineering notation with sensible rounding (2 ns, 500 µs, 1.91 V).
- One-gesture operations: drag the trigger marker, scroll to zoom time, shift-scroll for volts/div, double-click to reset.
- State is visible: running, stopped, armed, triggered, auto, disconnected. No silent failures.
- No modal dialogs during acquisition.
- Accessibility: scalable fonts, colour-blind-safe palette, light and dark themes, keyboard operation of main controls.

### 8.5 Platform and licensing

- Linux first. Windows and macOS follow the driver's `vendored` libusb build path; they need testing and signed
  packaging, and are a post-v1 goal.
- Licence compatible with the driver (MIT). Hantek is mentioned only to describe compatibility; no vendor SDK code or
  trademarks are shipped.

## 9. Roadmap

| Milestone | Content | Exit criterion |
|---|---|---|
| **M0 Spike** | Tauri + Svelte project skeleton, Channel throughput test, WebGL and Canvas 2D waveform renderers, type-generation tool choice, device trait draft, simulator | 60 fps with 4 channels of 4096 samples on the Linux target, latency measured |
| **M1 Live view** | Device manager with hotplug, acquisition thread, Yt display, run/stop, vertical and horizontal controls, edge trigger | US-01, 03, 04, 09 pass on hardware |
| **M2 Measure** | Cursors, automatic measurements, interpolation, export CSV/PNG, setups | US-06, 12, 13, 17, 18 |
| **M3 Analyse** | Roll mode, persistence, FFT, math, XY, auto-setup, pulse trigger | US-02, 05, 07, 08, 10, 11, 15, 16 |
| **M4 Polish** | Calibration wizard, accuracy documentation, packaging, headless mode, themes | US-14, 19; AppImage and .deb builds |
| **Later** | Second device, SCPI server, mask test, deep-memory devices, Windows/macOS | |

## 10. Risks and open questions

| Risk | Impact | Mitigation |
|---|---|---|
| Update rate of the USB frame cycle is lower than users expect | Looks sluggish next to bench scopes | Measure early (M0/M1), show capture rate, set expectations in docs; ask the driver to expose lower-latency readback if it helps |
| Only one unit measured by the driver (volts, zero model) | Wrong voltages on other units | Verification markers, zero-cal wizard, collect reports |
| 4096-sample verified window versus the datasheet's 64 K | Smaller records than advertised | Report to the driver project; extend only after verification |
| Device wedges after an early open during re-enumeration | Needs replug, bad first impression | Open only through `DeviceMonitor`; stress test (US-03) |
| WebKitGTK rendering problems on some GPUs (NVIDIA, DMABUF) and uncertain IPC throughput on Linux | Stuttering or blank waveform view for some users | M0 spike on several machines; Canvas 2D fallback (DSP-9); documented environment workaround; downsample on the Rust side |
| Two languages and a bridge (Rust core, TypeScript UI) | Contract drift, harder debugging | Generated shared types; logic in Rust; frontend kept thin |
| Windows/macOS untested | Smaller audience | Linux first, plan CI builds, community testing |
| Scope creep toward a general lab suite | Delays v1 | Non-goals in section 2.2; device trait but no second device in v1 |

Open questions:

1. Repository layout (the name is decided: SweepScope; Cargo workspace with `core` and `device` crates plus the Tauri app, and a `frontend/` folder)?
2. Should software features that the hardware lacks (hold-off, averaging, hi-res) be shown with an indicator that they are computed in software?
3. Is the 10 × 8 graticule fixed, or should a user see the entire 16-division record by default?
4. Licensing of the app: MIT like the driver, or GPL if sigrok-derived code is ever reused?
5. Which driver changes are wanted: lower-latency readback, selectable record length, overrun reporting, video trigger verification?
6. What accuracy statements are acceptable to publish given one measured unit?

## 11. Sources

- [Hantek 6004BC series product page](https://hantek.com/products/detail?id=9163) and [6254BC datasheet](https://www.rcscomponents.kiev.ua/datasheets/Hantek6254BC.pdf): 250 MHz, 1 GS/s, 64K memory, 8 bit, 2 mV to 10 V/div, 1 MΩ ‖ 25 pF.
- [sigrok wiki: Hantek 6254BD](https://sigrok.org/wiki/Hantek_6254BD): sibling model reference.
- `hantek6xx4-lib` `docs/protocol.md` (local repo `/home/kr/workspace/hantek6xx4-lib`): hardware, acquisition, time base, trigger, volts, verification status.
- [PicoScope software](https://www.picotech.com/oscilloscope/6000/picoscope-6000-software), [Digilent WaveForms reference](https://digilent.com/reference/waveforms3/refmanual), [TiePie oscilloscope software](https://www.tiepie.com/oscilloscope-software/oscilloscope), [NI scope display window](https://ni.com/docs/en-US/bundle/ni-elvis-iii-using-instruments/page/display-window-scope.html): feature survey.
- [Digilent WaveForms in depth (element14 review)](https://community.element14.com/members-area/personalblogs/b/gough-lui-s-blog/posts/digilent-adp3450-in-depth-ch4-tour-of-digilent-waveforms): persistence and decoding limitations.
- [OpenHantek6022](https://github.com/msiegert/OpenHantek6022), [Hantek6022API](https://www.github.com/Ho-Ro/Hantek6022API), [sigrok](https://sourceforge.net/p/sigrok/): prior art for Hantek and open scope software.
- [Tektronix: evaluating oscilloscopes](https://www.tek.com/document/online/primer/xyzs-scopes/ch3/evaluating-oscilloscopes), [NI: digitizer for time-domain measurements](https://www.ni.com/en/shop/electronic-test-instrumentation/oscilloscopes/using-a-digitizer-for-time-domain-measurements.html): sin(x)/x, bandwidth and sample-rate rules, waveform capture rate.
- [Slint real-time plotting experiments](https://gh.hhj520.top/okhsunrog/slint_realtime_plotting_experiments): GPU min/max per pixel column technique.
- [Tauri: Linux graphics issues](https://v2.tauri.app/develop/debug/linux-graphics/): WebKitGTK/DMABUF problems and WebGL fallback advice.
- [Tauri IPC performance discussion](https://github.com/orgs/tauri-apps/discussions/11915), [Tauri channel source](https://docs.rs/crate/tauri/2.8.4/source/src/ipc/channel.rs): Channel-based binary streaming and informal throughput figures.
