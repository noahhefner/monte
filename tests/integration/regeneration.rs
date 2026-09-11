//! T022/T023/T024 - Integration tests for regeneration: deterministic
//! byte-for-byte output, updated docstrings, and removed elements (FR-006).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use pydoc_gen::model::{DocComment, ImportPath};
use pydoc_gen::pipeline::build_project;

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "pydoc_regen_{}_{}_{}",
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

fn write_fixture(input: &Path) {
    fs::create_dir_all(input.join("samplepkg")).unwrap();
    fs::write(
        input.join("samplepkg/__init__.py"),
        "\"\"\"Sample package.\"\"\"\n",
    )
    .unwrap();
    fs::write(
        input.join("samplepkg/utils.py"),
        "def zeta():\n    \"\"\"First utility.\n    \"\"\"\n    pass\n\n\
         \ndef alpha():\n    \"\"\"Second utility.\n    \"\"\"\n    pass\n",
    )
    .unwrap();
    fs::write(
        input.join("samplepkg/extra.py"),
        "def top_extra():\n    \"\"\"Extra helper.\n    \"\"\"\n    pass\n",
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

fn relative_files(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    fn walk(dir: &Path, root: &Path, out: &mut Vec<String>) {
        for entry in fs::read_dir(dir).unwrap() {
            let p = entry.unwrap().path();
            if p.is_dir() {
                walk(&p, root, out);
            } else {
                out.push(
                    p.strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
    }
    walk(root, root, &mut out);
    out.sort();
    out
}

#[test]
fn unchanged_source_generates_byte_identical_output() {
    let binary = env!("CARGO_BIN_EXE_pydoc-gen");
    let root = temp_root("determinism");
    let input = root.join("in");
    let out1 = root.join("out1");
    let out2 = root.join("out2");
    write_fixture(&input);

    run(binary, &input, &out1);
    run(binary, &input, &out2);

    let first = fs::read(out1.join("index.html")).unwrap();
    let second = fs::read(out2.join("index.html")).unwrap();
    assert_eq!(
        first, second,
        "unchanged source must produce identical HTML (FR-006, SC-004)"
    );
    assert_eq!(
        relative_files(&out1),
        relative_files(&out2),
        "unchanged source must produce identical output file sets"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn rerun_over_existing_output_is_idempotent() {
    let binary = env!("CARGO_BIN_EXE_pydoc-gen");
    let root = temp_root("idempotent");
    let input = root.join("in");
    let output = root.join("out");
    write_fixture(&input);

    run(binary, &input, &output);
    let before = fs::read(output.join("index.html")).unwrap();

    run(binary, &input, &output);
    let after = fs::read(output.join("index.html")).unwrap();

    assert_eq!(
        before, after,
        "re-run into the same output dir must not change bytes"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn updated_docstring_is_reflected_on_regeneration() {
    let binary = env!("CARGO_BIN_EXE_pydoc-gen");
    let root = temp_root("update");
    let input = root.join("in");
    let output = root.join("out");
    write_fixture(&input);

    run(binary, &input, &output);

    fs::write(
        input.join("samplepkg/utils.py"),
        "def zeta():\n    \"\"\"First utility, v2.\n    \"\"\"\n    pass\n\n\
         \ndef alpha():\n    \"\"\"Second utility.\n    \"\"\"\n    pass\n",
    )
    .unwrap();

    run(binary, &input, &output);

    // The rebuilt representation reflects the updated docstring.
    let project = build_project(&input).expect("build_project");
    let zeta = project
        .index
        .get(&ImportPath::new(vec![
            "samplepkg".into(),
            "utils".into(),
            "zeta".into(),
        ]))
        .expect("zeta in index");
    match &zeta.doc {
        DocComment::Ok(content) => {
            assert!(content.description.contains("First utility, v2."));
        }
        other => panic!("expected ok, got {other:?}"),
    }
    let alpha = project
        .index
        .get(&ImportPath::new(vec![
            "samplepkg".into(),
            "utils".into(),
            "alpha".into(),
        ]))
        .expect("alpha in index");
    match &alpha.doc {
        DocComment::Ok(content) => {
            assert!(content.description.contains("Second utility."));
        }
        other => panic!("expected ok, got {other:?}"),
    }

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn removed_module_disappears_from_regenerated_output() {
    let binary = env!("CARGO_BIN_EXE_pydoc-gen");
    let root = temp_root("remove");
    let input = root.join("in");
    let output = root.join("out");
    write_fixture(&input);

    run(binary, &input, &output);
    let first_run = fs::read_to_string(output.join("index.html")).unwrap();
    assert!(first_run.contains("samplepkg.extra"));

    fs::remove_file(input.join("samplepkg/extra.py")).unwrap();

    run(binary, &input, &output);
    let second_run = fs::read_to_string(output.join("index.html")).unwrap();
    assert!(
        !second_run.contains("samplepkg.extra"),
        "a removed module must not leave stale output (T025)"
    );
    assert!(
        second_run.contains("samplepkg.utils"),
        "remaining modules must still be documented"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn elements_are_path_sorted_regardless_of_source_order() {
    let binary = env!("CARGO_BIN_EXE_pydoc-gen");
    let root = temp_root("ordered");
    let input = root.join("in");
    let output = root.join("out");
    write_fixture(&input);

    run(binary, &input, &output);
    let html = fs::read_to_string(output.join("index.html")).unwrap();

    let alpha = html.find("samplepkg.utils.alpha").expect("alpha absent");
    let zeta = html.find("samplepkg.utils.zeta").expect("zeta absent");
    assert!(
        alpha < zeta,
        "output must be path-sorted: alpha should render before zeta (FR-006)"
    );

    let _ = fs::remove_dir_all(&root);
}
