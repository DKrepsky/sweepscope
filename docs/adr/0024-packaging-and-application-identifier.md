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
5. A manual or tag-only CI `bundle` job builds both packages and uploads artifacts; it publishes nothing.

## Alternatives considered
`cargo-deb` plus a hand-made AppImage (two tools, two sources of truth); installing the rule from the app with elevation (privileged action from the UI); keeping the placeholder (later data migration); Flatpak first (USB portal story).

## Consequences
Developers' old config folder is orphaned once. The AppImage tooling is downloaded at build time (supply-chain note for the release story). Packages need glibc 2.39 or newer.
