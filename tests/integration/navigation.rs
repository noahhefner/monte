//! T027/T028 - Integration tests for index navigation: every element is
//! reachable from the index by full path, and duplicate simple names in
//! different modules are resolved unambiguously (FR-005, SC-003, SC-006).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pydoc_nav_{}_{}_{}",
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

fn write_package(input: &Path) {
    fs::create_dir_all(input.join("pkg")).unwrap();
    fs::write(
        input.join("pkg/__init__.py"),
        "\"\"\"Package with duplicates.\"\"\"\n",
    )
    .unwrap();
    fs::write(
        input.join("pkg/a.py"),
        "class Helper:\n    \"\"\"Helper A.\n    \"\"\"\n\n    def go(self):\n        \"\"\"Run A.\n        \"\"\"\n        pass\n",
    )
    .unwrap();
    fs::write(
        input.join("pkg/b.py"),
        "class Helper:\n    \"\"\"Helper B.\n    \"\"\"\n    pass\n",
    )
    .unwrap();
}

fn run(binary: &str, input: &Path, output: &Path) {
    let status = Command::new(binary)
        .arg(input)
        .arg(output)
        .status()
        .expect("failed to run binary");
    assert!(status.success(), "binary failed for {output:?}");
}

#[test]
fn index_reaches_every_documented_element() {
    let binary = env!("CARGO_BIN_EXE_pydoc-gen");
    let root = temp_root("coverage");
    let input = root.join("in");
    let output = root.join("out");
    write_package(&input);

    run(binary, &input, &output);

    let index_html =
        fs::read_to_string(output.join("index.html")).expect("index.html");
    // Modules link to their own pages.
    assert!(index_html.contains("href=\"pkg/a.html#pkg.a\""));
    assert!(index_html.contains("href=\"pkg/b.html#pkg.b\""));
    // Classes and methods link to their containing module's anchors.
    assert!(index_html.contains("href=\"pkg/a.html#pkg.a.Helper\""));
    assert!(index_html.contains("href=\"pkg/b.html#pkg.b.Helper\""));
    assert!(index_html.contains("href=\"pkg/a.html#pkg.a.Helper.go\""));

    // Module pages exist and carry content.
    let a_page =
        fs::read_to_string(output.join("pkg/a.html")).expect("pkg/a.html");
    assert!(a_page.contains("Helper A."));
    assert!(a_page.contains("href=\"index.html\""));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn duplicate_simple_names_resolved_by_full_path() {
    let binary = env!("CARGO_BIN_EXE_pydoc-gen");
    let root = temp_root("duplicates");
    let input = root.join("in");
    let output = root.join("out");
    write_package(&input);

    run(binary, &input, &output);

    let index_html =
        fs::read_to_string(output.join("index.html")).expect("index.html");
    assert!(
        index_html.contains("pkg/a.html#pkg.a.Helper"),
        "both same-named classes must be listed with distinct full paths"
    );
    assert!(
        index_html.contains("pkg/b.html#pkg.b.Helper"),
        "both same-named classes must be listed with distinct full paths"
    );

    let a_page =
        fs::read_to_string(output.join("pkg/a.html")).expect("pkg/a.html");
    let b_page =
        fs::read_to_string(output.join("pkg/b.html")).expect("pkg/b.html");
    assert!(a_page.contains("pkg.a.Helper"), "A page shows its own path");
    assert!(b_page.contains("pkg.b.Helper"), "B page shows its own path");

    let _ = fs::remove_dir_all(&root);
}
