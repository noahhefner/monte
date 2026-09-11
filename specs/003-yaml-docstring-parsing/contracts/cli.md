# Contract: CLI Interface

**Branch**: `001-pydoc-generator` | **Date**: 2026-09-08 | **Plan**: [plan.md](plan.md)

## Command Shape

The tool is a command-line utility. Its primary interface is:

```text
pydoc-gen <input_dir> <output_dir>
```

- `<input_dir>` — the source directory to scan (recursively) for Python
  modules. Required. (FR-001)
- `<output_dir>` — the directory where the generated HTML site is written.
  Required. (FR-009)

## Behavior

- The tool scans `<input_dir>` (and subdirectories) and produces an HTML
  documentation site at `<output_dir>`.
- `<output_dir>` is created if it does not exist; existing files may be
  overwritten.
- Output structure (FR-005):
  - `index.html` — the navigation page: every module/package, class, and
    function listed by full import path as a link to its documentation.
  - One page per module/package, e.g. module `sample.utils` →
    `sample/utils.html`, package `sample` → `sample.html`.
- Every module, class, and function is represented (FR-001, FR-004,
  FR-008).
- Regeneration (FR-006): the site is regenerated on every run; HTML for
  modules removed from the source is deleted from the output directory, and
  re-running on identical source produces byte-identical output.

## Error Handling

- Non-existent or unreadable `<input_dir>`: the tool reports a clear error
  and exits non-zero.
- Empty directory or no Python files: the tool completes, producing an empty
  (or minimal) site without crashing.
- Malformed docstring in one element (FR-007): the tool does NOT abort; the
  affected element's entry shows an error message, and all other elements are
  documented normally.

## Exit Codes

| Code | Meaning |
|------|---------|
| 0    | Success (site generated; may include per-element malformed-docstring error messages) |
| non-zero | Fatal error (e.g. invalid/unreadable input path) |

## Not in Scope (v1)

- Multiple output formats (HTML only).
- Hosting/publishing the generated site.
- Configurable styling/themes.
- Type extraction from source (names only from the AST, per the spec).
