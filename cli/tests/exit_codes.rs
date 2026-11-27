use sweepscope_cli::run;

fn run_with(args: &[&str]) -> (u8, String, String) {
    let args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(&args, &|_| None, &mut out, &mut err);
    (code, String::from_utf8(out).unwrap(), String::from_utf8(err).unwrap())
}

#[test]
fn given_a_bad_option_when_run_then_the_exit_code_is_2_and_the_usage_is_printed() {
    let (code, out, err) = run_with(&["capture", "--setup", "s.json", "--bogus"]);

    assert_eq!(code, 2);
    assert!(out.is_empty());
    assert!(
        err.contains("unknown option --bogus") && err.contains("usage:"),
        "{err}"
    );
}

#[test]
fn given_help_when_run_then_the_usage_goes_to_stdout_and_the_exit_code_is_0() {
    let (code, out, _) = run_with(&["--help"]);

    assert_eq!(code, 0);
    assert!(out.contains("exit codes:"));
}

#[test]
fn given_version_when_run_then_the_version_goes_to_stdout_and_the_exit_code_is_0() {
    let (code, out, err) = run_with(&["--version"]);

    assert_eq!(code, 0);
    assert_eq!(out, format!("sweepscope {}\n", env!("CARGO_PKG_VERSION")));
    assert!(err.is_empty(), "{err}");
}

#[test]
fn given_an_unknown_capture_option_when_run_then_the_error_and_usage_name_the_application() {
    let (code, _, err) = run_with(&["capture", "--bogus"]);

    assert_eq!(code, 2);
    assert!(err.starts_with("sweepscope: unknown option --bogus\n"), "{err}");
    assert!(
        err.contains("usage: sweepscope capture --setup FILE [options]"),
        "{err}"
    );
}

#[test]
fn given_no_config_folder_available_when_run_then_the_exit_code_is_1() {
    let (code, _, err) = run_with(&["capture", "--setup", "s.json", "--simulator"]);

    assert_eq!(code, 1);
    assert!(err.contains("--config-dir"), "{err}");
}
