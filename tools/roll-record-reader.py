#!/usr/bin/env python3
"""Reference reader for .ssroll v1 (docs/formats/roll-record-v1.md).

Usage: roll-record-reader.py FILE
Prints one JSON document: {"version", "records": [...], "truncated"}.
Standard library only. A truncated last record is ignored and reported.
"""
import json
import struct
import sys

MAGIC = b"SSRL"
KNOWN_VERSION = 1
END_REASONS = {0: "stopped", 1: "left roll", 2: "disconnected"}


def read_records(data):
    position = 8
    records = []
    channels = 0
    while position < len(data):
        if len(data) - position < 5:
            return records, True
        kind, length = struct.unpack_from("<BI", data, position)
        start = position + 5
        if start + length > len(data):
            return records, True
        payload = data[start:start + length]
        if kind == 1:
            segment = json.loads(payload.decode("utf-8"))
            channels = len(segment["channels"])
            records.append({"kind": "segment", **segment})
        elif kind == 2:
            first_index, count = struct.unpack_from("<QI", payload, 0)
            codes = [list(payload[12 + c * count:12 + (c + 1) * count]) for c in range(channels)]
            records.append({"kind": "samples", "first_index": first_index, "count": count, "codes": codes})
        elif kind == 3:
            after, missed, estimated = struct.unpack("<QQB", payload)
            records.append({"kind": "gap", "after": after, "missed": missed, "estimated": bool(estimated)})
        elif kind == 4:
            records.append({"kind": "end", "reason": END_REASONS.get(payload[0], "unknown")})
        position = start + length
    return records, False


def main(path):
    with open(path, "rb") as handle:
        data = handle.read()
    if len(data) < 8 or data[:4] != MAGIC:
        sys.exit("not a .ssroll file")
    version, _reserved = struct.unpack_from("<HH", data, 4)
    if version != KNOWN_VERSION:
        sys.exit("unknown version %d" % version)
    records, truncated = read_records(data)
    json.dump({"version": version, "records": records, "truncated": truncated}, sys.stdout)


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
