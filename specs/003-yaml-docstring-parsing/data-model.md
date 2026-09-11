# Data Model: YAML Docstring Parsing

**Branch**: `003-yaml-docstring-parsing` | **Date**: 2026-09-11 | **Plan**: [plan.md](plan.md)

This document defines the intermediary object representation and the YAML
deserialization schema used by the extraction stage. The model is unchanged
from the original feature (`001-pydoc-generator`); the change is solely in
how the extraction stage populates it — from YAML instead of `@`-tag lines.

## YAML Deserialization Schema

The extraction stage attempts to deserialize each docstring into this Rust
struct (via `serde_yml`):

```rust
// Not in model.rs — lives in extract.rs as an extraction-local type.
struct YamlDoc {
    description: Option<String>,
    args:        Option<Vec<YamlArg>>,
    returns:     Option<YamlReturn>,
    raises:      Option<Vec<YamlRaise>>,
}

struct YamlArg {
    name:        String,
    description: String,
}

struct YamlReturn {
    description: String,
}

struct YamlRaise {
    #[serde(alias = "exception")]
    r#type:      String,
    description: String,
}
```

Three-way dispatch on the raw docstring text:

| YAML parse result | Value type | Outcome |
|---|---|---|
| Mapping with recognized keys | `YamlDoc` | `DocComment::Ok(DocContent { ... })` |
| Scalar (plain prose) | `String` | `DocComment::Ok(DocContent { description: <text>, .. })` |
| Parse failure or wrong shape | — | `DocComment::Error("...")` |
| Missing / empty docstring | — | `DocComment::None` |

Unknown keys in the mapping are silently discarded by serde (they are not
present in the struct and serde's default behavior ignores them).

### Example: demo/demo_2.py (structural reference)

```yaml
description: A really cool method on this class.

args:
  - name: with_this_str
    description: A nice string to do something with.

returns:
  type: str          # ← ignored (not in YamlReturn struct)
  description: A cooler string.

raises:
  - type: ValueError
    description: If the string tries to be cool.

example: |           # ← ignored (not in YamlDoc struct)
  my_cooler_string: str = my_instance.do_something(my_str)
```

Extraction produces: `description = "A really cool method on this class."`,
`args = [{ name: "with_this_str", description: "A nice string..." }]`,
`returns = { description: "A cooler string." }`,
`raises = [{ exception: "ValueError", description: "If the string..." }]`.
The `type` and `example` keys have no effect on output.

## Intermediary Object Representation

These entities are unchanged from `001-pydoc-generator` (data-model.md).
The only difference is the extraction source: YAML mapping → these structs,
not `@`-tag lines → these structs.

### DocumentedElement (abstract)

| Field | Type | Description |
|-------|------|-------------|
| `path` | ImportPath | Full Python package import path; unique identifier. |
| `kind` | `Module \| Class \| Function` | Element type discriminator. |
| `doc` | DocComment | Parsed documentation (None, Error, or Ok). |
| `children` | `List<DocumentedElement>` | Nested elements. |

### ImportPath

| Field | Type | Description |
|-------|------|-------------|
| `segments` | `List<String>` | Ordered path segments. |
| `text` | `String` | Dot-joined rendering; display and index keys. |

### DocComment (tagged union)

| Variant | Meaning | Fields |
|---------|---------|--------|
| `None` | No docstring. Existence-only entry (FR-008). | — |
| `Error` | Docstring present but malformed YAML or wrong shape (FR-007). | `message: String` |
| `Ok` | Docstring parsed successfully. | `DocContent` (below) |

### DocContent

| Field | Type | YAML source key | Description |
|-------|------|----------------|-------------|
| `description` | `String` | `description` | Module/element description. |
| `args` | `Vec<ArgDoc>` | `args` | Argument documentation list. |
| `returns` | `Option<ReturnDoc>` | `returns` | Return value documentation. |
| `raises` | `Vec<RaisesDoc>` | `raises` | Exception documentation list. |

### ArgDoc

| Field | Type | YAML source key | Description |
|-------|------|----------------|-------------|
| `name` | `String` | `args[].name` | Argument name. |
| `description` | `String` | `args[].description` | Argument description. |

### ReturnDoc

| Field | Type | YAML source key | Description |
|-------|------|----------------|-------------|
| `description` | `String` | `returns.description` | Return value description. |

### RaisesDoc

| Field | Type | YAML source key | Description |
|-------|------|----------------|-------------|
| `exception` | `String` | `raises[].type` | Exception type/name. |
| `description` | `String` | `raises[].description` | When/why the exception is raised. |

### DocumentedProject (root)

| Field | Type | Description |
|-------|------|-------------|
| `root_path` | ImportPath | Import path prefix derived from source tree. |
| `elements` | `Vec<DocumentedElement>` | Top-level elements. |
| `index` | `BTreeMap<ImportPath, DocumentedElement>` | Flat index for navigation (FR-005). |

## Validation Rules (from requirements)

- **Unique identity (FR-004, SC-006)**: Every element has a unique
  `ImportPath`. Same-named elements in different modules remain distinct.
- **Coverage (FR-001, FR-008, SC-002)**: Every scanned element has a
  corresponding `DocumentedElement`, whether documented or not.
- **YAML extraction (FR-001, FR-003)**: Documentation content is sourced
  exclusively from the YAML interpretation of docstrings. `@`-tag parsing
  is removed; legacy `@`-tag docstrings are treated as malformed.
- **Malformed handling (FR-007, SC-004)**: Invalid YAML or wrong-shaped
  recognized keys yield `Error` for that element only; all other elements
  remain intact.
- **Prose handling (FR-010)**: A docstring that is valid YAML but not a
  mapping (plain scalar) is used as the description string; the element's
  `DocContent` is populated with `description` only and empty `args`/`raises`.
- **Determinism (FR-006, SC-005)**: Identical source produces an identical
  representation (path-sorted, stable ordering).

## State Transitions

The element's `doc` field transitions through the pipeline:

```
Source docstring ──► docstring_lines (dedent/trim) ──► YAML dispatch ──► DocComment
   (raw text)                                              │
                                            ┌──────────────┼──────────────┐
                                            ▼              ▼              ▼
                                        Empty/blank    YAML mapping   YAML scalar
                                            │              │          (prose)
                                            ▼              ▼              ▼
                                       DocComment::   DocComment::  DocComment::
                                         None           Ok            Ok
                                                         │          (description
                                                         ▼           only)
                                                    DocContent
                                                   (description,
                                                    args, returns,
                                                    raises)
```

There is no mutable lifecycle beyond construction; the representation is
immutable once produced and consumed by the HTML generator.
