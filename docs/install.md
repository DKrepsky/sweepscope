# Install SweepScope on Linux

SweepScope is built for Linux (glibc 2.39 or newer, for example Ubuntu 24.04). Two packages are built with
`npm run build -- --bundles deb,appimage`; they land in `target/release/bundle/`.

## Debian package (recommended)

```sh
sudo apt install ./SweepScope_<version>_amd64.deb
```

The package:

- declares its dependencies (WebKitGTK 4.1, GTK 3, libusb 1.0), which `apt` installs;
- installs the udev rule `/lib/udev/rules.d/99-hantek6xx4.rules` and reloads the rules, so the scope can be opened
  without root as soon as it is plugged in (unplug and plug it again if it was connected during the install);
- installs the program as `/usr/bin/sweepscope`, the desktop entry and the icons (16 to 512 px).

Removing the package removes the rule and reloads the rules.

## AppImage

```sh
chmod +x SweepScope_<version>_amd64.AppImage
./SweepScope_<version>_amd64.AppImage
```

An AppImage cannot install a udev rule. Install it once by hand, as described in
[usb-permissions.md](usb-permissions.md); the rule file is in the repository at `packaging/udev/`.

## Command line

There is one executable: `sweepscope` opens the window, `sweepscope capture ...` captures without it, and
`sweepscope --help` and `sweepscope --version` print and exit. Any other argument opens the window, also a mistyped
command. Command line mode needs no display, but the WebKitGTK and GTK libraries must still be installed. Options,
output and exit codes are in [cli.md](cli.md).

How to get the `sweepscope` command:

| Installed from | Command |
|---|---|
| `.deb` | `sweepscope` is already on your `PATH`. |
| AppImage | `mkdir -p ~/.local/bin && install -m 755 SweepScope_<version>_amd64.AppImage ~/.local/bin/sweepscope` (`~/.local/bin` must be on your `PATH`), or run the `.AppImage` file itself. |
| Source build | `npm install && npm --prefix frontend install && npm run build -- --no-bundle`, then `sudo install -m 755 target/release/sweepscope /usr/local/bin/sweepscope` (prerequisites in the [README](../README.md#prerequisites)). |

With the AppImage, give absolute paths and always `--out`: it does not look up relative paths in your current folder.

```sh
sweepscope capture --setup "$PWD/s.json" --count 3 --out "$PWD/cap"
```

Without FUSE, start it as `sweepscope --appimage-extract-and-run ...` (AppImage only).

## Where data is kept

Settings, setups and calibration live in the Tauri config folder of the application identifier
`io.github.dkrepsky.sweepscope` (`~/.config/io.github.dkrepsky.sweepscope/`). Earlier development builds used
`dev.sweepscope.app`; that folder is not migrated.

## Check an installed package

- `dpkg -L sweep-scope` lists the rule, desktop file and icons.
- `lsusb -d 04b5:6cde` lists the scope; see [usb-permissions.md](usb-permissions.md) if the app reports
  "Permission denied".
