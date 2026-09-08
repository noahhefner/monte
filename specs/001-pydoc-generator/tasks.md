---

description: "Task list for Python Documentation Generator (Rust CLI)"

---

# Tasks: Python Documentation Generator

**Input**: Design documents from `/specs/001-pydoc-generator/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: The project constitution (Testing Standards) mandates automated tests for every feature and bug fix, including happy paths, edge cases, and error conditions. Unit, contract, and integration tests are therefore included and written before implementation (TDD-style per plan.md).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single Rust crate**: `src/`, `tests/` at repository root (per plan.md Project Structure)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [X] T001 Create Rust project structure per implementation plan (src/, tests/, Cargo.toml)
- [X] T002 Add `rustpython-parser` (0.4.0) dependency in Cargo.toml
- [X] T003 [P] Configure linting and formatting (`rustfmt`, `clippy`) per constitution
- [X] T004 Define module skeleton in src/lib.rs and src/main.rs per plan.md structure

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T005 Create intermediary data model (DocumentedElement, ImportPath, DocComment union, ArgDoc/ReturnDoc/RaisesDoc, DocumentedProject) in src/model.rs per data-model.md
- [X] T006 Implement docstring capture — read the module/class/function first-statement string literal from the AST into the RawElement in src/parser.rs per research.md D-002
- [X] T007 Implement JavaDoc tag parser for `@arg`, `@return`, `@raises` + description text in src/tags.rs per contracts/docstring-syntax.md (FR-002)
- [X] T008 Implement ImportPath construction from file directory + element name in src/path.rs (FR-004, SC-006)
- [X] T009 Implement basic HTML escaping utility in src/html.rs (used by US1)
- [X] T010 Configure error handling so malformed docstrings and undocumented elements never abort the run in src/error.rs (FR-007)

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Generate documentation from annotated code (Priority: P1) 🎯 MVP

**Goal**: Scan a source tree, parse modules/classes/functions via rustpython_parser, extract JavaDoc-style docstrings into the intermediary representation, and emit HTML documenting every element.

**Independent Test**: Run the generator over a small documented sample package and confirm the HTML output contains every module, class, and function with full description where documented and an existence-only entry otherwise (SC-002, FR-004).

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T011 [P] [US1] Unit test for tag parser (`@arg`/`@return`/`@raises`) in tests/unit/tags.rs
- [X] T012 [P] [US1] Unit test for ImportPath construction in tests/unit/path.rs
- [X] T013 [P] [US1] Unit test for docstring normalization (dedent, blank-line trimming) in tests/unit/docstring.rs
- [X] T014 [P] [US1] Contract test asserting documented + undocumented element output in tests/contract/generate_docs.rs (per contracts/cli.md, docstring-syntax.md)
- [X] T014a [P] [US1] Unit test asserting expected behavior for empty directory, no Python files, and unrecognized tags in tests/unit/edges.rs
- [X] T014b [P] [US1] Integration test for non-existent input path returns clear error and non-zero exit in tests/integration/invalid_input.rs

### Implementation for User Story 1

- [X] T015 [P] [US1] Implement parser module wrapping rustpython_parser to collect modules/classes/functions in src/parser.rs (FR-001, FR-003)
- [X] T016 [P] [US1] Implement docstring extraction + tag parsing into DocComment for each element in src/extract.rs (FR-002)
- [X] T017 [US1] Implement HTML page generation from DocumentationProject in src/html.rs (FR-004; depends on T015, T016)
- [X] T018 [US1] Implement directory walker and end-to-end pipeline orchestration in src/pipeline.rs (FR-001, FR-009)
- [X] T019 [US1] Implement CLI wiring (`pydoc-gen <input_dir> <output_dir>`) including a clear error and non-zero exit for a non-existent or unreadable input dir per contracts/cli.md in src/cli.rs + src/main.rs
- [X] T020 [US1] Add per-element malformed-docstring error message rendering (FR-007)
- [X] T021 [US1] Add logging to satisfy constitution observability expectations

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently (MVP)

---

## Phase 4: User Story 2 - Maintain accurate docs on code changes (Priority: P2)

**Goal**: Re-running the generator on unchanged source yields byte-identical output, and on changed source reflects the update; removed elements disappear from output.

**Independent Test**: Change a function docstring and re-run; confirm only the affected HTML updates. Delete a documented element and re-run; confirm it is no longer present (SC-004, FR-006).

### Tests for User Story 2

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T022 [P] [US2] Integration test for deterministic regeneration (byte-for-byte) in tests/integration/regeneration.rs
- [X] T023 [P] [US2] Integration test for updated docstring reflected + removed element removed in tests/integration/regeneration.rs

### Implementation for User Story 2

- [X] T024 [P] [US2] Ensure deterministic, stable (path-sorted) ordering of elements in src/model.rs / src/pipeline.rs (FR-006)
- [X] T025 [US2] Ensure clean handling of stale output files when elements are removed in src/pipeline.rs (FR-006)
- [X] T026 [US2] Optimize regeneration path to be incremental/non-destructive where practical in src/pipeline.rs

**Checkpoint**: At this point, User Story 2 should be fully functional and tested (deterministic regeneration, docstring updates, removed elements, path-sorted output).

## Notes on US3 implementation

- The site is now multi-page: `index.html` (navigation) plus one page per
  module/package (`pkg.utils` → `pkg/utils.html`, package `pkg` →
  `pkg.html`). Index entries link to each element's page anchor, so any
  element is reachable in two clicks (SC-003).
- Per-module pages make US2's T025 real: `write_site` deletes stale HTML for
  modules removed from the source (covered by regeneration tests).
- T029 (flat index map) was already satisfied by `ImportPath`-keyed
  `BTreeMap` + `rebuild_index` in the model consumed by the rendering/docs.

## Notes on US2 implementation

- T025/T026 resolution: the site is a single self-contained `index.html`
  regenerated from scratch on every run, so removals never leave stale
  generated output and re-running over an existing output dir is idempotent
  (covered by `rerun_over_existing_output_is_idempotent` and
  `removed_module_disappears_from_regenerated_output`). Per-element
  incremental writes were deferred: the constitution requires measuring
  before optimizing (T035a), and a single-page site leaves no incremental
  benefit until per-module pages exist (US3).

---

## Phase 5: User Story 3 - Navigate large codebases efficiently (Priority: P3)

**Goal**: Provide a navigation index keyed by full Python package import path so readers can locate any element quickly, including same-named elements in different modules.

**Independent Test**: Generate docs for a multi-file package; confirm every documented element is reachable from the index within three clicks, and two same-named elements (e.g. `pkg.a.Helper`, `pkg.b.Helper`) remain individually addressable (SC-003, SC-006, FR-005).

### Tests for User Story 3

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T027 [P] [US3] Integration test for index navigation coverage in tests/integration/navigation.rs
- [X] T028 [P] [US3] Integration test for duplicate simple names resolved by full path in tests/integration/navigation.rs

### Implementation for User Story 3

- [X] T029 [P] [US3] Build flat index map keyed by full ImportPath in src/model.rs / src/index.rs (FR-005)
- [X] T030 [P] [US3] Generate index/navigation HTML pages in src/index.rs (FR-005)
- [X] T031 [US3] Add navigation links between pages and index entries in src/html.rs (SC-003)
- [X] T032 [US3] Ensure full package import path is displayed on every entry in src/html.rs (FR-004, SC-006)

**Checkpoint**: All user stories complete. Polish tasks T033-T038 done:
README.md added; dead code and stale comment references removed; baseline
benchmark recorded (mean ≈ 28 ms for 200 modules, SC-001 PASS, so T035b
concluded "no optimization warranted"); edge-case unit tests extended;
HTML escaping verified against injected markup; quickstart scenarios 1-5
passed end-to-end via scripts/validate-quickstart.sh.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [X] T033 [P] Documentation updates in README.md and contracts/
- [X] T034 Code cleanup and refactoring across src/ (per constitution Readability)
- [X] T035a Establish baseline benchmark of the parse → extract → generate pipeline against the sample project and record the measured time against SC-001 (site generated in under 60 seconds) in benches/generate.rs / bench-results.md
- [X] T035b [P] Optimize pipeline hot spots only after T035a baseline exists, per constitution Performance (measure before optimize) in src/parser.rs, src/extract.rs, src/html.rs
- [X] T036 [P] Additional unit tests for edge cases (empty dir, no Python files, unrecognized tags, non-existent input path) in tests/unit/
- [X] T037 Security hardening of HTML escaping (prevent injection via docstring content)
- [X] T038 Run quickstart.md validation scenarios end-to-end

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**:
  - All depend on Foundational phase completion
  - US2 (Phase 4) depends on US1 (Phase 3) — regeneration and element-removal behavior build on the primary generation pipeline
  - US3 (Phase 5) depends on US1 — the index iterates over generated entries but is otherwise independent of US2
  - US2 and US3 can run in parallel once US1 is done
- **Polish (Phase 6)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Depends on US1 (shares the pipeline); independently testable via regeneration tests
- **User Story 3 (P3)**: Depends on US1; does not depend on US2

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Models before services
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel (T003)
- All Foundational tasks can run in parallel (T005–T010)
- Once Foundational completes, US1 starts; story-specific [P] tasks within US1 run in parallel (tests T011–T014, then T015–T016, then T017+)
- Once US1 completes, US2 and US3 can be worked in parallel
- All tests for a story marked [P] run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Unit test for tag parser in tests/unit/tags.rs"
Task: "Unit test for ImportPath construction in tests/unit/path.rs"
Task: "Unit test for docstring normalization in tests/unit/docstring.rs"
Task: "Contract test for documented + undocumented output in tests/contract/generate_docs.rs"

# Launch independent implementation tasks together:
Task: "parser module in src/parser.rs"
Task: "docstring extraction into DocComment in src/extract.rs"

# After the two above complete, launch serial chain:
Task: "HTML page generation in src/html.rs (depends on parser + extract)"
Task: "pipeline orchestration in src/pipeline.rs"
Task: "CLI wiring in src/cli.rs + src/main.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (parse → extract → generate HTML)
4. **STOP and VALIDATE**: Run quickstart scenario 1 (documented elements) and scenario 2 (undocumented), plus malformed handling (scenario 3)
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently (regeneration/determinism) → Deploy/Demo
4. Add User Story 3 → Test independently (navigation) → Deploy/Demo
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done and US1 lands:
   - Developer A: User Story 2 (regeneration)
   - Developer B: User Story 3 (navigation)
3. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence
