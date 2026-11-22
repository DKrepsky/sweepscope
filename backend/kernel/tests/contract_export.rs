use std::path::Path;

use sweepscope_kernel::KernelError;
use ts_rs::{Config, TS};

#[test]
fn given_the_kernel_error_when_exported_then_the_generated_file_has_category_and_message() {
    KernelError::export_all(&Config::from_env()).unwrap();

    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../frontend/src/api/generated/KernelError.ts");
    let text = std::fs::read_to_string(file).unwrap();
    assert!(
        text.contains("category: ErrorCategory") && text.contains("message: string"),
        "{text}"
    );
}
