# pydoc-gen

Generate HTML documentation for Python source from JavaDoc-style docstrings.

`pydoc-gen` scans a source directory, parses each `.py` file with
[`rustpython_parser`](https://crates.io/crates/rustpython-parser), extracts
JavaDoc-style tags from module/class/function docstrings, and writes a
navigable HTML site.

## Features

- **Docstring-only source**: documentation is read from Python docstrings
  (`"""..."""`); `#` comment blocks are ignored.
- **Three tags**: `@arg <name> <description>`, `@return <description>`,
  `@raises <exception> <description>`, plus free description text.
- **Full-path identity**: every module, class, and function is identified by
  its full Python package import path, so duplicate simple names in
  different modules stay unambiguous (e.g. `pkg.a.Helper` vs `pkg.b.Helper`).
- **Unambiguous coverage**: undocumented elements still get an existence-only
  entry (`No documentation provided.`).
- **Never aborts on malformed docs**: a malformed docstring renders an
  in-entry error and the rest of the site is generated normally.
- **Deterministic output**: identical source produces byte-for-byte identical
  output, and pages for removed modules are cleaned up on re-generation.
- **Navigation index**: `index.html` links to every element; each module gets
  its own page.

## Example

```python
def format_value(text):
    """Formats text for display.

    @arg text  The text to format.
    @return    The formatted string.
    @raises ValueError  Raised when text is empty.
    """
    ...
```

## Build & run

Requires a stable Rust toolchain (edition 2024).

```bash
cargo build --release
cargo run --release -- <input_dir> <output_dir>
```

Output:

```text
<output_dir>/
├── index.html            # navigation index (every element, by full path)
├── sample.html           # package `sample`
└── sample/
    ├── formatting.html   # module `sample.formatting`
    └── util.html         # module `sample.util`
```

## Testing & benchmarks

```bash
cargo test        # unit, contract, and integration tests
cargo bench       # parse → extract → generate baseline (SC-001)
cargo clippy --all-targets -- -D warnings
```

## Specification

Design docs live in [`specs/001-pydoc-generator/`](specs/001-pydoc-generator/):

- [spec.md](specs/001-pydoc-generator/spec.md) - requirements and user stories
- [contracts/docstring-syntax.md](specs/001-pydoc-generator/contracts/docstring-syntax.md)
- [contracts/cli.md](specs/001-pydoc-generator/contracts/cli.md)
- [data-model.md](specs/001-pydoc-generator/data-model.md)
- [quickstart.md](specs/001-pydoc-generator/quickstart.md)
- [bench-results.md](specs/001-pydoc-generator/bench-results.md)