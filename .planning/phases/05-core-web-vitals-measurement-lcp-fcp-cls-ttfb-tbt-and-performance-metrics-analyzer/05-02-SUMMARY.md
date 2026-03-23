---
phase: "05"
plan: "02"
subsystem: analyzers/performance
tags: [performance, cwv, lcp, fcp, cls, ttfb, tbt, chromiumoxide, javascript, tdd]
dependency_graph:
  requires: [CategoryScores.performance, severity_points.perf-*, analyzers.performance.stub]
  provides: [analyze_vitals, classify_lcp, classify_fcp, classify_cls, classify_ttfb, classify_tbt]
  affects: [src/analyzers/performance.rs]
tech_stack:
  added: []
  patterns: [PerformanceObserver with buffered:true, eval_f64 error-absorbing helper, pub(crate) classify_* for direct unit testing]
key_files:
  created: []
  modified:
    - src/analyzers/performance.rs
decisions:
  - "eval_f64 returns -1.0 on any CDP error — non-panicking, maps CDP failures to the same 'unmeasured' path as missing data"
  - "classify_* are pub(crate) not pub — visible to unit tests but not exposed as part of the public module API"
  - "TBT value 0.0 is a Pass not an error — 0ms TBT is legitimately good (no long tasks)"
  - "LCP/CLS/TBT use buffered: true in PerformanceObserver; FCP/TTFB use synchronous reads (no observer needed)"
metrics:
  duration_seconds: 90
  completed_date: "2026-03-23"
  tasks_completed: 1
  files_modified: 1
---

# Phase 5 Plan 2: Core Web Vitals Measurement Module Summary

**One-liner:** Implemented `analyze_vitals(page: &Page) -> Vec<AnalysisResult>` in `src/analyzers/performance.rs` with 5 private `measure_*` helpers, 5 JS constants using PerformanceObserver (buffered: true where required), a non-panicking `eval_f64` CDP helper, and 30 unit tests covering all boundary cases for all 5 Google CWV thresholds via `pub(crate) classify_*` functions.

## What Was Built

Replaced the compile stub in `src/analyzers/performance.rs` with the full Core Web Vitals measurement module. The module exposes one public function (`analyze_vitals`) that measures all 5 metrics via JavaScript evaluation in a chromiumoxide page context and returns exactly 5 scored `AnalysisResult` entries.

Key design points:
- LCP uses `PerformanceObserver` with `buffered: true` and a 5-second `setTimeout` fallback
- CLS sums layout-shift entries where `hadRecentInput` is false
- TBT sums `Math.max(0, entry.duration - 50)` for longtask entries (prevents negative values)
- FCP reads `performance.getEntriesByName('first-contentful-paint')[0].startTime` synchronously
- TTFB reads `performance.getEntriesByType('navigation')[0].responseStart` synchronously
- `eval_f64` absorbs all CDP failures (returns -1.0), which classify_* functions map to `Status::Fail` with descriptive "could not be measured" messages

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 (RED) | Add failing tests for classify_* functions | e9be588 | src/analyzers/performance.rs |
| 1 (GREEN) | Implement full performance.rs with analyze_vitals + classify_* | c197321 | src/analyzers/performance.rs |

## Decisions Made

- `eval_f64` returns -1.0 on any CDP error — maps CDP failures to the same "unmeasured" code path as missing performance data, keeping error handling uniform
- `classify_*` functions are `pub(crate)` — directly testable in unit tests without requiring a browser, but not exposed as part of the public module API
- TBT 0.0ms is a `Status::Pass` — 0ms TBT means no long tasks fired, which is legitimately excellent performance (not an error state)
- LCP, CLS, and TBT use `PerformanceObserver` with `buffered: true` so already-fired entries are included; FCP and TTFB use synchronous reads since their entries are always available post-load

## Test Coverage

30 unit tests added (TDD: RED commit then GREEN commit), all passing:

**LCP (6 tests):** pass below/at boundary, warn above/at boundary, fail above boundary, fail unmeasured
**FCP (6 tests):** pass below/at boundary, warn above/at boundary, fail above boundary, fail unmeasured
**CLS (6 tests):** pass at zero/at boundary, warn above/at boundary, fail above boundary, fail unmeasured
**TTFB (6 tests):** pass below/at boundary, warn above/at boundary, fail above boundary, fail unmeasured
**TBT (6 tests):** pass at zero/at boundary, warn above/at boundary, fail above boundary, fail unmeasured

All 111 total tests pass (81 unit + 7 integration, plus 30 new classify tests = 111 unit total, 7 integration).

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None. The `analyze_vitals` stub from Plan 01 has been fully replaced with a working implementation. The function now calls 5 real measure_* helpers backed by CDP JavaScript evaluation.

## Self-Check: PASSED

- `src/analyzers/performance.rs` exists — FOUND
- `pub async fn analyze_vitals` at line 61 — FOUND
- `buffered: true` appears 3 times (LCP line 14, CLS line 32, TBT line 51) — FOUND
- `Math.max(0` at line 48 — FOUND
- `cargo test classify` — 30 passed, 0 failed — CONFIRMED
- `cargo test` (all) — 111 unit + 7 integration tests pass — CONFIRMED
- Commit e9be588 (RED) exists — FOUND
- Commit c197321 (GREEN) exists — FOUND
