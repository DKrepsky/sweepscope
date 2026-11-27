# USB permissions on Linux

SweepScope talks to the oscilloscope over USB. On Linux a normal user may not open the device until a udev rule
grants access. Without the rule the app shows a banner "Permission denied" with the install command and a
`Retry` button.

## Install the rule

The `.deb` package installs the rule itself ([install.md](install.md)). For the AppImage or a source build, from the
repository root:

```sh
sudo cp packaging/udev/99-hantek6xx4.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Then unplug and plug the scope again, and press `Retry` in the app if the banner is still shown.

## What the rule does

`packaging/udev/99-hantek6xx4.rules` matches the Hantek 6xx4 series (vendor `04b5`, product `6cde`) and:

- sets the mode to `0660` and the group to `plugdev`;
- adds the `uaccess` tag, so the user logged in at the machine gets access without joining a group.

If your distribution has no `plugdev` group, `uaccess` is enough on a desktop session. On a headless or remote
session add yourself to a group of your own and change `GROUP` in the rule.

## Check that it works

- `lsusb -d 04b5:6cde` lists the scope.
- `ls -l /dev/bus/usb/<bus>/<device>` shows the group `plugdev` and mode `crw-rw----`, plus an ACL entry for you
  (`getfacl`) when `uaccess` applied.

## Trying the app without a scope

Start it with the built-in simulator:

```sh
SWEEPSCOPE_SIMULATOR=1 npm run dev
```

Without the variable the simulator stays off until it is chosen from the banner. `RUST_LOG=debug` prints the driver and
adapter logs.

To make sure a run never opens a connected scope, add `SWEEPSCOPE_NO_HARDWARE=1`.
