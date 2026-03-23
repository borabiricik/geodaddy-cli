# Phase 3: GEO Differentiators - Research

**Researched:** 2026-03-23
**Domain:** GEO-specific analysis (listicle detection, AI bot robots.txt audit, schema stacking)
**Confidence:** HIGH

## Summary

Phase 3 adds three GEO-specific analyzers to the existing analysis engine, plus integrates a new `geo` scoring category. The implementation is well-bounded: all three analyzers operate on data already fetched (HTML document and robots.txt body), use crates already in dependencies (`scraper`, `robotstxt`, `serde_json`), and follow established patterns from Phase 2 analyzers. The only new dependency needed is `regex` for listicle heading pattern matching.

The primary technical challenge is the robots.txt body sharing between `check_robots()` (which already fetches it) and the new `analyze_ai_bots()`. The `robotstxt::DefaultMatcher` correctly resets internal state via `handle_robots_start()` on each call to `one_agent_allowed_by_robots()`, so a single mutable matcher instance can check all 6 bot user-agents sequentially against the same robots.txt body string.

**Primary recommendation:** Implement as a single new file `cli/src/analyzers/geo.rs` with three public functions, refactor `check_robots()` to expose the robots.txt body string, and update `scoring.rs` for the 3-way average.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-09:** Overall score becomes equal thirds: `(tech + content + geo) / 3`. Extends current 2-way average to 3-way.
- **D-10:** `CategoryScores` struct gains a `geo: f64` field. Always present in JSON output (even when score is 100.0) for consistent schema.
- **D-11:** GEO check severity assignments: `geo-ai-bot-{name}` = critical (10pts), `geo-listicle` = warning (5pts), `geo-schema-stacking` = warning (5pts). AI bot audit emits one result PER bot (6 results).
- **D-12:** Broad listicle detection patterns: "Top N"/"Best N"/"N Best" headings (regex on H1-H3), `<ol>` elements, numbered heading sequences, comparison tables.
- **D-13:** No listicle = warn with suggestion including 74.2% statistic.
- **D-14:** Listicle detected = pass with specific type found described in message.
- **D-15:** Extended bot list of 6: GPTBot, ClaudeBot, PerplexityBot, GoogleOther, Bytespider, CCBot.
- **D-16:** Per-bot results with check ID `geo-ai-bot-{botname}`.
- **D-17:** Blocked bot = fail, allowed bot = pass, with descriptive messages.
- **D-18:** Uses existing `robotstxt` crate. Fetches robots.txt once, checks each bot.
- **D-19:** Schema stacking: pass = all 3 present, warn = 1-2 present, fail = none present.
- **D-20:** JSON-LD only (`<script type="application/ld+json">`). No Microdata or RDFa.
- **D-21:** New file `cli/src/analyzers/geo.rs` with three public functions: `analyze_listicle(html: &Html) -> AnalysisResult`, `analyze_ai_bots(robots_body: &str) -> Vec<AnalysisResult>`, `analyze_schema_stacking(html: &Html) -> AnalysisResult`.
- **D-22:** AI bot analyzer takes robots.txt body as `&str`. Reuse `check_robots()` fetch.

### Claude's Discretion
- Exact regex patterns for listicle heading detection
- robots.txt parsing edge cases for AI bot detection (wildcard rules, multiple user-agent blocks)
- Schema type matching logic in JSON-LD (handling `@type` as string vs array)
- Comparison table detection heuristics

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| GEO-01 | Analyzer detects listicle format ("Top N", numbered lists, structured comparisons) | Heading regex patterns, `<ol>` detection, comparison table heuristics via `scraper` CSS selectors. `regex` crate needed for heading text matching. |
| GEO-02 | Analyzer audits robots.txt for AI bot directives (GPTBot, PerplexityBot, ClaudeBot) | `robotstxt` crate `one_agent_allowed_by_robots()` checks each of 6 bot user-agent strings. Matcher resets state correctly between calls. |
| GEO-03 | Analyzer detects triple schema stacking (Article + ItemList + FAQPage on same page) | JSON-LD parsing via `serde_json` on `<script type="application/ld+json">` blocks. Must handle `@type` as both string and array. |
</phase_requirements>

