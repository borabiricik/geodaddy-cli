# Phase 7: Research and Implement Missing GEO Metrics - Context

**Gathered:** 2026-03-29
**Status:** Ready for planning

<domain>
## Phase Boundary

Add comprehensive GEO-specific analyzers covering five metric areas: llms.txt support, expanded AI crawler directives (meta tags + HTTP headers), citation signal detection, entity coverage analysis, and conversational query optimization. Also folds in all v2 deferred GEO requirements (GEO-04 through GEO-08: FAQ quality, quick answer blocks, freshness signals, citation density, HowTo schema).

All new checks route to the existing `geo` scoring category. No new scoring categories or sub-categories are introduced.

</domain>

<decisions>
## Implementation Decisions

### Metric Scope & Priority
- **D-01:** All 5 named areas are implemented: llms.txt, AI crawler directives expansion, citation signals, entity coverage, conversational query optimization.
- **D-02:** All 5 v2 deferred requirements folded in: GEO-04 (FAQ quality scoring), GEO-05 (quick answer block detection), GEO-06 (content freshness signals), GEO-07 (citation/statistic density), GEO-08 (HowTo schema validation).
- **D-03:** All new checks use `geo-` prefix and route to the existing geo scoring category. Overall score formula unchanged (3-way or 4-way average depending on --vitals).
- **D-04:** Severity model: clear-cut detections (llms.txt missing, freshness signals absent) can be critical (10pts). Heuristic-heavy checks (entity coverage, conversational optimization) use warning (5pts) or info (2pts) severity to avoid false-positive score tanking.

