# Roll recording file, version 1 (`SSRL`)

SweepScope writes a recording of roll mode (500 ms/div and slower) as a `.ssroll` file (Record button). The file is written as a stream, one record after another, and flushed
once per second, so a crash or a pulled cable leaves a readable file: a reader ignores a last record that is cut short. The reference reader is `tools/roll-record-reader.py`
(standard library only); it prints every record as JSON.

## Layout

Integers are little-endian.

    bytes 0..4        magic, the ASCII text SSRL
    bytes 4..6        version, unsigned 16-bit integer, 1
    bytes 6..8        reserved, 0
    after the header  records, until the end of the file

Every record is `u8 kind`, `u32 length` (payload bytes), then the payload:

    kind 1  segment   UTF-8 JSON, see below
    kind 2  samples   u64 first_index, u32 count, then for each channel listed in the segment, in that order,
                      count bytes: one unsigned 8-bit converter code per sample
    kind 3  gap       u64 after, u64 missed, u8 estimated
    kind 4  end       u8 reason: 0 stopped, 1 left roll, 2 disconnected

A recording starts with a segment record. A change of the setup (volts per division, time base, probe, invert, channels) starts a new segment in the same file, and the sample
indices count from 0 again. Records between a segment and the next one belong to the first. A segment is written when its first samples arrive.

Segment keys:

    started_unix_us    Unix time in microseconds when the recorder opened the segment
    first_index        index of the first sample of the segment
    session            roll session number, increases with every setup change
    sample_interval_s  seconds between two samples
    seconds_per_div    time base setting
    counts_per_div     converter codes per vertical division
    channels           list, ascending channel number, enabled channels only; each entry has
                       channel (1 to 4), volts_per_div (at the input), probe (1, 10 or 100),
                       zero_code (the code of 0 V, may be fractional), inverted

Volts of a sample: (code - zero_code) / counts_per_div * volts_per_div * probe. Time of the sample with index k: k * sample_interval_s, counted from the first sample of the
session. `inverted` tells that the display inverts the trace; the codes are stored as captured.

## Gaps

A gap record marks samples that were not recorded. `after` is the index of the sample before the gap and `missed` the number of samples lost. `estimated` is 1 when the
count comes from elapsed time (the scope's buffer overran between two polls) and 0 when the application knows it exactly (items were dropped between its own modules).
A gap before the first samples of a segment has `after` 0. To place a gap, a reader should not rely on `after`: the lost samples are the indices between the end of the
previous samples record (`first_index + count`) and the `first_index` of the next one. Where the loss was estimated, that distance can differ from `missed` by a few samples;
the `first_index` of the next record is the truth for the time axis.

## End

An end record closes the recording: the user stopped it, roll mode ended, or the scope was disconnected. A file without an end record was cut by a failure (full disk, removed file,
crash) and is still readable up to the last complete record.

## Compatibility

A reader must refuse a different magic or a `version` it does not know. A new version changes the version field and this document.
