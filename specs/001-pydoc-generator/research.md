# Research: Python Documentation Generator

**Branch**: `001-pydoc-generator` | **Date**: 2026-09-08 | **Plan**: [plan.md](plan.md)

## Decisions

### D-001: Implementation language — Rust

**Decision**: Implement the tool in Rust (current stable toolchain).

**Rationale**: User-specified. Rust provides strong type safety and a single
statically-linked CLI binary, and aligns with the constitution's Performance
principle (predictable, bounded resource usage).

**Alternatives considered**: None — language was mandated by the user.

### D-002: AST parsing — use `rustpython_parser` (no hand-written parser)

**Decision**: Use the `rustpython_parser` crate (published as
`rustpython-parser`, current version 0.4.0, MSRV 1.72.1) to parse Python
source into an AST. Do NOT implement a parser from scratch.

**Rationale**: User-mandated, and prudent: the crate already handles lexing
and LALRPOP-based parsing of Python 3, exposing `ast::Suite::parse` /
`parser::parse_program`. Reusing it avoids the large, error-prone effort of a
hand-written parser and keeps the tool verifiable against the Python grammar.

**Key finding (important for design)**: The AST returned by
`rustpython_parser` does NOT retain source `#` comments; the parsed AST drops
them. Documentation content therefore comes from **docstrings**, which the
AST DOES retain: a module, class, or function docstring is the string
literal that appears as the first statement of the module or body. We read it
directly from the AST (`Stmt::Expr` → `Expr::Constant` → `Constant::Str`),
associating each docstring with its enclosing module/class/function (FR-003).
No comment capture or location alignment is needed, and `#` comment blocks
are NOT a documentation source (FR-002).

All documentation in v1 is JavaDoc-style content inside Python docstrings,
with `@arg`/`@return`/`@raises` tags on their own lines. This must be
documented as the tool's supported syntax (see
contracts/docstring-syntax.md).

**Alternatives considered**: Writing a bespoke parser (rejected — 
user-mandated against it); other Python parsers in Rust such as
`ruff_python_parser` (a valid alternative but user explicitly requested
`rustpython_parser`; note the current RustPython project itself now uses the
Ruff parser internally, but we honor the explicit user choice of
`rustpython_parser`).

### D-003: Docstring/tag scope — `@arg`, `@return`, `@raises`

**Decision**: Support exactly three JavaDoc-style tags: `@arg` (for
arguments), `@return` (for the return value), and `@raises` (for exceptions
that might be raised), plus free description text. Tags are parsed from the
docstring body, each on its own line.

**Rationale**: User-mandated scope limit. Keeps v1 focused and testable;
other tags are out of scope.

**Alternatives considered**: A generic `@param`/`@exception` style tag set
(rejected — user chose `@arg`/`@return`/`@raises`); inferring parameter types
from the AST (rejected as out of scope; content comes solely from docstrings,
element names from the AST only).

### D-004: Content source — docstrings only, not code types

**Decision**: Documentation content is derived solely from developer-provided
docstrings. Element names are taken from the AST; parameter types and other
type information are NOT inferred from the source in this version. `#`
comment blocks are not a documentation source.

**Rationale**: User-mandated. Simplifies v1 and defers type extraction to a
future version that may read types from the code itself.

**Alternatives considered**: Extracting types from the AST (deferred to a
future version).

### D-005: HTML generation — lightweight, self-contained

**Decision**: Generate a self-contained, navigable HTML site directly from
the intermediary object representation, with hand-rolled HTML escaping (no
heavy web framework). Pages are organized by full Python package import path,
with an index/navigation view.

**Rationale**: The scope is static docs; a lightweight approach keeps the
binary small, fast, and dependency-light, matching the Performance principle.
Full package paths uniquely identify elements (resolving duplicate simple
names) per the spec's SC-006/FR-004.

**Alternatives considered**: An HTML templating engine (rejected for v1 to
minimize dependencies; can be introduced later if styling needs grow).

## Alternatives Evaluation Summary

| Alternative | Evaluated? | Outcome |
|-------------|-----------|---------|
| Hand-written Python parser | Yes | Rejected (user-mandated against) |
| `ruff_python_parser` | Yes | Rejected (user explicitly asked for `rustpython_parser`) |
| Generic `@param` tag set | Yes | Rejected (user chose `@arg`/`@return`/`@raises`) |
| Type inference from AST | Yes | Deferred to future version |
| HTML templating engine | Yes | Deferred; lightweight escaping for v1 |
