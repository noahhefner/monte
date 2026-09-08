//! Extraction of documentation content from Python docstrings.
//!
//! A docstring's raw text is indented to match the enclosing block. This
//! module normalizes it (dedent + trim) into plain lines that the tag parser
//! (`tags.rs`) can consume, so `@arg`/`@return`/`@raises` lines are
//! recognized (see contracts/docstring-syntax.md).

/// Split a docstring's raw text into cleaned, dedented, non-blank-edge lines.
///
/// - Removes a common leading indentation across lines (PEP 257).
/// - Drops leading/trailing blank lines and strips each line's leading/trailing
///   whitespace.
///
/// Returns an empty vector for a docstring with no meaningful lines.
pub fn docstring_lines(raw: &str) -> Vec<String> {
    let mut lines: Vec<String> = raw.lines().map(str::to_string).collect();

    strip_blank_edges(&mut lines);
    if lines.is_empty() {
        return lines;
    }

    // Common leading whitespace across all non-blank lines.
    let indent = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);

    for line in &mut lines {
        let idx = line.len().min(indent);
        let trimmed = line[idx..].trim();
        *line = trimmed.to_string();
    }

    strip_blank_edges(&mut lines);
    lines
}

fn strip_blank_edges(lines: &mut Vec<String>) {
    while lines.first().is_some_and(|l| l.trim().is_empty()) {
        lines.remove(0);
    }
    while lines.last().is_some_and(|l| l.trim().is_empty()) {
        lines.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedents_docstring_body() {
        let raw = "    Does a thing.\n\n    @arg x  the input\n    ";
        assert_eq!(
            docstring_lines(raw),
            vec![
                "Does a thing.".to_string(),
                "".to_string(),
                "@arg x  the input".to_string(),
            ]
        );
    }

    #[test]
    fn handles_indented_tags() {
        let raw = "\n        @return str  output\n    ";
        assert_eq!(
            docstring_lines(raw),
            vec!["@return str  output".to_string()]
        );
    }

    #[test]
    fn empty_or_blank_docstring_yields_no_lines() {
        assert_eq!(docstring_lines(""), Vec::<String>::new());
        assert_eq!(docstring_lines("   \n   \n"), Vec::<String>::new());
    }
}
