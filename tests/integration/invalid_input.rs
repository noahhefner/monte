//! T014b - Integration test: non-existent input path returns a clear error
//! and a non-zero exit.

use std::process::Command;

#[test]
fn non_existent_input_path_exits_nonzero_with_error() {
    let binary = env!("CARGO_BIN_EXE_pydoc-gen");
    let missing = "/nonexistent/path/that/does/not/exist";
    let out_dir = std::env::temp_dir().join("pydoc_invalid_out");

    let output = Command::new(binary)
        .arg(missing)
        .arg(&out_dir)
        .output()
        .expect("failed to run binary");

    assert!(
        !output.status.success(),
        "expected non-zero exit, got {:?}",
        output.status
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("input path")
            && stderr.to_lowercase().contains("error"),
        "expected a clear error on stderr, got: {stderr}"
    );
}
