//! T013 - Unit tests for docstring content extraction.

use pydoc_gen::docstring::docstring_lines;

#[test]
fn extracts_description_and_tags_from_docstring() {
    let raw = "Formats a value.\n\n@arg value  The value to format.\n@return str  Output.";
    let lines = docstring_lines(raw);
    assert_eq!(
        lines,
        vec![
            "Formats a value.".to_string(),
            "".to_string(),
            "@arg value  The value to format.".to_string(),
            "@return str  Output.".to_string(),
        ]
    );
}

#[test]
fn dedents_indented_docstring() {
    let raw = "        Does a thing.\n\n        @arg x  the input\n    ";
    let lines = docstring_lines(raw);
    assert_eq!(
        lines,
        vec![
            "Does a thing.".to_string(),
            "".to_string(),
            "@arg x  the input".to_string(),
        ]
    );
}

#[test]
fn absent_docstring_yields_no_content() {
    assert!(docstring_lines("").is_empty());
    assert!(docstring_lines("   \n  \n").is_empty());
}
