# Contract: CLI — Theme Option

**Created**: 2026-09-08
**Feature**: [spec.md](spec.md) | Extends feature-001 [cli.md](../../001-pydoc-generator/contracts/cli.md)

## Usage

```text
pydoc-gen [--theme <styles.css>] <input_dir> <output_dir>
```

The optional `--theme` flag supplies a user-generated stylesheet applied to
the entire generated site. `--theme` may appear before or after the
positional arguments. Exactly two positional arguments are required.

## Behavior

| Input | Behavior |
|-------|----------|
| no `--theme` | Default hand-written theme (light + dark) is embedded in every page. |
| `--theme <file>`, file readable | File bytes appended to default CSS inside the page `<style>` block; user rules override defaults (cascade). |
| `--theme <file>`, file missing/unreadable | Fatal error: `could not read theme '<path>': <io error>`; exits non-zero; nothing written. |
| `--theme <file>`, file empty/whitespace-only | Allowed; site renders with the default theme. |

Example failure:

```text
$ pydoc-gen --theme missing.css src out
error: could not read theme 'missing.css': No such file or directory (os error 2)
```

## Determinism

The same `input_dir` + same `--theme` file (or absence of it) always
produces byte-for-byte identical output (SC-005). Theme resolution is
performed once per run and reused for the index and every module page.

## Interaction with existing behavior

- Stale-HTML cleanup, page layout (`index.html` + one page per module),
  escaping, and per-element docstring handling are unchanged.
- `--theme` affects presentation only; it never modifies documented content.