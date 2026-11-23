# M1 hardware checklist (run by the human on the Hantek 6254BC)

Closes the M1 exit criterion (PRD 9): US-01, US-03, US-04, US-09 on hardware. Spec: [implementation-spec.md](implementation-spec.md).
Agents never touch the scope; they write the build and the instructions, the human runs this list and reports pass or fail per item.
Record results as pass or fail with a word of context; keep measured figures (times, rates) in your own notes (ADR 0008), the QA report carries outcomes.

## Before you start
| # | Step |
|---|---|
| P-1 | `sudo apt install libusb-1.0-0-dev` (needed to build; D8). Build the app: `npm run build` or run `npm run dev`. Use `RUST_LOG=debug` to see the driver and adapter logs; keep the terminal open. |
| P-2 | Scope powered from USB, probes attached, CH1 probe on the `CAL` terminal (1 kHz square, about 1.9 V peak to peak). A second signal source is optional. |
| P-3 | Close the vendor software and any other program using the scope. |
| P-4 | Start with the udev rule **not installed** for H-05, then install it (H-06). If it is already installed, remove `/etc/udev/rules.d/99-hantek6xx4.rules` and run `sudo udevadm control --reload && sudo udevadm trigger` first. |

## US-01 first connection (stories #5, #6)
| ID | Do | Expect |
|---|---|---|
| H-01 | Start the app with the scope **unplugged**. | Status `Disconnected`, hint "Connect a scope or use the simulator", `Use simulator` action. |
| H-02 | Press `Use simulator`, then plug the real scope in. | `Ready` with `Simulated` and a live trace first; after the settle time the real scope takes over and the simulator session ends. |
| H-03 | Unplug everything, restart the app, plug the scope in with the rule installed. Start a stopwatch at plug-in. | About 2 s of `Disconnected` (settle), then `Connecting`, then `Ready` with model (`Hantek 6xx4 ...`), FW, FPGA and calibration. Ready within 5 s after the settle. |
| H-04 | Unplug and plug the scope **20 times** in a row (wait for `Ready` each time, a few seconds apart). | Every time `Ready`; no wedge (no replug needed beyond the one you did), no error banner left behind, no hang. Note the count reached. |
| H-05 | With the rule not installed, plug the scope in. | Error Banner naming the udev rule and showing the install command; app does not crash. |
| H-06 | Press `Copy command`, paste it in a terminal, run it, replug the scope, press `Retry` if it did not connect by itself. | Copy confirms; after the rule the app connects **without restarting**. If it needed `usermod -aG plugdev`, note it (D7). |
| H-07 | With the scope ready, read the connection item and tooltip. | Model, firmware, FPGA version and calibration status are all present. |
| H-08 | Start the app with the scope already plugged in and settled. | Goes to `Ready` without the 2 s wait being a problem; no error. |

