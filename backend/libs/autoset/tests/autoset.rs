mod harness;

use autoset::{FailReason, Step};
use harness::{drive, observe, run_on_channel_one, run_with, tables, Signal};

const FREQUENCIES_HZ: [f64; 7] = [10.0, 100.0, 1e3, 10e3, 100e3, 1e6, 10e6];
const AMPLITUDES_VOLTS: [f64; 7] = [0.005, 0.05, 0.5, 5.0, 20.0, 0.02, 2.0];
const NARROWEST_SWING_VOLTS: f64 = 0.005;

fn describe(signal: &Signal) -> String {
    format!("{} Hz {} V", signal.frequency_hz, signal.peak_to_peak_volts)
}

#[test]
fn given_a_10_hz_square_of_3_volts_when_autoset_runs_then_the_plan_shows_2_to_6_periods_and_3_to_7_divisions() {
    let signal = Signal::square(10.0, 3.0);

    let run = run_on_channel_one(signal);

    assert!(matches!(run.step, Step::Done(_)), "{:?}", run.step);
    assert!(
        (2.0..=6.0).contains(&run.screen_periods(&signal)),
        "{}",
        run.screen_periods(&signal)
    );
    assert!(
        (3.0..=7.0).contains(&run.screen_divs(&signal)),
        "{}",
        run.screen_divs(&signal)
    );
}

#[test]
fn given_seven_frequencies_and_amplitudes_when_autoset_runs_then_each_run_is_done_with_2_to_6_periods_in_14_probes() {
    for frequency_hz in FREQUENCIES_HZ {
        for volts in AMPLITUDES_VOLTS {
            for signal in [Signal::square(frequency_hz, volts), Signal::sine(frequency_hz, volts)] {
                let run = run_on_channel_one(signal);

                assert!(
                    matches!(run.step, Step::Done(_)),
                    "{}: {:?}",
                    describe(&signal),
                    run.step
                );
                let periods = run.screen_periods(&signal);
                assert!(
                    (2.0..=6.0).contains(&periods),
                    "{}: {periods} periods",
                    describe(&signal)
                );
                assert!(run.probes <= 14, "{}: {} probes", describe(&signal), run.probes);
            }
        }
    }
}

#[test]
fn given_seven_frequencies_and_amplitudes_when_autoset_runs_then_the_swing_fills_3_to_7_divisions_where_the_table_allows(
) {
    for frequency_hz in FREQUENCIES_HZ {
        for volts in AMPLITUDES_VOLTS {
            let signal = Signal::square(frequency_hz, volts);

            let divs = run_on_channel_one(signal).screen_divs(&signal);

            let narrowest_table_step_allows_less = volts <= NARROWEST_SWING_VOLTS;
            let wanted = if narrowest_table_step_allows_less {
                2.5..=7.0
            } else {
                3.0..=7.0
            };
            assert!(wanted.contains(&divs), "{}: {divs} divisions", describe(&signal));
        }
    }
}

#[test]
fn given_a_signal_with_a_dc_offset_when_autoset_runs_then_the_result_is_done_and_the_swing_is_not_clipped() {
    let signal = Signal {
        offset_volts: 3.0,
        ..Signal::square(1e3, 1.0)
    };

    let run = run_on_channel_one(signal);

    let seen = observe(&signal, run.plan(), &run.tables);
    let channel = seen.channels[0].unwrap();
    assert!(matches!(run.step, Step::Done(_)), "{:?}", run.step);
    assert!(!channel.clipped, "{channel:?} {:?}", run.plan());
}

#[test]
fn given_14_5_millivolts_peak_to_peak_when_autoset_runs_then_the_nearest_range_is_chosen() {
    let signal = Signal::square(1e3, 0.0145);

    let run = run_on_channel_one(signal);

    let divs = run.screen_divs(&signal);
    assert!(matches!(run.step, Step::Done(_)), "{:?}", run.step);
    assert!((divs - 2.9).abs() < 0.01 || (divs - 7.25).abs() < 0.01, "{divs}");
}

