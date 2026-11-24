# Raw frame file, version 1 (`SSR1`)

SweepScope writes one captured frame as a `.bin` file (menu File, Export raw). A text copy of the Layout section is written beside it as
`<stem>.format.txt`, where `<stem>` is the file name without its extension (`cap.bin` gets `cap.format.txt`), so the file stays readable without the application. The reader in the code is `decode_raw` in the `export` crate.

## Layout

A SweepScope raw capture (SSR1, version 1). Integers are little-endian.

    bytes 0..4        magic, the ASCII text SSR1
    bytes 4..8        H, unsigned 32-bit integer, the length of the header in bytes
    bytes 8..8+H      header, UTF-8 JSON (see the keys below)
    after the header  sample data: for each channel listed in the header, in that order,
                      record_points bytes, one unsigned 8-bit converter code per sample

Header keys:

    format            "sweepscope-raw"
    version           1
    timestamp_utc     ISO 8601, when the frame reached the application
    sample_interval_s seconds between two samples
    seconds_per_div   time base setting
    record_points     samples per channel
    counts_per_div    converter codes per vertical division
    trigger_index     index of the trigger sample, null when there was none
    channels          list, ascending channel number, enabled channels only; each entry has
                      channel (1 to 4), volts_per_div (at the input), probe (1, 10 or 100),
                      zero_code (the code of 0 V, may be fractional), inverted, calibrated
                      (true when the zero calibration offset was applied to zero_code)

Volts of a sample: (code - zero_code) / counts_per_div * volts_per_div * probe. Time of sample k: k * sample_interval_s, counted from the
first sample of the record. `inverted` tells that the display inverts the trace; the codes are stored as captured.

## Compatibility

Files written before the zero calibration have `range_verified` instead of `calibrated`; a reader ignores it and treats a missing `calibrated` as false. The version stays 1.

A reader must refuse a different magic or a `version` it does not know. A new version changes the `version` key and this document.
