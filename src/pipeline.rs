//! End-to-end pipeline orchestration (FR-001, FR-009).
//!
//! Scans a source directory for Python files, parses each into the
//! intermediary representation, and writes the generated HTML site.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::extract;
use crate::index;
use crate::model::{DocumentedElement, DocumentedProject, ImportPath};
use crate::parser;
use crate::path;
use crate::theme::{self, Theme};

/// Run the full pipeline: scan `input`, build the representation, and render
/// HTML into `output`, applying the resolved theme (`theme_path` is the
/// optional user stylesheet, contracts/cli.md).
pub fn generate(
    input: &Path,
    output: &Path,
    theme_path: Option<&Path>,
) -> Result<()> {
    let theme = theme::resolve_theme(theme_path)?;
    let project = build_project(input)?;
    write_site(&project, output, &theme)
}

/// Write the generated HTML site for `project` into `output`, creating the
/// directory if needed.
///
/// The site is one index/navigation page plus one HTML page per module
/// (FR-005). Every page embeds the resolved `theme`'s stylesheet (FR-008,
/// FR-012, SC-007). The output directory is fully regenerated on every run:
/// stale generated HTML for modules removed from the source is deleted
/// (FR-006).
pub fn write_site(
    project: &DocumentedProject,
    output: &Path,
    theme: &Theme,
) -> Result<()> {
    if output.exists() && !output.is_dir() {
        return Err(Error::WriteOutput(
            output.display().to_string(),
            io::Error::from(io::ErrorKind::NotADirectory),
        ));
    }
    fs::create_dir_all(output)
        .map_err(|e| Error::WriteOutput(output.display().to_string(), e))?;

    let mut written: Vec<String> = Vec::new();
    
    write_file(
        output,
        "index.html",
        &index::render_index(project, &theme.css),
        &mut written,
    )?;

    // TODO: Render pages for individual modules

    remove_stale_html(output, &written)
}

/// Write `content` to `<output>/<rel>`, creating parent directories, and
/// record the relative path in `written`.
fn write_file(
    output: &Path,
    rel: &str,
    content: &str,
    written: &mut Vec<String>,
) -> Result<()> {
    let target = output.join(rel);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| Error::WriteOutput(parent.display().to_string(), e))?;
    }
    fs::write(&target, content)
        .map_err(|e| Error::WriteOutput(target.display().to_string(), e))?;
    written.push(rel.to_string());
    Ok(())
}

/// Delete generated HTML files under `output` that were not written in this
/// run (stale pages from removed modules), then prune empty directories.
fn remove_stale_html(output: &Path, written: &[String]) -> Result<()> {
    let mut files: Vec<String> = Vec::new();
    let mut dirs: Vec<PathBuf> = Vec::new();
    collect_html_files(output, output, &mut files, &mut dirs)
        .map_err(|e| Error::WriteOutput(output.display().to_string(), e))?;

    for rel in files {
        if !written.iter().any(|w| w == &rel) {
            let target = output.join(&rel);
            fs::remove_file(&target).map_err(|e| {
                Error::WriteOutput(target.display().to_string(), e)
            })?;
        }
    }

    dirs.sort_by_key(|d| std::cmp::Reverse(d.components().count()));
    for dir in dirs {
        let empty = fs::read_dir(&dir)
            .map(|mut it| it.next().is_none())
            .unwrap_or(false);
        if empty {
            let _ = fs::remove_dir(&dir);
        }
    }
    Ok(())
}

/// Walk `dir`, collecting relative `.html` file paths and directory paths.
fn collect_html_files(
    dir: &Path,
    root: &Path,
    files: &mut Vec<String>,
    dirs: &mut Vec<PathBuf>,
) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            dirs.push(path.clone());
            collect_html_files(&path, root, files, dirs)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("html") {
            files.push(
                path.strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }
    Ok(())
}

/// Scan `input` and build the `DocumentedProject` representation.
pub fn build_project(input: &Path) -> Result<DocumentedProject> {
    if !input.exists() || !input.is_dir() {
        return Err(Error::InvalidInputPath(input.display().to_string()));
    }

    let files = collect_py_files(input)?;
    let mut elements: Vec<DocumentedElement> = Vec::new();
    let mut root_path = ImportPath::new(Vec::new());

    for file in &files {
        let relative = file.strip_prefix(input).unwrap_or(file);
        let Some(segments) = path::module_path_from_relative(relative) else {
            continue;
        };
        let module_path = ImportPath::new(segments);

        if root_path.segments.is_empty() && !module_path.segments.is_empty() {
            root_path = ImportPath::new(vec![module_path.segments[0].clone()]);
        }

        let source = fs::read_to_string(file)
            .map_err(|e| Error::ReadFile(file.display().to_string(), e))?;
        let parsed = parser::parse_source(&source)?;
        let module = extract::module_element(
            &module_path,
            &parsed.elements,
            parsed.module_doc,
        );
        elements.push(module);
    }

    for el in &mut elements {
        el.sort_by_path();
    }
    elements.sort_by(|a, b| a.path.cmp(&b.path));

    let mut project = DocumentedProject {
        root_path,
        elements,
        index: Default::default(),
    };
    project.rebuild_index();
    Ok(project)
}

/// Recursively collect `.py` files under `dir`, sorted for determinism.
fn collect_py_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = Vec::new();
    collect_py_files_inner(dir, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_py_files_inner(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries = fs::read_dir(dir)
        .map_err(|e| Error::ReadFile(dir.display().to_string(), e))?;
    for entry in entries {
        let entry =
            entry.map_err(|e| Error::ReadFile(dir.display().to_string(), e))?;
        let path = entry.path();
        if path.is_dir() {
            // Skip common VCS/build directories.
            let skip =
                path.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
                    matches!(
                        n,
                        ".git" | ".hg" | ".svn" | "target" | "node_modules"
                    )
                });
            if skip {
                continue;
            }
            collect_py_files_inner(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("py") {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn builds_project_from_directory() {
        let dir = std::env::temp_dir()
            .join(format!("pydoc_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("pkg")).unwrap();
        fs::write(dir.join("pkg/__init__.py"), "\"\"\"Package docs.\"\"\"\n")
            .unwrap();
        fs::write(
            dir.join("pkg/mod.py"),
            "def format_value(text):\n    \"\"\"Formatting helpers.\n\n    @arg text  The input.\n    @return str  Output.\n    \"\"\"\n    return text\n",
        )
        .unwrap();

        let project = build_project(&dir).expect("build");
        assert!(!project.elements.is_empty());
        assert!(
            project.index.contains_key(&ImportPath::new(vec![
                "pkg".into(),
                "mod".into()
            ]))
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn invalid_input_path_errors() {
        let dir = PathBuf::from("/nonexistent/path/xyzzy");
        let err = build_project(&dir).unwrap_err();
        assert!(matches!(err, Error::InvalidInputPath(_)));
    }
}
