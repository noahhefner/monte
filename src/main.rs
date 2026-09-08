//! Binary entry point for pydoc-gen.

use std::process::ExitCode;

fn main() -> ExitCode {
    // Full CLI wiring is implemented in cli.rs and invoked here (T019).
    match pydoc_gen::cli::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
