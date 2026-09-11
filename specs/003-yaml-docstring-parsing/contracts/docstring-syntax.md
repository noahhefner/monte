# Contract: Docstring Syntax (YAML for Python docstrings)

**Branch**: `003-yaml-docstring-parsing` | **Date**: 2026-09-11 | **Plan**: [plan.md](plan.md)

The tool reads documentation from Python docstrings interpreted as YAML
documents. This replaces the previous JavaDoc-style `@arg`/`@return`/`@raises`
tag format. A docstring is the string literal that opens a module, class, or
function body (PEP 257). `#` comment blocks remain unsupported as a
documentation source.

## Document Association (FR-002)

A docstring is associated with the module, class, or function it opens — the
module docstring is the first statement of a file when it is a string literal;
a class/function docstring is the first statement of its body when it is a
string literal. An element with no docstring is treated as undocumented
(FR-008).

## YAML Docstring Format

A well-formed docstring is a valid YAML mapping at the document root. The
following keys are recognized:

| Key | Type | Meaning | Applies to |
|-----|------|---------|-----------|
| `description` | Scalar | Free-text description of the module, class, or function. | Any |
| `args` | List | Arguments accepted by the function. Each entry has a `name` (scalar) and a `description` (scalar). | Functions |
| `returns` | Mapping | Return value documentation. Contains a `description` (scalar). | Functions |
| `raises` | List | Exceptions the function might raise. Each entry has a `type` (exception name, scalar) and a `description` (scalar). | Any |

### Structural reference

The file `demo/demo_2.py` in the repository demonstrates the intended YAML
authoring style. Additional keys present in that file (e.g. `example`,
`returns.type`) are tolerated but silently ignored by the tool — only the
four recognized keys above are extracted (FR-003).

### Example

```python
def format_value(text: str) -> str:
    """Provides formatting helpers.

    description: Formats text for display.

    args:
      - name: text
        description: The input string to format.

    returns:
      description: The formatted string.

    raises:
      - type: ValueError
        description: Raised when text is empty.
    """
    ...
```

## Plain-prose docstrings (FR-010)

A docstring that is valid YAML but not a mapping (i.e. a plain scalar) is
treated as a description-only entry: the entire text becomes the element's
`description`. No structured fields (`args`, `returns`, `raises`) are
extracted.

```python
def simple():
    """A brief description of what this does."""
    pass
```

## Undocumented element (FR-008)

No docstring yields an existence-only entry (name and full path, no extra
detail).

## Malformed / unparseable docstring (FR-007)

A docstring that is invalid YAML, or a YAML mapping whose recognized keys
have the wrong shape (e.g. `args` is a string instead of a list), produces a
`DocComment::Error` entry for the affected element. The error message states
the docstring was not written correctly. The run continues and all other
elements are documented normally.

Legacy JavaDoc-style `@arg`/`@return`/`@raises` docstrings fall into this
category — the `@` prefix causes a YAML parse failure, and the entry shows
the error message.

## Prose-with-colons risk

A docstring whose text contains a colon and space (e.g. "Formats text.
Returns: a string.") may parse as a YAML mapping of unrecognized keys. In
this case the entry has no `description` and the text is effectively lost.
Authors should avoid colons in plain-prose docstrings, or use the structured
mapping format instead.

## Requirements & Rules

- **Content source**: Only developer-provided docstrings are used for
  documentation content. Element names come from the AST; types are NOT
  inferred from source code.
- **Docstring normalization**: The docstring is dedented (common leading
  whitespace removed) and leading/trailing blank lines are dropped before YAML
  parsing, consistent with PEP 257.
- **Unknown keys**: Silently ignored. No error is raised for keys not in the
  recognized set (e.g. `example`, `type` on returns).
- **Deterministic output**: Identical docstrings produce identical extracted
  data (FR-006).

## Out of Scope (this version)

- `example`, `returns.type`, and other keys shown in `demo/demo_2.py`.
- `#` comment blocks as a documentation source.
- Type inference from source code.
- Migration or auto-conversion of legacy JavaDoc-style docstrings.