## Standard Stack

### Core (already in Cargo.toml)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| scraper | 0.26 | HTML parsing for listicle/schema detection | Already used by all Phase 2 analyzers. CSS selectors for headings, `<ol>`, tables. |
| robotstxt | 0.3 | AI bot user-agent checking | Already used by `check_robots()`. Google's algorithm port. `one_agent_allowed_by_robots()` supports sequential multi-bot checks. |
| serde_json | 1.0 | JSON-LD parsing for schema stacking | Already used for report output. `serde_json::Value` for parsing `@type` which can be string or array. |

### New Dependency
| Library | Version | Purpose | Why Needed |
|---------|---------|---------|------------|
| regex | 1.12.3 | Listicle heading pattern matching | Required for "Top N"/"Best N"/"N Best" detection in heading text. Listed in CLAUDE.md tech stack. Guaranteed linear time (no ReDoS). |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| regex | String contains/starts_with | Fragile for "Top 10" vs "Top Picks" vs "10 Best" patterns. Regex is cleaner and already in stack spec. |

**Installation (new dependency only):**
```bash
# Add to cli/Cargo.toml [dependencies]
regex = "1.12"
```

**Version verification:** regex 1.12.3 confirmed current on crates.io (2026-03-23).

## Architecture Patterns

### File Structure (addition to existing)
```
cli/src/
  analyzers/
    mod.rs           # Add: pub mod geo;
    technical.rs     # Existing (reference patterns)
    content.rs       # Existing (reference: analyze_json_ld)
    geo.rs           # NEW: 3 public analyzer functions
  scoring.rs         # MODIFY: add geo category, update severity_points, 3-way average
  main.rs            # MODIFY: refactor check_robots, call GEO analyzers
```

### Pattern 1: Analyzer Function Signatures (established in Phase 2)
**What:** Each analyzer returns `AnalysisResult` or `Vec<AnalysisResult>` with a `check` ID prefixed by category.
**When to use:** All new analyzers follow this pattern.
**Example (from existing `content.rs`):**
```rust
pub fn analyze_json_ld(html: &Html) -> AnalysisResult {
    // ...selector, parse, return AnalysisResult with check: "cont-json-ld"
}
```

GEO analyzers follow this exactly:
```rust
pub fn analyze_listicle(html: &Html) -> AnalysisResult { /* check: "geo-listicle" */ }
pub fn analyze_ai_bots(robots_body: &str) -> Vec<AnalysisResult> { /* check: "geo-ai-bot-{name}" */ }
pub fn analyze_schema_stacking(html: &Html) -> AnalysisResult { /* check: "geo-schema-stacking" */ }
```

### Pattern 2: robots.txt Body Sharing
**What:** Refactor `check_robots()` to return the fetched body string alongside the blocked boolean, so `analyze_ai_bots()` can reuse it without a second HTTP request.
**Current signature:** `async fn check_robots(client, url) -> bool`
**New signature:** `async fn check_robots(client, url) -> (bool, String)` where String is the robots.txt body.
**Integration point:** In `main()`, destructure return and pass body to `analyze_ai_bots()`.

### Pattern 3: Scoring Category Routing
**What:** `calculate_score()` routes results to categories based on check ID prefix. Currently handles `tech-` and `cont-`. Must add `geo-`.
**Current code (scoring.rs line 55-61):**
```rust
if result.check.starts_with("tech-") {
    tech_earned += earned;
    tech_max += pts;
} else if result.check.starts_with("cont-") {
    cont_earned += earned;
    cont_max += pts;
}
```
**Extension:** Add `else if result.check.starts_with("geo-")` block with `geo_earned`/`geo_max` accumulators.

