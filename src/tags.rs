//! JavaDoc-style tag parser for `@arg`, `@return`, and `@raises`.
//!
//! Tags are parsed from the normalized lines of a Python docstring.

use crate::model::{ArgDoc, DocComment, DocContent, RaisesDoc, ReturnDoc};

/// Result of parsing a docstring's lines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TagParseResult {
    /// The docstring parsed successfully.
    Ok(DocContent),
    /// The docstring is malformed; include the error message in the entry.
    Error(String),
}

impl From<TagParseResult> for DocComment {
    fn from(result: TagParseResult) -> Self {
        match result {
            TagParseResult::Ok(content) => DocComment::Ok(content),
            TagParseResult::Error(msg) => DocComment::Error(msg),
        }
    }
}

/// Parse the normalized lines of a docstring into a [`DocContent`].
///
/// Lines starting with `@` are treated as tags; every other non-empty line is
/// part of the description. Returns [`TagParseResult::Error`] if the block is
/// malformed (e.g. an `@arg` with no argument name) per FR-007.
pub fn parse_block(lines: &[String]) -> TagParseResult {
    let mut content = DocContent::default();
    let mut description_lines: Vec<&str> = Vec::new();

    for raw in lines {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix('@') {
            if let Err(msg) = parse_tag(rest, &mut content) {
                return TagParseResult::Error(msg);
            }
        } else if !line.is_empty() {
            description_lines.push(line);
        }
    }

    content.description = description_lines.join(" ");
    TagParseResult::Ok(content)
}

fn parse_tag(rest: &str, content: &mut DocContent) -> Result<(), String> {
    // Split on the first whitespace to isolate the keyword.
    let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
    let keyword = &rest[..end];
    let remainder = rest[end..].trim();

    match keyword {
        "arg" => {
            if remainder.is_empty() {
                return Err(
                    "malformed @arg: expected an argument name after the tag"
                        .to_string(),
                );
            }
            let name_end = remainder
                .find(char::is_whitespace)
                .unwrap_or(remainder.len());
            let name = remainder[..name_end].trim().to_string();
            let description = remainder[name_end..].trim().to_string();
            content.args.push(ArgDoc { name, description });
            Ok(())
        }
        "return" => {
            content.returns = Some(ReturnDoc {
                description: remainder.to_string(),
            });
            Ok(())
        }
        "raises" => {
            if remainder.is_empty() {
                return Err(
                    "malformed @raises: expected an exception name after the tag"
                        .to_string(),
                );
            }
            let name_end = remainder
                .find(char::is_whitespace)
                .unwrap_or(remainder.len());
            let exception = remainder[..name_end].trim().to_string();
            let description = remainder[name_end..].trim().to_string();
            content.raises.push(RaisesDoc {
                exception,
                description,
            });
            Ok(())
        }
        // Unrecognized tags are ignored (per docstring-syntax.md); they do
        // not fail the run. The text after the keyword is not captured.
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_description_and_tags() {
        let lines = vec![
            "Formats the given text.".to_string(),
            "@arg text  The input to format.".to_string(),
            "@return    The formatted string.".to_string(),
            "@raises ValueError  Raised when text is empty.".to_string(),
        ];
        match parse_block(&lines) {
            TagParseResult::Ok(content) => {
                assert!(
                    content.description.contains("Formats the given text.")
                );
                assert_eq!(content.args.len(), 1);
                assert_eq!(content.args[0].name, "text");
                assert_eq!(content.args[0].description, "The input to format.");
                let ret = content.returns.as_ref().expect("return present");
                assert_eq!(ret.description, "The formatted string.");
                assert_eq!(content.raises.len(), 1);
                assert_eq!(content.raises[0].exception, "ValueError");
                assert_eq!(
                    content.raises[0].description,
                    "Raised when text is empty."
                );
            }
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    #[test]
    fn malformed_arg_yields_error() {
        let lines = vec!["@arg".to_string()];
        match parse_block(&lines) {
            TagParseResult::Error(msg) => {
                assert!(msg.contains("expected an argument name"));
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn malformed_raises_yields_error() {
        let lines = vec!["@raises".to_string()];
        match parse_block(&lines) {
            TagParseResult::Error(msg) => {
                assert!(msg.contains("expected an exception name"));
            }
            other => panic!("expected Error, got {other:?}"),
        }
    }

    #[test]
    fn unrecognized_tag_is_ignored() {
        let lines = vec!["@author John".to_string(), "description".to_string()];
        match parse_block(&lines) {
            TagParseResult::Ok(content) => {
                assert!(!content.description.is_empty());
                assert!(content.args.is_empty());
            }
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    #[test]
    fn blank_comment_lines_are_ignored() {
        let lines = vec!["".to_string(), "just some text".to_string()];
        match parse_block(&lines) {
            TagParseResult::Ok(content) => {
                assert_eq!(content.description, "just some text");
            }
            other => panic!("expected Ok, got {other:?}"),
        }
    }
}
