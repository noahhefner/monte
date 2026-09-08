//! Construction of Python import paths from the directory structure.

use std::path::Path;

/// Convert a source file path (relative to the scanned source root) into
/// the dotted module import path segments.
///
/// - `pkg/sub/mod.py`         → `["pkg", "sub", "mod"]`
/// - `pkg/sub/__init__.py`    → `["pkg", "sub"]` (package itself)
/// - `utils.py`               → `["utils"]`
/// - `pkg/__init__.py`        → `["pkg"]` (package itself)
///
/// Returns `None` when the file is not a Python module (not `.py`).
pub fn module_path_from_relative(relative: &Path) -> Option<Vec<String>> {
    if relative.extension().and_then(|e| e.to_str()) != Some("py") {
        return None;
    }

    let mut segments: Vec<String> = Vec::new();
    for component in relative.parent().iter().flat_map(|p| p.components()) {
        if let Some(name) =
            component.as_os_str().to_str().filter(|n| !n.is_empty())
        {
            segments.push(name.to_string());
        }
    }

    let file_stem = relative
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    if file_stem == "__init__" {
        // The module is the package itself; the file stem is not a segment.
        if segments.is_empty() {
            segments.push("__init__".to_string());
        }
    } else {
        segments.push(file_stem);
    }

    Some(segments)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn converts_nested_module() {
        let rel = PathBuf::from("pkg/sub/mod.py");
        let segs = module_path_from_relative(&rel).expect("module");
        assert_eq!(segs, vec!["pkg", "sub", "mod"]);
    }

    #[test]
    fn converts_init_as_package() {
        let rel = PathBuf::from("pkg/sub/__init__.py");
        let segs = module_path_from_relative(&rel).expect("module");
        assert_eq!(segs, vec!["pkg", "sub"]);
    }

    #[test]
    fn converts_top_level_module() {
        let rel = PathBuf::from("utils.py");
        let segs = module_path_from_relative(&rel).expect("module");
        assert_eq!(segs, vec!["utils"]);
    }

    #[test]
    fn rejects_non_python_file() {
        let rel = PathBuf::from("readme.md");
        assert_eq!(module_path_from_relative(&rel), None);
    }
}
