---

description: "Task list for YAML docstring parsing"

---

# Tasks: YAML Docstring Parsing

**Input**: Design documents from `/specs/003-yaml-docstring-parsing/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Included. The Monte Constitution (Testing Standards) mandates automated
tests for every feature change; test coverage is mapped to the spec
requirements and quickstart.md scenarios per story.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- This is a single Rust crate; all paths are relative to the repo root.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add the YAML parsing dependency chosen in research.md (D-002).

- [ ] T001 Add the YAML crate (`serde_yml`; fall back to `yaml-rust2` if the crate is unavailable/has issues) to `[dependencies]` in `Cargo.toml`, then run `cargo build` to confirm the dependency resolves and the existing code still compiles unchanged.

  Per research.md D-002, prefer `serde_yml` (serde integration → YAML schema as typed Rust structs). If it cannot be added, use `yaml-rust2` and adjust the extraction logic in T002 to parse into a generic YAML Value instead of typed structs.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Implement the YAML docstring parser core and remove the legacy `@`-tag parser. MUST be complete before ANY user story can be implemented.

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [ ] T002 Add YAML deserialization schema structs (`YamlDoc { description, args, returns, raises }`, `YamlArg { name, description }`, `YamlReturn { description }`, `YamlRaise { #[serde(rename = "type", alias = "exception")] exception, description }`) and a `parse_yaml_docstring(text: &str) -> TagParseResult` function implementing three-way dispatch in `src/extract.rs`:
  1. `serde_yml::from_str::<YamlDoc>` succeeds → `TagParseResult::Ok(DocContent{..})` mapping `description`/`args`/`returns`/`raises` (unknown keys, e.g. `example`, `returns.type`, are silently ignored per data-model.md);
  2. else `serde_yml::from_str::<String>` succeeds (plain scalar / prose) → `TagParseResult::Ok(DocContent{ description: <text>, args: vec![], returns: None, raises: vec![] })` (FR-010);
  3. else → `TagParseResult::Error(<message>)` (FR-007).
- [ ] T003 Rewire `docstring_comment()` in `src/extract.rs` to join the normalized lines from `docstring_lines()` with newlines, call `parse_yaml_docstring`, and convert the result via the existing `TagParseResult → DocComment` conversion; remove the `tags::parse_block` call.
- [ ] T004 [P] Remove the legacy `tags` module: delete `src/tags.rs`, remove `pub mod tags;` from `src/lib.rs`, and repoint the `[[test]]` entry `unit_tags` (path `tests/unit/tags.rs`) to the new `unit_extract` (path `tests/unit/extract.rs`) in `Cargo.toml`. Create an empty (or initial) `tests/unit/extract.rs`.
- [ ] T005 [P] Update `tests/unit/edges.rs`: remove `use pydoc_gen::tags::{TagParseResult, parse_block};` and delete the `unrecognized_tags_everywhere_still_parse` test (it tested the removed `@`-tag parser; unknown-`@`-line behavior is now covered by US2/US3 error tests). Leave the empty-dir, no-python-files, and invalid-input-path tests intact.

**Checkpoint**: Foundation ready — run `cargo test`; the crate compiles, the YAML parser is wired into the pipeline, and all non-removed existing tests pass. User story implementation can now begin.

---

## Phase 3: User Story 1 - Document code with structured YAML docstrings (Priority: P1) 🎯 MVP

**Goal**: Valid YAML docstrings yield full documentation (description, args, returns, raises) and plain-prose docstrings yield a description-only entry (FR-001, FR-003, FR-010, SC-003).

**Independent Test**: Point the generator at a package whose docstrings are YAML mappings (mirroring `demo/demo_2.py`) and confirm every documented element's page shows the YAML values field-for-field; a plain-prose docstring shows its text as the description.

### Tests for User Story 1 ⚠️

> **NOTE**: Write these tests FIRST, ensure they FAIL against the pre-US1 state, then complete implementation.

- [ ] T006 [P] [US1] Add unit tests in `tests/unit/extract.rs` for valid YAML mapping extraction: a docstring with all four keys (`description`, `args` list of `name`/`description`, `returns.description`, `raises` list of `type`/`description`), a partial mapping (description only), and `raises[].type` mapping onto the model's `exception` field. Assert field-for-field equality with the YAML values (SC-003).
- [ ] T007 [P] [US1] Add unit tests in `tests/unit/extract.rs` for (a) plain-prose scalar docstrings → `DocComment::Ok` with that text as `description` and empty `args`/`raises` (FR-010), and (b) empty/blank docstrings → `DocComment::None`.

### Implementation for User Story 1

- [ ] T008 [US1] Update the shared fixtures in `tests/contract/generate_docs.rs` (`sample/__init__.py` and `sample/formatting.py`) from `@arg`/`@return`/`@raises` tag syntax to YAML mapping docstrings, updating assertions as needed so `generates_documented_and_undocumented_entries` and `writes_html_covering_all_elements` still verify description/args/returns/raises extraction.
- [ ] T009 [US1] Add a YAML docstring fixture that includes unrecognized keys (`example:` and a `returns.type`) mirroring `demo/demo_2.py`, and extend a `tests/contract/generate_docs.rs` assertion to confirm those keys are accepted without affecting the extracted fields (unknown-key tolerance, docstring-syntax contract).

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently (valid YAML + prose docstrings generate correct documentation; MVP).

---

## Phase 4: User Story 2 - Receive clear feedback for docstrings that are not valid YAML (Priority: P2)

**Goal**: Docstrings that are invalid YAML or have wrong-shaped recognized keys produce an in-entry error message and never abort the run (FR-007, SC-004).

**Independent Test**: Feed the generator a package where one docstring is invalid YAML and another element is documented correctly; verify the run completes, the invalid element's entry shows an error, and the healthy element is fully documented.

### Tests for User Story 2 ⚠️

> **NOTE**: Write these tests FIRST, ensure they FAIL against the pre-US2 state, then complete implementation.

- [ ] T010 [P] [US2] Add unit tests in `tests/unit/extract.rs` for invalid-YAML docstrings → `DocComment::Error`: malformed YAML (bad indentation / tabs), and content that cannot be a mapping or scalar (includes at least one legacy `@`-prefixed line; see US3). Assert the run-level behavior is preserved elsewhere (error is per-element).
- [ ] T011 [P] [US2] Add unit tests in `tests/unit/extract.rs` for wrong-shaped recognized keys → `DocComment::Error`: `args:` given a scalar value instead of a list, and `returns:` given a list instead of a mapping (per spec edge case and data-model.md validation rules).

### Implementation for User Story 2

- [ ] T012 [US2] Ensure error messages produced by `parse_yaml_docstring` in `src/extract.rs` are descriptive and include stable phrasing such as "docstring was not written correctly" (the exact text the contract tests assert), distinguishing invalid-YAML from wrong-shape failures where practical (FR-007).
- [ ] T013 [US2] Update the `malformed_docstring_renders_error_without_aborting_run` test in `tests/contract/generate_docs.rs` to YAML-based malformed fixtures (e.g. an `@arg`-line docstring and an `args: not_a_list` docstring), keeping the assertions that the run completes, the bad element's page shows the error message, and the healthy element still renders.

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently.

---

## Phase 5: User Story 3 - Migrate existing JavaDoc-style documentation (Priority: P3)

**Goal**: Legacy JavaDoc-style `@arg`/`@return`/`@raises` docstrings are predictably treated as malformed (they fail YAML parsing), producing an error entry, and no repo assets still advertise them as a supported format (spec Q1:A, FR-007).

**Independent Test**: Point the generator at a package documented with legacy `@`-tag docstrings; verify each such element shows an error entry while the run completes, and that no remaining docstring fixtures/README examples use `@`-tags as valid input.

### Tests for User Story 3 ⚠️

> **NOTE**: Write these tests FIRST, ensure they FAIL against the pre-US3 state, then complete implementation.

- [ ] T014 [P] [US3] Add unit tests in `tests/unit/extract.rs` proving legacy JavaDoc-style docstrings (`@arg x  input`, `@return str  output`, `@raises ValueError  ...`) fail YAML parsing and yield `DocComment::Error`; assert the message and that other (valid) elements are unaffected (migration story, spec US3 acceptance).

### Implementation for User Story 3

- [ ] T015 [US3] Sweep the repository for remaining legacy `@`-tag docstring fixtures (`rg -n "@arg|@return|@raises" src/ tests/ demo/ README.md`) and convert reader-facing examples to the YAML format from `contracts/docstring-syntax.md`; convert `demo/demo.py` to YAML docstrings (keeping `demo/demo_2.py` as the canonical structural reference). Keep only intentional negative examples (in US2/US3 error tests).

**Checkpoint**: All user stories should now be independently functional.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories.

- [ ] T016 Update `README.md`: replace the "JavaDoc-style" feature description and the inline `@arg`/`@return`/`@raises` example with the YAML docstring format documented in `contracts/docstring-syntax.md` (include the four recognized keys and a plain-prose note).
- [ ] T017 [P] Update user-facing crate metadata: `description` in `Cargo.toml` and the module-level doc comment in `src/lib.rs` to describe YAML-docstring input instead of "JavaDoc-style docstrings".
- [ ] T018 [P] Update the benchmark fixture in `benches/generate.rs` to use YAML docstrings (per quickstart.md), run `cargo bench`, and compare against the pre-change baseline captured in `specs/001-pydoc-generator/bench-results.md` to confirm no performance regression (Constitution: Performance principle; measure before optimizing).
- [ ] T019 Run the full validation pass and fix failures: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (all unit/contract/integration suites), and execute every scenario in `quickstart.md` against the built binary.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately.
- **Foundational (Phase 2)**: Depends on Setup (T001). BLOCKS all user stories.
- **User Stories (Phase 3+)**:
  - **US1 (T006–T009)**: Depends on Foundational (T002, T003; T004/T005 for test wiring).
  - **US2 (T010–T013)**: Depends on Foundational (T002).
  - **US3 (T014–T015)**: Depends on Foundational (T002).
  - All three depend on the same extraction module and `tests/unit/extract.rs` / `tests/contract/generate_docs.rs`, so sequential priority order (US1 → US2 → US3) is recommended to avoid file conflicts.
- **Polish (Phase 6)**: Depends on all desired user stories being complete.

### Story-Level Dependency Note

- **US1**: No dependencies on other stories — the MVP increment.
- **US2**: Independent of US1's data path (error branch of the same parser); shares `tests/unit/extract.rs` and `tests/contract/generate_docs.rs`.
- **US3**: Independent; relies on the invalid-YAML path built for US2.

### Within Each User Story

- Tests are written in the same commit/session as the implementation they cover (Constitution: Testing Standards); the ⚠️ notes above call out writing them before/alongside the change so failing-then-passing is observable.
- Schema structs and the parser core (T002) precede wiring (T003) and cleanup (T004/T005).
- Error-path tests (T010/T011 → T012) then fixture updates (T013).

### Parallel Opportunities

| Window | Tasks | File independence |
|--------|-------|-------------------|
| Phase 2 cleanup | T004, T005 | Different files (`Cargo.toml`/`src/lib.rs`/`src/tags.rs` vs `tests/unit/edges.rs`) |
| US1 tests | T006, T007 | Same file `tests/unit/extract.rs` — write together, single commit |
| US2 tests | T010, T011 | Same file `tests/unit/extract.rs` — write together, single commit |
| Polish | T016 vs T017 vs T018 | `README.md` / `Cargo.toml`+`src/lib.rs` / `benches/generate.rs` |

The three user stories touch overlapping files (`src/extract.rs`, `tests/unit/extract.rs`, `tests/contract/generate_docs.rs`), so run them sequentially (P1 → P2 → P3); parallel only staffed teams split by phase.

---

## Parallel Example: User Story 1

```bash
# Unit test coverage for the YAML extraction paths (write together):
Task: "T006: unit tests for valid mapping extraction in tests/unit/extract.rs"
Task: "T007: unit tests for prose/scalar and empty/blank docstrings in tests/unit/extract.rs"

# Contract fixtures (after unit tests pass):
Task: "T008: convert contracts fixtures to YAML in tests/contract/generate_docs.rs"
Task: "T009: unknown-key tolerance fixture in tests/contract/generate_docs.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001).
2. Complete Phase 2: Foundational (T002–T005, CRITICAL — blocks all stories).
3. Complete Phase 3: User Story 1 (T006–T009).
4. **STOP and VALIDATE**: valid-YAML and plain-prose docstrings generate correct docs (`cargo test` + quickstart scenarios 1–3).
5. Deploy/demo the MVP.

### Incremental Delivery

1. Setup + Foundational → foundation ready.
2. Add User Story 1 → test independently → demo (MVP!).
3. Add User Story 2 → invalid-YAML error feedback is in place → demo.
4. Add User Story 3 → legacy docstrings handled + repo migrated → demo.
5. Each story adds value without breaking previous stories.

### Parallel Team Strategy

After Foundational, teams can split by phase; within a phase, follow the Parallel Opportunities table. Keep story work sequential per shared files.

---

## Notes

- **[P] tasks** = different files, no dependencies on incomplete tasks.
- **[Story] labels** map tasks to user stories for traceability.
- Each user story is independently completable and testable (see "Independent Test" per phase).
- Commit after each task or logical group.
- Stop at any checkpoint to validate the story independently.
- Avoid: vague tasks, same-file conflicts without sequencing, cross-story dependencies that break independence.
- The exact YAML crate may need adjustment based on availability: `serde_yml` preferred (research.md D-002), `yaml-rust2` fallback — if the fallback is used, replace typed-struct extraction in T002 with generic Value parsing.