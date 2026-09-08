# Implementation Plan: Python Documentation Generator

**Branch**: `001-pydoc-generator` | **Date**: 2026-09-08 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/001-pydoc-generator/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command; its definition describes the execution workflow.

## Summary

Build a command-line tool in Rust that generates HTML documentation for
Python code from JavaDoc-style docstrings. The tool parses Python source into
an AST using the `rustpython_parser` crate, extracts `@arg`, `@return`, and
`@raises` tags plus description text from module, class, and function
docstrings into an intermediary object representation, and emits a navigable
HTML site for the documented code. Every element is identified by its full
Python package import path, undocumented elements still receive an
existence-only entry, and malformed docstrings produce an in-entry error
message without failing the run.

## Technical Context

**Language/Version**: Rust (current stable).

**Primary Dependencies**: `rustpython_parser` (AST parsing). HTML generation
from the intermediary representation; a lightweight templating/escaping
approach rather than a full framework.

**Storage**: N/A. Reads source files from disk, writes HTML files to a
user-specified output directory.

**Testing**: `cargo test` with unit tests for tag parsing and
intermediary-representation construction, plus snapshot/end-to-end tests
driving the full parse → extract → generate pipeline against small sample
sources.

**Target Platform**: Local developer machines (Linux/macOS/Windows via
standard Rust toolchain). CLI tool.

**Project Type**: CLI tool / library.

**Performance Goals**: Documentation site for a sample project generated in
under 60 seconds (SC-001); fast enough for interactive local use.

**Constraints**: Must rely only on developer-provided docstrings for
documentation content (element names from the AST; no type inference from
source). Must use `rustpython_parser` — no hand-written parser. Must not fail
on malformed docstrings or undocumented elements.

**Scale/Scope**: Small to medium Python packages (potentially thousands of
elements); bounded, predictable memory and throughput are expected per the
constitution's Performance principle.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Gates derived from the Monte Constitution (`.specify/memory/constitution.md`):

1. **Code Readability**: The tool MUST be written with clarity as the
   primary objective; descriptive names, documented public interfaces, and
   consistent formatting. Gate: PASS (design follows these rules).
2. **Testing Standards**: Every feature and bug fix MUST have automated
   tests covering happy paths, edge cases, and error conditions; tests MUST
   be deterministic and isolated. Gate: PASS (cargo test suite with unit,
   snapshot, and integration coverage planned).
3. **Performance**: Critical paths MUST have benchmarks before optimization;
   profiling precedes optimization; resource usage MUST be bounded and
   predictable. Gate: PASS (parse → extract → generate pipeline is
   single-pass and linear in source size; performance goals defined in
   SC-001).

No gate violations require justification in Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/001-pydoc-generator/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── main.rs              # CLI entry point: args, orchestration
├── cli.rs               # Argument parsing (input dir, output dir)
├── parser.rs            # Parse source → AST via rustpython_parser; collect modules/classes/functions
├── docstring.rs         # Normalize raw docstring text (dedent, trim blank lines)
├── tags.rs              # Parse JavaDoc-style tags (@arg, @return, @raises) from docstring lines
├── model.rs             # Intermediary object representation (DocumentedElement, Module, etc.)
├── html.rs              # Emit HTML from the intermediary representation
├── index.rs             # Build navigation/index pages by full package path
└── lib.rs               # Library surface for the pipeline

tests/
├── contract/            # End-to-end fixture-driven tests (parse → extract → generate)
├── integration/         # Multi-file package fixtures, malformed/undocumented handling
└── unit/                # Tag parsing, docstring association, path resolution
```

**Structure Decision**: Single Rust crate. The three-stage pipeline (parse →
extract into intermediary representation → generate HTML) maps to dedicated
modules (`parser.rs`, `docstring.rs`/`model.rs`, `html.rs`), keeping each
concern small and independently testable in line with the constitution's
Readability and Testing principles.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations to justify. Complexity is bounded: the pipeline
is a linear three-stage transform, and reuse of `rustpython_parser` avoids
the significant complexity of a hand-written parser.
