//! Command-line interface wiring (contracts/cli.md).
//!
//! Usage: `pydoc-gen <input_dir> <output_dir> [--theme <css filename>]`

use std::path::Path;

use crate::error::Result;
use crate::pipeline;
use crate::theme::resolve_theme;
use clap::Parser;
use std::path::PathBuf;

/// Documentation generator for Python
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Directory with Python source code
    input_dir: String,

    /// Directory to place documentation in
    output_dir: String,

    /// Custom CSS for generated documentation
    #[arg(short, long)]
    theme_path: Option<PathBuf>,
}

/// Parse command-line arguments and run the pipeline.
pub fn run() -> Result<()> {
    let args = Args::parse();

    let input = Path::new(&args.input_dir);
    let output = Path::new(&args.output_dir);

    let theme = resolve_theme(args.theme_path.as_deref())?;

    let project = pipeline::build_project(input)?;
    pipeline::write_site(&project, output, &theme)?;

    println!(
        "generated documentation for {} module(s) at {}",
        project.elements.len(),
        output.display()
    );

    Ok(())
}
