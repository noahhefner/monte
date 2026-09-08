# Feature Specification: Python Documentation Generator

**Feature Branch**: `001-pydoc-generator`

**Created**: 2026-09-08

**Status**: Draft

**Input**: User description: "Build a JavaDoc-style documentation generator but for Python. Code authors put docstrings in modules, functions, and classes, which are read by the tool. Then, the tool produces documentation in HTML format for the code."

## Clarifications

### Session 2026-09-08

- Q: How should the tool handle modules, classes, or functions that have no
  docstring? → A: Still create an entry for them, without extra
  information — just indicate that the element exists.
- Q: How should the tool handle malformed docstrings? → A: The entry for
  the affected element displays an error message stating the docstring was
  not written correctly; the overall run continues and still produces docs
  for all other properly documented code.
- Q: How should the tool avoid ambiguity between elements that share the
  same name across modules? → A: Each entry presents the element's full
  Python package import path (the text used to import the code), which
  uniquely identifies it.
- Q: What implementation language, parser, and supported tags should be
  used, and what is the high-level processing flow? → A: Built in Rust,
  parsing the AST with the rustpython_parser crate (no parser written from
  scratch). Documentation is sourced from Python docstrings only (FR-002
  update: comment blocks are NOT supported); supported tags are `@arg`,
  `@return`, and `@raises`. Information is sourced solely from
  developer-provided docstrings (element names come from the AST; inferring
  types from source is out of scope). Flow: parse source to AST → extract
  docstrings into an intermediary object representation → generate HTML
  from that representation.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Generate documentation from annotated code (Priority: P1)

A developer writes JavaDoc-style docstrings in modules, functions, and
classes in their Python codebase, describing each element's purpose,
arguments, and return values, along with any relevant tags. The developer
runs the generator, which scans the code, reads these docstrings, and
produces an HTML documentation site that organizes the documented elements
in a navigable structure so readers can understand the code's API.

**Why this priority**: This is the core value the tool delivers. Without
transforming source into browsable HTML, the tool does nothing.

**Independent Test**: Can be fully tested by running the generator over a
small sample codebase and confirming the HTML output contains every code
element — with full detail when documented and a bare existence entry
otherwise.

**Acceptance Scenarios**:

1. **Given** a codebase with JavaDoc-style docstrings on a module, a class,
   and a function, **When** the generator runs, **Then** the HTML output
   contains a page/section for each documented element with its full
   description.
2. **Given** a function documented with arguments and a return value,
   **When** the generator runs, **Then** the HTML output lists each
   argument with its description and shows the return value description.

---

### User Story 2 - Maintain accurate docs on code changes (Priority: P2)

A developer updates a function's signature and its docstring, then
re-regenerates documentation. The regenerated HTML reflects the updated
information without the developer editing the HTML by hand.

**Why this priority**: Keeping documentation in sync with code is what makes
it trustworthy and useful over time. Regeneration is a fundamental
expectation.

**Independent Test**: Can be tested by changing a function docstring and
re-running the generator, then confirming the HTML reflects only the updated
content.

**Acceptance Scenarios**:

1. **Given** previously generated documentation, **When** the source
   docstring for a function changes, **Then** re-running the generator
   replaces the affected HTML content with the updated description.
2. **Given** a documented element removed from the source, **When** the
   generator runs again, **Then** the corresponding documentation is no
   longer present in the output.

---

### User Story 3 - Navigate large codebases efficiently (Priority: P3)

A developer reading a large package uses the generated HTML site to locate a
specific class or function quickly via the navigation index rather than
reading the raw source.

**Why this priority**: Navigation is a meaningful enhancement but not
required for the basic tool value; it becomes important at scale.

**Independent Test**: Can be tested by generating docs for a multi-file
package and confirming a reader can reach any documented element from the
index in a few clicks.

**Acceptance Scenarios**:

1. **Given** a package with several documented modules, **When** the
   generator runs, **Then** the HTML site provides an index listing all
   documented modules and their contained classes and functions.
2. **Given** a documented function nested inside a class, **When** a user
   selects it from the index, **Then** the site navigates to the relevant
   documentation.

---

### Edge Cases

- What happens when a function, class, or module has no docstring at all?
  (Resolved: still produce an entry indicating the element exists, without
  extra information.)
- How does the generator handle incomplete or malformed docstrings, such as
  an `@arg` tag with no argument name? (Resolved: the affected element's
  entry displays an error message stating the docstring was not written
  correctly, and the run continues for all other code.)