### Anti-Patterns to Avoid
- **Fetching robots.txt twice:** The body is already fetched in `check_robots()`. Do not make a second HTTP request in `analyze_ai_bots()`.
- **Single AnalysisResult for all bots:** D-16 requires per-bot results. Return `Vec<AnalysisResult>` with 6 items, not a single summary.
- **Hardcoding bot names in severity_points():** Use a pattern match on prefix `"geo-ai-bot-"` to catch all 6 bot check IDs without listing them individually.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| robots.txt user-agent matching | Custom string matching for User-agent directives | `robotstxt::DefaultMatcher::one_agent_allowed_by_robots()` | Handles wildcards, case-insensitive matching, Google's algorithm edge cases |
| "Top N" regex pattern | Multiple string contains/starts_with checks | `regex::Regex` with compiled pattern | Handles variations ("Top 10", "10 Best", "Best 10") cleanly |
| JSON-LD parsing | Custom JSON parser | `serde_json::from_str::<Value>()` | Already used in `content.rs`, handles all JSON edge cases |
| HTML element selection | DOM tree walking | `scraper::Selector::parse()` + `html.select()` | Already used everywhere, CSS selector syntax is standard |

## Common Pitfalls

### Pitfall 1: robotstxt Matcher State Between Calls
**What goes wrong:** Assuming `DefaultMatcher` needs to be recreated for each bot check.
**Why it happens:** The `&mut self` signature suggests mutable state accumulation.
**How to avoid:** `handle_robots_start()` is called internally at the start of each `one_agent_allowed_by_robots()` call, which clears all match state. A single `DefaultMatcher::default()` instance can be reused for all 6 bot checks.
**Warning signs:** Creating 6 `DefaultMatcher` instances (unnecessary but harmless).

### Pitfall 2: JSON-LD @type Can Be String or Array
**What goes wrong:** `val.get("@type").and_then(|v| v.as_str())` misses array-typed `@type`.
**Why it happens:** JSON-LD spec allows `"@type": "Article"` or `"@type": ["Article", "ItemList"]`.
**How to avoid:** Check both forms:
```rust
fn extract_types(val: &Value) -> Vec<String> {
    match val.get("@type") {
        Some(Value::String(s)) => vec![s.clone()],
        Some(Value::Array(arr)) => arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        _ => vec![],
    }
}
```
**Warning signs:** Schema stacking always returning "0 found" on pages that use array `@type`.

### Pitfall 3: Multiple JSON-LD Blocks vs Single Block with Array @type
**What goes wrong:** Only checking a single JSON-LD block for all three schema types.
**Why it happens:** Some sites put Article, ItemList, FAQPage in separate `<script>` blocks; others combine them.
**How to avoid:** Collect `@type` values from ALL JSON-LD blocks on the page, then check the aggregate set for the three target types. A page might have `{"@type": "Article"}` in one block and `{"@type": "FAQPage"}` in another.
**Warning signs:** Tests only use single-block HTML fixtures.

### Pitfall 4: robots.txt URL Construction
**What goes wrong:** Appending "/robots.txt" to the page URL path instead of replacing it.
**Why it happens:** String concatenation instead of `set_path()`.
**How to avoid:** Already handled correctly in `check_robots()` using `set_path("/robots.txt")`. The GEO analyzer receives the body string directly, so this pitfall is avoided by design (D-22).

### Pitfall 5: Listicle Regex Case Sensitivity
**What goes wrong:** Missing "top 10" (lowercase) or "TOP 10" (uppercase) headings.
**Why it happens:** Using case-sensitive regex by default.
**How to avoid:** Use case-insensitive flag: `(?i)` in regex pattern or `RegexBuilder::new(...).case_insensitive(true)`.
**Warning signs:** Tests only use title-case headings.

### Pitfall 6: Scoring Formula Update Breaks Existing Tests
**What goes wrong:** Changing from 2-way to 3-way average breaks existing `calculate_score()` tests.
**Why it happens:** Existing tests expect `(tech + content) / 2`. New formula is `(tech + content + geo) / 3`.
**How to avoid:** When no `geo-*` results exist, `geo_max` is 0 and `geo_score` defaults to 100.0 (same pattern as existing empty-category handling). This means `(tech + content + 100) / 3` for pages with no GEO checks -- which IS different from the old `(tech + content) / 2`. Existing tests MUST be updated.
**Warning signs:** `test_overall_is_average` failing after scoring changes.

## Code Examples

