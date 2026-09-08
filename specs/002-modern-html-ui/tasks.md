# Tasks: Modern HTML UI

**Input**: Design documents from `/specs/002-modern-html-ui/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Included — the project's testing standard (constitution) and `plan.md` Constitution Check commit to automated unit/contract/integration tests plus a quickstart validation script.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/`, `assets/` at repository root (per plan.md)

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [ ] T001 [P] Create `src/style.rs` with `pub fn default_theme_css() -> &'static str` returning `include_str!("../assets/default.css")`, and add `pub mod style;` to `src/lib.rs`
- [ ] T002 [P] Create `assets/default.css` with the theme scaffold: `:root` light palette plus `@media (prefers-color-scheme: dark)` declaring the seven required tokens (`--color-bg`, `--color-surface`, `--color-text`, `--color-muted`, `--color-accent`, `--color-border`, `--color-code-bg`) for both modes (must exist so `include_str!` compiles)

**Checkpoint**: `cargo build` compiles; the bundled stylesheet string is non-empty.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T003 Create `src/theme.rs` per `data-model.md`: `Theme { css: String, source: ThemeSource }`, `enum ThemeSource { Default, Custom { path: PathBuf } }`, `Theme::default()` (bundles `style::default_theme_css()`), and `resolve_theme(theme_path: Option<&Path>) -> Result<Theme>`; add `pub mod theme;` to `src/lib.rs`
- [ ] T004 Add `Error::ReadTheme(String, io::Error)` variant and its `Display` arm ("could not read theme '{path}': {err}") in `src/error.rs`
- [ ] T005 Add `&Theme` (or `&str` css) parameter to `html::wrap_page` and `html::render_module_page`, emitting `<meta name="color-scheme" content="light dark">` and a single `<style>` block with the resolved theme css in `<head>` per `contracts/theme.md` §1; update `index::render_index` call site
- [ ] T006 Thread theme resolution through the pipeline: `pipeline::generate(input, output, theme_path: Option<&Path>)` resolves the theme once via `theme::resolve_theme` and passes it to `write_site`, `render_index`, and each `render_module_page` call in `src/pipeline.rs`
- [ ] T007 Parse the optional `--theme <styles.css>` flag in `src/cli.rs` (2 or 3 CLI args), update the usage string, pass it to `pipeline::generate`, and keep the existing two-positional-args error for bare `pydoc-gen a b` + also hard-error on unknown extra args per `contracts/cli.md`
- [ ] T008 Add `[[test]]` entries to `Cargo.toml` for `unit_style`, `unit_contrast`, `contract_styled_output`, and `integration_custom_theme` (mirroring existing entries)
- [ ] T009 [P] Write contract test `tests/contract/styled_output.rs`: every generated page (index + module pages) contains `<style>` and `<meta name="color-scheme" content="light dark">`; no `<link`/external `http` references; all element content and anchors preserved; user CSS bytes appear appended after default css; two runs on identical input produce byte-for-byte identical output (with and without a theme)
- [ ] T010 [P] Write integration test `tests/integration/custom_theme.rs`: end-to-end `pipeline::generate` with a temp sample package and a temp `--theme` stylesheet embeds the custom bytes in 100% of pages; a missing/unreadable theme file returns `Error::ReadTheme` and nothing is written
- [ ] T011 [P] Write unit test `tests/unit/style.rs`: `default_theme_css()` is non-empty and contains all seven tokens in both the light `:root` block and the `prefers-color-scheme: dark` block; `Theme::default()` mirrors it; `resolve_theme(None)` → `Default`; `resolve_theme(Some(empty file))` is allowed; `resolve_theme(Some(missing))` → `ReadTheme`; custom css is appended and ordered after default css

**Checkpoint**: Foundation ready — pages embed a resolvable theme and the `--theme` mechanism works end-to-end; contract/integration/unit tests for the mechanism pass.

---

## Phase 3: User Story 1 - Modern styled documentation pages (Priority: P1) MVP