## US-04 vertical (story #9)
| ID | Do | Expect |
|---|---|---|
| H-20 | CH1 on `CAL`, 200 us/div (default), run. | A stable-looking square wave, frame updates visible. |
| H-21 | Step CH1 volts/div through all 12 ranges (wheel or stepper). | Labels update; 12 ranges reachable; the screen scale follows. Ranges outside 50 mV to 1 V show the unverified marker. |
| H-22 | Set probe to 10x with a real 10x probe on `CAL`. | Scale and labels read 10 times larger; the trace does not change height. The CAL square reads about 19 V on the label scale. |
| H-23 | Toggle AC and DC on CH1. | The display reflects it (the square's DC offset disappears with AC on a 1 kHz square; low frequencies sag). |
| H-24 | Toggle BW limit, invert. | BW: noise or edge changes; invert: trace mirrors about its ground marker. |
| H-25 | Enable CH2, CH3, CH4 one by one and watch the status bar. | Sample rate `Fs` goes 1, 2, then 250 MS/s steps at fast time bases (set 20 ns/div to see it); disabled channels are not drawn and not captured. |
| H-26 | Move the position of CH1 to +2 div, -2 div; try +4 div on 200 mV/div and on 500 mV/div. | The ground marker and trace move together; on 50 mV, 500 mV and 5 V the marker stops near +-1.6 div with a tooltip. |
| H-27 | Check each channel colour, label and enable switch. | Distinct per channel. |

## US-09 edge trigger and horizontal (stories #10, #11)
| ID | Do | Expect |
|---|---|---|
| H-30 | Auto sweep, drag the trigger level marker across the CAL square, release. | Display becomes stable at the level; the marker tag shows level and slope; `Trig'd` chip. Note how long the trace takes to respond after release (each change reconfigures the scope). |
| H-31 | Drag the level above the signal, sweep Auto. | Chip `Auto`, display keeps updating (free running). |
| H-32 | Switch to Normal, level above the signal. | The last frame stays, chip `Waiting for trigger`, `Armed` between attempts. |
| H-33 | Press `Force` in that state. | One frame appears; note the delay (re-configure). |
| H-34 | Change source to CH2 with a signal on it (or only check the control), change slope. | Trigger follows on the next frame; only enabled channels can be the source. |
| H-35 | Time/div: step through all 25 frame-mode steps (2 ns to 200 ms). | Axis labels update; the one unmeasured frame-mode step (100 ms/div) shows the unverified marker; the 11 roll steps are disabled with the reason (D1). |
| H-36 | Trigger position at 10 %, 50 %, 90 % of the screen (drag or stepper) on the CAL edge at 200 us/div. | The rising edge sits within about +-0.2 division of the marker (+-6 samples is 5 us, 0.02 div, so any larger offset is a defect). |
| H-37 | At 2 ns/div and 5 ns/div with the trigger position at 50 %. | Records which part of the edge shows; offset of several divisions is expected from the +-6 sample accuracy (PRD 5.2); note it. |
| H-38 | Read sample rate and record length in the status bar for three time bases. | They update with the time base and channel count and match the table (4 ns interval at 4 channels and 1 us/div, 800 ns at 200 us/div). |

## US-03 unplug and replug (story #12), and run control (story #7)
| ID | Do | Expect |
|---|---|---|
| H-10 | Run, then press Stop. | Toggle reads `Run`, chip `Stop`, the last frame stays. Note the delay until you can press Run again and see `Armed`. |
| H-11 | Running at 200 us/div, pull the USB cable. Stopwatch. | Within 1 s the trace freezes (dimmed), chip and banner `Disconnected`, controls disabled but readable, no crash. |
| H-12 | Plug it back in. | After the settle time the app reconnects, restores the previous channels, time base and trigger (check volts/div, time/div, level, slope, sweep, probe) and, per D4, resumes running. No user action. |
| H-13 | Repeat unplug and replug **20 times** while running; once unplug during the first second after plug-in (power-up re-enumeration). | No hang, no crash, no wedge (device still enumerates and opens), memory and thread count flat in `top` or `ls /proc/<pid>/task | wc -l`. |
| H-14 | While running at 200 us/div, read capture rate and dead time in the status bar for 1 and 4 channels, and at 2 ms/div. | Both are live numbers that change with the settings. Write down the capture rate for each (replaces the simulator's assumed 15 ms readback and 5 ms re-arm). |
| H-15 | Drag windows and open Settings while running. | UI stays responsive, never blocks on USB. |
| H-16 | Pull the cable while the app is Stopped, replug. | Reconnects, setup restored, stays stopped. |

## Display (story #8)
| ID | Do | Expect |
|---|---|---|
| H-40 | Run at 200 us/div with CAL, look at the graticule. | 10 x 8 graticule, per channel scale chips readable at your screen scale. |
| H-41 | Toggle the frame-budget probe (dev build, WebKit inspector console) while running for two minutes; repeat three times. | Summary printed every 10 s; note whether any animation frame interval exceeds 33 ms and when (spike #2 left long frames unexplained). |
| H-42 | With `WEBKIT_DISABLE_DMABUF_RENDERER=1` and without, if the view is blank or unstable. | Records which works. Only needed if you see a problem. |

## Timing to record (feeds PRD 5.2 and the simulator constants)
Time from a trigger-level release to the next new frame (H-30), time from `Force` to the frame (H-33), time from Stop to being able to Run again (H-10),
capture rate at 200 us/div with 1 and 4 channels and at 2 ms/div (H-14), time from plug-in to `Ready` (H-03). Keep the numbers in your notes.

## Result
Pass when every item passes or has a recorded, accepted deviation. Failures go to the PM as bugs with the item id. The `#[ignore]` smoke test of WP4
can be run first as a quick check: `cargo test -p sweepscope-device -- --ignored hardware_`.
