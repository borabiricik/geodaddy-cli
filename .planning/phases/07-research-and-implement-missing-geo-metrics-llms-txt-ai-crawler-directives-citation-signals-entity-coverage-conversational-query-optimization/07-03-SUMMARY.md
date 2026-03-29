---
phase: 07-research-and-implement-missing-geo-metrics
plan: 03
subsystem: analyzers
tags: [rust, geo, query-optimization, freshness, howto, json-ld, regex, scraper]

requires:
  - phase: 07-01
    provides: "geo.rs extract_types pattern, scoring.rs severity_points for geo-* checks"
provides:
  - "4 conversational query optimization checks (qa-patterns, summary, snippet, faq)"
  - "Content freshness signal detection (JSON-LD dateModified, Last-Modified header, meta tags)"
  - "HowTo schema validation with step structure checking"
affects: [07-04, main-integration]

tech-stack:
  added: []
  patterns: ["heading regex detection for content structure analysis", "sibling DOM traversal for heading+paragraph pairs"]

key-files:
  created:
    - src/analyzers/geo_query.rs
    - src/analyzers/geo_freshness.rs
  modified: []

key-decisions:
  - "Snippet detection uses sibling traversal to find paragraph after heading, not generic paragraph scanning"
  - "FAQ detection checks both heading text and FAQPage JSON-LD schema for comprehensive coverage"
  - "find_howto_object is local to geo_freshness.rs rather than reusing extract_types from geo.rs"

patterns-established:
  - "Heading regex pattern for question detection: starts with what/how/why/when/where/who/which/can/does/is/are/should/will"
  - "CSS class/id attribute selectors for semantic section detection (tldr, summary, takeaway)"

requirements-completed: [D-12, D-14, D-15, D-17]

duration: 2min
completed: 2026-03-29
---

# Phase 07 Plan 03: Query Optimization, Freshness, and HowTo Schema Summary

**4 conversational query optimization checks plus freshness signal detection and HowTo schema validation -- 6 new GEO checks with 25 unit tests**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-29T16:52:23Z
- **Completed:** 2026-03-29T16:54:52Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Implemented 4 query optimization checks: QA heading patterns, TL;DR/summary section detection, featured snippet formatting (40-60 word paragraphs), and FAQ section detection
- Implemented freshness signal detection across 3 sources: JSON-LD dateModified, Last-Modified HTTP header, and meta tag equivalents
- Implemented HowTo schema validation checking both type presence and step property structure
- Full test coverage: 25 tests across both modules, all 159 lib tests pass

## Task Commits

Each task was committed atomically:

1. **Task 1: geo_query.rs -- 4 conversational query optimization checks** - `89277eb` (feat)
2. **Task 2: geo_freshness.rs -- freshness signals + HowTo schema validation** - `cef5253` (feat)

## Files Created/Modified
- `src/analyzers/geo_query.rs` - 4 query optimization checks: qa-patterns, summary, snippet, faq
- `src/analyzers/geo_freshness.rs` - Freshness signal detection and HowTo schema validation

## Decisions Made
- Snippet detection uses DOM sibling traversal to find the first `<p>` after each heading, then checks word count (40-60 range) -- more precise than scanning all paragraphs
- FAQ detection checks both heading text patterns and FAQPage JSON-LD schema for comprehensive coverage
- find_howto_object helper kept local to geo_freshness.rs rather than making extract_types public in geo.rs -- simpler module boundary

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All 6 checks ready for integration in main analysis pipeline (07-04)
- Check IDs already registered in scoring.rs severity_points map
- 159/159 lib tests passing

## Self-Check: PASSED

- src/analyzers/geo_query.rs: FOUND
- src/analyzers/geo_freshness.rs: FOUND
- Commit 89277eb: FOUND
- Commit cef5253: FOUND

---
*Phase: 07-research-and-implement-missing-geo-metrics*
*Completed: 2026-03-29*
