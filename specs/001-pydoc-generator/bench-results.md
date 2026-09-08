# Baseline Benchmark Results: Python Documentation Generator

**Branch**: `001-pydoc-generator` | **Date**: 2026-09-08 | **Plan**: [plan.md](plan.md)

Recorded from `cargo bench` (release profile) using `benches/generate.rs`
(T035a). Run again with `cargo bench`.

## Setup

- Synthetic project: 200 modules, 1 class + 10 functions each (with
  `@arg`/`@return` docstrings), `cargo bench`.

## Results

| Metric | Value |
|--------|-------|
| Modules | 200 |
| Elements | 200 (top-level modules) |
| Iterations | 5 |
| Min full-pipeline time | 27.94 ms |
| Mean full-pipeline time | 28.59 ms |

Per-iteration timings: 29.21 / 27.94 / 28.29 / 28.82 / 28.69 ms.

## SC-001 Check

Site generated in **under 60 seconds**: **PASS** (mean ≈ 28 ms, ~2000x
headroom).

## T035b Assessment

Per the constitution's Performance principle (measure before optimizing),
the baseline is already far below the SC-001 target. **No optimization of
src/parser.rs, src/extract.rs, or src/html.rs is warranted at this time**
(T035b). If the tool is later used on very large codebases, re-run this
bench before optimizing.