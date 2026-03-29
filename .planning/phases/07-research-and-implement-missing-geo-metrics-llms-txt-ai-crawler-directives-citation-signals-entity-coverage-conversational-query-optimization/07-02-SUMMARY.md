---
phase: 07-research-and-implement-missing-geo-metrics
plan: 02
subsystem: analyzers
tags: [regex, scraper, json-ld, citations, entities, faq, geo]

requires:
  - phase: 03-geo-specific-analysis
    provides: geo.rs analyzer patterns, extract_types helper, AnalysisResult/Status types
provides:
  - 4 citation signal checks (geo-citation-stats, geo-citation-sources, geo-citation-quotes, geo-citation-references)
  - FAQ quality scoring (geo-faq-quality) for 40-60 word optimal range
  - 4 entity coverage checks (geo-entity-schema, geo-entity-about, geo-entity-proper-nouns, geo-entity-author)
affects: [07-03, 07-04, main.rs integration]

tech-stack:
  added: []
  patterns: [shared extract_types via pub(crate), mid-sentence proper noun detection to avoid sentence-start false positives]

key-files:
  created:
    - src/analyzers/geo_citations.rs
    - src/analyzers/geo_entities.rs
  modified:
    - src/analyzers/geo.rs
    - src/analyzers/mod.rs

key-decisions:
  - "Made extract_types pub(crate) in geo.rs for DRY reuse in geo_entities.rs (option a from plan)"
  - "Proper noun regex uses mid-sentence pattern (lowercase word before capitalized) to avoid sentence-start false positives"

patterns-established:
  - "Citation checks: regex-based detection on body text with Pass/Warn binary signals"
  - "Entity checks: JSON-LD parsing + body text analysis combined for multi-signal detection"

requirements-completed: [D-09, D-10, D-11, D-13, D-16]

duration: 3min
completed: 2026-03-29
---

# Phase 07 Plan 02: Citation Signals and Entity Coverage Summary

**9 GEO checks: 4 citation signal detectors, FAQ answer quality scorer, and 4 entity coverage analyzers using regex + JSON-LD parsing**

## Performance

- **Duration:** 3 min
- **Started:** 2026-03-29T16:52:13Z
- **Completed:** 2026-03-29T16:55:00Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- 4 citation signal checks detecting statistics, source attributions, blockquotes, and reference sections
- FAQ quality check validating answer word counts against 40-60 word optimal range from FAQPage JSON-LD
- 4 entity coverage checks detecting Person/Organization schema, about/mentions properties, proper noun density, and author attribution
- Shared extract_types helper from geo.rs via pub(crate) visibility

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement geo_citations.rs** - `e7c9afa` (feat)
2. **Task 2: Implement geo_entities.rs** - `5f2a84d` (feat)

## Files Created/Modified
- `src/analyzers/geo_citations.rs` - 5 checks: 4 citation signals + FAQ quality scoring
- `src/analyzers/geo_entities.rs` - 4 entity coverage checks
- `src/analyzers/geo.rs` - Made extract_types pub(crate) for cross-module reuse
- `src/analyzers/mod.rs` - Registered geo_citations and geo_entities modules

## Decisions Made
- Made extract_types pub(crate) in geo.rs for DRY reuse (plan option a preferred over duplication)
- Proper noun regex uses mid-sentence pattern to avoid sentence-start false positives (per Pitfall 4 in plan)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Citation and entity analyzers ready for integration into main analysis pipeline
- Check IDs follow geo- prefix convention, compatible with scoring.rs routing
- 30 new unit tests (17 citation + 13 entity) all passing alongside 143 total lib tests

---
*Phase: 07-research-and-implement-missing-geo-metrics*
*Completed: 2026-03-29*

## Self-Check: PASSED
