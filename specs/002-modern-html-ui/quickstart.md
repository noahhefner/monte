# Quickstart: Modern HTML UI

**Created**: 2026-09-08
**Feature**: [spec.md](spec.md)

Runnable validation scenarios proving the feature works end-to-end. They
validate **behavior**, which will be automated in the implementation phase
(`tests/contract/styled_output.rs`, `tests/unit/style.rs`,
`tests/integration/custom_theme.rs`, and a
`scripts/validate-theme-quickstart.sh` mirroring feature 001's quickstart).

## Prerequisites

- Rust toolchain (`cargo`, including tests/bench from the toolchain used in
  feature 001).
- Run from the repository root.

```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release
PDG=./target/release/pydoc-gen
DEMO=/tmp/opencode/pydoc_demo        # docstring-style sample package (feature 001)
OUT=/tmp/opencode/monte_ui_out
```

## Scenario 1 — Default modern theme on every page (SC-001, US1)

```bash
rm -rf "$OUT" && "$PDG" "$DEMO" "$OUT"
for f in "$OUT/index.html" $(find "$OUT" -name '*.html'); do
  grep -q '<style>' "$f" || echo "MISSING style block: $f"
done
```

**Expected**: all pages contain a `<style>` block; the site renders as a
styled, consistent documentation site (open `index.html` in a browser); no
`MISSING` lines.

## Scenario 1b — Default theme is a real CSS file (FR-013)

```bash
test -f assets/default.css && test -s assets/default.css \
  && echo "PASS: assets/default.css exists and is non-empty"
rg -q -- '--color-accent' assets/default.css && echo "tokens defined in the CSS file"
```

**Expected**: both lines print — the authored default theme is a real,
non-empty CSS file in the repository (compiled into the tool, per FR-013),
not a string inside source code.

## Scenario 2 — No third-party / self-contained (FR-008, FR-010)

```bash
rg -n 'https?://|<link\b|@import' "$OUT" && echo "FAIL: external references" || echo "PASS: fully self-contained"
```

Open `$OUT/index.html` from `file://` with Wi-Fi off — **Expected**: identical
rendering, no external requests.

## Scenario 3 — Light + dark mode (FR-011, SC-008)

```bash
grep -q 'prefers-color-scheme: dark' "$OUT/index.html" && echo "dark block present"
grep -q -- '--color-text' "$OUT/index.html" && echo "design tokens present"
```

**Expected**: both lines print. In a browser, toggle OS appearance — the
site switches between a light and a dark palette and text stays readable in
both (AA contrast).

## Scenario 4 — Custom user theme applied site-wide (FR-012, SC-007)

```bash
cat > /tmp/opencode/my-theme.css <<'CSS'
:root { --color-accent: #8b2bd6; --color-bg: #faf8ff; }
@media (prefers-color-scheme: dark) {
  :root { --color-accent: #b98aff; --color-bg: #120b1e; }
}
CSS
"$PDG" --theme /tmp/opencode/my-theme.css "$DEMO" "$OUT"
grep -q '#8b2bd6' "$OUT/index.html" && echo "custom accent present"
grep -q '#120b1e' "$OUT/index.html" && echo "custom dark present"
for f in $(find "$OUT" -name '*.html'); do
  grep -q '#8b2bd6' "$f" || echo "theme NOT applied: $f"
done
```

**Expected**: `custom accent present` + `custom dark present`, and no
`theme NOT applied` lines — the user stylesheet is embedded in 100% of
pages and overrides the default accent/background.

## Scenario 5 — Theme file failure is fatal and clean (T019-style error path)

```bash
"$PDG" --theme /tmp/opencode/does-not-exist.css "$DEMO" "$OUT"; echo "exit=$?"
```

**Expected**: a clear `could not read theme '…'` message and a non-zero
`exit=`; check `$OUT` was not regenerated (previous content preserved).

## Scenario 6 — Determinism with and without a theme (SC-005)

```bash
rm -rf "$OUT" "$OUT2"
"$PDG" "$DEMO" "$OUT"
"$PDG" "$DEMO" "$OUT2"
diff -r "$OUT" "$OUT2" && echo "PASS: default theme deterministic"

"$PDG" --theme /tmp/opencode/my-theme.css "$DEMO" "$OUT3"
"$PDG" --theme /tmp/opencode/my-theme.css "$DEMO" "$OUT4"
diff -r "$OUT3" "$OUT4" && echo "PASS: themed output deterministic"
```

**Expected**: both `PASS` lines print.

## Scenario 7 — Content preserved and unbreakable (FR-003, FR-009)

```bash
rg -l 'No documentation provided' "$OUT" >/dev/null && echo "undocumented entries still present"
rg -n '&lt;script&gt;' "$OUT/index.html" >/dev/null && echo "escaping intact"
```

**Expected**: both lines print (a sample with an undocumented element and a
docstring containing markup would need to exist in the demo package).

## Notes

- `$OUT2`/`$OUT3`/`$OUT4` are additional scratch output dirs
  (`=/tmp/opencode/monte_ui_out2` etc.).
- Full expected-output details (tokens, element presentation rules, user
  theme precedence) are in [contracts/theme.md](contracts/theme.md) and
  [contracts/cli.md](contracts/cli.md).
- Run the suite as a script once `scripts/validate-theme-quickstart.sh` is
  added during implementation.