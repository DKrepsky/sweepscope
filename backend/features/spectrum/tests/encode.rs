use sweepscope_spectrum::{encode, stamp_sent, ChannelSpectrum, FrameHeader, SpectrumWindow, MATH_SLOT};

fn header() -> FrameHeader {
    FrameHeader {
        linear: false,
        window: SpectrumWindow::Hann,
        bin_hz: 12.5,
        top: 10.0,
        units_per_div: 10.0,
        setup_revision: 7,
    }
}

fn channel(slot: u8, peak_bin: Option<u32>, levels: [f32; 5]) -> ChannelSpectrum {
    ChannelSpectrum {
        slot,
        peak_bin,
        divisions: levels.iter().map(|level| (10.0 - level) / 10.0).collect(),
        levels: levels.to_vec(),
    }
}

fn golden() -> Vec<u8> {
    let channels = [
        channel(0, Some(3), [-20.0, -40.0, -10.0, -6.0, -50.0]),
        channel(MATH_SLOT, None, [-200.0, -60.0, -30.0, -70.0, -80.0]),
    ];
    encode(&header(), &channels, 42, 1_700_000_000_000_000.0)
}

#[test]
fn given_the_golden_spectrum_when_encoded_then_it_equals_the_committed_fixture() {
    let bytes = golden();

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/spectrum-v1.bin");
    if std::env::var_os("SWEEPSCOPE_UPDATE_FIXTURES").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &bytes).unwrap();
    }
    let committed = std::fs::read(&path).expect("fixture missing; run with SWEEPSCOPE_UPDATE_FIXTURES=1");
    assert_eq!(bytes, committed);
}

#[test]
fn given_two_channels_when_encoded_then_the_header_and_the_length_follow_the_layout() {
    let bytes = golden();

    assert_eq!(&bytes[..4], b"SSS1");
    assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 42);
    assert_eq!(bytes[24], 0b1_0001);
    assert_eq!(bytes[26], 1);
    assert_eq!(u32::from_le_bytes(bytes[28..32].try_into().unwrap()), 5);
    assert_eq!(bytes.len(), 56 + 2 * 4 + 2 * 5 * 2 * 4);
}

#[test]
fn given_a_channel_without_a_peak_when_encoded_then_its_peak_bin_is_u32_max() {
    let bytes = golden();

    assert_eq!(u32::from_le_bytes(bytes[56..60].try_into().unwrap()), 3);
    assert_eq!(u32::from_le_bytes(bytes[60..64].try_into().unwrap()), u32::MAX);
}

#[test]
fn given_an_encoded_frame_when_the_send_time_is_stamped_then_only_bytes_16_to_24_change() {
    let original = golden();
    let mut stamped = original.clone();

    stamp_sent(&mut stamped, 99.25);

    assert_eq!(f64::from_le_bytes(stamped[16..24].try_into().unwrap()), 99.25);
    assert_eq!(stamped[..16], original[..16]);
    assert_eq!(stamped[24..], original[24..]);
}
