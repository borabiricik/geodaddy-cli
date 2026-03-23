---
phase: 04-site-wide-crawling-polish
plan: "01"
subsystem: crawling
tags: [rust, quick-xml, scraper, reqwest, url, sitemap, bfs, robots-txt]

requires:
  - phase: 03-geo-differentiators
    provides: CategoryScores struct and calculate_score in scoring.rs

provides:
  - src/crawling.rs with 8 public crawl-logic functions
  - fetch_sitemap_urls: parses sitemap.xml, returns priority-sorted URL list
  - collect_links_bfs: BFS link-following crawler up to max_depth
  - normalize_url: strips fragments and trailing slashes
  - extract_crawl_delay: parses crawl-delay from robots.txt body
  - needs_js_rendering: detects thin/JS-rendered pages
  - aggregate_scores: averages score and CategoryScores across all pages
  - format_progress_known/format_progress_unknown: progress line formatters

affects:
  - 04-02 (wires these functions into main.rs crawl loop)

tech-stack:
  added: []
  patterns:
    - "aggregate_scores accepts &[(f64, CategoryScores)] tuples to avoid cross-module PageResult dependency"
    - "UrlSet/UrlEntry deserialization via quick_xml::de::from_str with default_priority fallback"
    - "Same-origin link filter via url::Url origin() equality check"

key-files:
  created:
    - src/crawling.rs
  modified:
    - src/main.rs (mod crawling; declaration added)
    - src/scoring.rs (#[derive(Clone)] added to CategoryScores)

key-decisions:
  - "aggregate_scores takes &[(f64, CategoryScores)] tuples instead of PageResult to avoid cross-module dependency (PageResult lives in main.rs)"
  - "CategoryScores derives Clone to support tuple-based aggregation pattern"
  - "fetch_sitemap_urls returns None for empty sitemap (triggers link-following fallback)"

patterns-established:
  - "Crawl-logic pure functions in crawling.rs keep async (HTTP) and sync (parsing) functions separate"
  - "All 15 unit tests in #[cfg(test)] block — pure functions test synchronously, no HTTP needed"

requirements-completed:
  - CRAWL-01
  - CRAWL-02

duration: 20min
completed: "2026-03-23"
---

# Phase 04 Plan 01: Crawling Module Summary

**Pure crawl-logic module (src/crawling.rs) with 8 public functions and 15 unit tests covering sitemap parsing, BFS link-following, URL normalization, crawl-delay extraction, JS detection, and score aggregation**

## Performance

- **Duration:** ~20 min
- **Started:** 2026-03-23T14:46:00Z
- **Completed:** 2026-03-23T15:06:00Z
- **Tasks:** 1 (TDD)
- **Files modified:** 3

## Accomplishments

- Created src/crawling.rs with all 8 required public functions
- All 15 unit tests compile and pass (cargo test: 71 passed)
- Added Clone derive to CategoryScores (required for aggregate_scores tuple pattern)
- Declared `mod crawling;` in main.rs

## Task Commits

1. **Task 1: Create src/crawling.rs with all crawl-logic functions and test scaffold** - `5deb992` (feat)

## Files Created/Modified

- `src/crawling.rs` - 8 public functions + 15 unit tests covering all crawl behaviors
- `src/scoring.rs` - Added `#[derive(Clone)]` to CategoryScores
- `src/main.rs` - Added `mod crawling;` declaration

## Decisions Made

- `aggregate_scores` accepts `&[(f64, CategoryScores)]` tuples instead of `&[PageResult]` to avoid cross-module dependency (PageResult is defined in main.rs, not scoring.rs)
- `CategoryScores` derives `Clone` to support the tuple-based aggregation pattern cleanly
- `fetch_sitemap_urls` returns `None` for empty sitemaps (triggers link-following fallback) matching the sitemap-first strategy

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Removed unused `anyhow::Result` import from crawling.rs**
- **Found during:** Task 1 (initial compile)
- **Issue:** Import was included in the action template but none of the public functions return `anyhow::Result` — they use `Option<T>` or concrete types
- **Fix:** Removed unused import to eliminate compiler warning
- **Files modified:** src/crawling.rs
- **Verification:** cargo test passes with no warnings
- **Committed in:** 5deb992 (part of task commit)

---

**Total deviations:** 1 auto-fixed (1 unused import cleanup)
**Impact on plan:** Minor cleanup. No scope creep.

## Issues Encountered

None — plan executed cleanly. All 15 tests passed on first run.

## Next Phase Readiness

- All 8 crawling.rs functions ready for wiring into main.rs (Plan 02)
- Plan 02 can use exact function signatures as specified in 04-02-PLAN.md interfaces block

---
*Phase: 04-site-wide-crawling-polish*
*Completed: 2026-03-23*
