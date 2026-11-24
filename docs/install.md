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
- installs the desktop entry and the icons (16 to 512 px).

Removing the package removes the rule and reloads the rules.

## AppImage

```sh
chmod +x SweepScope_<version>_amd64.AppImage
./SweepScope_<version>_amd64.AppImage
```

An AppImage cannot install a udev rule. Install it once by hand, as described in
[usb-permissions.md](usb-permissions.md); the rule file is in the repository at `packaging/udev/`.

## Where data is kept

Settings, setups and calibration live in the Tauri config folder of the application identifier
`io.github.dkrepsky.sweepscope` (`~/.config/io.github.dkrepsky.sweepscope/`). Earlier development builds used
`dev.sweepscope.app`; that folder is not migrated.

## Check an installed package

- `dpkg -L sweep-scope` lists the rule, desktop file and icons.
- `lsusb -d 04b5:6cde` lists the scope; see [usb-permissions.md](usb-permissions.md) if the app reports
  "Permission denied".
