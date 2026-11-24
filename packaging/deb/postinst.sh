#!/bin/sh
set -e
udevadm control --reload-rules >/dev/null 2>&1 || true
udevadm trigger --subsystem-match=usb --attr-match=idVendor=04b5 >/dev/null 2>&1 || true
