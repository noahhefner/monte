# Data Model: Modern HTML UI

**Created**: 2026-09-08
**Feature**: [spec.md](spec.md) | Decision source: [research.md](research.md)

The styling feature introduces exactly one new runtime entity — the resolved
**Theme** — plus one configuration input (the optional `--theme` path). The
existing `DocumentedElement` / `DocumentedProject` model from feature 001 is
**unchanged**; styling is presentation-only (FR-003, FR-007).

## Entities

### Theme

The complete stylesheet text embedded into every generated page.

| Field | Type | Description |
|-------|------|-------------|
| `css` | `String` | Full CSS text for the `<style>` block (default CSS + optional user CSS appended). |
| `source` | `ThemeSource` | Where the CSS came from (for diagnostics). |

### ThemeSource

| Variant | Meaning |
|---------|---------|
| `Default` | The built-in hand-written theme authored in `assets/default.css`, bundled into the tool (FR-013). |
| `Custom { path: PathBuf }` | A user-provided stylesheet read from `path` at build time. |

## Relationships

```text
CLI (--theme path, optional)
        │
        ▼
resolve_theme(path) ──► Theme { css, source } ──► wrap_page(title, body, &css)
                                                          │
                                        <style> block in <head> of every page
                                        (index page + one per module)
```

- `Theme::default()` → constructs the default-theme CSS bundled from
  `assets/default.css` (compiled into the tool via `include_str!`, FR-013),
  tagged `ThemeSource::Default`.
- `resolve_theme(optional_path)` → `Default` when no path; otherwise reads the
  file into `ThemeSource::Custom` and **appends** its bytes to default CSS.
- Theme resolution happens **once per run**; the same `Theme` is passed to
  every page (FR-012, SC-007).
- No entity persists beyond the run; nothing is stored on disk beyond the
  generated HTML itself.

## Validation Rules (FR-010, FR-012, FR-013, FR-007)

| Rule | Required | Behavior |
|------|----------|----------|
| `--theme` path points to a readable file | yes | Otherwise **fatal** `Error::ReadTheme(path, err)`; no output written. |
| User theme file is empty / whitespace-only | no | Allowed; renders with default theme (graceful degradation). |
| User theme CSS precedence over default | yes | User bytes appended after default CSS in the same `<style>` block; cascade decides (D-004). |
| Styling applied to 100% of pages | yes | Index + every module page carry the same resolved theme (SC-001, SC-007). |
| Output remains self-contained & offline | yes | Only inline `<style>`; no `<link>`, no external URLs, no JS (FR-008). |
| Other pages' existing content/structure intact | yes | No element content, paths, or anchors are changed (FR-003). |

## State Transitions

None — the theme is immutable once resolved for a run. The pipeline's
existing "generate → write page → clean stale HTML" flow (feature 001) is
unchanged; only the bytes each page contains change.

## Rendering Contract (input → output)

| Input | Output |
|-------|--------|
| default (no `--theme`) | `<head>` gains `<style>…default light+dark CSS…</style>`; `wrap_page` renders it. |
| `--theme custom.css` | `<head>` gains `<style>…default CSS + custom bytes…</style>`. |
| identical inputs (`--theme` or not), second run | byte-for-byte identical pages (SC-005). |