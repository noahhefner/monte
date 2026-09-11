//! Intermediary object representation of documented elements.
//!
//! This is the contract between the docstring-extraction stage and the
//! HTML-generation stage (see data-model.md).

use std::fmt;

/// The full dotted Python import path of an element, e.g. `pkg.mod.Class`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct ImportPath {
    /// Ordered segments from the package root to the element.
    pub segments: Vec<String>,
}

impl ImportPath {
    /// Build a path from an ordered list of segments.
    pub fn new(segments: Vec<String>) -> Self {
        Self { segments }
    }

    /// Dot-joined rendering, used for display and index keys.
    pub fn text(&self) -> String {
        self.segments.join(".")
    }

    /// The final segment (simple name) of the path.
    pub fn name(&self) -> &str {
        self.segments.last().map(String::as_str).unwrap_or("")
    }

    /// Append a child segment, producing a nested path.
    pub fn child(&self, segment: &str) -> Self {
        let mut segments = self.segments.clone();
        segments.push(segment.to_string());
        Self { segments }
    }
}

impl fmt::Display for ImportPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text())
    }
}

/// The kind of a documented element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementKind {
    Module,
    Class,
    Function,
}

impl ElementKind {
    /// Human-readable label used in the generated HTML.
    pub fn label(&self) -> &'static str {
        match self {
            ElementKind::Module => "Module",
            ElementKind::Class => "Class",
            ElementKind::Function => "Function",
        }
    }
}

/// A single documented argument (the `args` YAML key).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArgDoc {
    pub name: String,
    pub description: String,
}

/// A documented return value (the `returns` YAML key).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnDoc {
    pub description: String,
}

/// A documented exception (the `raises` YAML key).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RaisesDoc {
    pub exception: String,
    pub description: String,
}

/// The outcome of parsing the comment block above an element.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DocComment {
    /// No comment present; the element exists without detail (FR-008).
    #[default]
    None,
    /// Comment present but malformed; entry shows an error (FR-007).
    Error(String),
    /// Comment parsed successfully.
    Ok(DocContent),
}

/// Parsed content of a well-formed comment block.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DocContent {
    pub description: String,
    pub args: Vec<ArgDoc>,
    pub returns: Option<ReturnDoc>,
    pub raises: Vec<RaisesDoc>,
}

/// A documented module, class, or function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentedElement {
    /// Full Python package import path; the unique identifier.
    pub path: ImportPath,
    pub kind: ElementKind,
    pub doc: DocComment,
    /// Nested elements (classes/functions inside modules/classes).
    pub children: Vec<DocumentedElement>,
}

impl DocumentedElement {
    pub fn new(path: ImportPath, kind: ElementKind) -> Self {
        Self {
            path,
            kind,
            doc: DocComment::None,
            children: Vec::new(),
        }
    }

    /// Recursively order this element's children by full import path so
    /// output is stable regardless of declaration order (FR-006).
    pub fn sort_by_path(&mut self) {
        for child in &mut self.children {
            child.sort_by_path();
        }
        self.children.sort_by(|a, b| a.path.cmp(&b.path));
    }
}

/// The root representation of a generated documentation site.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DocumentedProject {
    /// Import path prefix derived from the scanned source tree.
    pub root_path: ImportPath,
    /// Top-level elements (packages/modules).
    pub elements: Vec<DocumentedElement>,
    /// Flat index of every element keyed by full path.
    pub index: std::collections::BTreeMap<ImportPath, DocumentedElement>,
}

impl DocumentedProject {
    /// Rebuild the flat index from the stored elements (path-sorted).
    pub fn rebuild_index(&mut self) {
        self.index.clear();

        fn collect(
            elements: &[DocumentedElement],
            index: &mut std::collections::BTreeMap<
                ImportPath,
                DocumentedElement,
            >,
        ) {
            for el in elements {
                index.insert(el.path.clone(), el.clone());
                collect(&el.children, index);
            }
        }

        collect(&self.elements, &mut self.index);
    }
}
