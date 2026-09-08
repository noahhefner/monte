//! Navigation index generation (FR-005, SC-003).
//!
//! Maps each documented element to an output page and renders the index
//! page that links to every module, class, and function by full path.

use crate::html;
use crate::model::{DocumentedElement, DocumentedProject, ImportPath};

/// Map an element import path to its output page path relative to the site
/// root, e.g. `sample.formatting` → `sample/formatting.html`.
pub fn page_file_path(path: &ImportPath) -> String {
    let mut rel = path.segments.join("/");
    rel.push_str(".html");
    rel
}

/// Render the index/navigation page: every module listed as a link to its
/// page, with the classes and functions it contains linked to that page's
/// element anchors (FR-005, SC-003).
pub fn render_index(project: &DocumentedProject, styles: &str) -> String {
    let mut body = String::new();
    body.push_str("<h1>pydoc-gen documentation</h1>\n");
    if !project.root_path.text().is_empty() {
        body.push_str(&format!(
            "<h2>Root: {}</h2>\n",
            html::escape_html(&project.root_path.text())
        ));
    }
    body.push_str("<h2>Index</h2>\n");
    if project.elements.is_empty() {
        body.push_str("<p>No Python modules found.</p>\n");
    } else {
        body.push_str("<ul>\n");
        for module in &project.elements {
            render_module_entry(&mut body, module);
        }
        body.push_str("</ul>\n");
    }
    body.push_str("<hr>\n");
    html::wrap_page("Index — pydoc-gen", &body, styles)
}

/// Render one top-level module entry and its contained elements.
fn render_module_entry(out: &mut String, module: &DocumentedElement) {
    let page = page_file_path(&module.path);
    let href = format!("{}#{}", page, html::escape_html(&module.path.text()));
    out.push_str(&format!(
        "<li><a href=\"{href}\">{path}</a> \
         <span class=\"kind\">{kind}</span>\n",
        href = html::escape_html(&href),
        path = html::escape_html(&module.path.text()),
        kind = html::escape_html(module.kind.label()),
    ));
    if !module.children.is_empty() {
        out.push_str("<ul>\n");
        render_children(out, &module.children, &module.path);
        out.push_str("</ul>\n");
    }
    out.push_str("</li>\n");
}

/// Render nested classes/functions under `module`; every element links to an
/// anchor on the containing module's page.
fn render_children(
    out: &mut String,
    elements: &[DocumentedElement],
    module: &ImportPath,
) {
    let page = page_file_path(module);
    for el in elements {
        let href = format!("{}#{}", page, html::escape_html(&el.path.text()));
        out.push_str(&format!(
            "<li><a href=\"{href}\">{path}</a> \
             <span class=\"kind\">{kind}</span>\n",
            href = html::escape_html(&href),
            path = html::escape_html(&el.path.text()),
            kind = html::escape_html(el.kind.label()),
        ));
        if !el.children.is_empty() {
            out.push_str("<ul>\n");
            render_children(out, &el.children, module);
            out.push_str("</ul>\n");
        }
        out.push_str("</li>\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DocComment, DocContent, ElementKind};

    #[test]
    fn maps_import_path_to_page() {
        let path = ImportPath::new(vec!["sample".into(), "formatting".into()]);
        assert_eq!(page_file_path(&path), "sample/formatting.html");
    }

    fn sample_module() -> DocumentedElement {
        let func = DocumentedElement {
            path: ImportPath::new(vec![
                "sample".into(),
                "mod".into(),
                "format_value".into(),
            ]),
            kind: ElementKind::Function,
            doc: DocComment::Ok(DocContent {
                description: "Formats values.".into(),
                ..DocContent::default()
            }),
            children: Vec::new(),
        };
        DocumentedElement {
            path: ImportPath::new(vec!["sample".into(), "mod".into()]),
            kind: ElementKind::Module,
            doc: DocComment::None,
            children: vec![func],
        }
    }

    #[test]
    fn index_links_every_element_to_its_page() {
        let styles = crate::style::default_theme_css();
        let mut project = DocumentedProject {
            root_path: ImportPath::new(vec!["sample".into()]),
            ..DocumentedProject::default()
        };
        project.elements.push(sample_module());

        let html_text = render_index(&project, styles);
        assert!(html_text.contains("href=\"sample/mod.html#sample.mod\""));
        assert!(
            html_text
                .contains("href=\"sample/mod.html#sample.mod.format_value\"")
        );
    }
}