#[test]
fn given_a_flat_input_when_autoset_runs_then_the_result_is_no_signal_with_1_volt_and_1_ms_per_division() {
    let signal = Signal::square(1e3, 0.0);

    let run = run_on_channel_one(signal);

    let Step::NoSignal(plan) = &run.step else {
        panic!("{:?}", run.step)
    };
    assert_eq!(run.tables.range_volts_per_div[plan.range_index[0] as usize], 1.0);
    assert_eq!(run.tables.time_base_s_per_div[plan.time_base_index as usize], 1e-3);
    assert_eq!(plan.position_divs[0], 0.0);
}

#[test]
fn given_a_start_that_clips_the_signal_when_autoset_runs_then_the_range_goes_up() {
    let signal = Signal::square(1e3, 20.0);

    let run = run_on_channel_one(signal);

    let start_range = harness::start_plan(&run.tables).range_index[0];
    assert!(run.plan().range_index[0] > start_range);
}

#[test]
fn given_a_3_hz_sine_when_autoset_runs_then_it_fails_without_a_period() {
    let run = run_on_channel_one(Signal::sine(3.0, 2.0));

    assert_eq!(run.step, Step::Failed(FailReason::NoPeriod));
}

#[test]
fn given_a_flat_input_when_autoset_runs_then_it_stops_within_14_probes() {
    let run = run_on_channel_one(Signal::square(1e3, 0.0));

    assert!(run.probes <= 14, "{}", run.probes);
}

#[test]
fn given_a_period_that_never_settles_when_autoset_runs_then_it_fails_after_one_correction() {
    let signal = Signal::square(1e3, 1.0);

    let run = drive(
        [true, false, false, false],
        |probe, plan, tables| {
            let mut seen = observe(&signal, plan, tables);
            let drifting = if probe % 2 == 0 { 40.0 } else { 0.4 };
            if let Some(channel) = seen.channels[0].as_mut() {
                channel.period_s = channel.period_s.map(|period| period * drifting);
            }
            seen
        },
        |autoset| autoset,
    );

    assert_eq!(run.step, Step::Failed(FailReason::UnstablePeriod));
}

#[test]
fn given_a_clipped_signal_at_every_range_when_autoset_runs_then_it_fails_without_exceeding_14_probes() {
    let signal = Signal::square(1e3, 20.0);

    let run = drive(
        [true, false, false, false],
        |_, plan, tables| {
            let mut seen = observe(&signal, plan, tables);
            if let Some(channel) = seen.channels[0].as_mut() {
                channel.clipped = true;
            }
            seen
        },
        |autoset| autoset,
    );

    assert!(matches!(run.step, Step::Failed(_)), "{:?}", run.step);
    assert!(run.probes <= 14);
}

#[test]
fn given_channel_two_with_the_signal_when_autoset_runs_then_the_trigger_follows_the_lowest_channel_with_a_period() {
    let signal = Signal::square(1e3, 1.0);
    let run = drive(
        [true, true, false, false],
        |_, plan, tables| {
            let mut seen = observe(&signal, plan, tables);
            seen.channels[1] = seen.channels[0].take();
            seen
        },
        |autoset| autoset,
    );

    let plan = run.plan();
    assert!(matches!(run.step, Step::Done(_)), "{:?}", run.step);
    assert_eq!(plan.trigger_source, 1);
}

#[test]
fn given_only_a_slow_signal_in_roll_territory_when_autoset_runs_then_a_roll_time_base_is_never_chosen() {
    let tables = tables();
    for frequency_hz in [5.0, 6.0, 10.0] {
        let run = run_with(Signal::square(frequency_hz, 1.0), [true, false, false, false]);

        assert!(
            (run.plan().time_base_index as usize) < tables.roll_from_index,
            "{frequency_hz} Hz"
        );
    }
}
