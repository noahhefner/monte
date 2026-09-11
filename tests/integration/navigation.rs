//! T027/T028 - Integration tests for index navigation: every element is
//! reachable from the index by full path, and duplicate simple names in
//! different modules are resolved unambiguously (FR-005, SC-003, SC-006).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use pydoc_gen::model::{DocComment, ImportPath};
use pydoc_gen::pipeline::build_project;

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
    // Every element is reachable from the index by full path.
    assert!(index_html.contains("pkg.a"), "module pkg.a listed");
    assert!(index_html.contains("pkg.b"), "module pkg.b listed");
    assert!(index_html.contains("pkg.a.Helper"), "class Helper in pkg.a");
    assert!(index_html.contains("pkg.b.Helper"), "class Helper in pkg.b");
    assert!(
        index_html.contains("pkg.a.Helper.go"),
        "method go in pkg.a.Helper"
    );

    // The representation carries correct docstrings for each element.
    let project = build_project(&input).expect("build_project");
    let a_helper = project
        .index
        .get(&ImportPath::new(vec![
            "pkg".into(),
            "a".into(),
            "Helper".into(),
        ]))
        .expect("Helper in pkg.a");
    match &a_helper.doc {
        DocComment::Ok(content) => {
            assert!(content.description.contains("Helper A."));
        }
        other => panic!("expected ok, got {other:?}"),
    }

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
        index_html.contains("pkg.a.Helper"),
        "both same-named classes must be listed with distinct full paths"
    );
    assert!(
        index_html.contains("pkg.b.Helper"),
        "both same-named classes must be listed with distinct full paths"
    );

    let project = build_project(&input).expect("build_project");
    let a_helper = project
        .index
        .get(&ImportPath::new(vec![
            "pkg".into(),
            "a".into(),
            "Helper".into(),
        ]))
        .expect("Helper in pkg.a");
    let b_helper = project
        .index
        .get(&ImportPath::new(vec![
            "pkg".into(),
            "b".into(),
            "Helper".into(),
        ]))
        .expect("Helper in pkg.b");
    match (&a_helper.doc, &b_helper.doc) {
        (DocComment::Ok(a), DocComment::Ok(b)) => {
            assert!(a.description.contains("Helper A."));
            assert!(b.description.contains("Helper B."));
        }
        (a, b) => panic!("expected both Ok, got a={a:?} b={b:?}"),
    }

    let _ = fs::remove_dir_all(&root);
}
