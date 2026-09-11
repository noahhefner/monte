//! Extraction of documentation content from Python docstrings.
//!
//! A docstring's raw text is indented to match the enclosing block. This
//! module provides two normalizations:
//!
//! - [`docstring_text`]: PEP 257-style dedent that removes the common leading
//!   indentation **while preserving relative indentation**. YAML is
//!   indentation-sensitive, so this is the input used for YAML parsing
//!   (see contracts/docstring-syntax.md).
//! - [`docstring_lines`]: line-based normalization (dedent + trim each line)
//!   used by the legacy line-oriented parsing; kept for callers that need
//!   flat lines.

/// Dedent a docstring's raw text while preserving relative indentation
/// (PEP 257), returning the text passed to the YAML parser.
///
/// - Removes a common leading indentation across non-blank lines, leaving any
///   extra indentation (e.g. nested YAML lists/mappings) intact.
/// - Blank/whitespace-only lines become empty lines.
/// - Leading/trailing blank lines are trimmed.
///
/// Returns an empty string for a docstring with no meaningful lines.
pub fn docstring_text(raw: &str) -> String {
    let lines: Vec<&str> = raw.split('\n').collect();

    // Common leading whitespace across all non-blank lines.
    let indent = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);

    let text = lines
        .iter()
        .map(|l| {
            if l.trim().is_empty() {
                String::new()
            } else {
                let leading = l.len() - l.trim_start().len();
                l[leading.min(indent)..].to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    text.trim().to_string()
}

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
    fn dedent_preserves_yaml_indentation() {
        let raw = "    description: cool\n\n    args:\n      - name: x\n        description: input\n    ";
        assert_eq!(
            docstring_text(raw),
            "description: cool\n\nargs:\n  - name: x\n    description: input"
        );
    }

    #[test]
    fn dedent_keeps_trailing_block_scalar_indent() {
        let raw = "    example: |\n      line1\n      line2\n    ";
        assert_eq!(docstring_text(raw), "example: |\n  line1\n  line2");
    }

    #[test]
    fn blank_only_docstring_yields_empty_text() {
        assert_eq!(docstring_text(""), "");
        assert_eq!(docstring_text("   \n   \n"), "");
    }

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
