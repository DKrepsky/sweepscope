# QA report: M0 Spike, product part (#3 ScopeDevice trait and simulated scope)

Build or commit: 30ef39c on `feat/3-scope-device-and-simulator` (WP3.1 798ea06, WP3.2 06fdea1, WP3.3 1a583af, fix round 30ef39c), working tree clean for product code. First pass was on 1a583af; retest of the fix round is in the section "Retest of 30ef39c".
Date: 2026-11-22

## Summary
Verdict: **pass** (after retest of 30ef39c; the first-pass text below is kept for history). Original first-pass verdict: pass with known issues. All five acceptance criteria are met by running tests. Two deviations found; BUG-1 (medium) should be fixed before G4 because it breaks a PRD 5.1 verification-status rule, BUG-2 (low) needs a staff decision or a doc line. Nothing was tested on real hardware (none needed for #3).

Runs: `cargo test --workspace` all green (device crate: 60 unit, 43 integration tests; the crate suite was repeated 20 times, no flaky result, including the real-clock unplug test); `npm run check` green (fmt, clippy -D warnings, eslint, stylelint, prettier, svelte-check 0 errors, check-architecture, check-test-names); generated TS files unchanged in git and type-check in `frontend`. (The spec's "npm run check" in `frontend` does not exist: the script is at the repo root.)

## Results
| Test ID | Result | Notes |
|---|---|---|
| T1 AC1 trait extensible | pass | `second_device.rs` implements `ScopeDevice` through `Box<dyn ScopeDevice>`; crate depends only on kernel, serde, ts-rs (architecture check ok). The test proves the trait is object-safe and implementable, not that the future UI is untouched (no UI yet) |
| T2 AC2 identity | pass | model "6254BC (simulated)", firmware `sim-1.0`, FPGA `sim-0x0001`, calibration Valid, `simulated` true |
| T3 types export | pass | camelCase, committed, frontend type-checks |
| T4 AC3 frames | pass | 4096 codes per enabled channel, trigger position 30 % and slopes verified by tests |
| T5 AC3 dead time | pass | cycle = poll-rounded acquisition + 15 + 5 ms (40 ms minimum), pinned by `timing.rs` and capture tests. Constants are unverified assumptions, documented as such in `timing.rs` |
| T6 sample interval | pass | 1/2/4 ns, 800 ns, 2 ms, 0.8 ms match PRD 5.1 and driver |
| T7 sweep | pass | Normal: Timeout after timeout in clock time; Auto: free-run frame |
| T8 disabled and clipping | pass | |
| T9 roll | pass | about 499 samples per second, empty second poll, no loss or repeat |
| T10 overrun | pass | 95 % rule, injected overrun once, fast time base roll refused |
| T11 AC4 unplug and events | pass (retest 30ef39c; was pass with BUG-2) | every call returns `Disconnected`, `Detached` once, `Attached` on plug in, `open` gives `NotFound`. No initial `Attached` (BUG-2) |
| T12 unplug during capture | pass | returns within one poll; 20 repeats clean |
| T13 invalid setups | pass | |
| T14 error mapping | pass | Disconnected and NotFound to DeviceLost, others Recoverable |
| T15 capability tables | pass (retest 30ef39c; was fail BUG-1) | 36 steps, streaming from index 25, 12 ranges, ranges verified 50 mV to 1 V correct; but all 36 time bases `verified: true` (PRD 5.1: only 0-22, 24, 25, 27, 30 measured) |
| T16 hotplug parity | pass (retest 30ef39c; was deviation BUG-2) | driver `hotplug.rs` replays already connected scopes on start; simulator does not |
| T17 zero model | pass, note | `zero = 128 + position_divs x counts_per_div`, counts 24.15/21.6/21.1 as driver section 9. Driver ends are 31..224 (k=4) at the position limits; the simulator does not clamp the position, so a position beyond about +-5 divisions yields a code range the real device cannot reach (codes still clip at 0..255). Info only, simulator limit to document |
| T18 AC5 tests without hardware | pass | no hardware, one real-clock test only |
| T19 flakiness | pass | 20 repeats |

## Developer deviations checked
- `profile.rs` extra file (identity, tables, zero model): fine, not in the spec file list but within `simulator/` and tested (6 unit tests).
- No initial Attached event: see BUG-2.
- `zero_code` model (128 + position x counts): consistent with driver section 9 (position 128 gives code 128); see T17 note.
- All 36 time bases marked verified: wrong, BUG-1.

## Bugs
| Bug | Severity | Status |
|---|---|---|
| BUG-1 all time bases `verified: true`, PRD 5.1 lists measured indexes 0-22, 24, 25, 27, 30 | medium | fixed, verified in 30ef39c |
| BUG-2 simulated catalog gives no `Attached` for an already plugged device (real monitor does) | low | fixed, verified in 30ef39c |

## Not tested and why
- Real 6254BC: no adapter exists yet.
- UI, themes, keyboard: no UI in this story.
- Trigger accuracy of the real device (+-6 samples) and real dead time: not simulated, documented as assumptions; need hardware measurement (spike #2).
- Spikes #2 and #4: no QA e2e by plan; harness review comes when their results exist.
- TS contract in CI: checked locally only (git shows no change).

## Screenshots and logs
None (no UI); `.claude/tmp/3/` not used.

## Retest of 30ef39c (2026-11-22)
Result: all fixes verified, no new bug. Bugs open: 0.

| Case | Result | Evidence |
|---|---|---|
| T15 verified flags | pass | `profile` unit test `only_the_prd_measured_indexes_are_verified` asserts verified = 0-22, 24, 25, 27, 30 (exactly PRD 5.1); ranges test still 12 ranges, 50 mV to 1 V verified. Code read: `VERIFIED_TIME_BASES` matches PRD |
| T11, T16 initial Attached | pass | `subscribe()` sends `Attached` first if plugged; tests `attached_comes_first` (catalog and control), none when unplugged at subscribe, then `Detached`, `Attached` on changes. Matches the real monitor replay |
| Handle invalidation | pass | generation counter bumps on unplug; test `old_handle_stays_disconnected` (old handle `Disconnected` after plug_in, `catalog.open` gives a working new handle). The old "works again after plug_in" test was correctly removed |
| T12 unplug during capture | pass | now a deterministic `FakeClock` test: unplug inside the 3rd sleep, result `Disconnected`, exactly 3 sleeps (no further sleep). Real-clock test is gone, so no timing flake source left |
| T13 input validation | pass | NaN position and infinite trigger level give `InvalidSetting` (2 new tests); earlier invalid-setup cases unchanged |
| Private modules | pass | `simulator` is private; `edge`, `signal`, `timing` private; public API is only `device_id`, `SignalKind`, `SimulatedCatalog`, `SimulatedDevice`, `SimulatorControl`. No other crate uses `simulator::` paths; workspace builds; `check-architecture` ok |
| Behaviour note | info | Documented in `simulator/mod.rs`: Auto sweep ignores the capture timeout (cycle always completes), covered by a new test. Consistent with Auto free-run in T7; not previously stated, flagging for staff awareness |

Runs on 30ef39c: `cargo test --workspace` green; device crate 110 tests (64 unit, 1 contract, 1 second_device, 22 capture, 8 faults, 4 identity, 10 roll), suite run 20 more times with zero failures; `npm run check` green (fmt, clippy, eslint, stylelint, prettier, svelte-check 0 errors 0 warnings, check-architecture, check-test-names).
Still not tested: real hardware and UI (as above).
