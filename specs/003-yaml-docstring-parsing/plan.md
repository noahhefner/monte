# Implementation Plan: YAML Docstring Parsing

**Branch**: `003-yaml-docstring-parsing` | **Date**: 2026-09-11 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/003-yaml-docstring-parsing/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command; its definition describes the execution workflow.

## Summary

Replace the current line-by-line manual tag parser (`tags.rs`) with a YAML
interpreter so that docstrings are read as YAML documents and documentation
data is extracted from structured YAML keys rather than parsed `@`-tag lines.
The existing three-stage pipeline — parse Python source to AST, extract
docstrings, generate HTML — is preserved; only the middle extraction stage
changes. The data model (`DocComment`, `DocContent`, `ArgDoc`, `ReturnDoc`,
`RaisesDoc`) remains structurally identical. Plain-prose docstrings that are
valid YAML scalars continue to be treated as description-only entries. Legacy
JavaDoc-style `@`-tag docstrings are treated as malformed (error entry) since
they do not parse as valid YAML. A YAML crate replaces the hand-written tag
parser.

## Technical Context

**Language/Version**: Rust (current stable, edition 2024 — unchanged).

**Primary Dependencies**: `rustpython_parser` (unchanged, AST parsing),
`serde` (already present, serialization), `serde_yml` or equivalent
(new, YAML document parsing with serde integration), `tera` (unchanged,
HTML templates).

**Storage**: N/A. Reads source files from disk; writes HTML files to a
user-specified output directory.

**Testing**: `cargo test` with unit tests for YAML docstring extraction,
contract tests for the full pipeline, and integration tests for
malformed/edge-case handling. Existing tag-parsing tests are replaced.

**Target Platform**: Local developer machines (Linux/macOS/Windows via
standard Rust toolchain). CLI tool.

**Project Type**: CLI tool / library.

**Performance Goals**: Documentation site for a sample project generated in
under 60 seconds (SC-001); fast enough for interactive local use.

**Constraints**: YAML docstring interpretation replaces manual parsing; no
fallback to `@`-tag parsing (FR-001, FR-007). Malformed YAML must never abort
the run (FR-007). Prose-only docstrings remain valid (FR-010). Deterministic
output must be preserved (FR-006).

**Scale/Scope**: Small to medium Python packages (potentially thousands of
elements); bounded, predictable memory and throughput per the constitution's
Performance principle.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Gates derived from the Monte Constitution (`.specify/memory/constitution.md`):

1. **Code Readability**: The YAML extraction logic uses serde-based
   deserialization into typed structs, which reads as a declarative schema
   rather than procedural parsing. Module structure and naming remain clear.
   Gate: PASS.
2. **Testing Standards**: Existing tag-parsing unit tests are replaced with
   YAML extraction unit tests covering valid mappings, plain prose, malformed
   YAML, and wrong-shape fields. Contract and integration tests cover the full
   pipeline with YAML docstrings. Gate: PASS.
3. **Performance**: YAML parsing adds one linear pass over each docstring
   (proportional to docstring length); no additional I/O or allocation beyond
   what already exists. Profiling the parse → extract → generate pipeline
   before and after will confirm no regression. Gate: PASS.

**Post-Phase 1 re-check** (after research/data-model/contract design):

1. **Code Readability**: The design expresses the YAML schema as typed Rust
   structs (`YamlDoc`, `YamlArg`, `YamlReturn`, `YamlRaise`) deserialized via
   serde — a declarative contract, not procedural field-picking. Only
   `tags.rs` is removed; `extract.rs` grows. Gate: PASS (no change).
2. **Testing Standards**: Unit tests cover all four dispatch outcomes
   (empty→None, mapping→Ok, scalar→prose description, parse-fail→Error) plus
   wrong-shape fields and unknown-key tolerance; fixtures use YAML docstrings.
   Gate: PASS (no change).
3. **Performance**: Design adds exactly one YAML parse per docstring — linear
   in docstring size, no extra I/O. Existing benchmarks are updated with YAML
   docstring inputs to confirm no regression. Gate: PASS (no change).

No gate violations require justification in Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/003-yaml-docstring-parsing/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
│   ├── cli.md           # (carried forward from 001; unchanged)
│   └── docstring-syntax.md
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── main.rs              # CLI entry point (unchanged)
├── cli.rs               # Argument parsing (unchanged)
├── parser.rs            # Parse source → AST via rustpython_parser (unchanged)
├── docstring.rs         # Normalize raw docstring text (unchanged — dedent/trim)
├── extract.rs           # NEW: YAML-based docstring → DocComment extraction
│                         #      (replaces tags.rs; calls yaml parse then maps to model)
├── tags.rs              # REMOVED — replaced by extract.rs YAML logic
├── model.rs             # Intermediary object representation (unchanged)
├── pipeline.rs          # Orchestration (unchanged — already calls extract.rs)
├── index.rs             # Navigation index generation (unchanged)
├── theme.rs             # Theme/styling (unchanged)
├── error.rs             # Error types (unchanged or minor additions)
├── lib.rs               # Library surface (update module list)
├── templates/           # Tera HTML templates (unchanged)
└── style.rs             # (unchanged)

tests/
├── unit/
│   ├── docstring.rs     # docstring normalization (unchanged)
│   ├── extract.rs       # NEW: YAML extraction unit tests (replaces unit/tags.rs)
│   ├── edges.rs         # Edge-case tests (update for YAML behavior)
│   └── path.rs          # Path resolution (unchanged)
├── contract/
│   └── generate_docs.rs # End-to-end fixture-driven tests (update fixtures to YAML)
├── integration/
│   ├── invalid_input.rs # Malformed docstring tests (update for YAML)
│   ├── regeneration.rs  # Determinism tests (update fixtures)
│   └── navigation.rs    # Navigation tests (update fixtures)
└── ...
```

**Structure Decision**: Single Rust crate. The change is localized to the
extraction stage: `tags.rs` is removed, `extract.rs` gains YAML-parsing
logic, and `model.rs` remains unchanged. All other modules (parser,
pipeline, index, theme, templates) are untouched. This keeps the
change small and testable per the constitution's Readability and Testing
principles.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations to justify. The feature replaces one extraction
method with another of equivalent complexity; the pipeline remains linear
and single-pass.
