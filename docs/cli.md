# Command line capture

The `sweepscope` executable is both the application window and a command line tool. `sweepscope capture` records frames
from a setup file without opening the window (ADR 0022).

```
sweepscope capture --setup FILE [--count N] [--out DIR] [--format csv|raw]
                   [--timeout S] [--device-wait S] [--simulator] [--config-dir DIR]
```

## Window or command line

The first argument decides:

| First argument | Result |
|---|---|
| `capture` | Captures, then exits with an [exit code](#exit-codes). A mistake after `capture` is a usage error (exit 2). |
| `--help`, `-h` | Prints the usage text and exits 0. |
| `--version`, `-V` | Prints `sweepscope <version>` and exits 0. |
| none, or anything else | Opens the window. |

The words must match exactly. A mistyped command (`sweepscope captur ...`) or an option before `capture`
(`sweepscope --simulator capture ...`) opens the window. A script would then wait until the window is closed, or fail
where there is no display.

## Requirements

Command line mode needs no display: it runs over SSH, from cron or in CI. The libraries of the window are still needed,
because there is one executable: WebKitGTK 4.1, GTK 3 and libusb 1.0 must be installed, even only to capture. The
`.deb` declares them. The AppImage carries WebKitGTK and GTK itself and takes libusb from the system.

Reading a real scope needs the udev rule, see [usb-permissions.md](usb-permissions.md).

## Get the `sweepscope` command

### From the Debian package

```sh
sudo apt install ./SweepScope_<version>_amd64.deb
sweepscope --version
```

The package installs `/usr/bin/sweepscope` and the udev rule. Nothing else is needed.

### From the AppImage

The AppImage is the same executable and passes its arguments through. To call it `sweepscope`, put it under that name in
a folder on your `PATH`:

```sh
mkdir -p ~/.local/bin
install -m 755 SweepScope_<version>_amd64.AppImage ~/.local/bin/sweepscope
sweepscope --version
```

If `sweepscope` is not found, add `~/.local/bin` to `PATH` or log in again. You can also run the file where it is, as
`./SweepScope_<version>_amd64.AppImage capture ...`. An AppImage installs no udev rule; install it by hand.

Relative paths work as for the installed program: `--setup`, `--out`, `--config-dir` and the default output folder refer
to the folder you run the command in:

```sh
sweepscope capture --setup s.json --count 3 --out cap
```

Each start mounts the image with FUSE. On a machine without FUSE, put `--appimage-extract-and-run` first; the image is
then unpacked to a temporary folder that is removed afterwards. Relative paths work the same way:

```sh
sweepscope --appimage-extract-and-run capture --setup s.json --out cap
```

The AppImage keeps options starting with `--appimage-` for itself.

### From a source build

Install the build prerequisites listed in the [README](../README.md#prerequisites), then in the repository:

```sh
npm install && npm --prefix frontend install
npm run build -- --no-bundle
```

The executable is `target/release/sweepscope`. Run it from there, or install it:

```sh
sudo install -m 755 target/release/sweepscope /usr/local/bin/sweepscope
sweepscope --version
```

Use `npm run build`, not `cargo build --release`: only the former builds an executable whose window also works. Install
the udev rule by hand.

During development, without installing anything, run it through cargo:

```sh
cargo run -p sweepscope-app -- capture --simulator --setup s.json --out cap
```

`npm run dev` (`tauri dev`) is for the window; passing capture arguments through it is not supported.

## Options

| Option | Meaning | Default |
|---|---|---|
| `--setup FILE` | Setup file written by the application (menu File, Save setup). Required. | |
| `--count N` | Number of frames to write. | 1 |
| `--out DIR` | Output folder, created when missing. | current folder |
| `--format csv\|raw` | CSV table or raw binary (`SSR1`, see [frame-raw-v1.md](formats/frame-raw-v1.md)). | csv |
| `--timeout S` | Seconds to wait for each frame. | 10 |
| `--device-wait S` | Seconds to wait for the scope to become ready. The scope needs about 2 s to settle after it is opened, which counts here. | 8 |
| `--simulator` | Use the built-in simulator instead of hardware. | off |
| `--config-dir DIR` | Folder with the application's stored files (zero offsets). | `$XDG_CONFIG_HOME/io.github.dkrepsky.sweepscope`, else `~/.config/io.github.dkrepsky.sweepscope` |

Seconds may have decimals (`--timeout 0.5`). `SWEEPSCOPE_NO_HARDWARE=1` turns hardware discovery off, as in the
application.

## What a run does

1. Refuses to start when any of the `N` target files already exists. Nothing is written and no file is replaced.
2. Waits for the scope to be ready.
3. Loads the setup file and starts the scope.
4. Writes each frame as it arrives to `capture-0001.csv`, `capture-0002.csv`, ... (`.bin` for raw; the first raw frame
   also writes `capture.format.txt`, the layout of the raw format).
5. Stops the scope. Frames already written stay on disk when a later one fails.

Each file holds exactly what File, Export (CSV or raw) writes in the application, including the timestamp of the moment
the frame reached the program. Every frame of the run is written, not only the newest.

## Stored files are read only

The command line mode reads the application's stored zero offsets, so a calibrated unit gives calibrated frames
(`chN_zero_offset_applied` in CSV, `calibrated` in the raw header). It never writes to the config folder: the last
setup, the settings and the calibration are unchanged after a run.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | All frames written. |
| 1 | Failure: unreadable or invalid setup, an existing target file, a write error other than permission. |
| 2 | Usage: unknown option, missing or invalid value. The usage text is printed. |
| 10 | No scope became ready within `--device-wait`. |
| 11 | No frame within `--timeout`, for example a Normal sweep trigger that never fires. |
| 12 | Permission denied: the output folder is not writable, or the scope's USB permission is missing (install the udev rule). |

Messages go to standard error, the list of written files to standard output.

## Accuracy note

The position of the trigger within a frame is uncertain by about 6 samples, as in the application.
