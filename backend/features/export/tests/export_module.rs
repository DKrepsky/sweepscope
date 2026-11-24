use std::path::PathBuf;
use std::sync::Arc;

use sweepscope_export::{decode_raw, ExportDeps, ExportFormat, ExportFrame, ExportModule, SaveScreenshot};
use sweepscope_kernel::contracts::{ChannelData, Frame, FrameCaptured, FrameContext, TriggerOutcome};
use sweepscope_kernel::testing::{ScriptedPicker, TempDir};
use sweepscope_kernel::{Bus, ErrorCategory};

const NOW_US: f64 = 1_700_000_000_000_000.0;
const GOLDEN: &str = "tests/frame-raw-v1.bin";

fn clock() -> f64 {
    NOW_US
}

struct Rig {
    bus: Bus,
    dir: TempDir,
    picker: Arc<ScriptedPicker>,
    _module: ExportModule,
}

fn rig(answers: Vec<Option<PathBuf>>) -> Rig {
    let bus = Bus::new();
    let picker = Arc::new(ScriptedPicker::new(answers, vec![]));
    let module = ExportModule::new(ExportDeps {
        bus: bus.clone(),
        picker: picker.clone(),
        unix_time_us: clock,
    })
    .unwrap();
    Rig {
        bus,
        dir: TempDir::new(),
        picker,
        _module: module,
    }
}

fn frame() -> FrameCaptured {
    let channel = |codes: Vec<u8>, volts_per_div: f64| {
        Some(ChannelData {
            codes,
            volts_per_div,
            zero_code: 128.0,
        })
    };
    FrameCaptured {
        frame: Arc::new(Frame {
            channels: [
                channel(vec![128, 153, 178, 255], 1.0),
                None,
                channel(vec![0, 64, 128, 192], 0.5),
                None,
            ],
            counts_per_div: 25.0,
            sample_interval_s: 1e-6,
            trigger: TriggerOutcome::Triggered { sample_index: 2 },
        }),
        context: FrameContext {
            setup_revision: 1,
            window_start: 0,
            window_len: 4,
            invert: [false, false, true, false],
            probe: [1.0, 1.0, 10.0, 1.0],
            range_verified: [true, true, false, true],
            seconds_per_div: 1e-6,
        },
        derived: None,
    }
}

#[test]
fn given_a_frame_and_a_picked_path_when_csv_is_exported_then_the_file_has_metadata_header_and_rows() {
    let dir = TempDir::new();
    let target = dir.path().join("out.csv");
    let rig = rig(vec![Some(target.clone())]);
    rig.bus.publish(frame());

    let saved = rig
        .bus
        .request(ExportFrame {
            format: ExportFormat::Csv,
        })
        .unwrap();

    assert_eq!(saved.unwrap().file_name, "out.csv");
    let text = std::fs::read_to_string(target).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], "# format: sweepscope-csv 1");
    assert_eq!(lines[1], "# timestamp_utc: 2023-11-14T22:13:20Z");
    assert!(lines.contains(&"# trigger_index: 2"));
    assert!(lines.contains(&"# ch3_probe: 10"));
    assert!(lines.contains(&"# ch3_range_verified: false"));
    assert!(!lines.iter().any(|line| line.starts_with("# ch2_")));
    let header = lines.iter().position(|line| !line.starts_with('#')).unwrap();
    assert_eq!(lines[header], "time_s,ch1_v,ch3_v");
    assert_eq!(lines.len() - header - 1, 4);
    // code 153: (153 - 128) / 25 * 1 V; code 64 on ch3: (64 - 128) / 25 * 0.5 V * 10 probe.
    assert_eq!(lines[header + 2], "0.000001000000,1.000000,-12.800000");
}

#[test]
fn given_a_frame_when_csv_is_exported_then_the_suggested_name_has_the_timestamp_and_extension() {
    let rig = rig(vec![None]);
    rig.bus.publish(frame());

    let _ = rig.bus.request(ExportFrame {
        format: ExportFormat::Csv,
    });

    assert_eq!(rig.picker.suggested_names(), vec!["sweepscope-20231114-221320.csv"]);
}

