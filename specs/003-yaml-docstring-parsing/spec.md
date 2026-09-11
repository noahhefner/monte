# Feature Specification: YAML Docstring Parsing

**Feature Branch**: `003-yaml-docstring-parsing`

**Created**: 2026-09-11

**Status**: Draft

**Input**: User description: "I want this application to interpret docstrings as yaml and extract data that way instead of manually parsing the string."

## Clarifications

### Session 2026-09-11

- Q: How should existing JavaDoc-style `@arg`/`@return`/`@raises` docstrings
  be handled? → A: Rejected as malformed. Such docstrings do not parse as
  YAML, the affected element's entry shows an error message, and the run
  continues (FR-007).
- Q: What fields should the YAML schema extract? → A: The current fields only
  — `description`, a list of `args` (each `name` + `description`), `returns`
  (`description`), and a list of `raises` (each exception `type`/name +
  `description`). `example` and `type` keys shown in the demo are out of
  scope (FR-003).
- Q: How should a docstring that is valid YAML but not a mapping (plain
  prose) be handled? → A: It remains valid and is used as a description-only
  entry (FR-010).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Document code with structured YAML docstrings (Priority: P1)

A developer writes a Python docstring whose body is a structured YAML
mapping instead of JavaDoc-style `@` tags. The tool reads each module, class,
and function docstring as YAML and extracts the documented information —
description, arguments, return value, and exceptions — into the generated
documentation, without hand-parsing the text line by line.

**Why this priority**: This is the core of the feature. Replacing manual
text parsing with YAML extraction is the entire purpose of the change, so it
must land first and deliver value on its own.

**Independent Test**: Can be fully tested by running the tool over a small
sample codebase whose docstrings are YAML mappings and confirming every
documented element's page shows the description, arguments, return value, and
exceptions extracted from the YAML (not from `@` tags).

**Acceptance Scenarios**:

1. **Given** a module, class, or function whose docstring contains a YAML
   mapping with `description`, `args`, `returns`, and `raises` keys, **When**
   the generator runs, **Then** the generated documentation shows the
   description, each argument with its description, the return value, and each
   exception with its description, matching the values in the YAML.
2. **Given** a docstring that is completely plain prose (no YAML structure),
   **When** the generator runs, **Then** the element is documented with that
   prose as its description.

---

### User Story 2 - Receive clear feedback for docstrings that are not valid YAML (Priority: P2)

A developer writes a docstring that is not valid YAML (for example, a
malformed mapping or leftover JavaDoc-style `@arg`/`@return`/`@raises`
lines). The run does not abort: every other element is still documented, and
the affected element's entry displays an error message stating the docstring
could not be interpreted.

**Why this priority**: Robust, non-fatal behavior is already a core
expectation of the tool (a bad docstring must never break the whole run).
Preserving that guarantee around YAML parsing keeps the tool trustworthy.

**Independent Test**: Can be tested by feeding the generator a file with one
valid YAML docstring and one invalid-YAML docstring, then confirming the
valid element is documented normally and the invalid element shows an error
message while the run completes.

**Acceptance Scenarios**:

1. **Given** a docstring that fails YAML parsing, **When** the generator runs,
   **Then** the affected element's entry displays an error message stating the
   docstring was not written correctly.
2. **Given** a source tree where one docstring is invalid YAML, **When** the
   generator runs, **Then** the run completes and all other properly
   documented elements render normally.

---

### User Story 3 - Migrate existing JavaDoc-style documentation (Priority: P3)

A developer who already documented their codebase with the current
JavaDoc-style `@arg`/`@return`/`@raises` format has a clear, predictable
migration story after upgrading the tool.

**Why this priority**: How existing users migrate is important for adoption,
but it is secondary to the new YAML format itself.

**Independent Test**: Can be tested by pointing the generator at a codebase
using the legacy tag format and confirming the outcome matches the chosen
migration behavior.

**Acceptance Scenarios**:

1. **Given** a codebase documented with JavaDoc-style `@tag` docstrings,
   **When** the upgraded generator runs, **Then** each such docstring is
   treated as malformed: the affected element's entry displays an error
   message, and the run completes with all other elements documented normally.

---

### Edge Cases

- What happens when a docstring is empty or blank? (Resolved: existing
  behavior is preserved — blank or empty docstrings produce an
  existence-only entry, not an error.)
- What happens when a docstring is plain prose rather than a YAML mapping?
  (Resolved: it is still valid; the prose is used as the element's
  description. Only mapping-structured docstrings are parsed for structured
  fields.)
