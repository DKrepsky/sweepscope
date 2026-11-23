# ADR 0016: Files: the backend picks paths and writes atomically; setups are acquisition state in a JSON file

Status: proposed
Date: 2026-11-23
Issue: #18, #19, #20; resolves architecture 10 items 3 and 5 for dialogs and setup files

## Context
M2 writes exports, screenshots and setups to places the user chooses and reads setups back. Architecture 4.5 rule 3 says the backend is the only
writer of files; architecture 10.5 asks for a review when dialogs or file-system plugins are added; the issues demand that a failed write names its
cause and leaves nothing half-written (#18 AC3, #19 AC3) and that a bad setup file leaves the current setup untouched (#20 AC4). The architecture
file tree says `settings` holds "setups later", but a setup is the state of `acquisition`, which owns and validates it (rule 4); `settings` stores
opaque blobs and cannot interpret a setup.

## Decision
1. **Kernel `files`** (small, no dependency): `FilePicker` trait (`pick_save(suggested_name, filter)`, `pick_open(filter)`, both `Result<Option<PathBuf>, KernelError>`,
   `None` is cancel), `write_atomic(path, bytes)` (temp file in the same folder, sync, rename, temp removed on any failure) and `read_bounded(path, max_bytes)`.
   Errors are `Recoverable` and name the cause in words ("permission denied", "no space left on the device", "the folder does not exist") plus the file name, never the full path.
   `kernel::testing` gets a scripted fake picker.
2. **Dialogs are opened by the backend.** The shell implements `FilePicker` with the official `tauri-plugin-dialog` used from Rust only (no JavaScript plugin,
   no `dialog:*` capability for the window). The frontend therefore never sends or sees a path: modules call the picker, write the file and reply
   `Option<SavedFile { file_name }>`. The dialog blocks the command thread, so these commands are `async` in the shell and run on a blocking task; the main
   thread and the bus are never blocked.
3. **Screenshots are composed in the frontend** (architecture 4.5 rule 4: they include cursors and layout) on an offscreen canvas at device resolution,
   encoded to PNG by the browser, and sent as a raw binary command body (`save_screenshot`) to `export`, which validates the PNG signature and a size
   limit (32 MiB), asks the picker and writes atomically. No new dependency.
4. **Setup file** (`format: "sweepscope-setup"`, `version: 1`): pretty-printed JSON with stable key order, written by `acquisition`: per channel
   `enabled`, `volts_per_div` (the range in volts, not an index), `position_divs`, `coupling`, `bandwidth_limit`, `probe`, `invert`; `time_base_s_per_div`;
   trigger `source`, `slope`, `sweep`, `level_volts`, `position_percent`. Values, not indexes, so a file stays meaningful on another device. JSON, not TOML,
   because `serde_json` is already a dependency (P15) and the file is small and flat.
5. **Loading** matches each value to the connected device's tables exactly (relative tolerance 1e-9). A value the device cannot do keeps the
   current setting and adds a line to the result `warnings` (for example `CH2 volts/div 2 mV is not available; kept 1 V`); nothing is dropped silently.
   An unreadable, wrong-format or newer-version file (`version` above the supported one) is a `Recoverable` error with the reason and changes nothing.
   Loading needs a ready device (ADR 0013 item 8).
6. **Last setup** is saved by `acquisition` through the existing `Storage` trait as `last-setup.json` (same format), at most once per second after a change and
   when the module stops, and read at start. It is held unresolved until the first device is ready (the tables are unknown before) and then applied like a load;
   warnings or an unreadable file become `InstrumentState.setup_notice` (a revisioned slice, so the frontend cannot miss it) and the frontend shows one Toast.
   An unreadable file is renamed to `last-setup.json.corrupt` (first one kept, as settings does) and defaults are used. Acquisition does not start by itself after a restore.
7. **Cursor positions and pinned rows** are presentation state: pinned measurement rows persist through the existing UI-state blob (namespace `measurements`);
   cursors are not persisted in M2.

## Alternatives considered
- **JS dialog plugin, frontend sends the path.** Needs a window capability and lets any script in the WebView name a path to overwrite; the Rust-side picker removes both.
- **Shell orchestrates dialog then module.** The shell would hold flow logic and modules could not be tested with a fake picker.
- **Setup in `settings`.** Settings would have to understand acquisition's data or store it opaquely and lose validation and the load report.
- **TOML.** Nicer for comments, costs a dependency for a flat file.
- **Screenshot by the backend.** WebKitGTK offers no page capture through Tauri; a native capture would miss the HiDPI canvas composition and the measurement table.

## Consequences
- One new dependency, `tauri-plugin-dialog` (Rust only), justified in the spec; architecture 10.5 (security review) is answered: no dialog or fs capability for the window.
- Modules that write files are tested with a temp directory and the fake picker; only the shell touches the real dialog.
- `docs/architecture.md` is updated after approval: settings holds UI-state blobs only; setups belong to `acquisition`.
