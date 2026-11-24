#!/bin/sh
set -e
udevadm control --reload-rules >/dev/null 2>&1 || true
