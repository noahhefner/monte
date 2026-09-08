//! T014 - Contract test: documented + undocumented element output.

use std::fs;
use std::path::{Path, PathBuf};

use pydoc_gen::model::{DocComment, ImportPath};
use pydoc_gen::pipeline::{build_project, generate};

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pydoc_contract_{}_{}",
        name,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    dir
}

fn write_sample(dir: &Path) {
    fs::create_dir_all(dir.join("sample")).unwrap();
    fs::write(
        dir.join("sample/__init__.py"),
        "\"\"\"The sample package.\"\"\"\n",
    )
    .unwrap();
    // Documented function.
    fs::write(
        dir.join("sample/formatting.py"),
        "def format_value(value):\n    \"\"\"Formatting helpers.\n\n    @arg value  The value to format.\n    @return str  The formatted output.\n    @raises ValueError  Raised on empty.\n    \"\"\"\n    return value\n",
    )
    .unwrap();
    // Undocumented function (existence-only).
    fs::write(
        dir.join("sample/util.py"),
        "def undocumented_helper():\n    return 1\n",
    )
    .unwrap();
}

#[test]
fn generates_documented_and_undocumented_entries() {
    let input = temp_dir("in");
    write_sample(&input);

    let project = build_project(&input).expect("build_project");

    let fmt_path = ImportPath::new(vec![
        "sample".into(),
        "formatting".into(),
        "format_value".into(),
    ]);
    let fmt = project
        .index
        .get(&fmt_path)
        .expect("documented function in index");
    match &fmt.doc {
        DocComment::Ok(content) => {
            assert!(content.description.contains("Formatting helpers."));
            assert_eq!(content.args.len(), 1);
            assert!(content.returns.is_some());
            assert_eq!(content.raises.len(), 1);
        }
        other => panic!("expected ok, got {other:?}"),
    }

    // The package's own docstring is captured.
    let pkg_path = ImportPath::new(vec!["sample".into()]);
    match &project.index.get(&pkg_path).expect("package in index").doc {
        DocComment::Ok(content) => {
            assert!(content.description.contains("The sample package."))
        }
        other => panic!("expected ok package doc, got {other:?}"),
    }

    let undoc_path = ImportPath::new(vec![
        "sample".into(),
        "util".into(),
        "undocumented_helper".into(),
    ]);
    let undoc = project
        .index
        .get(&undoc_path)
        .expect("undocumented function in index");
    assert_eq!(undoc.doc, DocComment::None);

    let _ = fs::remove_dir_all(&input);
}

#[test]
fn writes_html_covering_all_elements() {
    let input = temp_dir("in2");
    write_sample(&input);
    let output = temp_dir("out");

    generate(&input, &output, None).expect("generate");

    // The index links every element by full path to its page.
    let index_html =
        fs::read_to_string(output.join("index.html")).expect("index.html");
    assert!(
        index_html
            .contains("sample/formatting.html#sample.formatting.format_value")
    );
    assert!(index_html.contains("sample.util.undocumented_helper"));

    // Documented function content lives on the module page.
    let fmt_page = fs::read_to_string(output.join("sample/formatting.html"))
        .expect("formatting.html");
    assert!(fmt_page.contains("format_value"));
    assert!(fmt_page.contains("The value to format."));

    // Undocumented element has an existence-only entry.
    let util_page =
        fs::read_to_string(output.join("sample/util.html")).expect("util.html");
    assert!(util_page.contains("undocumented_helper"));
    assert!(util_page.contains("No documentation provided."));

    let _ = fs::remove_dir_all(&input);
    let _ = fs::remove_dir_all(&output);
}

#[test]
fn malformed_docstring_renders_error_without_aborting_run() {
    // FR-007: a malformed block (e.g. `@arg` with no name) yields an
    // in-entry error and does NOT abort the run; other elements still render.
    let input = temp_dir("in3");
    fs::create_dir_all(input.join("sample")).unwrap();
    fs::write(
        input.join("sample/__init__.py"),
        "\"\"\"The sample package.\"\"\"\n",
    )
    .unwrap();
    fs::write(
        input.join("sample/ok.py"),
        "def good(x):\n    \"\"\"Fine docs.\n\n    @arg x  input\n    \"\"\"\n    pass\n",
    )
    .unwrap();
    fs::write(
        input.join("sample/bad.py"),
        "def broken(arg):\n    \"\"\"Broken docs.\n\n    @arg\n    \"\"\"\n    pass\n",
    )
    .unwrap();

    let output = temp_dir("out3");
    generate(&input, &output, None)
        .expect("generate succeeds despite malformed docstring");

    // The malformed element's page carries the error and does not abort.
    let bad_page =
        fs::read_to_string(output.join("sample/bad.html")).expect("bad.html");
    assert!(bad_page.contains("docstring was not written correctly"));
    assert!(bad_page.contains("broken"));

    // The healthy function is still fully documented on its own page.
    let ok_page =
        fs::read_to_string(output.join("sample/ok.html")).expect("ok.html");
    assert!(ok_page.contains("Fine docs."));

    // The index still lists both elements.
    let index_html =
        fs::read_to_string(output.join("index.html")).expect("index.html");
    assert!(index_html.contains("sample.ok.good"));
    assert!(index_html.contains("sample.bad.broken"));

    let _ = fs::remove_dir_all(&input);
    let _ = fs::remove_dir_all(&output);
}
