# Accuracy

What SweepScope can and cannot promise about its readings on a Hantek 6254BC, and where each statement comes from.

## How to read this page

Every number here is a measurement on **one 6254BC unit**. They are not tolerances and not a specification: another
unit can differ. Statements say "about" because they describe what was seen, not a limit. The measurements are in the
driver's protocol document (`hantek6xx4-lib`, `docs/protocol.md`, section 10 and appendix A). Where this page says
"verified" it means checked on that unit; "not verified" means nobody has checked it, not that it is wrong.

The scope is an 8-bit instrument. A reading has a resolution of one count, 1/24 of a division.

## Capability table

| Item | Status |
|---|---|
| Four channels, independently enabled | Verified (three channels: data layout only) |
| Analog bandwidth 250 MHz | Vendor specification, not measured |
| Bandwidth limit | Verified (the relay switches) |
| ADC, 8-bit HMCAD1511 | Hardware fact, not a measurement |
| Hotplug, connect and disconnect, 2 s settle time | Verified (the scope re-enumerates about 1.2 s after plug-in) |
| Sample rate 1 GS/s (1 channel), 500 MS/s (2), 250 MS/s (3 or 4) | Verified by rise-time measurements |
| Twelve vertical ranges, relays, bandwidth limit, AC/DC coupling | Verified |
| Edge trigger, pulse trigger | Verified (pulses of 500 us) |
| Video trigger | Not verified (vendor only) |
| Scan and roll streaming | Verified |
| Volts conversion | Verified from 50 mV to 1 V per division |
| Trigger position | Lands within about 6 samples of the requested position |

## Vertical ranges

Gain was checked with a DC source from 50 mV to 1 V per division (counts per division 24.1 against the nominal
24.15, volts within about 1.5 %). Gain was not checked on the other ranges. The zero level of a shorted input was
measured on all twelve ranges, so those ranges are partly verified. Typical zero error is the offset one unit showed
with its input shorted, in volts and in counts.

| Range | Status | Typical zero error |
|---|---|---|
| 2 mV/div | Partly verified | +0.66 mV (8 counts) |
| 5 mV/div | Partly verified | +1.04 mV (5 counts) |
| 10 mV/div | Partly verified | +0.83 mV (2 counts) |
| 20 mV/div | Partly verified | +0.83 mV (1 count) |
| 50 mV/div | Verified | up to 2 counts between channels |
| 100 mV/div | Verified | up to 2 counts between channels |
| 200 mV/div | Verified | up to 2 counts between channels |
| 500 mV/div | Verified | up to 2 counts between channels |
| 1 V/div | Verified | up to 2 counts between channels |
| 2 V/div | Partly verified | up to 2 counts between channels |
| 5 V/div | Partly verified | up to 2 counts between channels |
| 10 V/div | Partly verified | up to 2 counts between channels |

On 2 mV/div the input noise of that unit was about 20 counts rms, on 5 mV/div about 9. Readings on the most sensitive
ranges are dominated by noise; the bandwidth limit lowers it. At extreme vertical positions readings can still be off
by 2 to 3 counts (about 0.1 division).

## Time bases

There are 36 time bases, 2 ns to 1000 s per division. Measured indexes: 0 to 22, 24, 25, 27, 30. The period of the
1 kHz calibrator was within about 0.001 % from 200 us to 50 ms per division. Time bases that were not measured share
the same divisor table but have not been checked.

## Trigger position

The trigger edge lands within about 6 samples of the requested position. The offset depends on the number of enabled
channels and the time base, and is not modelled. At 200 us per division it was about +5, -2 and -6 samples for 1, 2
and 4 channels.

## What calibration removes and what it does not

Zero calibration measures each channel with the input shorted, on every range, averaging about 12 000 samples. It
removes the zero error of that channel on that range for that unit, and the trigger level follows the correction.

It does not remove:

- gain error: about 2 % from 50 mV to 1 V per division on the unit measured, not verified outside that range;
- the 2 to 3 count error at extreme vertical positions (about 0.1 division);
- input noise, and the one-count resolution of the converter;
- the skew of about 2 counts between channels.

## Report another unit

If you have a different 6254BC or a 6254BD, measurements are welcome: open an issue with the model, the zero
calibration table from the Done step, and what you connected (a DC source with a multimeter reading, or the probe
compensation output). The driver project collects the same data for its protocol document.
