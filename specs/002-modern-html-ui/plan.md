# Implementation Plan: Modern HTML UI

**Branch**: `002-modern-html-ui` | **Date**: 2026-09-08 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/002-modern-html-ui/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command; its definition describes the execution workflow.

## Summary

Give the generated documentation site (feature `001-pydoc-generator`) a
modern, hand-written visual design instead of raw, unstyled pages. The
default theme provides light and dark presentation modes, is written
entirely by hand (no third-party CSS libraries or frameworks), and is
replaceable: users can supply their own stylesheet at build time so the
entire generated site renders with their theme. Styling is presentation-only
and MUST preserve existing content, structure, and byte-for-byte determinism.

## Technical Context

**Language/Version**: Rust (current stable, edition 2024, MSRV compatible with
existing toolchain). Styling output is CSS emitted as static text from the
generator; no runtime language.

**Primary Dependencies**: None new. `rustpython_parser` unchanged (existing).
Specifically NO third-party CSS libraries or frameworks — all styling is
hand-written and embedded in the generated pages (FR-010).

**Storage**: Files. Source Python in, generated HTML out. The default theme
is a real CSS source file (`assets/default.css`) compiled into the binary at
build time (`include_str!`, FR-013) and embedded into every page; a
user-provided theme is a CSS file read from disk and embedded into every
page.

**Testing**: `cargo test` — unit tests for theme resolution and light/dark
CSS generation, contract tests asserting styled markup (style block present,
correct classes/content preserved) and determinism, integration tests for
the `--theme` CLI path end-to-end. `cargo bench` re-run to confirm no
performance regression (existing baseline: mean ≈ 28 ms / 200 modules).

**Target Platform**: Local developer machines (Linux/macOS/Windows). Generated
site viewed in any modern browser supporting `@media (prefers-color-scheme)`
and CSS custom properties.

**Project Type**: CLI tool / library (single Rust crate, extends feature
`001-pydoc-generator`).

**Performance Goals**: SC-001 (site generated in under 60 seconds) preserved;
styling adds bounded, deterministic text emission with negligible time cost.
Re-run existing benchmark to confirm.

**Constraints**: Hand-written CSS only (FR-010); light and dark modes tested
and readable (FR-011, SC-008); user-provided theme applied site-wide
(FR-012, SC-007); self-contained offline output (FR-008); deterministic
output preserved (FR-007, SC-005); no injection via docstring content
(FR-009); works at 200+ elements (SC-006).

**Scale/Scope**: Styling at small-to-medium package scale (benchmark target
200 modules / 2000+ elements). CSS size is bounded and static. One default
theme plus optional user themes.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

Gates derived from the Monte Constitution (`.specify/memory/constitution.md`):

1. **Code Readability**: The tool MUST be written with clarity as the
   primary objective; descriptive names, documented public interfaces, and
   consistent formatting. Gate: PASS (a dedicated theme module encapsulates
   CSS generation with documented functions; CSS authoring follows a
   documented variable/token convention; existing codebase style is
   maintained).
2. **Testing Standards**: Every feature and bug fix MUST have automated
   tests covering happy paths, edge cases, and error conditions; tests MUST
   be deterministic and isolated. Gate: PASS (unit tests for theme/default
   CSS and mode output; contract tests for styled markup + content
   preservation + determinism; integration test for user theme flag;
   edge cases: missing theme file, empty theme file, markup in docstrings).
3. **Performance**: Critical paths MUST have benchmarks before optimization;
   profiling precedes optimization; resource usage MUST be bounded and
   predictable. Gate: PASS (existing T035a baseline exists for the parse →
   extract → generate pipeline; styling is linear string emission; re-run
   the benchmark post-change to confirm no regression before any
   optimization consideration).

No gate violations require justification in Complexity Tracking.

### Phase 1 Re-check (post design): PASS

Re-evaluated after the design artifacts were produced
(`research.md`, `data-model.md`, `contracts/theme.md`,
`contracts/cli.md`, `quickstart.md`):

- **Readability**: The design keeps CSS in one purpose-built module
  (`style.rs`) and resolution in another (`theme.rs`) with a documented
  token naming convention (`--color-*`), matching existing module style.
- **Testing**: Every behavior above maps to an automated test —
  `tests/unit/style.rs` (tokens, light/dark emission, custom theme
  resolution), `tests/contract/styled_output.rs` (style block present,
  content preserved, determinism), `tests/integration/custom_theme.rs`
  (CLI path end-to-end), plus a `validate-theme-quickstart.sh` runner.
  No behavior is untested.
- **Performance**: Styling adds only a single `push_str` of static CSS per
  page; linear, bounded, deterministic. The existing benchmark baseline
  (~28 ms mean, 200 modules) is re-run in implementation to confirm no
  regression (Constitution: measure before optimizing).

## Project Structure

### Documentation (this feature)

```text
specs/002-modern-html-ui/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

Extends the existing single-crate layout from feature `001` (`src/`, `tests/`
at repository root):

```text
assets/
└── default.css          # NEW: the authored default-theme stylesheet (FR-013)

src/
├── main.rs              # CLI entry point (unchanged)
├── cli.rs               # Add --theme <path> option parsing (custom stylesheet)
├── style.rs             # NEW: emits default-theme CSS (bundled assets/default.css) + user CSS into pages
├── theme.rs             # NEW: theme resolution (default vs. user-provided), reading/validating CSS input
├── html.rs              # Embed resolved stylesheet into every page (<style> block)
├── index.rs             # Navigation index rendering (unchanged, receives styled page wrapper)
├── pipeline.rs          # Thread theme resolution into site generation
└── ...                  # parser.rs, extract.rs, model.rs, path.rs, docstring.rs, tags.rs, error.rs (unchanged)

tests/
├── contract/
│   └── styled_output.rs # NEW: styled pages keep all element content; style block present; determinism
├── integration/
│   └── custom_theme.rs  # NEW: --theme CLI path applied across the site end-to-end
└── unit/
    └── style.rs         # NEW: default CSS token consistency, light/dark blocks, custom theme resolution
```

**Structure Decision**: Single Rust crate. The styling feature is isolated in
new, well-named modules (`style.rs` for emitting the bundled default-theme
CSS from `assets/default.css` plus user CSS, `theme.rs` for theme
resolution/loading) so the existing pipeline stays untouched except where the
stylesheet must be threaded into page emission. The default theme is
authored as a real CSS file (`assets/default.css`, FR-013) compiled into the
tool at build time.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No constitution violations to justify. The theme subsystem is a small,
well-bounded addition: default CSS is static text, custom themes are read as
files, and page emission gains a single style-embedding step.