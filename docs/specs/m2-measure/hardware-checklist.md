# M2 hardware checklist (run by the human on the Hantek 6254BC)

Closes the M2 numbers that the simulator cannot prove. Spec: [implementation-spec.md](implementation-spec.md). Agents never touch the scope.
Record pass or fail with a word of context per item; keep measured figures in your own notes (ADR 0008), the QA report carries outcomes.
Setup: CH1 probe on the `CAL` terminal (1 kHz square, about 1.9 V peak to peak), 1x probe, CH1 only, DC, 200 us/div, Run. Run `RUST_LOG=info` in the terminal.

| ID | Do | Expect |
|---|---|---|
| H-01 | Pin all rows. Read frequency, period, Vpp, duty at 200 us/div, then at 100 us/div, 500 us/div and 1 ms/div. | Frequency `1.000 kHz` and Vpp about `1.9 V` at every base; duty about 50 %. Note any base where frequency is off by more than 0.01 % (R2). |
| H-02 | Watch the frequency for a minute at 200 us/div. | The last digit is stable or moves by one count; a persistent drift larger than 0.01 % is a failure to report. |
| H-03 | Time base 50 ns/div, CH1 only. Read rise time and fall time; step sin(x)/x, Linear, Dots. | Rise and fall 240 to 256 ns. sin(x)/x shows an edge, not a staircase; Dots shows only points, no lines. Default (after restart) is sin(x)/x at this base. |
| H-04 | Time base 2 ns/div. | Frequency and period show `-` with the reason "Fewer than one period"; rise time shows part of the edge or a reason, never a number from nowhere. |
| H-05 | CH1 at 200 mV/div (the 1.9 V square exceeds 8 divisions). | `Clip` badge on the CH1 scale chip; Vpp, Vmax, Vmin, rise and fall show `-` with the clip reason; frequency still reads 1.000 kHz. |
| H-06 | CH1 at 20 mV/div and at 2 V/div. | Unverified marker (`?` and amber edge) on the values, the accuracy note under the table. |
| H-07 | Cursors (`C`) on the CAL square at 200 us/div. Put X1 and X2 on two rising edges; Y1 and Y2 on the levels. Move with mouse and with arrow keys. | ΔX 1.00 ms, 1/ΔX 1.00 kHz, ΔY about 1.9 V; values follow when CH1 volts/div or time/div changes; unplug the scope: cursors still work on the frozen trace and measurements read `Frozen`. |
| H-08 | Export CSV and raw (`Ctrl+E` flow). Open the CSV in a spreadsheet; read the raw file with the Python reader of `docs/formats/frame-raw-v1.md`. Export into a folder you cannot write to. | CSV has the metadata and 4096 rows per enabled channel, values match the screen; raw reloads with the same codes; the read-only folder gives an error Toast naming permission denied and leaves no file. |
| H-09 | Screenshot (`Ctrl+P`) on your normal display, and again at 200 % OS scale if you can. | PNG shows graticule, traces, scale labels, measurements and cursors; lines are crisp at device resolution; the dialog opens in front of the window and cancelling does nothing. |
| H-10 | Save a setup (`Ctrl+S`), change CH1 range, time base and trigger, load it back; hand-edit a copy so CH1 range is 3 V and load that; quit and restart the app with the scope attached. | Loading restores the saved values; the edited copy loads with a warning naming the 3 V range and keeps the current range; after restart the last setup is back and acquisition is stopped. |
