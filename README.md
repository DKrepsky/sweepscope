# SweepScope

A desktop application for USB oscilloscopes, starting with the Hantek 6254BC. Rust backend, Svelte 5 + TypeScript
frontend, packaged with Tauri 2. Requirements are in [docs/prd.md](docs/prd.md), the design rules in
[docs/architecture.md](docs/architecture.md) and [docs/design.md](docs/design.md).

## Prerequisites

- Rust, installed with [rustup](https://rustup.rs). The toolchain version is pinned in `rust-toolchain.toml`.
- Node 24 and npm.
- WebKitGTK development packages for Tauri 2:
  - Ubuntu 24.04: `sudo apt install build-essential pkg-config libwebkit2gtk-4.1-dev libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`
  - Fedora: `sudo dnf install @development-tools pkgconf-pkg-config webkit2gtk4.1-devel libxdo-devel openssl-devel libappindicator-gtk3-devel librsvg2-devel`
- The libusb development package, used by the scope driver:
  - Ubuntu 24.04: `sudo apt install libusb-1.0-0-dev`
  - Fedora: `sudo dnf install libusb1-devel`
- USB access for the scope on Linux: install the udev rule, see [docs/usb-permissions.md](docs/usb-permissions.md).

## Running without a scope

`SWEEPSCOPE_SIMULATOR=1 npm run dev` starts the app with the built-in simulated oscilloscope. `RUST_LOG=debug` turns on
driver and adapter logs.

`SWEEPSCOPE_NO_HARDWARE=1` leaves out the hardware discovery completely, so the app never opens a connected scope. Use it
for tests and demos: `SWEEPSCOPE_NO_HARDWARE=1 SWEEPSCOPE_SIMULATOR=1 npm run dev`.

## Using the app

### Accuracy

Voltage readings are accurate to about 2 to 3 counts (about 5 % of a division at extreme positions). Ranges below 50 mV/div and above 1 V/div are unverified. Verified on one unit.

### Keyboard shortcuts

| Keys | Action |
|---|---|
| `C` | Add the four measurement cursors (two vertical, two horizontal); needs a waveform on screen |
| `Ctrl+S` | Save the setup (File menu) |
| `Ctrl+E` | Export the current frame as CSV (File menu, Export CSV) |
| `Ctrl+P` | Save a screenshot as PNG |

Shortcuts do nothing while a text field has focus. Arrow keys move the selected cursor by 1/250 of the screen, with Shift by ten steps; Delete removes it.

### File formats

- **CSV** (File, Export CSV): metadata as comment lines, then a header (`time_s,ch1_v`, one voltage column per enabled channel) and one row per sample, in seconds from the first sample of the record and in volts.
- **Raw frame** (File, Export raw): the converter codes as they were captured, in the self-describing `SSR1` format: `<name>.bin` plus a text description of the layout beside it, `<name>.format.txt` (same name without the `.bin` extension). See [docs/formats/frame-raw-v1.md](docs/formats/frame-raw-v1.md), which includes how to read it.
- **Setup** (File, Save setup and Load setup): a JSON file with `format` and `version` keys, the channel settings, time base and trigger in volts and seconds. Values the device cannot apply on load are kept at their current value and listed in a warning. A file from a newer version is refused with a message. The last setup is also restored at start; acquisition starts stopped.

## Troubleshooting

- Blank or stuttering window on Linux (often NVIDIA with WebKitGTK): start with `WEBKIT_DISABLE_DMABUF_RENDERER=1`, for
  example `WEBKIT_DISABLE_DMABUF_RENDERER=1 npm run dev`. See [docs/troubleshooting.md](docs/troubleshooting.md).
- The scope is not found or access is denied: [docs/usb-permissions.md](docs/usb-permissions.md).

## Commands

| Command | What it does |
|---|---|
| `npm install && npm --prefix frontend install` | Install the Tauri CLI and the frontend dependencies |
| `npm run dev` | Start the app with hot reload (`tauri dev`) |
| `npm run check` | Format, lint, type check and the architecture and test-name checks |
| `npm test` | Rust tests, frontend tests and the tests of the repository checks |
| `npm run build` | Build the release bundles (`tauri build`) |

`npm run check` and `npm test` are what CI runs, split into jobs (see `.github/workflows/ci.yml`). The repository checks
live in `tools/` and need no dependencies.