- How does the output handle duplicate names across modules (e.g., two
  classes named `Helper` in different files)? (Resolved: each entry displays
  the element's full Python package import path, which uniquely identifies
  it and prevents ambiguity between same-named elements.)
- How does the generator behave when given an empty directory, a directory
  with no Python files, or a path that does not exist?
- What output is produced when a docstring uses a tag the tool does not
  recognize?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST scan a provided source directory (and its
  subdirectories) and identify all Python modules, classes, and functions,
  regardless of whether they carry a docstring.
- **FR-002**: The system MUST read documentation from Python docstrings
  (the string literal that opens a module, class, or function). Exactly
  three tags are parsed from the docstring: `@arg` (arguments), `@return`
  (return value), and `@raises` (exceptions that might be raised), along
  with the docstring's description text. It MUST rely solely on these
  developer-provided docstrings and MUST NOT infer parameter types or other
  type information from the source code. Python `#` comment blocks are NOT
  treated as documentation.
- **FR-003**: The system MUST associate a module docstring with its module,
  a class docstring with its class, and a function docstring with its
  function.
- **FR-004**: The system MUST generate an HTML documentation site containing
  a dedicated entry for every module, class, and function. Each entry MUST
  include the element's full Python package import path so the element is
  uniquely identified, even when other elements elsewhere in the project
  share the same name. Documented elements MUST include their description,
  parameters, return values, and raised exceptions as applicable; elements
  without docstrings MUST still have an entry that simply indicates the
  element exists, with no extra information.
- **FR-005**: The system MUST provide an index/navigation view that lets a
  user locate any documented module, class, or function by its full Python
  package import path.
- **FR-006**: The system MUST regenerate output deterministically so that
  re-running on unchanged source produces identical results, and on changed
  source produces updated results reflecting the current docstrings.
- **FR-007**: The system MUST handle malformed or partial docstrings by
  generating an entry for the affected element that includes an error
  message stating the docstring was not written correctly. The overall run
  MUST NOT fail; documentation MUST still be produced for all other
  properly documented code.
- **FR-008**: The system MUST generate an existence entry for any module,
  class, or function that has no docstring. (This documents the
  undocumented-element behavior specified in FR-004, retained here as a
  distinct acceptance anchor for SC-002.)
- **FR-009**: The system MUST accept a target output location where the
  generated HTML site is written.

### Key Entities *(include if feature involves data)*

- **Source Project**: The directory tree being scanned, containing the
  Python modules to document.
- **Documented Element**: A module, class, or function in the source
  project. Each element is uniquely identified by its full Python package
  import path and is represented in the generated site, with parsed
  docstring detail if a docstring is present or a bare existence entry if
  it is not.
- **Generated Documentation Site**: The HTML output that organizes
  documented elements into navigable pages.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A user can generate a complete HTML documentation site for a
  sample project in under 60 seconds using the standard command.
- **SC-002**: 100% of elements in the scanned source are represented in the
  generated HTML — with full detail when documented and an existence-only
  entry when not.
- **SC-003**: A user can locate and open the documentation for any documented
  class or function from the navigation index within three clicks.
- **SC-004**: Re-running the generator on unchanged source produces identical
  output, verified by byte-for-byte comparison.
- **SC-005**: Malformed docstring input does not stop the run; the tool
  completes and produces docs for all other code, and the affected element's
  entry displays an error message stating the docstring was not written
  correctly.
- **SC-006**: Every documented element's entry displays its full Python
  package import path, and elements sharing the same simple name in
  different modules remain individually addressable without ambiguity.

## Assumptions

- The tool is a command-line utility intended for local use by individual
  developers.
- The tool is written in Rust and parses Python source using the
  rustpython_parser crate; the parser is not implemented from scratch.
- The processing flow is: (1) parse source to an AST, (2) extract docstrings
  into an intermediary object representation, (3) generate HTML from that
  intermediary representation.
- The scanned source tree maps directly to a Python package structure so
  each element can be assigned a full package import path from its location
  in the tree.
- Documentation content is sourced solely from developer-provided Python
  docstrings (not `#` comment blocks). Only `@arg`, `@return`, and `@raises`
  tags are supported. Element names come from the AST; inferring types from
  source code is out of scope for this version and may be added later from
  the code's types.
- Support for multiple output formats is out of scope for the initial
  version; HTML is the only required output for v1.
- The tool may be used on a source tree that includes undocumented elements;
  per FR-008 these always receive an existence-only entry.
- A developer can point the tool at a source directory and an output
  directory; no hosting or publishing to a remote location is in scope for
  v1.
- Configurable styling/themes are out of scope for v1 unless requested.