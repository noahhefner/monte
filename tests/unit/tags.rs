//! T011 - Unit tests for the JavaDoc tag parser.

use pydoc_gen::tags::{TagParseResult, parse_block};

#[test]
fn parses_all_supported_tags() {
    let lines = vec![
        "Formats a value.".to_string(),
        "@arg value  The value to format.".to_string(),
        "@return     The formatted string.".to_string(),
        "@raises ValueError  Raised when value is empty.".to_string(),
    ];
    match parse_block(&lines) {
        TagParseResult::Ok(content) => {
            assert!(content.description.contains("Formats a value."));
            assert_eq!(content.args.len(), 1);
            assert_eq!(content.args[0].name, "value");
            assert_eq!(content.args[0].description, "The value to format.");
            let ret = content.returns.as_ref().expect("return present");
            assert_eq!(ret.description, "The formatted string.");
            assert_eq!(content.raises.len(), 1);
            assert_eq!(content.raises[0].exception, "ValueError");
            assert_eq!(
                content.raises[0].description,
                "Raised when value is empty."
            );
        }
        other => panic!("expected ok, got {other:?}"),
    }
}

#[test]
fn missing_arg_name_is_an_error() {
    match parse_block(&["@arg".to_string()]) {
        TagParseResult::Error(msg) => assert!(msg.contains("argument name")),
        other => panic!("expected error, got {other:?}"),
    }
}

#[test]
fn missing_raises_name_is_an_error() {
    match parse_block(&["@raises".to_string()]) {
        TagParseResult::Error(msg) => assert!(msg.contains("exception name")),
        other => panic!("expected error, got {other:?}"),
    }
}

#[test]
fn unrecognized_tags_are_ignored_not_errors() {
    match parse_block(&["@author Jane".to_string(), "text".to_string()]) {
        TagParseResult::Ok(content) => {
            assert!(content.description.contains("text"));
            assert!(content.args.is_empty());
        }
        other => panic!("expected ok, got {other:?}"),
    }
}
