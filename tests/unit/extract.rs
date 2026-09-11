//! Unit tests for YAML docstring extraction (FR-001, FR-007, FR-010):
//! structured mappings, plain-prose docstrings, invalid YAML, wrong-shape
//! keys, and legacy `@`-tag docstrings.

use pydoc_gen::extract::{YamlParseResult, parse_yaml_docstring};
use pydoc_gen::model::DocContent;

fn ok(content: DocContent) -> YamlParseResult {
    YamlParseResult::Ok(content)
}

fn empty_content(description: &str) -> DocContent {
    DocContent {
        description: description.to_string(),
        ..DocContent::default()
    }
}

// --- US1: well-formed YAML docstrings ---

#[test]
fn full_mapping_extracts_all_sections() {
    let text = r#"description: A real docstring.

args:
  - name: with_this_str
    description: A nice string.

returns:
  description: A cooler string.

raises:
  - type: ValueError
    description: If the string is too cool.
  - exception: TypeError
    description: Wrong kind of cool."#;
    match parse_yaml_docstring(text) {
        YamlParseResult::Ok(content) => {
            assert_eq!(content.description, "A real docstring.");
            assert_eq!(content.args.len(), 1);
            assert_eq!(content.args[0].name, "with_this_str");
            assert_eq!(content.args[0].description, "A nice string.");
            assert!(content.returns.is_some());
            assert_eq!(
                content.returns.as_ref().unwrap().description,
                "A cooler string."
            );
            assert_eq!(content.raises.len(), 2);
            assert_eq!(content.raises[0].exception, "ValueError");
            assert_eq!(content.raises[1].exception, "TypeError");
        }
        other => panic!("expected Ok, got {other:?}"),
    }
}

#[test]
fn description_only_mapping_is_ok() {
    let text = "description: Just a description.";
    match parse_yaml_docstring(text) {
        YamlParseResult::Ok(content) => {
            assert_eq!(content.description, "Just a description.");
            assert!(content.args.is_empty());
            assert!(content.returns.is_none());
            assert!(content.raises.is_empty());
        }
        other => panic!("expected Ok, got {other:?}"),
    }
}

#[test]
fn unknown_keys_are_tolerated() {
    let text = r#"description: Structured docs.

example: |
  fn do_it(s: str) -> str: ...

returns:
  type: str
  description: A string.

args:
  - name: s
    description: input
    optional: false"#;
    match parse_yaml_docstring(text) {
        YamlParseResult::Ok(content) => {
            assert_eq!(content.description, "Structured docs.");
            assert_eq!(content.args.len(), 1);
            assert_eq!(content.args[0].name, "s");
            assert_eq!(content.args[0].description, "input");
            assert!(content.returns.is_some());
            assert_eq!(
                content.returns.as_ref().unwrap().description,
                "A string."
            );
        }
        other => panic!("expected Ok, got {other:?}"),
    }
}

// --- US1: plain-prose docstrings (FR-010) ---

#[test]
fn prose_docstring_is_description_only() {
    let text = "A documentation comment without structure.";
    assert_eq!(
        parse_yaml_docstring(text),
        ok(empty_content("A documentation comment without structure."))
    );
}

#[test]
fn multi_line_prose_is_a_description() {
    let text = "First line.\n\nSecond line.";
    match parse_yaml_docstring(text) {
        YamlParseResult::Ok(content) => {
            assert!(content.description.starts_with("First line."));
            assert!(content.description.contains("Second line."));
            assert!(content.args.is_empty());
        }
        other => panic!("expected Ok, got {other:?}"),
    }
}

// --- US2: invalid YAML / wrong-shape keys (FR-007) ---

#[test]
fn malformed_yaml_is_an_error() {
    let text = "args: [unclosed";
    match parse_yaml_docstring(text) {
        YamlParseResult::Error(msg) => {
            assert!(msg.contains("docstring was not written correctly"));
        }
        other => panic!("expected Error, got {other:?}"),
    }
}

#[test]
fn tab_indentation_is_an_error() {
    let text = "args:\n\t- name: x\n\t  description: y";
    assert!(matches!(
        parse_yaml_docstring(text),
        YamlParseResult::Error(_)
    ));
}

#[test]
fn args_wrong_shape_is_an_error() {
    let text = "args: nope";
    assert!(matches!(
        parse_yaml_docstring(text),
        YamlParseResult::Error(_)
    ));
}

#[test]
fn missing_required_field_is_an_error() {
    let text = "args:\n  - name: x\n  - {}\n";
    assert!(matches!(
        parse_yaml_docstring(text),
        YamlParseResult::Error(_)
    ));
}

#[test]
fn returns_wrong_shape_is_an_error() {
    let text = "returns: not a mapping";
    assert!(matches!(
        parse_yaml_docstring(text),
        YamlParseResult::Error(_)
    ));
}

#[test]
fn raises_without_type_is_an_error() {
    let text = "raises:\n  - description: no type here";
    assert!(matches!(
        parse_yaml_docstring(text),
        YamlParseResult::Error(_)
    ));
}

// --- US3: legacy `@`-tag docstrings (FR-007) ---

#[test]
fn legacy_arg_tag_is_an_error() {
    let text = "@arg x  the input";
    match parse_yaml_docstring(text) {
        YamlParseResult::Error(msg) => {
            assert!(msg.contains("docstring was not written correctly"));
        }
        other => panic!("expected Error, got {other:?}"),
    }
}

#[test]
fn legacy_return_tag_is_an_error() {
    let text = "Legacy docs.\n\n@return str  output";
    assert!(matches!(
        parse_yaml_docstring(text),
        YamlParseResult::Error(_)
    ));
}

#[test]
fn legacy_raises_tag_is_an_error() {
    let text = "@raises ValueError  when it goes wrong";
    assert!(matches!(
        parse_yaml_docstring(text),
        YamlParseResult::Error(_)
    ));
}
