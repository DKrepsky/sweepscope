# ADR 0023: Verification markers leave the product; device verification facts stay as documentation and an internal hint

Status: Accepted
Date: 2026-11-24
Issue: #50, #34

## Context
The human removed every UNVERIFIED and PARTIALLY VERIFIED tag (#50). Earlier decisions built a data path for them (Q-E3, Q-E4, per-channel and horizontal flags, a measurement cell state, `FrameContext.range_verified`, flags in CSV and raw headers). Capability `verified` flags also drive an auto-setup tie-break (ADR 0020).

## Decision
1. Remove everything that exists only to show a marker, in backend and frontend. Q-E3 and Q-E4 are void.
2. Keep `verified` on vertical ranges and time bases as internal device facts (auto-setup, accuracy-page consistency test); the frontend never reads them.
3. `FrameContext.range_verified` becomes `calibrated` (ADR 0021). CSV and raw headers carry `calibrated`; raw format version 1 is kept, old files still open.
4. Behaviour is unchanged: nothing is blocked. Accuracy information moves to the status tooltip and the accuracy page.

## Alternatives considered
Keep the fields unused (dead, misleading); remove the capability flags too (loses the tie-break and the page guard); raw format version 2 (migration for a key nobody reads).

## Consequences
Generated types change; export files differ in two keys; many tests are updated.
