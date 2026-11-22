# Test plan: M0 Spike, product part (#3 ScopeDevice trait and simulated scope)

Spikes #2 and #4 get no QA e2e (implementation spec section 5); only #3 is planned here.

## Scope and risks
- Scope: crate `sweepscope-device` (traits, types, `SimulatedDevice`, `SimulatedCatalog`, `SimulatorControl`), `Clock::sleep` in the kernel, generated TypeScript types.
- No UI exists for #3, so there is no UI check, theme or keyboard case. Verification is by unit and integration tests on `FakeClock`, plus code and data inspection against PRD 5.1/5.2, the driver docs (`hantek6xx4-lib/docs/protocol.md`) and ADR 0007.
- Risks: simulator tables drift from the PRD verification status (UI shows "unverified" markers from them); simulator events differ from the real adapter; dead-time constants are assumptions; the second-device test proves AC1 only for the trait shape.
- Not applicable: a real 6254BC (the Hantek adapter is a later story).

## Test cases
Kind: AUTO = existing automated test run by me; INSP = inspection of code/data against a source; EXP = exploratory check. Hardware: none for all.

| ID | Given / when / then | Kind | Covers |
|---|---|---|---|
| T1 | given a second `ScopeDevice` in a test when used as `Box<dyn ScopeDevice>` then it runs with no crate change (`second_device.rs`); the trait and `DeviceCatalog` import nothing device specific; no feature depends on the crate (`check-architecture`) | AUTO + INSP | AC1 |
| T2 | given the simulator when identity is read then model contains "simulated", firmware, FPGA, calibration present, `simulated` true (`simulator_identity.rs`, `profile.rs`) | AUTO | AC2 |
| T3 | given the exported types when generated then camelCase TS files equal committed ones and `frontend` type-checks | AUTO | AC2, ADR 0003 |
| T4 | given 4 channels, edge trigger, sine when captured then 4096 codes per channel, trigger at configured position, rising and falling slope respected | AUTO | AC3 |
| T5 | given two captures then the clock advances exactly one frame cycle (acquisition rounded to 20 ms + 15 ms + 5 ms); slow time bases round up | AUTO | AC3 |
| T6 | given 1, 2, 4 channels then sample interval is 1, 2, 4 ns at 2 ns/div and tb/250 at 200 us/div; 4 ch 200 us/div is 800 ns; 500 ms/div is 2 ms | AUTO | AC3, PRD 5.1 |
| T7 | given normal sweep and level above signal then `Timeout` after the timeout in clock time; auto sweep then `TriggerOutcome::Auto` | AUTO | AC3, ACQ-2 |
| T8 | given a disabled channel then `None`; given a sine above the range then codes clip at 0 and 255 | AUTO | AC3 |
| T9 | given roll at 500 ms/div, 1 s elapsed then about 499 samples per channel, again at once empty, no loss or repeat across polls | AUTO | PRD 5.2 |
| T10 | given roll beyond 95 % of the buffer span then `Overrun` once, then data; `inject_overrun` then `Overrun` once; fast time base roll then `InvalidSetting` | AUTO | AC4 |
| T11 | given `unplug` then `configure`, `capture`, `poll_roll`, `start_roll`, `stop` return `Disconnected`; catalog emits `Detached` once; `plug_in` emits `Attached`; `open` while unplugged `NotFound`; recovery after plug in | AUTO | AC4 |
| T12 | given a capture waiting for a trigger when unplugged from another thread then `Disconnected` within one poll (real clock) | AUTO (repeat 20x for flakiness) | AC4 |
| T13 | given invalid setups (range, time base, no channel, source disabled, not configured) then `InvalidSetting` | AUTO | AC4 |
| T14 | given each `DeviceError` then `category()` and `KernelError` conversion follow ADR 0007 | AUTO | AC4 |
| T15 | given the capability tables then 36 time bases 2 ns to 1000 s, streaming from index 25, 12 ranges, only 50 mV to 1 V verified; time base `verified` flags match PRD 5.1 | AUTO + INSP | AC2, PRD 5.1 |
| T16 | given the real driver's hotplug behaviour (already connected scopes are reported on start) then the simulated catalog behaves alike | INSP | AC4 |
| T17 | given the zero model then simulator codes match the driver's volts formula `(code - zero) * V/div / counts` and counts per div 24.15 / 21.6 / 21.1 | INSP | AC3 |
| T18 | given no hardware when `cargo test --workspace` runs then all pass; `npm run check` (fmt, clippy -D warnings, lint, typecheck, architecture, test names) passes; no real-clock wait beyond one test | AUTO | AC5 |
| T19 | EXP: repeat the device crate tests 20 times for flakiness | EXP | AC5 |

## Environment
Linux, `cargo test --workspace`, `npm run check` in the working tree on branch `feat/3-scope-device-and-simulator` at 1a583af. No real 6254BC.

## Entry and exit criteria
Entry: WP3.1 to WP3.3 committed locally. Exit: every AC mapped to a passed case, no open bug above low, deviations decided by the staff engineer.
