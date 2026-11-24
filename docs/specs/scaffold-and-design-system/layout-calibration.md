# Layout calibration (spec 11.10)

Date: 2026-11-22. Text scale 100 %, Chromium (Playwright 1.65) and WebKit (Playwright build). Both engines gave identical numbers.
Commit: the commit that adds this file (see `git log -- docs/specs/scaffold-and-design-system/layout-calibration.md`).

Method: throwaway Playwright script (scratchpad, not committed) against the dev server. For each (column, tier) the window was set
inside the tier's range (width 1024 = narrow, 1440 = wide) and the need was computed as
`toolbar + status bar + (content scroll height - scope stage height) + stage minimum`, where the stage minimum is 25.6 rem of
screen plus 1.25 + 1 rem of gutters (445.6 px). Measured values: toolbar 44 px, status bar 25 px (one line) or 49 px (two lines),
dock 334 px at narrow h1.
Rule: final = measured + 0.25 rem, rounded up to 0.25 rem; the h4 threshold is raised to the content floor (45 rem), because
under the floor the tier is `scroll`.

| Column | Tier | Estimate (rem) | Measured need (px) | Measured (rem) | Final threshold (rem) |
|---|---|---|---|---|---|
| wide | h1 | 45.10 | 748.9 | 46.81 | 47.25 |
| wide | h2 | 44.10 | 732.9 | 45.81 | 46.25 |
| wide | h3 | not reached | 732.9 | 45.81 (same as h2: channel tabs save nothing, the Horizontal and Trigger groups are taller) | not used |
| wide | h4 | not reached | 673.6 | 42.10 | 45 (floor) |
| narrow | h1 | 53.85 | 937.6 | 58.60 | 59 |
| narrow | h2 | 52.10 | 905.6 | 56.60 | 57 |
| narrow | h3 | 49.48 | 797.9 | 49.87 | 50.25 |
| narrow | h4 | 42.98 | 673.6 | 42.10 | 45 (floor) |

Estimates off by more than 1.5 rem: wide h1, wide h2, narrow h1, narrow h2 (the W3 dock is 20.9 rem high, not 16.25). Reported to the
staff engineer on the bus. The h4 need is under 45 rem, so the floor stays at 45 rem (1024 x 720).

Verification of AC-L1 with the final table: window heights from 45 rem to 62 rem in 2 px steps at widths 1024 and 1440, text scale 100 %,
both engines: scope screen 10:8 and at least 32 x 25.6 rem, inside the stage, no vertical scroll of the content region, toolbar, scale
strip, connection and sample rate visible. No failures.

## Recalibration for M3 (row M and the Pulse trigger layout)

Date: 2026-11-24. Text scale 100 %, Chromium and WebKit, identical numbers. Method as above, with the final UI: row M in the Vertical
group (a fifth row, a channel-style tab at h3 and h4), the Pulse Trigger group (Type, Source with polarity, Level, Width, Sweep and a
reserved two-line message slot) and the stub snapshot (channels 1 to 3 on). Need = window height - stage height + 445.6 px; where the
content still scrolled at the measured height (stage at its minimum) the overflow was added to the need.

| Column | Tier | Dock edge (px) | Dock Pulse (px) | Need edge (rem) | Need Pulse (rem) | Threshold (rem) | Previous (rem) |
|---|---|---|---|---|---|---|---|
| wide | h1 | 174.5 | 214.2 | 47.13 | 49.61 | 50 | 47.25 |
| wide | h2 | 154.5 | 194.2 | 45.88 | 48.35 | 48.75 | 46.25 |
| wide | h4 (Vertical tab) | 94 | 94 | 42.04 | 42.04 | 45 (floor) | 45 |
| narrow | h1 | 323 | 388.7 | 57.91 | 62.02 | 62.5 | 59 |
| narrow | h2 | 287 | 348.7 | 55.66 | 59.54 | 60 | 57 |
| narrow | h3 | 197.5 | 259.2 | 48.57 | 52.42 | 52.75 | 50.25 |
| narrow | h4 (Vertical tab) | 94 | 94 | 42.04 | 42.04 | 45 (floor) | 45 |

Row M is exactly as high as a channel row (28 px at h1, 24 px at h2), one line at 1024 and 1440 px and at 200 % text scale. The Pulse
Trigger group is 213 px high (edge: 131 to 173 px), 35 px more than the estimate of the implementation spec (section 7), which left out the
row gaps; it sets the dock height, so the thresholds are 2.25 to 3.5 rem above the estimate.
Known limit, not changed here: at h4 the Trigger tab is taller than the default Vertical tab (edge 161 px, Pulse 223 px dock), so at
720 px the content region scrolls by 22 px (edge) or 83 px (Pulse) while that tab is shown; the thresholds above are measured on the
default tab, as before.
Verification: `e2e/tiers.spec.ts` (every threshold and one pixel below it, Pulse layout at each tier, row M height), both engines, no failures.
