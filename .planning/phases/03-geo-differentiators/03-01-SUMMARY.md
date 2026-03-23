---
phase: 03-geo-differentiators
plan: 01
subsystem: analyzers
tags: [geo, listicle, robots-txt, schema-stacking, regex, robotstxt]

# Dependency graph
requires:
  - phase: 02-core-analysis
    provides: AnalysisResult and Status types in scoring.rs, analyzer module pattern
provides:
  - analyze_listicle function for listicle format detection
  - analyze_ai_bots function for AI crawler robots.txt audit
  - analyze_schema_stacking function for triple schema stacking detection
  - geo module registered in analyzers/mod.rs
affects: [03-02, scoring integration, main orchestration]

# Tech tracking
tech-stack:
  added: [regex 1.12]
  patterns: [static check ID constants for &'static str fields, extract_types helper for recursive JSON-LD parsing]

key-files:
  created: [cli/src/analyzers/geo.rs]
  modified: [cli/Cargo.toml, cli/src/analyzers/mod.rs]

key-decisions:
  - "Used static &str constants for AI bot check IDs instead of Box::leak or format! -- avoids runtime allocation and unsafe"
  - "Comparison table detection requires th + 3 tr rows -- filters noise from layout tables"
  - "extract_types helper recursively handles @graph arrays for JSON-LD schema stacking"

patterns-established:
  - "GEO analyzer pattern: geo-* check ID prefix for GEO-specific analyzers"
  - "AI bot constant tuple array: (user_agent, check_id, service_description)"

requirements-completed: [GEO-01, GEO-02, GEO-03]

# Metrics
duration: 2min
completed: 2026-03-23
---

# Phase 03 Plan 01: GEO Analyzers Summary

**Three GEO-specific analyzers: listicle detection (4 pattern types), AI bot robots.txt audit (6 crawlers), and triple schema stacking (Article + ItemList + FAQPage) with JSON-LD @graph support**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-23T13:54:36Z
- **Completed:** 2026-03-23T13:56:54Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Listicle analyzer detects Top N headings, Best N headings, ordered lists, numbered heading sequences, and comparison tables
- AI bot analyzer checks GPTBot, ClaudeBot, PerplexityBot, GoogleOther, Bytespider, and CCBot against robots.txt with per-bot pass/fail results
- Schema stacking analyzer detects Article + ItemList + FAQPage in JSON-LD, handling @type as string/array, @graph arrays, and multiple script blocks
- 17 unit tests covering all three analyzers passing

## Task Commits

Each task was committed atomically:

1. **Task 1: Add regex dependency and register geo module** - `9e1ab69` (chore)
2. **Task 2 RED: Failing tests for GEO analyzers** - `6d72da6` (test)
3. **Task 2 GREEN: Implement GEO analyzers** - `55a6eef` (feat)

## Files Created/Modified
- `cli/src/analyzers/geo.rs` - Three GEO analyzer functions with 17 unit tests
- `cli/Cargo.toml` - Added regex = "1.12" dependency
- `cli/src/analyzers/mod.rs` - Registered geo module

## Decisions Made
- Used static &str constants for AI bot check IDs instead of Box::leak -- avoids runtime allocation and is safer
- Comparison table detection requires th headers + 3 tr rows to filter noise from layout tables
- extract_types helper recursively handles @graph arrays for JSON-LD schema stacking detection

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- GEO analyzers ready to be wired into scoring and main.rs orchestration (Plan 03-02)
- All functions follow established analyzer pattern with AnalysisResult return types

---
*Phase: 03-geo-differentiators*
*Completed: 2026-03-23*
