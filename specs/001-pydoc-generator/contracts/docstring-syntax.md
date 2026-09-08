# Contract: Docstring Syntax (JavaDoc-style for Python)

**Branch**: `001-pydoc-generator` | **Date**: 2026-09-08 | **Plan**: [plan.md](plan.md)

The tool reads documentation from JavaDoc-style Python docstrings. A
docstring is the string literal that opens a module, class, or function body
(PEP 257). **`#` comment blocks are NOT supported** as a documentation
source (FR-002).

## Document Association (FR-003)

A docstring is associated with the module, class, or function it opens — the
module docstring is the first statement of a file when it is a string
literal; a class/function docstring is the first statement of its body when
it is a string literal. An element with no docstring is treated as
undocumented (FR-008).

## Supported Tags (FR-002)

Exactly three tags are supported, plus free description text (the docstring
body):

| Tag | Applies to | Meaning |
|-----|-----------|---------|
| `@arg <name> <description>` | Function | Documents a single argument. |
| `@return <description>` | Function | Documents the return value. |
| `@raises <exception> <description>` | Any | Documents an exception that might be raised. |

All other content in the docstring is treated as description text. Any tag
outside this set is not recognized (see "Unrecognized tags" below).

## Example

```python
def format_value(text: str) -> str:
    """Provides formatting helpers.

    @arg text  The input string to format.
    @return    The formatted string.
    @raises ValueError  Raised when text is empty.
    """
    ...
```

## Requirements & Rules

- **Content source**: Only developer-provided docstrings are used for
  documentation content. Element names come from the AST; types are NOT
  inferred from source code.
- **Docstring normalization**: The docstring is dedented (common leading
  whitespace removed) and leading/trailing blank lines are dropped before
  parsing tags, so `@arg` lines are recognized regardless of indentation.
- **Missing description / malformed block (FR-007)**: If a docstring is
  malformed (e.g. `@arg` with no name, or malformed `@raises`), the affected
  element's entry displays an error message stating the docstring was not
  written correctly. The run continues and other elements are documented
  normally.
- **Undocumented element (FR-008)**: No docstring yields an existence-only
  entry (name and full path, no extra detail).

## Out of Scope (v1)

- Type extraction from the source AST/signature.
- Tags other than `@arg`, `@return`, `@raises`.
- `#` comment blocks as a documentation source.

## Unrecognized tags

A tag that is not in the supported set is reported/ignored without failing
the run. The surrounding description text is still captured. The exact
treatment is a validation case in [quickstart.md](../quickstart.md).