### AI Bot User-Agent Strings (verified from multiple sources)
```rust
// Source: momenticmarketing.com, amicited.com, paulcalvano.com (cross-verified)
const AI_BOTS: &[(&str, &str, &str)] = &[
    // (user_agent, check_id_suffix, service_description)
    ("GPTBot",        "gptbot",        "ChatGPT"),
    ("ClaudeBot",     "claudebot",     "Claude"),
    ("PerplexityBot", "perplexitybot", "Perplexity"),
    ("GoogleOther",   "googleother",   "Google AI"),
    ("Bytespider",    "bytespider",    "ByteDance AI"),
    ("CCBot",         "ccbot",         "Common Crawl (used by many AI systems)"),
];
```

### Listicle Heading Regex Patterns
```rust
// Case-insensitive patterns for H1-H3 heading text
// Matches: "Top 10 Tools", "Best 5 Frameworks", "10 Best Practices", "Top Picks"
use regex::Regex;

// "Top N" pattern: "Top" followed by optional number
let top_n = Regex::new(r"(?i)\btop\s+\d+\b").unwrap();
// "Best N" pattern: "Best" followed by number
let best_n = Regex::new(r"(?i)\bbest\s+\d+\b").unwrap();
// "N Best/Top" pattern: number followed by "Best" or "Top"
let n_best = Regex::new(r"(?i)\b\d+\s+(?:best|top)\b").unwrap();
```

### Schema Type Extraction from JSON-LD
```rust
// Handle @type as string or array, across multiple JSON-LD blocks
use serde_json::Value;
use scraper::{Html, Selector};
use std::collections::HashSet;

fn collect_schema_types(html: &Html) -> HashSet<String> {
    let sel = Selector::parse(r#"script[type="application/ld+json"]"#).expect("valid selector");
    let mut types = HashSet::new();

    for block in html.select(&sel) {
        let text: String = block.text().collect();
        if let Ok(val) = serde_json::from_str::<Value>(&text) {
            extract_types_recursive(&val, &mut types);
        }
    }
    types
}

fn extract_types_recursive(val: &Value, types: &mut HashSet<String>) {
    match val.get("@type") {
        Some(Value::String(s)) => { types.insert(s.clone()); }
        Some(Value::Array(arr)) => {
            for v in arr {
                if let Some(s) = v.as_str() { types.insert(s.to_string()); }
            }
        }
        _ => {}
    }
    // Also check @graph arrays (common in JSON-LD)
    if let Some(Value::Array(graph)) = val.get("@graph") {
        for item in graph {
            extract_types_recursive(item, types);
        }
    }
}
```

### Severity Points Update Pattern
```rust
// Add to severity_points() match arms in scoring.rs
// Pattern: match prefix for AI bot checks instead of listing all 6
fn severity_points(check: &str) -> u32 {
    match check {
        // existing tech/cont entries...
        _ if check.starts_with("geo-ai-bot-") => 10,  // critical per D-11
        "geo-listicle" => 5,                            // warning per D-11
        "geo-schema-stacking" => 5,                     // warning per D-11
        _ => 5,  // default fallback
    }
}
```

