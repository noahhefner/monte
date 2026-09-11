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
    // Documented function (also exercises tolerance of unknown keys such as
    // `example` and `returns.type`, see contracts/docstring-syntax.md).
    fs::write(
        dir.join("sample/formatting.py"),
        "def format_value(value):\n    \"\"\"\n    description: Formatting helpers.\n\n    args:\n      - name: value\n        description: The value to format.\n\n    returns:\n      type: str\n      description: The formatted output.\n\n    raises:\n      - type: ValueError\n        description: Raised on empty.\n\n    example: |\n      format_value(1)\n    \"\"\"\n    return value\n",
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
            assert_eq!(content.args[0].name, "value");
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
fn writes_html_index_covering_all_elements() {
    let input = temp_dir("in2");
    write_sample(&input);
    let output = temp_dir("out");

    generate(&input, &output, None).expect("generate");

    // The index links every element by full path.
    let index_html =
        fs::read_to_string(output.join("index.html")).expect("index.html");
    assert!(index_html.contains("sample.formatting.format_value"));
    assert!(index_html.contains("sample.util.undocumented_helper"));

    // Documented elements are present even when there is no prose content.
    let project = build_project(&input).expect("build_project");
    let fmt = project
        .index
        .get(&ImportPath::new(vec![
            "sample".into(),
            "formatting".into(),
            "format_value".into(),
        ]))
        .expect("format_value in index");
    match &fmt.doc {
        DocComment::Ok(content) => {
            assert!(content.description.contains("Formatting helpers."));
        }
        other => panic!("expected ok, got {other:?}"),
    }

    let undoc = project
        .index
        .get(&ImportPath::new(vec![
            "sample".into(),
            "util".into(),
            "undocumented_helper".into(),
        ]))
        .expect("undocumented_helper in index");
    assert_eq!(undoc.doc, DocComment::None);

    let _ = fs::remove_dir_all(&input);
    let _ = fs::remove_dir_all(&output);
}

#[test]
fn malformed_docstring_yields_inrun_error_without_aborting() {
    // FR-007: a malformed docstring (here an empty legacy `@arg` tag) yields
    // an in-entry error and does NOT abort the run; other elements still
    // render.
    let input = temp_dir("in3");
    fs::create_dir_all(input.join("sample")).unwrap();
    fs::write(
        input.join("sample/__init__.py"),
        "\"\"\"\nThe sample package.\n\"\"\"\n",
    )
    .unwrap();
    fs::write(
        input.join("sample/ok.py"),
        "def good(x):\n    \"\"\"\n    description: Fine docs.\n\n    args:\n      - name: x\n        description: input\n    \"\"\"\n    pass\n",
    )
    .unwrap();
    fs::write(
        input.join("sample/bad.py"),
        "def broken(arg):\n    \"\"\"\n    Broken docs.\n\n    @arg\n    \"\"\"\n    pass\n",
    )
    .unwrap();

    let output = temp_dir("out3");
    generate(&input, &output, None)
        .expect("generate succeeds despite malformed docstring");

    // The malformed element is still indexed, with an error entry.
    let project = build_project(&input).expect("build_project");
    let bad = project
        .index
        .get(&ImportPath::new(vec![
            "sample".into(),
            "bad".into(),
            "broken".into(),
        ]))
        .expect("broken in index");
    match &bad.doc {
        DocComment::Error(msg) => {
            assert!(msg.contains("docstring was not written correctly"));
        }
        other => panic!("expected error, got {other:?}"),
    }

    // The healthy function is still fully documented.
    let good = project
        .index
        .get(&ImportPath::new(vec![
            "sample".into(),
            "ok".into(),
            "good".into(),
        ]))
        .expect("good in index");
    match &good.doc {
        DocComment::Ok(content) => {
            assert!(content.description.contains("Fine docs."));
            assert_eq!(content.args.len(), 1);
        }
        other => panic!("expected ok, got {other:?}"),
    }

    // The index still lists both elements.
    let index_html =
        fs::read_to_string(output.join("index.html")).expect("index.html");
    assert!(index_html.contains("sample.ok.good"));
    assert!(index_html.contains("sample.bad.broken"));

    let _ = fs::remove_dir_all(&input);
    let _ = fs::remove_dir_all(&output);
}
