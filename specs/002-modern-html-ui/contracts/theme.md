# Contract: Theme (Styling)

**Created**: 2026-09-08
**Feature**: [spec.md](spec.md) | Decision source: [research.md](research.md)

## Purpose

Define what "modern UI" means in the generated site and how default and
user-provided themes behave. This contract is the specification for the
hand-written default theme (`assets/default.css`), its emission via
`style.rs`, and the resolution logic in `theme.rs`.

## 1. Page structure (unchanged from feature 001)

Every generated page is self-contained HTML with no external references:

```text
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="color-scheme" content="light dark">   ← added by this feature
    <title>…</title>
    <style>…resolved theme…</style>                    ← added by this feature
  </head>
  <body>
    …existing content…
  </body>
</html>
```

Existing selector surface used by the theme (no HTML changes elsewhere):

- `body`, `h1`–`h4`, `a`, `code`
- `.nav`, `.element`, `.kind`, `.path`
- `.description`, `.undocumented`, `.error`
- `table`, `tr`, `td`, `ul`, `li`

## 2. Default theme

Hand-written, no third-party code (FR-010). The default theme is authored
as a real CSS file, `assets/default.css`, which is compiled into the tool
and inlined into every page (FR-013) — it is not a string embedded in
source code. Design tokens are CSS custom properties on `:root`; dark mode
replaces the same tokens in `@media (prefers-color-scheme: dark)`
(FR-011, D-002).

Required tokens (names are the public override surface for user themes):

| Token | Purpose |
|-------|---------|
| `--color-bg` | page background |
| `--color-surface` | panels/tables/code backgrounds |
| `--color-text` | body text |
| `--color-muted` | subdued text (paths, secondary labels) |
| `--color-accent` | links, element names, focus |
| `--color-border` | table/horizontal-rule borders |
| `--color-code-bg` | `<code>` background |

Contract of the default theme:

- Readable: body text meets WCAG AA (≥ 4.5:1) against `--color-bg` in both
  light and dark palettes (SC-003, SC-008).
- Modern look: generous line height/spacing, subtle borders, accent-colored
  links, prominent element headings, type labels styled as subtle badges via
  the existing `.kind` span, `code` styled distinctly.
- Responsive: content stays readable from 320 px to desktop without
  horizontal scrolling (SC-004); no fixed widths.
- Distinct sections: descriptions/arguments/returns/raises blocks visually
  separated (US2).
- No JavaScript, no external fonts, no images, no `<link>` stylesheets.

## 3. Element presentation rules (US2, SC-002)

| Element | Required visual treatment |
|---------|---------------------------|
| heading `.element` | prominent (larger, accent on name) |
| `.kind` | subtle badge/label, distinct from name text |
| `.path` | muted monospace under the heading |
| `.description` | body text, clear block spacing |
| `h4` (Arguments/Returns/Raises) | section header distinct from body text |
| `table` | bordered/row-separated, readable in both modes |
| `.undocumented` | muted note, still legible (AA contrast) |
| `.error` | error-styled but readable (AA contrast), does not break layout |

## 4. User themes (FR-012)

- A user theme is supplied as a CSS file via `--theme` (see
  [cli.md](cli.md)).
- Embedding: user bytes are appended **after** default CSS inside the same
  `<style>` block of **every** page (SC-007). Identical-specificity rules
  therefore cascade user-side once more → user styles win.
- Users may override the token variables above (e.g. a different light/dark
  palette, accent color) or add wholly new rules.
- A user theme may define its **own** light/dark palette by re-declaring the
  tokens inside their own `@media (prefers-color-scheme: dark)` block.
- Default theme stays present underneath: partial user themes degrade
  gracefully (missing tokens fall back to the default design).
- Trust boundary: `--theme` is build-time user input (the same user already
  runs the generator on their machine); its CSS is embedded verbatim. It is
  never sourced from documented content (see FR-009).

## 5. Guarantees

- Determinism (SC-005): same inputs → byte-for-byte identical pages,
  whether or not `--theme` is used.
- Offline/self-contained (FR-008): works from `file://`, no network, no
  external assets.
- Injection safety (FR-009): docstring content stays HTML-escaped and never
  enters the CSS block.
- 200+ elements scale (SC-006): styling must not break or truncate large
  pages (tested via the existing benchmark corpus).