//! Convert raw AST elements into documented elements with docstrings.
//!
//! This stage takes each raw element (from `parser.rs`) and parses its
//! docstring into a [`DocComment`], producing the [`DocumentedElement`] tree
//! consumed by `html.rs` (FR-003).

use crate::docstring::docstring_lines;
use crate::model::{DocComment, DocumentedElement, ElementKind, ImportPath};
use crate::parser::RawElement;
use crate::tags;

/// Build the module `DocumentedElement` for a single parsed file.
///
/// `module_doc` is the module docstring extracted by the parser (the first
/// statement of the file when it is a string literal).
pub fn module_element(
    module_path: &ImportPath,
    raws: &[RawElement],
    module_doc: Option<String>,
) -> DocumentedElement {
    let doc = docstring_comment(module_doc.as_deref());

    let children = raws
        .iter()
        .map(|raw| build_element(module_path, raw))
        .collect();

    DocumentedElement {
        path: module_path.clone(),
        kind: ElementKind::Module,
        doc,
        children,
    }
}

/// Build a `DocumentedElement` for a single raw definition and its children,
/// referencing the full import path of the parent scope.
fn build_element(parent: &ImportPath, raw: &RawElement) -> DocumentedElement {
    let path = parent.child(&raw.name);
    let doc = docstring_comment(raw.docstring.as_deref());
    let children = raw
        .children
        .iter()
        .map(|child| build_element(&path, child))
        .collect();

    DocumentedElement {
        path,
        kind: raw.kind,
        doc,
        children,
    }
}

/// Parse a raw docstring into a `DocComment`: malformed tag blocks produce
/// `DocComment::Error` (FR-007); a missing docstring produces `None`
/// (FR-008); otherwise `Ok` (FR-002).
fn docstring_comment(raw: Option<&str>) -> DocComment {
    match raw {
        None => DocComment::None,
        Some(text) => {
            let lines = docstring_lines(text);
            if lines.is_empty() {
                DocComment::None
            } else {
                tags::parse_block(&lines).into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn module(name: &str) -> ImportPath {
        ImportPath::new(vec![name.to_string()])
    }

    #[test]
    fn undocumented_elements_get_none() {
        let source = "def f():\n    pass\n";
        let pm = crate::parser::parse_source(source).unwrap();
        let el = module_element(&module("m"), &pm.elements, pm.module_doc);
        assert_eq!(el.doc, DocComment::None);
        assert_eq!(el.children[0].doc, DocComment::None);
    }

    #[test]
    fn documented_function_gets_docstring() {
        let source = "def f():\n    \"\"\"Does a thing.\n\n    @return int  the count\n    \"\"\"\n    return 1\n";
        let pm = crate::parser::parse_source(source).unwrap();
        let el = module_element(&module("m"), &pm.elements, pm.module_doc);
        let child = &el.children[0];
        match &child.doc {
            DocComment::Ok(content) => {
                assert!(content.description.contains("Does a thing."));
                assert!(content.returns.is_some());
            }
            other => panic!("expected Ok docstring, got {other:?}"),
        }
    }

    #[test]
    fn malformed_docstring_yields_error() {
        let source = "def f():\n    \"\"\"@arg\n    \"\"\"\n    pass\n";
        let pm = crate::parser::parse_source(source).unwrap();
        let el = module_element(&module("m"), &pm.elements, pm.module_doc);
        match &el.children[0].doc {
            DocComment::Error(msg) => {
                assert!(msg.contains("argument name"));
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn children_are_nested_under_parent_path() {
        let source = "class A:\n    def m(self):\n        pass\n";
        let pm = crate::parser::parse_source(source).unwrap();
        let el = module_element(&module("m"), &pm.elements, pm.module_doc);
        assert_eq!(el.children[0].path.text(), "m.A");
        assert_eq!(el.children[0].children[0].path.text(), "m.A.m");
    }

    #[test]
    fn module_gets_docstring() {
        let source = "\"\"\"Module level docs.\"\"\"\n\ndef f():\n    pass\n";
        let pm = crate::parser::parse_source(source).unwrap();
        let el = module_element(&module("m"), &pm.elements, pm.module_doc);
        match &el.doc {
            DocComment::Ok(content) => {
                assert!(content.description.contains("Module level docs."))
            }
            other => panic!("expected Ok module docstring, got {other:?}"),
        }
    }
}
