---
phase: 07-research-and-implement-missing-geo-metrics
plan: 04
subsystem: api
tags: [rust, analyzers, llms-txt, geo, integration, pipeline]

requires:
  - phase: 07-01
    provides: llms.txt and directive analyzers
  - phase: 07-02
    provides: citation, entity, and query analyzers
  - phase: 07-03
    provides: freshness and howto analyzers
provides:
  - Complete Phase 7 analyzer integration in lib.rs analysis pipeline
  - llms.txt site-wide fetch before page loop
  - HTTP response header capture for directive and freshness checks
affects: [scoring, cli-output, future-analyzers]

tech-stack:
  added: []
  patterns: [site-wide-fetch-before-loop, headers-cloned-before-body-consumption]

key-files:
  created: []
  modified: [src/lib.rs]

key-decisions:
  - "llms.txt fetched once before page loop (site-wide resource like robots.txt)"
  - "HTTP headers cloned before .text() consumes response body"
  - "All 9 new analyzer calls added after existing analyze_schema_stacking"

patterns-established:
  - "Site-wide resources (robots.txt, llms.txt) fetched once before page loop"
  - "Response headers captured via clone before body consumption for downstream analyzers"

requirements-completed: [D-01, D-02, D-03, D-05, D-07, D-15]

duration: 1min
completed: 2026-03-29
---

# Phase 7 Plan 4: Wire Phase 7 Analyzers Summary

**Integrated all 6 Phase 7 analyzer modules (18 new checks) into lib.rs analysis pipeline with llms.txt site-wide fetch and HTTP header capture**

## Performance

- **Duration:** 1 min
- **Started:** 2026-03-29T17:19:48Z
- **Completed:** 2026-03-29T17:21:04Z
- **Tasks:** 2
- **Files modified:** 1

## Accomplishments
- Wired 9 new analyzer function calls producing 18 check results per page
- Added fetch_llms_txt async function fetched once per site before page loop
- Captured HTTP response headers before body consumption for directive and freshness analyzers
- All 189 unit tests pass, all 8 integration tests pass

## Task Commits

Each task was committed atomically:

1. **Task 1: Add llms.txt fetch function and capture HTTP headers in analyze()** - `68c2442` (feat)
2. **Task 2: Full test suite verification and integration smoke test** - verification only, no code changes

## Files Created/Modified
- `src/lib.rs` - Added imports for 6 new analyzer modules, fetch_llms_txt function, HTTP header capture, and 9 new analyzer calls in page loop

## Decisions Made
- llms.txt fetched once before page loop (site-wide resource like robots.txt) -- avoids redundant fetches per page
- HTTP headers cloned before .text() consumes response body -- required by Rust ownership model
- New analyzer calls placed after existing analyze_schema_stacking, before vitals section

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All Phase 7 analyzers fully integrated into the analysis pipeline
- 18 new GEO checks emitted per page (llms.txt, directives, citations, FAQ quality, entities, query optimization, freshness, howto schema)
- Phase 7 complete -- all 4 plans executed successfully

---
*Phase: 07-research-and-implement-missing-geo-metrics*
*Completed: 2026-03-29*
