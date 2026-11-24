use std::io::{stderr, stdout};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let env = |name: &str| std::env::var(name).ok();
    ExitCode::from(sweepscope_cli::run(&args, &env, &mut stdout(), &mut stderr()))
}