#[test]
fn given_a_frame_when_raw_is_exported_then_decoding_returns_the_codes_and_header() {
    let target = TempDir::new();
    let rig = rig(vec![Some(target.path().join("cap.bin"))]);
    rig.bus.publish(frame());

    rig.bus
        .request(ExportFrame {
            format: ExportFormat::Raw,
        })
        .unwrap();

    let capture = decode_raw(&std::fs::read(target.path().join("cap.bin")).unwrap()).unwrap();
    assert_eq!(capture.record_points, 4);
    assert_eq!(capture.trigger_index, Some(2));
    assert_eq!(capture.counts_per_div, 25.0);
    assert_eq!(capture.timestamp_utc, "2023-11-14T22:13:20Z");
    assert_eq!(capture.channels.len(), 2);
    assert_eq!(capture.channels[0].channel, 1);
    assert_eq!(capture.channels[0].codes, vec![128, 153, 178, 255]);
    assert_eq!(capture.channels[1].channel, 3);
    assert_eq!(capture.channels[1].codes, vec![0, 64, 128, 192]);
    assert_eq!(capture.channels[1].probe, 10.0);
    assert!(capture.channels[1].inverted);
    assert!(!capture.channels[1].range_verified);
}

#[test]
fn given_a_frame_when_raw_is_exported_then_the_bytes_equal_the_golden_file() {
    let target = TempDir::new();
    let rig = rig(vec![Some(target.path().join("cap.bin"))]);
    rig.bus.publish(frame());
    rig.bus
        .request(ExportFrame {
            format: ExportFormat::Raw,
        })
        .unwrap();
    let written = std::fs::read(target.path().join("cap.bin")).unwrap();
    let golden = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(GOLDEN);
    if std::env::var_os("UPDATE_GENERATED").is_some() {
        std::fs::write(&golden, &written).unwrap();
    }

    assert_eq!(written, std::fs::read(golden).unwrap());
}

#[test]
fn given_the_golden_file_when_decoded_then_it_holds_the_fixture_frame() {
    let bytes = std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(GOLDEN)).unwrap();

    let capture = decode_raw(&bytes).unwrap();

    assert_eq!(capture.channels[1].codes, vec![0, 64, 128, 192]);
}

#[test]
fn given_a_raw_export_when_it_succeeds_then_the_sidecar_holds_the_layout_section() {
    let target = TempDir::new();
    let rig = rig(vec![Some(target.path().join("cap.bin"))]);
    rig.bus.publish(frame());

    rig.bus
        .request(ExportFrame {
            format: ExportFormat::Raw,
        })
        .unwrap();

    let sidecar = std::fs::read_to_string(target.path().join("cap.format.txt")).unwrap();
    assert!(sidecar.starts_with("A SweepScope raw capture (SSR1, version 1)."));
    assert!(sidecar.contains("record_points bytes"));
    assert!(!sidecar.contains("## "));
}

#[test]
fn given_a_picker_that_cancels_when_exporting_then_nothing_is_written_and_the_reply_is_none() {
    let rig = rig(vec![None, None]);
    rig.bus.publish(frame());

    let csv = rig
        .bus
        .request(ExportFrame {
            format: ExportFormat::Csv,
        })
        .unwrap();
    let raw = rig
        .bus
        .request(ExportFrame {
            format: ExportFormat::Raw,
        })
        .unwrap();

    assert!(csv.is_none() && raw.is_none());
    assert_eq!(std::fs::read_dir(rig.dir.path()).unwrap().count(), 0);
}

#[test]
fn given_a_missing_folder_when_exporting_raw_then_the_error_names_the_cause_and_leaves_no_file() {
    let rig = rig(vec![Some(PathBuf::from("/nonexistent-sweepscope-folder/cap.bin"))]);
    rig.bus.publish(frame());

    let error = rig
        .bus
        .request(ExportFrame {
            format: ExportFormat::Raw,
        })
        .unwrap_err();

    assert_eq!(error.category, ErrorCategory::Recoverable);
    assert!(error.to_string().contains("the folder does not exist"), "{error}");
}

