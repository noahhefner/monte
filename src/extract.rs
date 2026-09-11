//! Convert raw AST elements into documented elements with docstrings.
//!
//! This stage takes each raw element (from `parser.rs`) and interprets its
//! docstring as a YAML document, producing the [`DocumentedElement`] tree
//! consumed by `html.rs` (FR-003). See contracts/docstring-syntax.md.

use serde::Deserialize;

use crate::docstring::docstring_text;
use crate::model::{
    ArgDoc, DocComment, DocContent, DocumentedElement, ElementKind, ImportPath,
    RaisesDoc, ReturnDoc,
};
use crate::parser::RawElement;

/// Result of interpreting a docstring as YAML.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YamlParseResult {
    /// The docstring parsed successfully.
    Ok(DocContent),
    /// The docstring is malformed; include the error message in the entry.
    Error(String),
}

impl From<YamlParseResult> for DocComment {
    fn from(result: YamlParseResult) -> Self {
        match result {
            YamlParseResult::Ok(content) => DocComment::Ok(content),
            YamlParseResult::Error(msg) => DocComment::Error(msg),
        }
    }
}

/// YAML deserialization schema for a structured docstring (FR-003).
///
/// Only the keys `description`, `args`, `returns`, and `raises` are
/// recognized. Unknown keys (e.g. `example`, `returns.type`) are silently
/// ignored by serde's default behavior, matching the documented contract.
#[derive(Debug, Clone, Deserialize, Default)]
struct YamlDoc {
    description: Option<String>,
    args: Option<Vec<YamlArg>>,
    returns: Option<YamlReturn>,
    raises: Option<Vec<YamlRaise>>,
}

#[derive(Debug, Clone, Deserialize)]
struct YamlArg {
    name: String,
    description: String,
}

#[derive(Debug, Clone, Deserialize)]
struct YamlReturn {
    description: String,
}

#[derive(Debug, Clone, Deserialize)]
struct YamlRaise {
    #[serde(rename = "type", alias = "exception")]
    exception: String,
    description: String,
}

/// Interpret a dedented docstring as a YAML document (FR-001, FR-007,
/// FR-010).
///
/// Three-way dispatch:
/// 1. The text deserializes into a [`YamlDoc`] mapping → build a
///    [`DocContent`] from the recognized keys.
/// 2. The text is a plain YAML scalar (prose, not a mapping) → use it as a
///    description-only entry (FR-010).
/// 3. YAML parsing fails (or a recognized key has the wrong shape) → an
///    error entry (FR-007).
pub fn parse_yaml_docstring(text: &str) -> YamlParseResult {
    if contains_legacy_tag(text) {
        return YamlParseResult::Error(LEGACY_TAG_ERROR.to_string());
    }

    match serde_yml::from_str::<YamlDoc>(text) {
        Ok(doc) => YamlParseResult::Ok(content_from_yaml(doc)),
        Err(yaml_err) => match serde_yml::from_str::<String>(text) {
            Ok(prose) => YamlParseResult::Ok(DocContent {
                description: prose.trim().to_string(),
                ..DocContent::default()
            }),
            Err(_) => YamlParseResult::Error(format!(
                "{MALFORMED_PREFIX}: could not be interpreted as YAML ({yaml_err})"
            )),
        },
    }
}

/// Stable prefix of every malformed-docstring error message; contract tests
/// assert on it.
const MALFORMED_PREFIX: &str = "docstring was not written correctly";

const LEGACY_TAG_ERROR: &str = "docstring was not written correctly: found a legacy @-tag; \
     rewrite as YAML (description/args/returns/raises)";

/// Detect a line beginning with `@word` at the document's top-level
/// indentation — the legacy JavaDoc-style syntax (D-005). Such docstrings are
/// treated as malformed rather than prose, since `@` is a legal YAML 1.2
/// plain-scalar character and would otherwise parse as a description.
///
/// Content nested under block scalars (deeper indentation) is not flagged.
fn contains_legacy_tag(text: &str) -> bool {
    let indent = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);

    text.lines().any(|line| {
        if line.len() - line.trim_start().len() != indent {
            return false;
        }
        let rest = line.trim_start();
        rest.starts_with('@')
            && rest[1..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic())
    })
}

/// Map a deserialized [`YamlDoc`] mapping onto the intermediary
/// [`DocContent`].
fn content_from_yaml(doc: YamlDoc) -> DocContent {
    let args = doc
        .args
        .unwrap_or_default()
        .into_iter()
        .map(|a| ArgDoc {
            name: a.name,
            description: a.description,
        })
        .collect();

    let returns = doc.returns.map(|r| ReturnDoc {
        description: r.description,
    });

    let raises = doc
        .raises
        .unwrap_or_default()
        .into_iter()
        .map(|r| RaisesDoc {
            exception: r.exception,
            description: r.description,
        })
        .collect();

    DocContent {
        description: doc.description.unwrap_or_default(),
        args,
        returns,
        raises,
    }
}

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

/// Interpret a raw docstring as a `DocComment`: malformed YAML produces
/// `DocComment::Error` (FR-007); a missing or blank docstring produces `None`
/// (FR-008); otherwise `Ok` (FR-001, FR-010).
fn docstring_comment(raw: Option<&str>) -> DocComment {
    match raw {
        None => DocComment::None,
        Some(text) => {
            let text = docstring_text(text);
            if text.is_empty() {
                DocComment::None
            } else {
                parse_yaml_docstring(&text).into()
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
    fn documented_function_gets_yaml_docstring() {
        let source = "def f():\n    \"\"\"\n    description: Does a thing.\n\n    returns:\n      description: the count\n    \"\"\"\n    return 1\n";
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
        let source =
            "def f():\n    \"\"\"args: not a list\n    \"\"\"\n    pass\n";
        let pm = crate::parser::parse_source(source).unwrap();
        let el = module_element(&module("m"), &pm.elements, pm.module_doc);
        match &el.children[0].doc {
            DocComment::Error(msg) => {
                assert!(msg.contains("docstring was not written correctly"));
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
        let source = "\"\"\"description: Module level docs.\"\"\"\n\ndef f():\n    pass\n";
        let pm = crate::parser::parse_source(source).unwrap();
        let el = module_element(&module("m"), &pm.elements, pm.module_doc);
        match &el.doc {
            DocComment::Ok(content) => {
                assert!(content.description.contains("Module level docs."))
            }
            other => panic!("expected Ok module docstring, got {other:?}"),
        }
    }

    #[test]
    fn prose_docstring_is_description_only() {
        let source = "def f():\n    \"\"\"Just some words.\"\"\"\n    pass\n";
        let pm = crate::parser::parse_source(source).unwrap();
        let el = module_element(&module("m"), &pm.elements, pm.module_doc);
        match &el.children[0].doc {
            DocComment::Ok(content) => {
                assert_eq!(content.description, "Just some words.");
                assert!(content.args.is_empty());
                assert!(content.returns.is_none());
                assert!(content.raises.is_empty());
            }
            other => panic!("expected Ok prose docstring, got {other:?}"),
        }
    }
}