**Goal**: Every generated page (index + module pages) renders a consistent, modern visual design — clean typography, consistent spacing, accent color, self-contained styling (SC-001, FR-001/FR-008/FR-010/FR-013, quickstart Scenario 1).

**Independent Test**: Generate a demo site and confirm every page contains a `<style>` block and renders styled/consistent in a browser (quickstart Scenario 1); no page appears as raw, unstyled browser defaults.

### Implementation for User Story 1

- [ ] T012 [US1] Author base layout and typography in `assets/default.css`: body background/text from tokens, sans-serif font stack, readable line-height, constrained centered content column (max-width), heading scale h1–h4, accent-colored links with hover/focus states, `hr`/borders from `--color-border`, consistent vertical spacing rhythm
- [ ] T013 [US1] Author component base styling in `assets/default.css`: `table` (Arguments) with bordered/row-separated cells, `ul`/`li` spacing, `code` styling with `--color-code-bg`, `.nav` back-link, `.path` muted monospace, `.undocumented` muted note, `.error` readable error state
- [ ] T014 [US1] Style the navigation index page in `assets/default.css`: site title, root-path line, scannable nested link list, visible kind labels

**Checkpoint**: User Story 1 fully functional and testable independently — the site looks modern and consistent on every page (quickstart Scenarios 1 & 2 pass).

---

## Phase 4: User Story 2 - Clear visual hierarchy and readability (Priority: P2)

**Goal**: Element type (module/class/function) distinguishable at a glance via type badges, prominent element names, distinct sections, and body text meeting WCAG AA contrast (4.5:1) in both light and dark palettes (FR-004/FR-005/FR-011, SC-002/SC-003/SC-008).

**Independent Test**: Generate docs for a package mixing modules/classes/functions and confirm types are visually distinct; unit contrast test asserts body-text pairs are ≥4.5:1 in both modes (quickstart Scenario 3 shows the dark palette).

### Tests for User Story 2

- [ ] T015 [P] [US2] Write unit test `tests/unit/contrast.rs`: parse the `:root` and dark-block token values from `default_theme_css()` and assert `--color-text` vs `--color-bg` (and `--color-muted` vs `--color-bg`) meet ≥4.5:1 relative-luminance contrast in both palettes; assert `.kind` badge rules exist

### Implementation for User Story 2

- [ ] T016 [US2] Author element hierarchy in `assets/default.css`: `.kind` rendered as a subtle badge (pill, padded, muted/accent), `.element` heading name emphasized with `--color-accent`, and section-level spacing that visually separates each element block
- [ ] T017 [US2] Tune the light palette in `assets/default.css` so `--color-text`/`--color-muted`/`--color-accent` against `--color-bg` meet ≥4.5:1 (record chosen values; drive them off the T015 contrast test)
- [ ] T018 [US2] Tune the dark palette in `assets/default.css` so the same body-text pairs meet ≥4.5:1 against the dark `--color-bg` (SC-008), keeping `prefers-color-scheme: dark` intact

**Checkpoint**: User Stories 1 AND 2 both work independently — types are identifiable at a glance and both modes pass the contrast test.

---

## Phase 5: User Story 3 - Readable across common screen sizes (Priority: P3)

**Goal**: Pages remain fully readable without horizontal scrolling or clipped content at widths from 320 px up to desktop (FR-006, SC-004).

**Independent Test**: Open a generated page in browser devtools at 320/768/1280 px widths and confirm no horizontal scrolling of main content (quickstart resize check).

### Implementation for User Story 3

- [ ] T019 [US3] Author responsive rules in `assets/default.css`: fluid content column with horizontal padding, `overflow-wrap`/`word-break` guards for long `.path`/`code` text, and `table` handling that stays readable at narrow widths
- [ ] T020 [US3] Verify at 320/768/1280 px in browser devtools and fix any remaining overflow (long names, error text, nested index lists) in `assets/default.css`