#[cfg(unix)]
#[test]
fn given_a_read_only_folder_when_exporting_raw_then_permission_denied_and_no_file_remains() {
    use std::os::unix::fs::PermissionsExt;
    let folder = TempDir::new();
    std::fs::set_permissions(folder.path(), std::fs::Permissions::from_mode(0o555)).unwrap();
    let rig = rig(vec![Some(folder.path().join("cap.bin"))]);
    rig.bus.publish(frame());

    let result = rig.bus.request(ExportFrame {
        format: ExportFormat::Raw,
    });

    let leftovers = std::fs::read_dir(folder.path()).unwrap().count();
    std::fs::set_permissions(folder.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
    if let Err(error) = result {
        assert!(error.to_string().contains("permission denied"), "{error}");
        assert_eq!(leftovers, 0);
    }
}

fn export_raw_to(path: PathBuf) -> Result<(), sweepscope_kernel::KernelError> {
    let rig = rig(vec![Some(path)]);
    rig.bus.publish(frame());
    rig.bus
        .request(ExportFrame {
            format: ExportFormat::Raw,
        })
        .map(drop)
}

#[test]
fn given_an_existing_raw_file_when_the_sidecar_cannot_be_written_then_the_old_raw_file_is_kept() {
    let folder = TempDir::new();
    std::fs::create_dir(folder.path().join("cap.format.txt")).unwrap();
    std::fs::write(folder.path().join("cap.bin"), b"old data").unwrap();

    let error = export_raw_to(folder.path().join("cap.bin")).unwrap_err();

    assert!(error.to_string().contains("cap.format.txt"), "{error}");
    assert_eq!(std::fs::read(folder.path().join("cap.bin")).unwrap(), b"old data");
}

#[test]
fn given_an_unreadable_sidecar_when_exporting_raw_then_it_fails_naming_the_sidecar_and_writes_nothing() {
    let folder = TempDir::new();
    std::fs::create_dir(folder.path().join("cap.format.txt")).unwrap();

    let error = export_raw_to(folder.path().join("cap.bin")).unwrap_err();

    let text = error.to_string();
    assert!(
        text.contains("cap.format.txt") && text.contains("nothing was exported"),
        "{text}"
    );
    assert!(!folder.path().join("cap.bin").exists());
    assert!(folder.path().join("cap.format.txt").is_dir());
}

#[cfg(unix)]
#[test]
fn given_a_permission_denied_sidecar_when_the_export_fails_then_the_old_sidecar_is_kept() {
    use std::os::unix::fs::PermissionsExt;
    let folder = TempDir::new();
    let sidecar = folder.path().join("cap.format.txt");
    std::fs::write(&sidecar, b"old layout").unwrap();
    std::fs::set_permissions(&sidecar, std::fs::Permissions::from_mode(0o000)).unwrap();
    let readable = std::fs::read(&sidecar).is_ok();
    std::fs::create_dir(folder.path().join("cap.bin")).unwrap();

    export_raw_to(folder.path().join("cap.bin")).unwrap_err();

    std::fs::set_permissions(&sidecar, std::fs::Permissions::from_mode(0o644)).unwrap();
    if !readable {
        assert_eq!(std::fs::read(&sidecar).unwrap(), b"old layout");
    }
}

#[test]
fn given_no_raw_file_when_the_sidecar_cannot_be_written_then_no_data_file_appears() {
    let folder = TempDir::new();
    std::fs::create_dir(folder.path().join("cap.format.txt")).unwrap();

    export_raw_to(folder.path().join("cap.bin")).unwrap_err();

    assert!(!folder.path().join("cap.bin").exists());
}

#[test]
fn given_an_existing_sidecar_when_the_raw_file_cannot_be_written_then_the_old_sidecar_is_kept() {
    let folder = TempDir::new();
    std::fs::create_dir(folder.path().join("cap.bin")).unwrap();
    std::fs::write(folder.path().join("cap.format.txt"), b"old layout").unwrap();

    export_raw_to(folder.path().join("cap.bin")).unwrap_err();

    assert_eq!(
        std::fs::read(folder.path().join("cap.format.txt")).unwrap(),
        b"old layout"
    );
}

#[test]
fn given_no_sidecar_when_the_raw_file_cannot_be_written_then_no_sidecar_appears() {
    let folder = TempDir::new();
    std::fs::create_dir(folder.path().join("cap.bin")).unwrap();

    export_raw_to(folder.path().join("cap.bin")).unwrap_err();

    assert!(!folder.path().join("cap.format.txt").exists());
}

#[test]
fn given_no_frame_when_exporting_then_no_waveform_and_the_picker_is_not_opened() {
    let rig = rig(vec![]);

    let error = rig
        .bus
        .request(ExportFrame {
            format: ExportFormat::Csv,
        })
        .unwrap_err();

    assert_eq!(error.category, ErrorCategory::Recoverable);
    assert!(error.to_string().contains("No waveform to export"));
    assert!(rig.picker.suggested_names().is_empty());
}

#[test]
fn given_a_frame_without_samples_when_exporting_then_no_waveform() {
    let rig = rig(vec![]);
    let mut empty = frame();
    empty.frame = Arc::new(Frame {
        channels: [None, None, None, None],
        ..(*empty.frame).clone()
    });
    rig.bus.publish(empty);

    let error = rig
        .bus
        .request(ExportFrame {
            format: ExportFormat::Raw,
        })
        .unwrap_err();

    assert!(error.to_string().contains("No waveform to export"));
}

#[test]
fn given_two_frames_when_exporting_then_the_newest_is_written() {
    let target = TempDir::new();
    let rig = rig(vec![Some(target.path().join("a.bin"))]);
    rig.bus.publish(frame());
    let mut newer = frame();
    newer.frame = Arc::new(Frame {
        sample_interval_s: 2e-6,
        ..(*newer.frame).clone()
    });
    rig.bus.publish(newer);

    rig.bus
        .request(ExportFrame {
            format: ExportFormat::Raw,
        })
        .unwrap();

    let capture = decode_raw(&std::fs::read(target.path().join("a.bin")).unwrap()).unwrap();
    assert_eq!(capture.sample_interval_s, 2e-6);
}

#[test]
fn given_a_png_when_saving_a_screenshot_then_it_is_written_byte_for_byte() {
    let target = TempDir::new();
    let rig = rig(vec![Some(target.path().join("shot.png"))]);
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    png.extend_from_slice(&[1, 2, 3, 4, 5]);

    let saved = rig.bus.request(SaveScreenshot { png: png.clone() }).unwrap();

    assert_eq!(saved.unwrap().file_name, "shot.png");
    assert_eq!(std::fs::read(target.path().join("shot.png")).unwrap(), png);
    assert_eq!(rig.picker.suggested_names(), vec!["sweepscope-20231114-221320.png"]);
}

#[test]
fn given_a_body_that_is_not_a_png_when_saving_a_screenshot_then_it_is_refused_without_a_dialog() {
    let rig = rig(vec![]);

    let error = rig.bus.request(SaveScreenshot { png: vec![1, 2, 3] }).unwrap_err();

    assert_eq!(error.category, ErrorCategory::Recoverable);
    assert!(rig.picker.suggested_names().is_empty());
}

#[test]
fn given_a_png_over_the_limit_when_saving_a_screenshot_then_it_is_refused() {
    let rig = rig(vec![]);
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    png.resize(32 * 1024 * 1024 + 1, 0);

    let error = rig.bus.request(SaveScreenshot { png }).unwrap_err();

    assert!(error.to_string().contains("32 MiB"));
}

#[test]
fn given_a_cancelled_dialog_when_saving_a_screenshot_then_the_reply_is_none() {
    let rig = rig(vec![None]);

    let saved = rig
        .bus
        .request(SaveScreenshot {
            png: b"\x89PNG\r\n\x1a\n".to_vec(),
        })
        .unwrap();

    assert!(saved.is_none());
}

#[test]
fn given_a_truncated_or_foreign_file_when_decoded_then_it_is_refused() {
    assert!(decode_raw(b"SSR1").is_err());
    assert!(decode_raw(b"NOPE\0\0\0\0").is_err());
    let mut bytes = std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(GOLDEN)).unwrap();
    bytes.pop();
    assert!(decode_raw(&bytes).is_err());
}
