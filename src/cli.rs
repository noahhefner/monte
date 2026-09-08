//! Command-line interface wiring (contracts/cli.md).
//!
//! Usage: `pydoc-gen <input_dir> <output_dir> [--theme <css filename>]`

use std::path::Path;

use crate::error::{Error, Result};
use crate::pipeline;
use crate::theme::resolve_theme;

/// Parse command-line arguments and run the pipeline.
pub fn run() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 && args.len() != 5 {
        return Err(Error::Message(
            "usage: pydoc-gen <input_dir> <output_dir> [--theme <css filename>]"
                .to_string(),
        ));
    }

    let input = Path::new(&args[1]);
    let output = Path::new(&args[2]);

    let theme_path = if args.len() == 5 {
        if args[3] != "--theme" {
            return Err(Error::Message(
                "usage: pydoc-gen <input_dir> <output_dir> [--theme <css filename>]"
                    .to_string(),
            ));
        }

        Some(Path::new(&args[4]))
    } else {
        None
    };

    let theme = resolve_theme(theme_path)?;
    let project = pipeline::build_project(input)?;
    pipeline::write_site(&project, output, &theme)?;

    println!(
        "generated documentation for {} module(s) at {}",
        project.elements.len(),
        output.display()
    );

    Ok(())
}