**Checkpoint**: All user stories now independently functional — quickstart Scenarios 1–6 pass end-to-end.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T021 Re-run the existing benchmark (`cargo bench --bench generate`) and confirm no regression vs the ~28 ms mean baseline (200-module corpus, `benches/generate.rs`); record results in `specs/002-modern-html-ui/bench-results.md`
- [ ] T022 Run the full verification gate and fix any issues: `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`
- [ ] T023 Create executable `scripts/validate-theme-quickstart.sh` implementing quickstart scenarios 1, 1b, 2 (auto parts), 4, 5, 6 from `specs/002-modern-html-ui/quickstart.md`
- [ ] T024 Run the theme quickstart script against a fresh demo output and confirm every scenario passes; fix any failures
- [ ] T025 Final review of a generated demo site in a browser (light + dark) to gather SC-001–SC-008 evidence; update `contracts/theme.md`, `contracts/cli.md`, and `quickstart.md` if the implemented tokens/behavior differ from the design

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion — BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - US1 → US2 → US3 sequentially FOR THE CSS FILE: all three author into the same `assets/default.css`, so they must run in order (US1 first)
  - Story-specific test files can be written in parallel with each story's CSS work
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) — no dependencies on other stories
- **User Story 2 (P2)**: Can start after US1's CSS is in place — adds hierarchy/contrast rules to `assets/default.css`
- **User Story 3 (P3)**: Can start after US2 — adds responsive rules on top of the final palette; independently testable via resize checks

### Within Each User Story

- Tests (included per constitution/plan) are written before/alongside the CSS rules they validate
- CSS authoring order matters (same file): base layout → components → index → hierarchy → contrast → responsive
- Story complete before moving to next priority

### Parallel Opportunities

- T001 and T002 are independent files (Setup)
- T009, T010, T011 (Foundational tests) are different test files and can run in parallel after T003–T008
- T015 (contrast unit test) is independent of the US2 CSS-rule tasks
- T023, T024 (quickstart script) can be prepared in parallel with the final review (T025)
- Everything authoring into `assets/default.css` (T012–T014, T016–T020) is intentionally NOT marked [P] to avoid same-file conflicts

---

## Parallel Example: Foundational

```bash
# Launch all foundational tests together (after T003-T008):
Task: "Write contract test tests/contract/styled_output.rs (T009)"
Task: "Write integration test tests/integration/custom_theme.rs (T010)"
Task: "Write unit test tests/unit/style.rs (T011)"
```

## Parallel Example: User Story 2

```bash
# Launch the contrast test alongside CSS rule authoring:
Task: "Write unit test tests/unit/contrast.rs (T015)"
Task: "Author element hierarchy rules in assets/default.css (T016)"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001–T002)
2. Complete Phase 2: Foundational (T003–T011) — CRITICAL, blocks all stories
3. Complete Phase 3: User Story 1 (T012–T014)
4. **STOP and VALIDATE**: quickstart Scenarios 1 & 2; browser review
5. Deploy/demo if ready — the styled site is already a complete, self-contained product

### Incremental Delivery

1. Complete Setup + Foundational → mechanism ready (styled pages + `--theme`)
2. Add User Story 1 → modern default theme → Deploy/Demo (MVP!)
3. Add User Story 2 → hierarchy + AA contrast in both modes → Deploy/Demo
4. Add User Story 3 → responsive at 320 px → desktop → Deploy/Demo
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Developer A: User Story 1 (CSS base) → then hands `assets/default.css` forward
3. Developer B: foundational/theme tests in parallel (T009–T011)
4. Once US1 lands, Developer C: US2 contrast test (T015) in parallel with US2 CSS authoring (T016–T018)
5. Continue US3; only one author on `assets/default.css` at a time

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- `assets/default.css` is a single shared file — never parallelize edits to it
- Each user story is independently completable and testable
- Tests added (project norm per constitution): T009, T010, T011, T015
- Verify the gate before moving on: `cargo test && cargo clippy -- -D warnings && cargo fmt --check`
- Commit after each task or logical group
- Stop at any checkpoint to validate a story independently