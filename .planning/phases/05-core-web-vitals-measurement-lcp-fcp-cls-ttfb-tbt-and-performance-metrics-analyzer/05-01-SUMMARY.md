---
phase: "05"
plan: "01"
subsystem: scoring
tags: [performance, scoring, CategoryScores, perf-vitals]
dependency_graph:
  requires: []
  provides: [CategoryScores.performance, severity_points.perf-*, calculate_score.4-way, analyzers.performance.stub]
  affects: [src/scoring.rs, src/crawling.rs, src/main.rs, src/analyzers/mod.rs, src/analyzers/performance.rs]
tech_stack:
  added: []
  patterns: [Option<f64> for optional category, pub(crate) for test-visible functions]
key_files:
  created:
    - src/analyzers/performance.rs
  modified:
    - src/scoring.rs
    - src/analyzers/mod.rs
    - src/crawling.rs
    - src/main.rs
decisions:
  - "performance: Option<f64> serializes as JSON null when None (not skipped) — consistent with D-05 design"
  - "severity_points made pub(crate) to allow direct unit test access"
  - "aggregate_scores averages only pages with Some(performance) — None pages not included in perf average"
metrics:
  duration_seconds: 131
  completed_date: "2026-03-23"
  tasks_completed: 2
  files_modified: 5
---

# Phase 5 Plan 1: Performance Scoring Foundation Summary

**One-liner:** Added `performance: Option<f64>` to `CategoryScores` with perf-lcp (10pts) and perf-fcp/cls/ttfb/tbt (5pts) severity weights, 4-way average scoring when vitals present, and a compile stub for the performance analyzer module.

## What Was Built

Extended the existing scoring infrastructure to support a fourth scoring category — performance — that integrates cleanly with the existing technical/content/geo 3-way average when no perf checks are present, and switches to a 4-way average when vitals checks are included.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Extend CategoryScores and scoring functions | 341d0a2 | src/scoring.rs, src/crawling.rs, src/main.rs |
| 2 | Declare performance module and create compile stub | bf4a320 | src/analyzers/mod.rs, src/analyzers/performance.rs |

## Decisions Made

- `performance: Option<f64>` field does NOT use `#[serde(skip_serializing_if)]` — it serializes as `null` in JSON when absent, making the field always present in the JSON output regardless of whether `--vitals` was passed
- `severity_points()` made `pub(crate)` so tests can assert on it directly without routing through `calculate_score`
- `aggregate_scores()` in crawling.rs computes performance average only over pages that have `Some(performance)` — pages without vitals data (None) are excluded from the perf average, preventing dilution

## Test Coverage

7 new unit tests added to scoring.rs, all green:
- `test_performance_category_null_when_absent`
- `test_performance_category_some_when_perf_checks_present`
- `test_perf_severity_points_lcp_is_10`
- `test_perf_severity_points_others_are_5`
- `test_three_way_average_without_perf`
- `test_four_way_average_with_perf`
- `test_perf_fail_does_not_affect_tech_cont_geo`

Zero regressions: 81 unit tests + 7 integration tests all pass.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed missing performance field in crawling.rs test fixtures**
- **Found during:** Task 1 (cargo test compile error)
- **Issue:** Two `CategoryScores` constructors in `crawling.rs` test block (lines ~416, ~424) also lacked the new `performance` field
- **Fix:** Added `performance: None` to both test fixture constructors
- **Files modified:** src/crawling.rs
- **Commit:** 341d0a2

**2. [Rule 2 - Missing functionality] Enhanced aggregate_scores to properly average performance**
- **Found during:** Task 1 (reviewing aggregate_scores logic)
- **Issue:** Plan specified `performance: None` for the aggregate path but aggregating performance as None loses data when pages have vitals
- **Fix:** Computed performance average only from pages with Some(performance), yielding None when no pages have vitals
- **Files modified:** src/crawling.rs
- **Commit:** 341d0a2

## Known Stubs

| File | Stub | Reason |
|------|------|--------|
| src/analyzers/performance.rs | `analyze_vitals` returns `vec![]` | Intentional — stub exists only to satisfy module declaration; replaced by Plan 05-02 |

## Self-Check: PASSED

- `src/scoring.rs` contains `performance: Option<f64>` — FOUND
- `src/scoring.rs` contains `"perf-lcp" => 10` — FOUND
- `src/scoring.rs` contains `perf_max` — FOUND
- `src/analyzers/mod.rs` contains `pub mod performance` — FOUND
- `src/analyzers/performance.rs` exists with `pub async fn analyze_vitals` — FOUND
- Commit 341d0a2 exists — FOUND
- Commit bf4a320 exists — FOUND
- All 88 tests pass — CONFIRMED
