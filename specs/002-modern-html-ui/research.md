# Research: Modern HTML UI

**Created**: 2026-09-08
**Feature**: [spec.md](spec.md)

All design decisions below are resolved — no open questions remain. Research
was grounded in the existing implementation (`src/html.rs`, `src/index.rs`,
`src/pipeline.rs`, `src/cli.rs`, `src/error.rs`), which was read as input.

## Decisions

### D-001: Hand-written styling, no third-party CSS (FR-010)

**Decision**: The default theme is authored as a real CSS file in the
repository (`assets/default.css`) and compiled into the tool at build time
via `include_str!`. Its contents are emitted into every generated page.
There are no CSS libraries, frameworks, or external assets anywhere in the
pipeline.

**Rationale**: FR-010 forbids third-party dependencies, and the
clarification (spec "Clarifications" session) requires the theme to live in
an actual CSS file rather than a string literal in source code — so
`assets/default.css` is maintainable with normal CSS tooling while
`include_str!` keeps the runtime behavior identical (embedded, offline,
deterministic). The default theme needs only a modest set of rules targeting
the existing page markup (`<body>`, `.nav`, `.element`, `.kind`, `.path`,
`.description`, `.undocumented`, `.error`, `h1`–`h4`, `table`, `ul`) plus
tokens; ~100–200 lines of hand-written CSS fully covers it.

**Alternatives considered**:
- Tailwind/Bootstrap or similar framework — rejected: violates FR-010 and
  breaks the self-contained output requirement.
- A separate vendored `site.css` shipped next to `pydoc-gen` — viable but
  adds a second artifact to manage and a path-resolution problem at runtime;
  rejected in favor of embedding for simplicity and determinism.

### D-002: Light + dark presentation modes (FR-011)

**Decision**: The default theme defines colors exclusively through CSS custom
properties (e.g. `--color-bg`, `--color-text`, `--color-accent`,
`--color-border`, `--color-muted`, `--color-code-bg`) declared on `:root`,
with a `@media (prefers-color-scheme: dark)` block overriding the same
properties for dark mode. The page also gets
`<meta name="color-scheme" content="light dark">` so form/scrollbar chrome
follows the reader's preference. Dark mode is selected automatically by the
reader's OS/browser; no toggle and no JavaScript (per assumptions).

**Rationale**: CSS custom properties are supported by every modern browser,
keep the theme DRY, and are the *same override mechanism* the user-theme
feature (FR-012) relies on (D-004). `prefers-color-scheme` needs zero
runtime code. This satisfies SC-008 (dark-body contrast) because each mode
declares its own palette with measured AA-contrast pairs.

**Alternatives considered**:
- Class-based toggle driven by JavaScript — rejected: no JS per assumptions.
- Two per-mode stylesheet files linked conditionally — works but duplicates
  rules and complicates the embed-in-page requirement.
- `light-dark()` CSS function with a single palette — newer CSS; custom
  properties + media query chosen for wider compatibility (SC-004 supports
  older browsers at 320px).

### D-003: Embedding the stylesheet in every page (FR-008)

**Decision**: Every generated page's `<head>` gets a single inline `<style>`
block containing the resolved theme. The page is still fully self-contained
HTML, identical in spirit to today's output.

**Rationale**: Keeps the site offline-capable and portable (open any page
from `file://`), matches FR-008, and avoids touching the stale-file cleanup
in `pipeline.rs` (a linked `.css` file would need to be tracked as an
artifact). Byte-for-byte output remains well-defined.

**Alternatives considered**:
- One shared `style.css` linked from every page — self-contained but new
  artifact to clean up; rejected.
- External CDN/hosted CSS — rejected (offline, self-contained, hand-written).

### D-004: User-provided themes (FR-012, SC-007, SC-008)

**Decision**: Users supply a custom stylesheet at build time via a new
optional CLI flag `--theme <styles.css>`. The file is read once, and its
contents are appended inside the same `<style>` block **after** the default
theme CSS. Because both share the same CSS custom-property names, a user
theme can override tokens (colors, spacing, fonts) or append whole rules; by
CSS cascade rules (later identical-specificity wins), the user's styles take
precedence. The default theme is always present underneath, so a partial user
theme degrades gracefully.

**Rationale**: This satisfies "users can provide their own stylesheets for
their own themes" with a minimal, build-time contract that needs no runtime
JS or server. Precedence-by-cascade-through-one-block is simple, predictable,
and matches how the handbook/theming cost here is lowest: no new parser
surface, no selector merging, one embed path reused for both themes.

**Alternatives considered**:
- Named, built-in theme selection (`--theme solarized`) — rejected: not
  "user-generated", adds a theme registry.
- Read-time theme switching in the browser (theme picker persisted in
  `localStorage`) — rejected: requires JavaScript, contradicts assumptions.
- Putting user CSS in a separate `<style>` block *before* the default —
  rejected: would make default win; order is inverted intentionally.

**Edge handling**:
- Missing/unreadable `--theme` file → **fatal error** (consistent with
  existing invalid-input handling); clear message, non-zero exit, nothing
  written. New `Error::ReadTheme(path, io_err)` variant.
- Empty or whitespace-only theme file → allowed; page still renders with the
  default theme. Documented, tested.
- Theme applied to **every** page (index + all module pages) → SC-007.

### D-005: Determinism (FR-007, SC-005)

**Decision**: The default-theme CSS comes from the bundled `assets/default.css`
asset (`include_str!`, D-001); a custom theme's bytes are embedded verbatim.
Theme resolution happens once per run and produces identical CSS text for
identical inputs. Combined with the existing deterministic pipeline,
regenerating on unchanged source stays byte-for-byte identical.

**Rationale**: No time stamps, hashes, or generated identifiers enter the
output. The contract test re-runs generation and asserts equality (mirrors
feature 001's regeneration test).

### D-006: Injection safety (FR-009)

**Decision**: Docstring/user content path continues through `escape_html`
unchanged; styling rules and theme CSS never interpolate parsed source
content. The `<style>` block is emitted statically or from an explicitly
user-supplied file (trust boundary: the build-time user who already has
arbitrary code execution), not from docstrings.

**Rationale**: The existing escaping contract (T037) already prevents
docstring markup injection; the theme block adds no new injection surface.

## Resolved unknowns

| Topic | Resolution |
|-------|------------|
| Where the default theme lives | `assets/default.css`, compiled into the binary via `include_str!` (FR-013) |
| Where custom themes come from | `--theme <file>` CLI flag (`src/theme.rs`) |
| How light/dark work | custom properties + `prefers-color-scheme` (D-002) |
| How user CSS wins | appended after default in the same block (D-004) |
| Error for missing theme file | `Error::ReadTheme` fatal error |
| HTML changes required | `<style>` block in `<head>` only; `wrap_page` gains a stylesheet param |

## Open questions

None.