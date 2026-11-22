# Troubleshooting

## Blank, black or stuttering window on Linux

SweepScope draws the waveform with Canvas 2D inside the Tauri webview (WebKitGTK). On some GPU and driver combinations,
most often NVIDIA, the WebKitGTK DMABUF renderer shows a blank window or drops frames.

Workaround: turn the DMABUF renderer off for the process.

```sh
WEBKIT_DISABLE_DMABUF_RENDERER=1 npm run dev
```

For an installed build, set the variable in the environment or in the desktop entry (`Exec=env
WEBKIT_DISABLE_DMABUF_RENDERER=1 sweepscope`). The status bar shows `Renderer Canvas 2D` when the app uses the
supported renderer. More background: [Tauri: Linux graphics issues](https://v2.tauri.app/develop/debug/linux-graphics/).

## The scope is not found or access is denied

Install the udev rule: [usb-permissions.md](usb-permissions.md).

## Keep the app away from a connected scope

`SWEEPSCOPE_NO_HARDWARE=1` skips the hardware discovery, so only the simulator is available:

```sh
SWEEPSCOPE_NO_HARDWARE=1 SWEEPSCOPE_SIMULATOR=1 npm run dev
```
