//! T035a - Baseline benchmark of the parse -> extract -> generate pipeline.
//!
//! Builds a synthetic project (200 modules, 10 functions each) and measures
//! how long the full pipeline takes. SC-001 requires a site to be generated
//! in under 60 seconds. Run with `cargo bench`. Baseline results are
//! recorded in specs/001-pydoc-generator/bench-results.md.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use pydoc_gen::pipeline::{build_project, generate};

const ITERATIONS: usize = 5;
const MODULES: usize = 200;
const FUNCTIONS_PER_MODULE: usize = 10;

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pydoc_bench_{}_{}_{}",
        std::process::id(),
        tag,
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn create_project(root: &Path) {
    for m in 0..MODULES {
        let dir = root.join(format!("mod{}", m / 10));
        fs::create_dir_all(&dir).unwrap();
        let mut source = format!(
            "class Mod{m}:\n    \"\"\"Module {m} docs.\n    \"\"\"\n    pass\n\n"
        );
        for f in 0..FUNCTIONS_PER_MODULE {
            source.push_str(&format!(
                "def func_{f}(a, b):\n    \"\"\"Function {f}.\n\n    \
                 @arg a  First value.\n    @arg b  Second value.\n    \
                 @return int  The sum.\n    \"\"\"\n    return a + b\n\n"
            ));
        }
        fs::write(dir.join(format!("mod{m}.py")), source).unwrap();
    }
}

fn main() {
    let input = temp_dir("in");
    let output = temp_dir("out");
    create_project(&input);

    let project = build_project(&input).expect("build");
    println!(
        "synthetic project: {} modules, {} elements",
        MODULES,
        project.elements.len()
    );

    let mut times: Vec<Duration> = Vec::new();
    for _ in 0..ITERATIONS {
        let _ = fs::remove_dir_all(&output);
        let start = Instant::now();
        generate(&input, &output).expect("generate");
        times.push(start.elapsed());
    }

    let min = *times.iter().min().unwrap();
    let mean = times.iter().sum::<Duration>() / ITERATIONS as u32;
    for (i, t) in times.iter().enumerate() {
        println!("iteration {i}: {t:?}");
    }
    println!("min generate time: {min:?}");
    println!("mean generate time: {mean:?}");
    println!(
        "sc-001 (site generated in under 60 seconds): {}",
        if min.as_secs() < 60 { "PASS" } else { "FAIL" }
    );

    let _ = fs::remove_dir_all(&input);
    let _ = fs::remove_dir_all(&output);
}
