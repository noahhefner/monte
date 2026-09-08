//! HTML generation from the intermediary representation.
//!
//! This module renders each documented module as a self-contained HTML page
//! (FR-004). Element content is HTML-escaped to prevent injection via
//! docstring content (T037). The navigation index is rendered by `index.rs`
//! (FR-005).

use crate::model::{DocComment, DocumentedElement};

/// HTML-escape a string so that raw docstring text cannot inject markup.
pub fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// Render a single module page: the module entry with its classes and
/// functions, plus a link back to the index (FR-004, SC-003).
pub fn render_module_page(
    module: &DocumentedElement,
    back_to: &str,
    styles: &str,
) -> String {
    let mut body = String::new();
    body.push_str(&format!(
        "<p class=\"nav\"><a href=\"{}\">&larr; Index</a></p>\n",
        escape_html(back_to)
    ));
    body.push_str("<hr>\n");
    render_element(&mut body, module, 0);
    wrap_page(
        &format!("{} — pydoc-gen", module.path.text()),
        &body,
        styles,
    )
}

/// Wrap body content in a minimal HTML document, embedding `styles` so every
/// page is self-contained and offline-capable (FR-008, contracts/theme.md).
pub fn wrap_page(title: &str, body: &str, styles: &str) -> String {
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\
         \n<meta name=\"color-scheme\" content=\"light dark\">\n<title>{}</title>\
         \n<style>\n{}\n</style>\n</head>\n<body>\n{}\n</body>\n</html>\n",
        escape_html(title),
        styles,
        body
    )
}

fn render_element(out: &mut String, el: &DocumentedElement, depth: usize) {
    let heading = format!("h{}", (depth + 2).min(6));
    out.push_str(&format!(
        "<section>\n<{heading} class=\"element\" id=\"{id}\">\
         <span class=\"kind\">{kind}</span> {name}</{heading}>\n",
        id = escape_html(&el.path.text()),
        kind = escape_html(el.kind.label()),
        name = escape_html(&el.path.text()),
    ));

    out.push_str(&format!(
        "<p class=\"path\"><code>{}</code></p>\n",
        escape_html(&el.path.text())
    ));

    match &el.doc {
        DocComment::None => {
            out.push_str(
                "<p class=\"undocumented\">No documentation provided.</p>\n",
            );
        }
        DocComment::Error(msg) => {
            out.push_str(&format!(
                "<p class=\"error\">Error: the docstring was not written \
                 correctly ({})</p>\n",
                escape_html(msg)
            ));
        }
        DocComment::Ok(content) => {
            if !content.description.is_empty() {
                out.push_str(&format!(
                    "<p class=\"description\">{}</p>\n",
                    escape_html(&content.description)
                ));
            }
            if !content.args.is_empty() {
                out.push_str("<h4>Arguments</h4>\n<table>\n");
                for arg in &content.args {
                    out.push_str(&format!(
                        "<tr><td><code>{}</code></td><td>{}</td></tr>\n",
                        escape_html(&arg.name),
                        escape_html(&arg.description),
                    ));
                }
                out.push_str("</table>\n");
            }
            if let Some(returns) = &content.returns {
                out.push_str(&format!(
                    "<h4>Returns</h4>\n<p>{}</p>\n",
                    escape_html(&returns.description)
                ));
            }
            if !content.raises.is_empty() {
                out.push_str("<h4>Raises</h4>\n<ul>\n");
                for raises in &content.raises {
                    out.push_str(&format!(
                        "<li><code>{}</code> — {}</li>\n",
                        escape_html(&raises.exception),
                        escape_html(&raises.description),
                    ));
                }
                out.push_str("</ul>\n");
            }
        }
    }

    for child in &el.children {
        render_element(out, child, depth + 1);
    }
    out.push_str("</section>\n");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ArgDoc, DocContent, ImportPath, ReturnDoc};

    #[test]
    fn escapes_special_characters() {
        assert_eq!(
            escape_html("<script>alert('&\"x\")</script>"),
            "&lt;script&gt;alert(&#39;&amp;&quot;x&quot;)&lt;/script&gt;"
        );
    }

    #[test]
    fn leaves_plain_text_unchanged() {
        assert_eq!(escape_html("plain text"), "plain text");
    }

    fn sample_module() -> DocumentedElement {
        let doc = DocContent {
            description: "A sample module.".to_string(),
            args: vec![ArgDoc {
                name: "text".to_string(),
                description: "The input.".to_string(),
            }],
            returns: Some(ReturnDoc {
                description: "The result.".to_string(),
            }),
            raises: Vec::new(),
        };

        let func = DocumentedElement {
            path: ImportPath::new(vec![
                "sample".to_string(),
                "mod".to_string(),
                "format_value".to_string(),
            ]),
            kind: crate::model::ElementKind::Function,
            doc: DocComment::Ok(doc),
            children: Vec::new(),
        };

        DocumentedElement {
            path: ImportPath::new(vec![
                "sample".to_string(),
                "mod".to_string(),
            ]),
            kind: crate::model::ElementKind::Module,
            doc: DocComment::None,
            children: vec![func],
        }
    }

    #[test]
    fn renders_module_page_with_element_content() {
        let styles = crate::style::default_theme_css();
        let html_text =
            render_module_page(&sample_module(), "index.html", styles);
        assert!(html_text.contains("sample.mod"));
        assert!(html_text.contains("format_value"));
        assert!(html_text.contains("The input."));
        assert!(html_text.contains("The result."));
        assert!(html_text.contains("href=\"index.html\""));
    }

    #[test]
    fn escapes_markup_injected_via_docstring() {
        let styles = crate::style::default_theme_css();
        let mut module = sample_module();
        if let DocComment::Ok(content) = &mut module.children[0].doc {
            content.description =
                "Unsafe <script>alert('x')</script> and \"quoted\"".to_string();
        }
        let html_text = render_module_page(&module, "index.html", styles);
        assert!(
            !html_text.contains("<script>"),
            "raw markup from a docstring must not appear in HTML (T037)"
        );
        assert!(html_text.contains(
            "Unsafe &lt;script&gt;alert(&#39;x&#39;)&lt;/script&gt; and \
             &quot;quoted&quot;"
        ));
    }
}
