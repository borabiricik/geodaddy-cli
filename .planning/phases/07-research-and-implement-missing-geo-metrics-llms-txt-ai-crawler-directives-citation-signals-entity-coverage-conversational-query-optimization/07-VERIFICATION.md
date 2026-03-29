---
phase: 07-research-and-implement-missing-geo-metrics
verified: 2026-03-29T20:15:00Z
status: passed
score: 14/14 must-haves verified
---

# Phase 7: Research and Implement Missing GEO Metrics Verification Report

**Phase Goal:** Add 18 new GEO-specific analyzers across 6 modules (llms.txt, AI directives, citation signals, entity coverage, conversational query optimization, freshness/HowTo) with severity-based scoring, expanding the geo category from 8 to 26 checks
**Verified:** 2026-03-29T20:15:00Z
**Status:** passed
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | severity_points() returns correct points for all 18 new geo check IDs | VERIFIED | scoring.rs lines 39-47: geo-llms-txt/geo-freshness => 10, 12 checks => 5, 4 checks => 2. Tests at lines 257-282 verify all 18 IDs. |
| 2 | analyze_llms_txt returns Fail when empty, Warn when missing H1, Pass when valid | VERIFIED | geo_llms.rs lines 13-47: three branches with correct status/check ID. 8 unit tests. |
| 3 | analyze_ai_meta_directives detects noai/noimageai/nosnippet in meta robots tags | VERIFIED | geo_directives.rs lines 12-45: CSS selector for meta[name="robots"], to_lowercase(), AI_DIRECTIVES const. 5 unit tests. |
| 4 | analyze_ai_header_directives detects noai/noimageai/nosnippet in X-Robots-Tag header | VERIFIED | geo_directives.rs lines 50-82: HeaderMap access, to_lowercase(), same AI_DIRECTIVES const. 5 unit tests. |
| 5 | 4 citation checks each return pass when signal present, warn when absent | VERIFIED | geo_citations.rs: analyze_citations returns Vec of 4 results with check IDs geo-citation-stats/sources/quotes/references. Regex + CSS selector detection. 13 unit tests. |
| 6 | geo-faq-quality check scores FAQ answers for 40-60 word optimal length | VERIFIED | geo_citations.rs lines 112-202: analyze_faq_quality parses FAQPage JSON-LD, counts words per answer, checks 40..=60 range. 3 unit tests. |
| 7 | 4 entity checks detect Person/Organization schema, about/mentions properties, proper nouns, and author bylines | VERIFIED | geo_entities.rs: analyze_entities returns Vec of 4 results. Uses shared extract_types from geo.rs (pub(crate)). 12 unit tests. |
| 8 | All check IDs use geo- prefix and kebab-case | VERIFIED | All 18 check IDs confirmed: geo-llms-txt, geo-ai-meta-directives, geo-ai-header-directives, geo-citation-stats/sources/quotes/references, geo-faq-quality, geo-entity-schema/about/proper-nouns/author, geo-query-qa-patterns/summary/snippet/faq, geo-freshness, geo-howto-schema. |
| 9 | 4 query optimization checks detect QA patterns, summaries, snippets, and FAQ sections | VERIFIED | geo_query.rs: analyze_query_optimization returns Vec of 4 results. Regex for question headings, CSS class/id selectors for summary, sibling walking for snippet, heading+JSON-LD for FAQ. 14 unit tests. |
| 10 | Freshness check detects dateModified in JSON-LD, Last-Modified header, and meta tag equivalents | VERIFIED | geo_freshness.rs lines 10-59: Three signal detection paths. Status::Fail when no signals (critical 10pts). 6 unit tests. |
| 11 | HowTo schema check validates presence and step structure | VERIFIED | geo_freshness.rs lines 80-133: find_howto_object walks @graph, checks "step" property. Pass with steps, Warn without. 4 unit tests. |
| 12 | llms.txt is fetched once before the page loop | VERIFIED | lib.rs line 103: fetch_llms_txt called once after robots.txt, before URL loop. Line 306: async fn fetch_llms_txt implementation. |
| 13 | HTTP response headers are captured before body consumption | VERIFIED | lib.rs lines 166-180: resp.headers().clone() on line 168 BEFORE resp.text().await on line 169. |
| 14 | All 18 new check IDs appear in analysis results (wired into pipeline) | VERIFIED | lib.rs lines 231-239: all 9 analyzer function calls (push/extend) adding 18 results per page. Imports on lines 28-33. |

