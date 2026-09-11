//! T014a/T036 - Unit tests for edge cases: empty dir, no Python files,
//! non-existent input path.

use std::fs;
use std::path::PathBuf;

use pydoc_gen::error::Error;
use pydoc_gen::model::DocComment;
use pydoc_gen::pipeline::build_project;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pydoc_edges_{}_{}",
        name,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    dir
}

#[test]
fn empty_directory_produces_empty_site() {
    let input = temp_dir("empty");
    fs::create_dir_all(&input).unwrap();

    let project =
        build_project(&input).expect("build_project succeeds on empty dir");
    assert!(project.elements.is_empty());
    assert!(project.index.is_empty());

    let _ = fs::remove_dir_all(&input);
}

#[test]
fn directory_with_no_python_files_completes() {
    let input = temp_dir("nopy");
    fs::create_dir_all(&input).unwrap();
    fs::write(input.join("readme.txt"), "not python").unwrap();

    let project = build_project(&input).expect("no_python completes");
    assert!(project.elements.is_empty());

    let _ = fs::remove_dir_all(&input);
}

#[test]
fn legacy_tag_docstring_is_an_error_not_a_panic() {
    let input = temp_dir("legacytags");
    fs::create_dir_all(&input).unwrap();
    fs::write(
        input.join("legacy.py"),
        "def f():\n    \"\"\"@arg x  the input\n    \"\"\"\n    pass\n",
    )
    .unwrap();

    let project = build_project(&input).expect("legacy tags do not crash");
    let el = project
        .elements
        .iter()
        .flat_map(|m| m.children.iter())
        .find(|e| e.path.text().ends_with("f"))
        .expect("function documented");
    let doc = el.doc.clone();
    assert!(
        matches!(doc, DocComment::Error(_)),
        "legacy tag docstring must be an error entry, got {doc:?}"
    );

    let _ = fs::remove_dir_all(&input);
}

#[test]
fn non_existent_input_path_is_a_clear_error() {
    let missing = "/nonexistent/path/that/does/not/exist";
    let err = build_project(std::path::Path::new(missing))
        .expect_err("non-existent input must error");
    assert!(
        matches!(err, Error::InvalidInputPath(_)),
        "expected InvalidInputPath, got {err}"
    );
}
