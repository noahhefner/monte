# Research: YAML Docstring Parsing

**Branch**: `003-yaml-docstring-parsing` | **Date**: 2026-09-11 | **Plan**: [plan.md](plan.md)

## Decisions

### D-001: Replace manual tag parsing with YAML extraction

**Decision**: Remove the line-by-line `@`-tag parser (`tags.rs`) and replace
it with a YAML-document parser in `extract.rs`. The existing docstring
normalization (`docstring.rs` — dedent, trim) is retained because it produces
a clean text block suitable for YAML input.

**Rationale**: The feature's core requirement (FR-001) is to interpret
docstrings as YAML instead of manually parsing `@`-prefixed lines. The data
model (`DocContent`, `ArgDoc`, `ReturnDoc`, `RaisesDoc`) is unchanged, so the
change is localized to extraction: YAML is parsed, then mapped onto the same
typed structs the rest of the pipeline already consumes.

**Alternatives considered**: Keeping both parsers and selecting based on
content (rejected — user explicitly chose YAML-only; maintaining two parsers
adds complexity with no benefit).

### D-002: YAML crate — `serde_yml` (serde-based)

**Decision**: Use the `serde_yml` crate (community-maintained successor to
`serde_yaml`, serde integration, pure-Rust, no C dependencies). Deserialize
each docstring into a typed `YamlDoc` struct that models the schema in
`data-model.md`.

**Rationale**: Serde integration means the YAML schema is expressed as Rust
struct definitions — the extraction code reads as a declarative contract
rather than procedural field-picking. This directly supports the Code
Readability principle. `serde_yml` is actively maintained; pure-Rust avoids
build-time C dependency issues.

**Alternatives considered**:
- `yaml-rust2` (pure-Rust, actively maintained, no serde integration — would
  require manual Value extraction; less readable and more error-prone for
  structured data, rejected for Readability reasons).
- `serde_yaml` (deprecated by original author in 2023; functional but no
  longer maintained; `serde_yml` is its successor).

### D-003: Docstring detection strategy — three-way dispatch

**Decision**: Each docstring is classified by a three-way dispatch:

1. **Empty or missing** → `DocComment::None` (existence-only entry, FR-008).
2. **Valid YAML, is a mapping** → extract structured fields from the mapping
   (`description`, `args`, `returns`, `raises`); unknown keys are silently
   ignored per the spec. Wrong-shaped recognized keys (e.g. `args` is a
   string instead of a list) → `DocComment::Error` (FR-007).
3. **Valid YAML, is not a mapping** (plain scalar) → use as the description
   string (FR-010, prose fallback).
4. **Invalid YAML** (parse fails) → `DocComment::Error` (FR-007).

This is implemented as: attempt `serde_yml::from_str::<YamlDoc>` first; on
deserialization error, fall back to `serde_yml::from_str::<String>` for prose.
If both fail, report malformed.

**Rationale**: Covers all spec-defined cases (FR-007, FR-010) with no
ambiguous middle states. The fallback from mapping to scalar is cheap and
deterministic.

**Alternatives considered**: Requiring a YAML type hint (e.g. `# yaml: doc`)
to distinguish prose from structure (rejected — adds authoring burden for no
clear benefit; a mapping's colon-delimited keys are unambiguous).

### D-004: Unknown-key handling — silent ignore

**Decision**: Any key in the YAML mapping that is not one of the four
recognized keys (`description`, `args`, `returns`, `raises`) is silently
discarded. No error is raised for unknown keys.

**Rationale**: Per the spec (edge cases, resolved): unknown keys are ignored
without failing the run. This allows `demo/demo_2.py`-style docstrings
containing `example` and `type` fields to be parsed successfully — those keys
simply have no effect on output.

**Alternatives considered**: Reporting an error for unrecognized keys (rejected
— would cause false positives for docstrings like `demo/demo_2.py` and reduce
adoption).

### D-005: Handling legacy `@`-tag docstrings — malformed error

**Decision**: Docstrings beginning with `@` (e.g. `@arg x input`) are
invalid YAML (the `@` character is reserved in YAML 1.1 and causes a parse
error in standard YAML 1.2). Such docstrings fail YAML parsing and produce a
`DocComment::Error` entry for the affected element. The run continues.

**Rationale**: Per the spec clarification (Q1:A) — legacy JavaDoc-style
docstrings are explicitly treated as malformed; the author must rewrite them.
This is a deliberate, documented breaking change.

**Alternatives considered**: A fallback `@`-tag parser for non-YAML docstrings
(rejected per Q1:A; maintaining two parsing paths contradicts the feature's
goal of YAML-only extraction).

## Alternatives Evaluation Summary

| Alternative | Evaluated? | Outcome |
|-------------|-----------|---------|
| Keep `@`-tag parser alongside YAML | Yes | Rejected (Q1:A: YAML-only) |
| `yaml-rust2` instead of `serde_yml` | Yes | Rejected (no serde; less readable) |
| Require YAML type-hint annotation | Yes | Rejected (authoring burden) |
| Error on unknown YAML keys | Yes | Rejected (false positives) |
| Fallback `@`-tag parser for legacy | Yes | Rejected per spec clarification |
