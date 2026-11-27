# ADR 0024: Linux packaging with the Tauri bundler, udev rule in the .deb, identifier `io.github.dkrepsky.sweepscope`

Status: Accepted
Date: 2026-11-24
Issue: #37

## Context
Bundling is off and the identifier `dev.sweepscope.app` is a placeholder. The Tauri config folder derives from the identifier, so it decides where settings, setups and calibration live and cannot change later without orphaning data. Nothing is released yet.

## Decision
1. Identifier `io.github.dkrepsky.sweepscope`; no migration of the old folder.
2. Tauri bundler, targets `deb` and `appimage`, all generated icon sizes listed.
3. The .deb installs the udev rule, reloads rules after install and removal, and declares WebKitGTK 4.1, GTK 3 and libusb. The rule also carries `uaccess`, so no group is created.
4. The AppImage cannot install a rule; the install guide covers it.
5. `mainBinaryName` is `sweepscope` (stable binary name); the .deb package name is `sweep-scope`, derived by the bundler from the product name `SweepScope`.
6. A manual or tag-only CI `bundle` job builds both packages and uploads artifacts; it publishes nothing.

## Amendment 2026-11-25: one executable
7. The installed `sweepscope` is both the window and the CLI (`sweepscope capture ...`, `--help`, `--version`; any other command line opens the window; ADR 0022 items 7 and 8). The .deb and AppImage carry this one binary, and no other CLI binary exists. CLI mode needs the WebKitGTK and GTK packages the .deb depends on, but no display. The AppImage passes arguments through and mounts a FUSE image at each start. It also starts the program inside its own `usr` folder, so in CLI mode, when the executable and the working directory are both under `$APPDIR`, the application changes to `$PWD` (the caller's folder, set by the bundler's bash `AppRun` before the change, in mounted and extract-and-run starts alike; `OWD` is missing in the latter). The window path is not touched, because the bundled WebKit relies on that working directory. A change of the bundler's `AppRun` would need this rechecked. The `.desktop` entry passes no arguments.

## Alternatives considered
`cargo-deb` plus a hand-made AppImage (two tools, two sources of truth); installing the rule from the app with elevation (privileged action from the UI); keeping the placeholder (later data migration); Flatpak first (USB portal story).

## Consequences
Developers' old config folder is orphaned once. The AppImage tooling is downloaded at build time (supply-chain note for the release story). Packages need glibc 2.39 or newer.