### llms.txt & AI Directives
- **D-05:** Check `/llms.txt` presence + basic validation. Fetch the file, report pass if exists with non-empty content of reasonable length. Warn/fail if missing. Don't deeply parse internal format since the spec is still evolving.
- **D-06:** llms.txt absence is **critical severity (10pts)** — strong stance that AI-readiness requires this file.
- **D-07:** Check AI-specific meta tags in HTML (e.g., `<meta name="robots" content="noai">`, Google's AI-specific directives) AND `X-Robots-Tag` HTTP headers with AI crawler values. Both directive mechanisms are covered.
- **D-08:** These expand the existing AI bot audit (Phase 3) — separate check IDs, not modifications to existing `geo-ai-bot-*` checks.

### Citation Signal Detection
- **D-09:** Four separate checks, each pass/fail independently:
  - `geo-citation-stats` — Statistics with numbers (e.g., "74.2% of...", "3 out of 4...", "$1.2 million")
  - `geo-citation-sources` — Source attributions (e.g., "according to [Source]", "a study by [X] found")
  - `geo-citation-quotes` — Blockquotes (`<blockquote>`) and quotation patterns ("As [Person] said...")
  - `geo-citation-references` — Reference/bibliography sections (headings like "References", "Sources" followed by links/citations)
- **D-10:** Threshold: at least 1 signal per page for each check type. Pass if present, warn if absent.

### Entity Coverage
- **D-11:** Separate checks for entity coverage:
  - `geo-entity-schema` — Person and Organization JSON-LD schema types present
  - `geo-entity-about` — Check for `about` and `mentions` properties in JSON-LD that link content to entities
  - `geo-entity-proper-nouns` — Detect proper noun density / named entities in text content
  - `geo-entity-author` — Author byline detection ("by [Name]", author meta tags, Person schema linked to article)

### Conversational Query Optimization
- **D-12:** Separate checks for query optimization:
  - `geo-query-qa-patterns` — Question-then-answer structures: H2/H3 headings phrased as questions followed by direct answers
  - `geo-query-summary` — TL;DR sections, key takeaways blocks, above-fold summary content
  - `geo-query-snippet` — Featured snippet formatting: definition paragraphs after question headings, concise 40-60 word answer blocks
  - `geo-query-faq` — Dedicated FAQ sections in content structure (with or without FAQPage schema)

### v2 Deferred Requirements (Folded In)
- **D-13:** GEO-04 (FAQ quality): Score FAQ answers for optimal 40-60 word length.
- **D-14:** GEO-05 (quick answer blocks): Covered by `geo-query-summary` check (D-12).
- **D-15:** GEO-06 (freshness signals): Check `dateModified` in JSON-LD schema + `Last-Modified` HTTP header. Check `<meta>` last-modified equivalents.
- **D-16:** GEO-07 (citation density): Covered by the 4 citation signal checks (D-09).
- **D-17:** GEO-08 (HowTo schema): Validate HowTo JSON-LD schema type presence and structure.

### Claude's Discretion
- Exact regex patterns for citation signal detection (statistics, source attributions, quotation patterns)
- Proper noun detection heuristics (capitalization-based vs. more sophisticated approaches)
- Featured snippet formatting validation details (word count thresholds, paragraph structure)
- FAQ section detection without schema (heading pattern matching)
- llms.txt content validation rules (what constitutes "reasonable length", basic structure checks)
- Severity assignment for individual checks within the D-04 guidelines
- How to efficiently extract HTTP headers (X-Robots-Tag) — whether to use existing reqwest response or make separate HEAD request
- GEO-04 FAQ quality scoring granularity (per-answer vs. aggregate)
- GEO-08 HowTo schema validation depth (presence-only vs. structural completeness)
- Implementation of GEO-06 freshness signals check ID naming

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing GEO Analyzers
- `src/analyzers/geo.rs` — Current GEO checks (listicle, AI bot audit, schema stacking). New checks extend this file or add new files in analyzers/.
- `src/scoring.rs` — `severity_points()` function needs new check ID entries. `calculate_score()` routes `geo-` prefix to geo category (no changes needed to routing logic).

### Phase 3 Context (GEO patterns to follow)
- `.planning/phases/03-geo-differentiators/03-CONTEXT.md` — D-09 through D-22: established GEO analyzer patterns, severity model, JSON-LD parsing approach, code architecture

### Project Requirements
- `.planning/REQUIREMENTS.md` — GEO-04 through GEO-08 (v2 deferred, now folded in), CRED-01 (author byline, partially addressed by D-11 entity-author)
- `.planning/ROADMAP.md` — Phase 7 goal and dependencies

### Tech Stack
- `CLAUDE.md` — Tech stack. Phase 7 uses existing deps: `scraper` (HTML parsing), `robotstxt` (robots.txt), `serde_json` (JSON-LD), `regex` (pattern matching), `reqwest` (HTTP for llms.txt fetch + headers).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `analyzers/geo.rs` — `extract_types()` helper for JSON-LD type extraction (reuse for HowTo, Person, Organization detection)
- `analyzers/geo.rs` — `AI_BOTS` constant pattern for defining check sets (reuse for citation signal types, entity types)
- `analyzers/geo.rs` — `analyze_ai_bots()` takes `robots_body: &str` pattern — llms.txt can follow similar "fetch body elsewhere, pass string to analyzer" approach
- `analyzers/content.rs` — JSON-LD parsing already exists in `check_json_ld()` — reuse patterns for entity schema detection
- `scoring.rs` — `severity_points()` match block with `geo-` prefix wildcard catch — new check IDs need explicit entries

### Established Patterns
- All analyzer functions return `AnalysisResult` or `Vec<AnalysisResult>` (for multi-result checks like AI bot audit)
- Check IDs use kebab-case with category prefix: `geo-listicle`, `geo-ai-bot-gptbot`
- HTML analysis via `scraper::Html` + CSS selectors
- JSON-LD analysis via `serde_json::Value` parsing of `<script type="application/ld+json">` blocks
- robots.txt body passed as `&str` (fetched in main.rs, passed to analyzers)

### Integration Points
- `main.rs` — needs to fetch `/llms.txt` (new HTTP request) and pass HTTP response headers to directive analyzers
- `analyzers/mod.rs` — may need new sub-modules if geo.rs gets too large (e.g., `geo_citations.rs`, `geo_entities.rs`)
- `scoring.rs` — `severity_points()` needs ~15+ new check ID entries

</code_context>

<specifics>
## Specific Ideas

- llms.txt gets critical severity (10pts) — user wants a strong stance on AI-readiness
- All 4 citation signal types get their own check for granular recommendations
- Entity and query optimization checks are also separate (not combined) — consistent with citation approach
- This phase produces ~15-20 new check IDs, significantly expanding the geo category
- Author byline detection (geo-entity-author) partially addresses deferred CRED-01 requirement

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 07-research-and-implement-missing-geo-metrics-llms-txt-ai-crawler-directives-citation-signals-entity-coverage-conversational-query-optimization*
*Context gathered: 2026-03-29*
