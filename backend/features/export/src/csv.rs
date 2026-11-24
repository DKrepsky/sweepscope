use std::fmt::Write;

use crate::capture::{Capture, ChannelCapture};

/// Probe scaled volts of one code; invert is not applied.
fn volts(capture: &Capture, channel: &ChannelCapture, code: u8) -> f64 {
    (f64::from(code) - channel.zero_code) / capture.counts_per_div * channel.volts_per_div * channel.probe
}

pub fn encode(capture: &Capture) -> String {
    let mut text = metadata(capture);
    text.push_str("time_s");
    for channel in &capture.channels {
        let _ = write!(text, ",ch{}_v", channel.channel);
    }
    text.push('\n');
    for index in 0..capture.record_points {
        let _ = write!(text, "{:.12}", index as f64 * capture.sample_interval_s);
        for channel in &capture.channels {
            let _ = write!(text, ",{:.6}", volts(capture, channel, channel.codes[index]));
        }
        text.push('\n');
    }
    text
}

fn metadata(capture: &Capture) -> String {
    let trigger = capture
        .trigger_index
        .map_or_else(|| "none".to_owned(), |index| index.to_string());
    let mut text = String::new();
    let mut line = |key: &str, value: String| {
        let _ = writeln!(text, "# {key}: {value}");
    };
    line("format", "sweepscope-csv 1".into());
    line("timestamp_utc", capture.timestamp_utc.clone());
    line("time_base_s_per_div", capture.seconds_per_div.to_string());
    line("sample_interval_s", capture.sample_interval_s.to_string());
    line("record_points", capture.record_points.to_string());
    line("trigger_index", trigger);
    line("time_origin", "first sample of the record".into());
    line("volts", "probe scaled, invert not applied".into());
    for channel in &capture.channels {
        let n = channel.channel;
        line(&format!("ch{n}_volts_per_div"), channel.volts_per_div.to_string());
        line(&format!("ch{n}_probe"), channel.probe.to_string());
        line(&format!("ch{n}_zero_offset_applied"), channel.calibrated.to_string());
    }
    text
}
