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
