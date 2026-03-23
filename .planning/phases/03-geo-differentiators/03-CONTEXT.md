# Phase 3: GEO Differentiators - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Add three GEO-specific analyzers (listicle detection, AI bot robots.txt audit, triple schema stacking) and integrate a new GEO scoring category into the existing severity-weighted scoring system. These features distinguish geodaddy from traditional SEO tools.

Requirements: GEO-01, GEO-02, GEO-03

</domain>

<decisions>
## Implementation Decisions

### Scoring Integration
- **D-09:** Overall score becomes equal thirds: `(tech + content + geo) / 3`. Extends current 2-way average to 3-way.
- **D-10:** `CategoryScores` struct gains a `geo: f64` field. Always present in JSON output (even when score is 100.0) for consistent schema.
- **D-11:** GEO check severity assignments (mixed):

  | Check ID | Requirement | Severity | Points |
  |----------|-------------|----------|--------|
  | geo-ai-bot-{name} | GEO-02 | critical | 10 |
  | geo-listicle | GEO-01 | warning | 5 |
  | geo-schema-stacking | GEO-03 | warning | 5 |

  Note: AI bot audit emits one result PER bot (6 results), each critical. This means AI bot blocking has heavy score impact — intentional, as blocking AI crawlers is the most actionable GEO issue.

### Listicle Detection (GEO-01)
- **D-12:** Broad detection. Patterns to detect:
  - "Top N" / "Best N" / "N Best" heading patterns (regex on H1-H3)
  - Ordered lists (`<ol>` elements)
  - Numbered heading sequences ("1. ...", "2. ..." in H2/H3 tags)
  - Comparison tables (`<table>` with structured data patterns)
- **D-13:** When NO listicle detected: status `warn` with suggestion ("Consider restructuring as numbered list or 'Top N' format — 74.2% of AI citations come from listicle-style content").
- **D-14:** When listicle IS detected: status `pass` with specific type found ("Listicle format detected: 'Top 10' heading pattern with ordered list").

### AI Bot Audit (GEO-02)
- **D-15:** Extended bot list — 6 bots:
  - GPTBot (OpenAI / ChatGPT)
  - ClaudeBot (Anthropic / Claude)
  - PerplexityBot (Perplexity)
  - GoogleOther (Google AI)
  - Bytespider (ByteDance)
  - CCBot (Common Crawl, used by many AI systems)
- **D-16:** Per-bot results. Each bot gets its own `AnalysisResult` with check ID `geo-ai-bot-{botname}` (e.g., `geo-ai-bot-gptbot`). This means 6 results from this analyzer.
- **D-17:** Each blocked bot = `fail` status. Each allowed bot = `pass`. Message includes bot name and which AI service it serves (e.g., "GPTBot is blocked in robots.txt. This prevents your content from appearing in ChatGPT search results.").
- **D-18:** Uses the existing `robotstxt` crate already in dependencies. Fetches robots.txt once, checks each bot's user-agent string against it.

### Schema Stacking (GEO-03)
- **D-19:** Partial stacking reported:
  - `pass` = all 3 present (Article + ItemList + FAQPage)
  - `warn` = 1-2 of 3 present (message lists which are found and which are missing)
  - `fail` = none of the 3 schema types present
- **D-20:** JSON-LD only. Parse `<script type="application/ld+json">` blocks. Do not scan for Microdata or RDFa. Consistent with existing CONT-02 JSON-LD check.

### Code Architecture
- **D-21:** New file `cli/src/analyzers/geo.rs` following flat module pattern (D-07). Three public functions:
  - `analyze_listicle(html: &Html) -> AnalysisResult`
  - `analyze_ai_bots(robots_body: &str) -> Vec<AnalysisResult>` (returns 6 results)
  - `analyze_schema_stacking(html: &Html) -> AnalysisResult`
- **D-22:** AI bot analyzer takes robots.txt body as `&str` rather than fetching itself. `main.rs` already fetches robots.txt in `check_robots()` — reuse that fetch, pass the body string to both `check_robots` logic and `analyze_ai_bots`.

### Claude's Discretion
- Exact regex patterns for listicle heading detection (e.g., how to match "Top N" vs "Top Picks")
- robots.txt parsing edge cases for AI bot detection (e.g., wildcard rules, multiple user-agent blocks)
- Schema type matching logic in JSON-LD (handling `@type` as string vs array)
- Comparison table detection heuristics (what makes a table "structured" enough to count as listicle-adjacent)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Technology Stack
- `CLAUDE.md` — Full tech stack. Phase 3 uses existing deps: `scraper` (HTML parsing), `robotstxt` (robots.txt parsing), `serde_json` (JSON-LD parsing). No new dependencies expected.

### Phase 2 Foundation
- `cli/src/scoring.rs` — `AnalysisResult`, `Status`, `CategoryScores`, `severity_points()`, `calculate_score()`. Must be modified to add GEO category.
- `cli/src/main.rs` — Orchestration, `check_robots()` function, `PageResult` struct. Must be modified to call GEO analyzers and pass robots.txt body.
- `cli/src/analyzers/mod.rs` — Module re-exports. Must add `pub mod geo;`.
- `cli/src/analyzers/content.rs` — `analyze_json_ld()` as reference for JSON-LD parsing pattern.
- `cli/src/analyzers/technical.rs` — Reference for analyzer function signatures and patterns.

### Requirements
- `.planning/REQUIREMENTS.md` — GEO-01, GEO-02, GEO-03 full descriptions

### Prior Context
- `.planning/phases/02-core-analysis-engine/02-CONTEXT.md` — D-05 (AnalysisResult shape), D-06 (scoring formula), D-07 (flat module structure), D-08 (broken links stub pattern)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `robotstxt` crate already in dependencies — `DefaultMatcher::one_agent_allowed_by_robots()` can check each AI bot user-agent
- `check_robots()` in `main.rs` already fetches and parses robots.txt body — refactor to expose the raw body string for reuse by GEO analyzer
- `analyze_json_ld()` in `content.rs` already parses `<script type="application/ld+json">` — schema stacking can follow the same selector pattern
- `scraper::Selector` for heading/list/table detection already used throughout technical.rs and content.rs

### Established Patterns
- Analyzer functions return `AnalysisResult` or `Vec<AnalysisResult>` — GEO analyzers follow this
- Check IDs use `{category}-{name}` format — GEO uses `geo-*` prefix
- `severity_points()` match arms in scoring.rs — add new `geo-*` entries
- All analyzers called sequentially in `main.rs` — GEO analyzers added to the sequence

### Integration Points
- `scoring.rs`: Add `geo: f64` to `CategoryScores`, update `severity_points()` with GEO check IDs, update `calculate_score()` for 3-way average
- `main.rs`: Refactor `check_robots()` to return robots body string, add GEO analyzer calls, import geo module
- `analyzers/mod.rs`: Add `pub mod geo;`

</code_context>

<specifics>
## Specific Ideas

- AI bot audit emitting per-bot results (6 items) is a deliberate choice — it gives granular visibility into which AI services can/can't access the content, and each blocked bot individually impacts the score
- The robots.txt body fetch should happen once and be shared between the existing `check_robots()` logic and the new `analyze_ai_bots()` — avoid duplicate HTTP requests
- Listicle pass messages should be descriptive about WHAT was found ("Top 10 heading pattern with ordered list") so users understand what's working

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 03-geo-differentiators*
*Context gathered: 2026-03-23*