**Score:** 14/14 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/analyzers/geo_llms.rs` | llms.txt presence and validation analyzer | VERIFIED | 122 lines, exports analyze_llms_txt, 8 unit tests |
| `src/analyzers/geo_directives.rs` | AI meta tag and X-Robots-Tag header directive analyzers | VERIFIED | 179 lines, exports analyze_ai_meta_directives + analyze_ai_header_directives, 10 unit tests |
| `src/analyzers/geo_citations.rs` | 4 citation signal checks + FAQ quality scoring | VERIFIED | 387 lines, exports analyze_citations + analyze_faq_quality, 16 unit tests |
| `src/analyzers/geo_entities.rs` | 4 entity coverage checks | VERIFIED | 283 lines, exports analyze_entities, uses shared extract_types, 12 unit tests |
| `src/analyzers/geo_query.rs` | 4 conversational query optimization checks | VERIFIED | 357 lines, exports analyze_query_optimization, 14 unit tests |
| `src/analyzers/geo_freshness.rs` | Freshness signal detection and HowTo schema validation | VERIFIED | 263 lines, exports analyze_freshness + analyze_howto_schema, 10 unit tests |
| `src/scoring.rs` | severity_points entries for all 18 new check IDs | VERIFIED | Lines 39-47: 3 severity tiers (10, 5, 2) covering all 18 IDs |
| `src/analyzers/mod.rs` | Module declarations for all 6 new modules | VERIFIED | All 6 pub mod declarations present |
| `src/lib.rs` | Wired integration of all Phase 7 analyzers | VERIFIED | Imports, fetch_llms_txt, header capture, all 9 analyzer calls |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| geo_llms.rs | scoring.rs | check ID geo-llms-txt matched in severity_points | WIRED | scoring.rs line 39: "geo-llms-txt" => 10 |
| geo_directives.rs | scoring.rs | check IDs geo-ai-meta-directives and geo-ai-header-directives | WIRED | scoring.rs lines 40-45: both in => 5 arm |
| geo_citations.rs | scoring.rs | check IDs geo-citation-* and geo-faq-quality | WIRED | scoring.rs lines 41-42 (5pts) and line 47 (2pts) |
| geo_entities.rs | scoring.rs | check IDs geo-entity-* | WIRED | scoring.rs lines 43, 46: schema/author at 5pts, about/proper-nouns at 2pts |
| geo_query.rs | scoring.rs | check IDs geo-query-* | WIRED | scoring.rs lines 44-45, 47: qa-patterns/summary/faq at 5pts, snippet at 2pts |
| geo_freshness.rs | scoring.rs | check IDs geo-freshness and geo-howto-schema | WIRED | scoring.rs line 39 (10pts) and line 45 (5pts) |
| lib.rs | geo_llms.rs | analyze_llms_txt call | WIRED | lib.rs line 231 |
| lib.rs | geo_directives.rs | analyze_ai_meta/header_directives calls | WIRED | lib.rs lines 232-233 |
| lib.rs | geo_citations.rs | analyze_citations + analyze_faq_quality calls | WIRED | lib.rs lines 234-235 |
| lib.rs | geo_entities.rs | analyze_entities call | WIRED | lib.rs line 236 |
| lib.rs | geo_query.rs | analyze_query_optimization call | WIRED | lib.rs line 237 |
| lib.rs | geo_freshness.rs | analyze_freshness + analyze_howto_schema calls | WIRED | lib.rs lines 238-239 |
| geo_entities.rs | geo.rs | shared extract_types helper | WIRED | geo.rs line 136: pub(crate) fn extract_types; geo_entities.rs line 1: use crate::analyzers::geo::extract_types |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| D-01 | 07-04 | All 5 named areas implemented | SATISFIED | 6 modules cover llms.txt, AI directives, citations, entities, query optimization, freshness |
| D-02 | 07-04 | All 5 v2 deferred requirements folded in | SATISFIED | GEO-04 via geo-faq-quality, GEO-05 via geo-query-summary, GEO-06 via geo-freshness, GEO-07 via 4 citation checks, GEO-08 via geo-howto-schema |
| D-03 | 07-01, 07-04 | All new checks use geo- prefix, route to geo category | SATISFIED | All 18 check IDs use geo- prefix |
| D-04 | 07-01 | Severity model: critical 10pts, warning 5pts, info 2pts | SATISFIED | scoring.rs lines 39-47 |
| D-05 | 07-01 | Check /llms.txt presence + basic validation | SATISFIED | geo_llms.rs: presence, H1, minimum length |
| D-06 | 07-01 | llms.txt absence is critical severity (10pts) | SATISFIED | scoring.rs line 39: "geo-llms-txt" => 10 |
| D-07 | 07-01 | Check AI-specific meta tags AND X-Robots-Tag headers | SATISFIED | geo_directives.rs: both functions implemented |
| D-08 | 07-01 | Separate check IDs from existing geo-ai-bot-* | SATISFIED | geo-ai-meta-directives and geo-ai-header-directives are distinct from geo-ai-bot-* |
| D-09 | 07-02 | Four separate citation checks | SATISFIED | geo_citations.rs: stats, sources, quotes, references |
| D-10 | 07-02 | Threshold: at least 1 signal per check type | SATISFIED | Each citation check: pass if present, warn if absent |
| D-11 | 07-02 | Four entity coverage checks | SATISFIED | geo_entities.rs: schema, about, proper-nouns, author |
| D-12 | 07-03 | Four query optimization checks | SATISFIED | geo_query.rs: qa-patterns, summary, snippet, faq |
| D-13 | 07-02 | GEO-04: FAQ quality scoring 40-60 word range | SATISFIED | geo_citations.rs analyze_faq_quality |
| D-14 | 07-03 | GEO-05: Quick answer block detection | SATISFIED | geo_query.rs check_summary (TL;DR, key takeaways) |
| D-15 | 07-03, 07-04 | GEO-06: Freshness signals | SATISFIED | geo_freshness.rs: dateModified, Last-Modified, meta tags |
| D-16 | 07-02 | GEO-07: Citation density | SATISFIED | Covered by 4 citation signal checks |
| D-17 | 07-03 | GEO-08: HowTo schema validation | SATISFIED | geo_freshness.rs analyze_howto_schema |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| None | - | - | - | No TODO, FIXME, placeholder, or stub patterns found in any Phase 7 files |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Full lib test suite passes | `cargo test --lib` | 189 passed, 0 failed | PASS |
| Release build succeeds | `cargo build --release` | Finished release in 6.52s | PASS |
| All 18 severity entries present | grep scoring.rs for geo- check IDs | 18 distinct IDs across 3 severity tiers | PASS |

### Human Verification Required

### 1. Live URL Analysis Output

**Test:** Run `cargo run -- https://example.com` and inspect JSON output for all 26 geo-prefixed check IDs
**Expected:** Output contains all 18 new checks plus 8 existing geo checks (26 total)
**Why human:** Requires network access to a live URL and manual JSON inspection

### 2. Score Impact Validation

**Test:** Compare overall and geo category scores before and after Phase 7 on a real site
**Expected:** Scores reflect new checks without disproportionate penalty from heuristic checks (D-04 severity model)
**Why human:** Requires subjective judgment on whether score changes are reasonable

---

_Verified: 2026-03-29T20:15:00Z_
_Verifier: Claude (gsd-verifier)_