- What happens when a docstring contains JavaDoc-style `@` lines? (Resolved:
  such docstrings do not parse as YAML and are treated as malformed; the
  affected element's entry shows an error message and the run continues.)
- What happens when a YAML mapping contains keys the tool does not recognize?
  (Resolved: unknown keys are ignored without failing the run; recognized
  keys are extracted. A docstring whose prose contains a colon, e.g.
  "Formats text. Returns: a string.", can parse as a mapping of unknown keys
  and therefore contribute no description — this is a documented authoring
  risk. Prose without colons remains a plain scalar and is used as the
  description.)
- What happens when a recognized key has the wrong shape, e.g. `args` is a
  string instead of a list? (Resolved: this is treated as a malformed
  docstring; the element's entry shows an error message, per FR-007.)
- What happens when YAML itself is ambiguous or uses flow syntax such as
  `{...}`? (Definition: any valid YAML document is acceptable input; the
  tool reads it with a standard YAML interpretation.)
- What happens when a docstring whose prose contains a colon (e.g.
  "Returns: a string.")? (Resolved: the colon can cause the prose to parse as
  a YAML mapping of unrecognized keys. Per the chosen behavior the entry then
  has no description; this risk is documented so authors avoid colons in
  prose, per the assumptions below.)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST interpret each module, class, and function
  docstring as a YAML document and extract documentation data from it. This
  replaces the current line-based manual parsing of `@arg`/`@return`/`@raises`
  tags as the source of documentation content.
- **FR-002**: The system MUST associate the extracted documentation with the
  same element the docstring opens (module, class, or function), preserving
  the existing association rules.
- **FR-003**: The system MUST extract the following information from a YAML
  docstring when present, and nothing more: the `description`, a list of
  `args` (each with a `name` and a `description`), the `returns` record (with
  its `description`), and a list of `raises` (each with an exception
  `type`/name and a `description`). No other fields (e.g. `example`, argument
  types, return types) are extracted in this version.
- **FR-004**: The system MUST still generate an entry for every module,
  class, and function — with full detail when its YAML docstring parses
  successfully, and an existence-only entry when there is no docstring.
- **FR-005**: The system MUST continue to provide a navigation/index view
  letting a user locate any documented element by its full Python package
  import path.
- **FR-006**: The system MUST preserve deterministic regeneration: re-running
  on unchanged source produces byte-for-byte identical output, and changed
  source produces updated output.
- **FR-007**: The system MUST handle a docstring that cannot be interpreted as
  YAML by giving the affected element an entry containing an error message
  stating the docstring was not written correctly. This includes legacy
  JavaDoc-style docstrings (`@arg`/`@return`/`@raises` lines), which do not
  parse as YAML and are therefore treated as malformed. The overall run MUST
  NOT fail; all other properly documented elements MUST still be documented.
- **FR-008**: The system MUST generate an existence-only entry for any
  module, class, or function that has no docstring.
- **FR-009**: The system MUST accept a target output location where the
  generated HTML site is written (unchanged).
- **FR-010**: The system MUST treat a docstring that is valid YAML but not a
  mapping (i.e. plain prose, a scalar or flow value) as a description-only
  entry, using the whole docstring as the element's description. Only
  mapping-structured docstrings are parsed for the structured fields in
  FR-003.

### Key Entities *(include if feature involves data)*

- **Documented Element**: A module, class, or function. Its documentation
  content now comes from a YAML-structure docstring instead of parsed `@`
  tag lines.
- **Documentation Content**: The data extracted from a YAML docstring —
  description, arguments, return value, and exceptions (the structured fields
  in FR-003; no additional fields in this version).
- **Generated Documentation Site**: The HTML output organizing documented
  elements into navigable pages (unchanged).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can generate a complete HTML documentation site for a
  sample project whose docstrings are YAML mappings, in under 60 seconds
  using the standard command.
- **SC-002**: 100% of elements in the scanned source are represented in the
  generated HTML — with full YAML-extracted detail when documented and an
  existence-only entry when not.
- **SC-003**: A YAML docstring's description, arguments, return value, and
  exceptions are reproduced exactly (field-for-field, value-for-value) in the
  generated documentation.
- **SC-004**: An invalid-YAML docstring never aborts the run; the affected
  element's entry displays a clear error message and all other elements are
  documented.
- **SC-005**: Re-running the generator on unchanged source produces identical
  output, verified by byte-for-byte comparison.
- **SC-006**: No path in the generated site relies on manual parsing of
  `@arg`/`@return`/`@raises` syntax — documentation content is sourced from
  the YAML interpretation of docstrings.

## Assumptions

- YAML is the agreed documentation format going forward; this is a deliberate,
  breaking change to the docstring contract previously defined in
  `contracts/docstring-syntax.md`. Legacy JavaDoc-style docstrings are not
  migrated or auto-converted; they are reported as malformed (FR-007) and the
  author must rewrite them.
- The extracted fields map exactly onto the existing documentation structure
  (description, arguments with name + description, return description,
  exceptions with name + description), so the generated-output shape and
  existing HTML structure remain largely unchanged. Richer fields shown in the
  demo (`demo/demo_2.py`) such as `example` and `type` keys are explicitly out
  of scope for this version (FR-003).
- Plain-prose docstrings remain valid: a docstring that is not a YAML mapping
  is used as a description-only entry (FR-010). Authors should avoid prose
  containing a colon followed by a space, since that can parse as a YAML
  mapping of unrecognized keys and yield an empty description.
- YAML parsing is delegated to a standard YAML reader; the tool does not write
  a YAML parser from scratch.
- The tool's other guarantees — determinism, existence-only entries for
  undocumented code, never-aborting on bad input, navigation index — are
  preserved by this change.
- Comment-block (`#`) documentation and type-inference from source remain out
  of scope, as before.