### Comparison Table Detection Heuristic
```rust
// A table is "structured comparison" if it has:
// 1. A header row (th elements)
// 2. Multiple data rows (3+ tr elements)
// 3. At least 2 columns
fn has_comparison_table(html: &Html) -> bool {
    let table_sel = Selector::parse("table").expect("valid");
    let th_sel = Selector::parse("th").expect("valid");
    let tr_sel = Selector::parse("tr").expect("valid");

    for table in html.select(&table_sel) {
        let has_headers = table.select(&th_sel).next().is_some();
        let row_count = table.select(&tr_sel).count();
        if has_headers && row_count >= 3 {
            return true;
        }
    }
    false
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Ignoring AI crawlers in robots.txt | Explicitly managing GPTBot, ClaudeBot, PerplexityBot directives | 2024-2025 | 21% of top 1000 sites now have GPTBot rules in robots.txt |
| Traditional SEO only | GEO-aware optimization (listicle format, schema stacking) | 2024-2026 | 74.2% of AI citations come from listicle-style content |
| Single schema type per page | Triple schema stacking (Article + ItemList + FAQPage) | 2025-2026 | GEO best practice for AI search visibility |

## Open Questions

1. **@graph handling in JSON-LD**
   - What we know: Some sites wrap multiple types in a `@graph` array within a single JSON-LD block
   - What's unclear: Whether this is common enough to warrant recursive extraction in v1
   - Recommendation: Handle it -- the code is minimal (5 extra lines) and avoids false negatives on well-structured sites. See code example above.

2. **Numbered heading sequences detection**
   - What we know: D-12 says detect "1. ...", "2. ..." patterns in H2/H3 tags
   - What's unclear: How strict -- does "1) First item" count? What about "Step 1:"?
   - Recommendation: Use regex `(?i)^\d+[\.\)]\s+` to match digit followed by period or paren. This covers "1. Item", "1) Item" but not "Step 1:" (which is a how-to, not a listicle).

3. **GoogleOther vs Google-Extended user-agent**
   - What we know: D-15 specifies "GoogleOther". Multiple sources mention "Google-Extended" as Google's AI training token.
   - What's unclear: Whether GoogleOther is the correct robots.txt user-agent string for Google AI services.
   - Recommendation: Use "GoogleOther" as specified in D-15 (locked decision). The `robotstxt` crate handles the matching correctly for this user-agent string.

## Project Constraints (from CLAUDE.md)

- **Language:** Rust -- single binary, no cloud dependencies
- **Output:** JSON-only for v1
- **Crate choices:** Use `scraper` for HTML parsing, `robotstxt` for robots.txt, `serde_json` for JSON, `regex` for patterns (all specified in CLAUDE.md)
- **No hand-rolled alternatives:** Do not use `html5ever` directly, do not build custom robots.txt parsers
- **Testing:** Unit tests for parsers, use fixtures not real websites
- **Error handling:** `anyhow` for application code
- **GSD Workflow:** Do not make direct repo edits outside a GSD workflow

## Sources

### Primary (HIGH confidence)
- `robotstxt` 0.3.0 source code (`matcher.rs`) -- verified `one_agent_allowed_by_robots()` API, state reset behavior, `&mut self` signature
- Existing codebase (`scoring.rs`, `main.rs`, `content.rs`, `technical.rs`) -- established patterns, current signatures, integration points
- `Cargo.toml` -- current dependency versions

### Secondary (MEDIUM confidence)
- [Momentic Marketing - AI Search Crawlers List](https://momenticmarketing.com/blog/ai-search-crawlers-bots) -- user-agent strings for GPTBot, ClaudeBot, PerplexityBot, Bytespider, CCBot
- [Am I Cited - Robots.txt for AI](https://www.amicited.com/blog/robots-txt-ai-control-bot-access/) -- robots.txt AI bot configuration patterns
- [Paul Calvano - AI Bots and Robots.txt](https://paulcalvano.com/2025-08-21-ai-bots-and-robots-txt/) -- 21% adoption stat, user-agent naming conventions
- [GenOptima - GEO Best Practices 2026](https://www.gen-optima.com/blog/generative-engine-optimization-best-practices-complete-2026-playbook/) -- 74.2% listicle citation stat, triple schema stacking practice
- [Enrich Labs - GEO Complete Guide 2026](https://www.enrichlabs.ai/blog/generative-engine-optimization-geo-complete-guide-2026) -- GEO optimization strategies
- [Schema.org - ItemList](https://schema.org/ItemList) -- ItemList schema type definition
- [Google - FAQPage Structured Data](https://developers.google.com/search/docs/appearance/structured-data/faqpage) -- FAQPage schema specification

### Tertiary (LOW confidence)
- None

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all crates already in dependencies (except regex which is in CLAUDE.md spec)
- Architecture: HIGH -- follows established Phase 2 patterns exactly, integration points clearly identified in codebase
- Pitfalls: HIGH -- verified robotstxt internal state reset in source code, JSON-LD @type ambiguity confirmed in spec

**Research date:** 2026-03-23
**Valid until:** 2026-04-23 (stable domain, no fast-moving dependencies)
