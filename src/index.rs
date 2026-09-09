//! Navigation index generation.

use serde::Serialize;
use tera::{Context, Tera};

use crate::model::{DocumentedElement, DocumentedProject};

#[derive(Debug, Serialize)]
struct TemplateElement {
    path: String,
    kind: String,
    children: Vec<TemplateElement>,
}

impl From<&DocumentedElement> for TemplateElement {
    fn from(element: &DocumentedElement) -> Self {
        Self {
            path: element.path.text(),
            kind: element.kind.label().to_string(),
            children: element
                .children
                .iter()
                .map(TemplateElement::from)
                .collect(),
        }
    }
}

pub fn render_index(project: &DocumentedProject, styles: &str) -> String {
    let mut tera = Tera::default();

    tera.add_raw_template(
        "index.html",
        include_str!("templates/index.html"),
    )
    .expect("failed to parse index template");

    let modules: Vec<TemplateElement> = project
        .elements
        .iter()
        .map(TemplateElement::from)
        .collect();

    let mut context = Context::new();

    context.insert("title", "Python Docs");
    context.insert("styles", styles);
    context.insert("modules", &modules);

    tera.render("index.html", &context)
        .expect("failed to render index template")
}