//! Parse Python source into the AST and collect modules, classes, and
//! functions together with their docstrings (FR-001, FR-003).
//!
//! Parsing is delegated to `rustpython_parser`; no parser is written from
//! scratch (research.md D-002).

use rustpython_parser::ast::Suite;
use rustpython_parser::text_size::TextSize;
use rustpython_parser::{Parse as _, ast};

use crate::error::{Error, Result};
use crate::model::ElementKind;

/// A raw element extracted from the AST, before docstring extraction.
///
/// The docstring-extraction stage (`extract.rs`) converts these into
/// [`crate::model::DocumentedElement`] values.
#[derive(Debug, Clone)]
pub struct RawElement {
    pub name: String,
    pub kind: ElementKind,
    /// Docstring literal that opens the body, if any (may be empty).
    pub docstring: Option<String>,
    /// 1-based source line where the `def`/`class` keyword begins.
    pub def_line: usize,
    pub children: Vec<RawElement>,
}

/// Result of parsing a source file.
#[derive(Debug, Clone)]
pub struct ParsedModule {
    /// Top-level documented elements (classes and functions). The module
    /// element itself is constructed by the extraction stage.
    pub elements: Vec<RawElement>,
    /// The module docstring (first statement of the file), if any.
    pub module_doc: Option<String>,
}

/// Parse a single Python source file and return its elements plus the module
/// docstring.
pub fn parse_source(source: &str) -> Result<ParsedModule> {
    let ast = Suite::parse(source, "<source>").map_err(|e| {
        Error::ParseSource("<source>".to_string(), e.to_string())
    })?;
    Ok(ParsedModule {
        elements: collect_statements(&ast, source),
        module_doc: docstring_of(&ast),
    })
}

/// Walk a suite of statements, collecting the definitions that appear
/// directly within the given scope.
fn collect_statements(stmts: &[ast::Stmt], source: &str) -> Vec<RawElement> {
    let mut out = Vec::new();
    for stmt in stmts {
        match stmt {
            ast::Stmt::FunctionDef(f) => out.push(RawElement {
                name: f.name.to_string(),
                kind: ElementKind::Function,
                docstring: docstring_of(&f.body),
                def_line: line_at(f.range.start(), source),
                children: Vec::new(),
            }),
            ast::Stmt::AsyncFunctionDef(f) => out.push(RawElement {
                name: f.name.to_string(),
                kind: ElementKind::Function,
                docstring: docstring_of(&f.body),
                def_line: line_at(f.range.start(), source),
                children: Vec::new(),
            }),
            ast::Stmt::ClassDef(c) => out.push(RawElement {
                name: c.name.to_string(),
                kind: ElementKind::Class,
                docstring: docstring_of(&c.body),
                def_line: line_at(c.range.start(), source),
                children: collect_statements(&c.body, source),
            }),
            _ => {}
        }
    }
    out
}

/// Return the docstring of a suite: the first statement, if it is a plain
/// (non-f-string) string literal. Empty strings count as "no docstring".
fn docstring_of(stmts: &[ast::Stmt]) -> Option<String> {
    match stmts.first() {
        Some(ast::Stmt::Expr(expr)) => match &*expr.value {
            ast::Expr::Constant(c) => match &c.value {
                ast::Constant::Str(s) if !s.is_empty() => Some(s.clone()),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

/// The 1-based line number for a byte offset into `source`.
fn line_at(offset: TextSize, source: &str) -> usize {
    let offset = offset.to_usize();
    let upto = source.as_bytes().get(..offset).unwrap_or(&[]);
    upto.iter().filter(|&&b| b == b'\n').count() + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collects_top_level_function() {
        let src = "def foo():\n    pass\n";
        let pm = parse_source(src).expect("parse");
        let els = &pm.elements;
        assert_eq!(els.len(), 1);
        assert_eq!(els[0].name, "foo");
        assert_eq!(els[0].kind, ElementKind::Function);
        assert_eq!(els[0].def_line, 1);
    }

    #[test]
    fn collects_class_with_method_children() {
        let src = "class A:\n    def m(self):\n        pass\n";
        let pm = parse_source(src).expect("parse");
        let els = &pm.elements;
        assert_eq!(els.len(), 1);
        assert_eq!(els[0].kind, ElementKind::Class);
        assert_eq!(els[0].name, "A");
        assert_eq!(els[0].children.len(), 1);
        assert_eq!(els[0].children[0].name, "m");
        assert_eq!(els[0].children[0].def_line, 2);
    }

    #[test]
    fn ignores_non_definition_statements() {
        let src = "import os\nx = 1\ndef f():\n    pass\n";
        let pm = parse_source(src).expect("parse");
        let els = &pm.elements;
        assert_eq!(els.len(), 1);
        assert_eq!(els[0].name, "f");
    }

    #[test]
    fn async_functions_are_functions() {
        let src = "async def a():\n    pass\n";
        let pm = parse_source(src).expect("parse");
        let els = &pm.elements;
        assert_eq!(els.len(), 1);
        assert_eq!(els[0].kind, ElementKind::Function);
        assert_eq!(els[0].name, "a");
    }

    #[test]
    fn captures_function_docstring() {
        let src = "def f():\n    \"\"\"Does a thing.\n\n    @arg x  input.\n    \"\"\"\n    return x\n";
        let pm = parse_source(src).expect("parse");
        let doc = pm.elements[0].docstring.as_deref().expect("docstring");
        assert!(doc.contains("Does a thing."));
        assert!(doc.contains("@arg x"));
    }

    #[test]
    fn captures_module_docstring() {
        let src = "\"\"\"Module docs.\"\"\"\n\ndef f():\n    pass\n";
        let pm = parse_source(src).expect("parse");
        let doc = pm.module_doc.as_deref().expect("module docstring");
        assert_eq!(doc, "Module docs.");
    }

    #[test]
    fn assignment_before_docstring_is_not_module_doc() {
        let src = "x = 1\n\"\"\"Not really a docstring here.\"\"\"\n";
        let pm = parse_source(src).expect("parse");
        assert_eq!(pm.module_doc, None);
    }

    #[test]
    fn empty_docstring_counts_as_none() {
        let src = "def f():\n    \"\"\"\"\"\"\n    pass\n";
        let pm = parse_source(src).expect("parse");
        assert_eq!(pm.elements[0].docstring, None);
    }
}
