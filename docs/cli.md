# Command line capture

`sweepscope-cli` captures frames from a setup file without the window. It does not link Tauri or WebKitGTK, so it runs on a machine without a display (ADR 0022).

```
sweepscope-cli capture --setup FILE [--count N] [--out DIR] [--format csv|raw]
                       [--timeout S] [--device-wait S] [--simulator] [--config-dir DIR]
```

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

Seconds may have decimals (`--timeout 0.5`). `SWEEPSCOPE_NO_HARDWARE=1` turns hardware discovery off, as in the application.

## What a run does

1. Refuses to start when any of the `N` target files already exists. Nothing is written and no file is replaced.
2. Waits for the scope to be ready.
3. Loads the setup file and starts the scope.
4. Writes each frame as it arrives to `capture-0001.csv`, `capture-0002.csv`, ... (`.bin` for raw; the first raw frame also writes `capture.format.txt`, the layout of the raw format).
5. Stops the scope. Frames already written stay on disk when a later one fails.

Each file holds exactly what File, Export (CSV or raw) writes in the application, including the timestamp of the moment the frame reached the program. Every frame of the run is written, not only the newest.

## Stored files are read only

The CLI reads the application's stored zero offsets, so a calibrated unit gives calibrated frames (`chN_zero_offset_applied` in CSV, `calibrated` in the raw header). It never writes to the config folder: the last setup, the settings and the calibration are unchanged after a run.

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
