# Phase 7: Research and Implement Missing GEO Metrics - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-29
**Phase:** 07-research-and-implement-missing-geo-metrics
**Areas discussed:** Metric scope & priority, llms.txt & AI directives, Citation signal detection, Entity & query optimization

---

## Metric Scope & Priority

### Ambition Level

| Option | Description | Selected |
|--------|-------------|----------|
| All 5 areas (Recommended) | Implement all named areas: llms.txt, AI directives expansion, citation signals, entity coverage, conversational query optimization | ✓ |
| Core 3 only | llms.txt + citation signals + entity coverage. Defer conversational query optimization and AI directives expansion | |
| Research-first | Research all 5, only implement what has clear detection heuristics | |

**User's choice:** All 5 areas
**Notes:** User wants geodaddy to be a comprehensive GEO tool.

### v2 Deferred Requirements

| Option | Description | Selected |
|--------|-------------|----------|
| Pull in relevant ones (Recommended) | Fold in GEO-04, GEO-06, GEO-07 — overlap with Phase 7 scope | |
| Pull in all 5 | Fold in GEO-04 through GEO-08 entirely. Clears v2 GEO backlog | ✓ |
| Keep them deferred | Phase 7 focuses on new metrics only | |

**User's choice:** Pull in all 5
**Notes:** Big phase — clears entire v2 GEO backlog.

### Scoring Category

| Option | Description | Selected |
|--------|-------------|----------|
| Keep geo as one category (Recommended) | All new checks use geo- prefix, route to existing geo category | ✓ |
| Split into sub-categories | Break geo into sub-scores (geo-ai-readiness, geo-content-signals, geo-entity) | |
| You decide | Claude picks | |

**User's choice:** Keep geo as one category

### Severity Model

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, use info/warning (Recommended) | Subjective checks get warning/info severity. Clear-cut checks can be critical | ✓ |
| Same severity model as existing | Apply severity based on GEO impact regardless of detection confidence | |
| You decide | Claude assigns based on both factors | |

**User's choice:** Yes, use info/warning for heuristic-heavy checks

---

## llms.txt & AI Directives

### llms.txt Detection

| Option | Description | Selected |
|--------|-------------|----------|
| Check presence + basic validation (Recommended) | Fetch /llms.txt, validate non-empty + reasonable length. Don't deeply parse format | ✓ |
| Check presence only | Just check HTTP 200. No content validation | |
| Deep validation | Parse internal structure per draft spec | |

**User's choice:** Check presence + basic validation

### Additional AI Directives

| Option | Description | Selected |
|--------|-------------|----------|
| Meta tags + HTTP headers (Recommended) | Check AI-specific meta tags AND X-Robots-Tag HTTP headers | ✓ |
| Meta tags only | HTML meta tags only, skip HTTP headers | |
| You decide | Claude researches current standards | |

**User's choice:** Meta tags + HTTP headers

### llms.txt Severity

| Option | Description | Selected |
|--------|-------------|----------|
| Warning (5pts) (Recommended) | Missed opportunity, actionable but not critical | |
| Info (2pts) | Light touch, standard is too new | |
| Critical (10pts) | Strong stance: llms.txt is essential for GEO | ✓ |

**User's choice:** Critical (10pts)
**Notes:** User takes a strong stance on AI-readiness — llms.txt absence is a significant issue.

---

## Citation Signal Detection

### Signal Types

| Option | Description | Selected |
|--------|-------------|----------|
| Statistics with numbers | Numeric data patterns (percentages, dollar amounts, ratios) | ✓ |
| Source attributions | "according to [Source]", "a study by [X] found" patterns | ✓ |
| Blockquotes & expert quotes | `<blockquote>` elements and quotation patterns | ✓ |
| Reference/bibliography sections | Headings like "References", "Sources" followed by links | ✓ |

**User's choice:** All four signal types (multi-select)

### Threshold

| Option | Description | Selected |
|--------|-------------|----------|
| At least 1 signal per page (Recommended) | Pass if present, warn if absent. Low bar catches zero-citation pages | ✓ |
| Density ratio | Signals-per-1000-words with tunable threshold | |
| You decide | Claude determines threshold from research | |

**User's choice:** At least 1 signal per page

### Granularity

| Option | Description | Selected |
|--------|-------------|----------|
| One combined check (Recommended) | Single geo-citation-signals check | |
| Separate per type | Four checks: geo-citation-stats, -sources, -quotes, -references | ✓ |
| You decide | Claude picks based on existing patterns | |

**User's choice:** Separate per type
**Notes:** More granular recommendations, consistent with the approach chosen for entity/query checks later.

---

## Entity & Query Optimization

### Entity Coverage Checks

| Option | Description | Selected |
|--------|-------------|----------|
| Person/Organization schema | JSON-LD Person and Organization types | ✓ |
| About/mentions schema properties | `about` and `mentions` JSON-LD properties | ✓ |
| Proper noun density | Named entity detection in text content | ✓ |
| Author byline detection | "by [Name]", author meta tags, Person schema | ✓ |

**User's choice:** All four (multi-select)

### Conversational Query Optimization Checks

| Option | Description | Selected |
|--------|-------------|----------|
| Q&A patterns in content | Question headings followed by direct answers | ✓ |
| TL;DR / summary blocks | Above-fold summaries, key takeaways | ✓ |
| Featured snippet formatting | Definition paragraphs, concise answer blocks | ✓ |
| FAQ section detection | Dedicated FAQ sections in content | ✓ |

**User's choice:** All four (multi-select)

### Grouping

| Option | Description | Selected |
|--------|-------------|----------|
| Separate checks each (Recommended) | Individual check IDs for each signal type | ✓ |
| Two combined checks | geo-entity-coverage + geo-query-optimization | |
| You decide | Claude groups logically | |

**User's choice:** Separate checks each

### v2 Requirements Details

| Option | Description | Selected |
|--------|-------------|----------|
| You decide details | Claude implements GEO-04 through GEO-08 consistently with above decisions | ✓ |
| Discuss each one | Go through each v2 requirement individually | |

**User's choice:** You decide details

---

## Claude's Discretion

- Exact regex patterns for all detection heuristics
- Proper noun detection approach
- Featured snippet formatting thresholds
- FAQ section detection without schema
- llms.txt content validation rules
- Individual check severity assignments within the D-04 guidelines
- HTTP header extraction efficiency
- v2 requirement implementation details (GEO-04, GEO-05, GEO-06, GEO-08)

## Deferred Ideas

None — discussion stayed within phase scope.
