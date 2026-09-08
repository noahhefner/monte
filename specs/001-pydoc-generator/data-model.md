# Data Model: Python Documentation Generator

**Branch**: `001-pydoc-generator` | **Date**: 2026-09-08 | **Plan**: [plan.md](plan.md)

This document defines the intermediary object representation produced in
step 2 of the pipeline (parse → **extract into intermediary representation** →
generate HTML). It is the contract between the docstring-extraction stage
and the HTML-generation stage.

## Entities

### DocumentedElement (abstract)

Any module, class, or function in the scanned source that can appear in the
generated documentation.

| Field | Type | Description |
|-------|------|-------------|
| `path` | ImportPath | Full Python package import path (e.g. `mypkg.sub.mod.Class.method`), the unique identifier. |
| `kind` | `Module \| Class \| Function` | Discriminator for the element type. |
| `doc` | DocComment | Parsed documentation. May be "none" (undocumented element), "error" (malformed docstring), or "ok" with fields populated. |
| `children` | List<DocumentedElement> | Nested elements (modules inside packages; classes/functions inside modules/classes). |

### ImportPath

The full dotted text used to import the element (e.g. `pkg.mod.Class`).

| Field | Type | Description |
|-------|------|-------------|
| `segments` | List<String> | Ordered segments from package root to the element. |
| `text` | String | Dot-joined rendering; used for display and index keys. |

### DocComment (tagged union)

Represents the outcome of parsing the docstring of an element.

| Variant | Meaning | Fields |
|---------|---------|--------|
| `None` | No docstring present. Element is documented as existing-without-detail (FR-008). | — |
| `Error` | Docstring present but malformed. Entry shows an error message and does not fail the run (FR-007). | `message: String` (e.g. "docstring was not written correctly") |
| `Ok` | Docstring parsed successfully. | `description: String`, `args: List<ArgDoc>`, `returns: Option<ReturnDoc>`, `raises: List<RaisesDoc>` |

### ArgDoc (`@arg`)

| Field | Type | Description |
|-------|------|-------------|
| `name` | String | Argument name as written in the docstring. |
| `description` | String | Free-text description of the argument. |

### ReturnDoc (`@return`)

| Field | Type | Description |
|-------|------|-------------|
| `description` | String | Description of the return value. |

### RaisesDoc (`@raises`)

| Field | Type | Description |
|-------|------|-------------|
| `exception` | String | Exception name/type as written in the docstring. |
| `description` | String | When/why the exception might be raised. |

### DocumentedProject (root)

The top-level representation of the generated site.

| Field | Type | Description |
|-------|------|-------------|
| `root_path` | ImportPath | The import path prefix derived from the scanned source tree. |
| `elements` | List<DocumentedElement> | Top-level elements (packages/modules). |
| `index` | Map<ImportPath, DocumentedElement> | Flat index of every element keyed by full path, used for navigation (FR-005). |

## Validation Rules (from requirements)

- **Unique identity (FR-004, SC-006)**: Every element MUST have a unique
  `ImportPath`. Elements with the same simple name in different modules MUST
  remain individually addressable.
- **Coverage (FR-001, FR-008, SC-002)**: Every scanned module, class, and
  function MUST have a corresponding element in the representation, whether
  documented or not.
- **Tag scope (FR-002)**: Only `@arg`, `@return`, and `@raises` tags plus
  description text are represented. No type information is extracted from
  source.
- **Malformed handling (FR-007, SC-005)**: A malformed docstring yields the
  `Error` variant for that element only; all other elements remain intact.
- **Determinism (FR-006, SC-004)**: Identical source MUST produce an
  identical representation (stable ordering, e.g. path-sorted).

## State Transitions

The element's `doc` field transitions through the pipeline:

```
Source docstring ──► parse ──► DocComment
   (raw text)                ├─ None   (no docstring)
                             ├─ Error  (malformed)
                             └─ Ok     (description, args, returns, raises)
```

There is no mutable lifecycle beyond construction; the representation is
immutable once produced and is consumed by the HTML generator.
