# Phase 7: Research and Implement Missing GEO Metrics - Research

**Researched:** 2026-03-29
**Domain:** GEO content analysis (llms.txt, AI directives, citations, entities, conversational optimization)
**Confidence:** HIGH

## Summary

Phase 7 adds approximately 15-20 new `geo-` prefixed check IDs across five metric areas. The implementation is entirely within existing dependencies -- no new crates are needed. The codebase already has well-established patterns from Phase 3 (geo.rs analyzers, severity_points matching, AnalysisResult return types) that these new checks follow directly.

The main architectural decision is splitting geo.rs into sub-modules since it will grow from ~215 lines to roughly 800-1000 lines. The llms.txt check requires a new HTTP fetch in the analysis pipeline (similar to how robots.txt is already fetched once at crawl start). AI directive meta tag and X-Robots-Tag header checks require passing HTTP response headers into the analyzer, which is the only new data flow needed in lib.rs.

**Primary recommendation:** Split new checks into separate files (geo_llms.rs, geo_citations.rs, geo_entities.rs, geo_query.rs, geo_directives.rs, geo_freshness.rs) under analyzers/, re-exporting from geo.rs or a new geo/ directory. Wire all new analyzers into lib.rs's analyze() function following the existing sequential pattern.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** All 5 named areas are implemented: llms.txt, AI crawler directives expansion, citation signals, entity coverage, conversational query optimization.
- **D-02:** All 5 v2 deferred requirements folded in: GEO-04 (FAQ quality scoring), GEO-05 (quick answer block detection), GEO-06 (content freshness signals), GEO-07 (citation/statistic density), GEO-08 (HowTo schema validation).
- **D-03:** All new checks use `geo-` prefix and route to the existing geo scoring category. Overall score formula unchanged (3-way or 4-way average depending on --vitals).
- **D-04:** Severity model: clear-cut detections (llms.txt missing, freshness signals absent) can be critical (10pts). Heuristic-heavy checks (entity coverage, conversational optimization) use warning (5pts) or info (2pts) severity to avoid false-positive score tanking.
- **D-05:** Check `/llms.txt` presence + basic validation. Fetch the file, report pass if exists with non-empty content of reasonable length. Warn/fail if missing. Don't deeply parse internal format since the spec is still evolving.
- **D-06:** llms.txt absence is **critical severity (10pts)** -- strong stance that AI-readiness requires this file.
- **D-07:** Check AI-specific meta tags in HTML (e.g., `<meta name="robots" content="noai">`, Google's AI-specific directives) AND `X-Robots-Tag` HTTP headers with AI crawler values. Both directive mechanisms are covered.
- **D-08:** These expand the existing AI bot audit (Phase 3) -- separate check IDs, not modifications to existing `geo-ai-bot-*` checks.
- **D-09:** Four separate citation checks: geo-citation-stats, geo-citation-sources, geo-citation-quotes, geo-citation-references.
- **D-10:** Threshold: at least 1 signal per page for each check type. Pass if present, warn if absent.
- **D-11:** Four separate entity checks: geo-entity-schema, geo-entity-about, geo-entity-proper-nouns, geo-entity-author.
- **D-12:** Four separate query optimization checks: geo-query-qa-patterns, geo-query-summary, geo-query-snippet, geo-query-faq.
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
- How to efficiently extract HTTP headers (X-Robots-Tag) -- whether to use existing reqwest response or make separate HEAD request
- GEO-04 FAQ quality scoring granularity (per-answer vs. aggregate)
- GEO-08 HowTo schema validation depth (presence-only vs. structural completeness)
- Implementation of GEO-06 freshness signals check ID naming

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| D-01 | All 5 named metric areas implemented | Architecture patterns section covers module structure for all 5 |
| D-02 | GEO-04 through GEO-08 folded in | Mapped to specific checks in Architecture Patterns |
| D-03 | geo- prefix, existing scoring category | scoring.rs analysis confirms geo- routing already works |
| D-04 | Severity model (critical vs warning vs info) | Severity assignment table in Architecture Patterns |
| D-05/D-06 | llms.txt presence check, critical severity | llms.txt spec research + fetch pattern |
| D-07/D-08 | AI meta tags + X-Robots-Tag headers | Meta tag directive research + header extraction pattern |
| D-09/D-10 | 4 citation signal checks | Citation regex patterns in Code Examples |
| D-11 | 4 entity coverage checks | Entity detection patterns in Code Examples |
| D-12 | 4 query optimization checks | Query pattern detection in Code Examples |
| D-13 | GEO-04 FAQ quality scoring | FAQ word count logic in Code Examples |
| D-14 | GEO-05 quick answer blocks | Covered by geo-query-summary |
| D-15 | GEO-06 freshness signals | Freshness detection patterns in Code Examples |
| D-16 | GEO-07 citation density | Covered by D-09 citation checks |
| D-17 | GEO-08 HowTo schema validation | HowTo JSON-LD pattern in Code Examples |
</phase_requirements>

## Standard Stack

### Core (already in Cargo.toml -- no new dependencies)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| scraper | 0.26 | HTML parsing + CSS selectors | Already used for all HTML analysis |
| serde_json | 1.0 | JSON-LD parsing | Already used for schema extraction |
| regex | 1.12 | Pattern matching for citations, entities | Already used in listicle detection |
| reqwest | 0.13 | HTTP fetch for llms.txt + response headers | Already used for page fetching |
| robotstxt | 0.3 | robots.txt parsing (existing) | Already used for AI bot audit |

### No New Dependencies Needed
All 5 metric areas use existing crates. Citation regex, entity detection, and query pattern matching are all achievable with `regex` + `scraper`. llms.txt fetch reuses the existing `reqwest::Client`. JSON-LD entity/HowTo checks reuse `serde_json::Value` parsing.

## Architecture Patterns

### Recommended Module Structure

The existing `geo.rs` is 215 lines with 3 analyzers. Adding ~15 more analyzers would push it to ~1000 lines. Split into sub-modules:

```
src/analyzers/
  geo.rs                  # Keep existing: analyze_listicle, analyze_ai_bots, analyze_schema_stacking, extract_types helper
  geo_llms.rs             # NEW: analyze_llms_txt (D-05/D-06)
  geo_directives.rs       # NEW: analyze_ai_meta_directives, analyze_ai_header_directives (D-07/D-08)
  geo_citations.rs        # NEW: 4 citation checks (D-09) + GEO-04 FAQ quality (D-13)
  geo_entities.rs         # NEW: 4 entity checks (D-11)
  geo_query.rs            # NEW: 4 query optimization checks (D-12)
  geo_freshness.rs        # NEW: freshness signals (D-15) + HowTo schema (D-17)
  mod.rs                  # Re-exports all geo_* + existing modules
```

Alternative: keep everything in geo.rs with clear section comments. Given ~15 new public functions, separate files are cleaner and match the existing flat module pattern.

### Complete Check ID Inventory

| Check ID | Area | Severity | Points | Source Decision |
|----------|------|----------|--------|-----------------|
| `geo-llms-txt` | llms.txt | critical | 10 | D-05, D-06 |
| `geo-ai-meta-directives` | AI directives | warning | 5 | D-07, D-08 |
| `geo-ai-header-directives` | AI directives | warning | 5 | D-07, D-08 |
| `geo-citation-stats` | Citations | warning | 5 | D-09 |
| `geo-citation-sources` | Citations | warning | 5 | D-09 |
| `geo-citation-quotes` | Citations | warning | 5 | D-09 |
| `geo-citation-references` | Citations | warning | 5 | D-09 |
| `geo-entity-schema` | Entities | warning | 5 | D-11 |
| `geo-entity-about` | Entities | info | 2 | D-11, D-04 |
| `geo-entity-proper-nouns` | Entities | info | 2 | D-11, D-04 |
| `geo-entity-author` | Entities | warning | 5 | D-11 |
| `geo-query-qa-patterns` | Query opt | warning | 5 | D-12 |
| `geo-query-summary` | Query opt | warning | 5 | D-12, D-14 |
| `geo-query-snippet` | Query opt | info | 2 | D-12, D-04 |
| `geo-query-faq` | Query opt | warning | 5 | D-12 |
| `geo-faq-quality` | FAQ quality | info | 2 | D-13 |
| `geo-freshness` | Freshness | critical | 10 | D-15 |
| `geo-howto-schema` | HowTo | warning | 5 | D-17 |

**Total: 18 new check IDs** (plus existing 8: listicle, 6 AI bots, schema stacking = 26 total geo checks)

### Severity Assignment Rationale (per D-04)

- **Critical (10pts):** `geo-llms-txt` (clear-cut: file exists or not), `geo-freshness` (clear-cut: dateModified present or not)
- **Warning (5pts):** Most checks -- citation signals, entity schema/author, query patterns, FAQ, directives, HowTo. These have clear detection but absence is less severe.
- **Info (2pts):** `geo-entity-about`, `geo-entity-proper-nouns`, `geo-query-snippet` -- heuristic-heavy, avoid false-positive score tanking per D-04.

### Data Flow Changes in lib.rs

The `analyze()` function in lib.rs needs two new data sources:

1. **llms.txt body** -- fetch once at crawl start (like robots.txt), pass `&str` to analyzer
2. **HTTP response headers** -- capture from the existing `client.get()` response before calling `.text()`, pass to directive analyzers

```rust
// In analyze(), before the page loop:
let llms_txt_body = fetch_llms_txt(client, &base_url).await;

// In the page loop, capture headers from existing fetch:
let resp = client.get(page_url.as_str()).send().await;
let headers = resp.headers().clone(); // capture before consuming body
let html_body = resp.text().await.unwrap_or_default();

// Pass to analyzers:
results.push(analyze_llms_txt(&llms_txt_body));
results.push(analyze_ai_meta_directives(&html_doc));
results.push(analyze_ai_header_directives(&headers));
results.extend(analyze_citations(&html_doc));
results.extend(analyze_entities(&html_doc));
results.extend(analyze_query_optimization(&html_doc));
results.push(analyze_freshness(&html_doc, &headers));
results.push(analyze_howto_schema(&html_doc));
results.push(analyze_faq_quality(&html_doc));
```

**Important:** `llms.txt` is a site-wide resource, not per-page. It should be fetched once and the same result emitted for every page (or only emitted once in the first page's results, then skipped). The simplest approach: include it in every page's results since it is a property of the site, consistent with how `geo-ai-bot-*` results are already emitted per-page from the same robots.txt body.

### Pattern: Analyzer Function Signatures

Following established patterns from geo.rs:

```rust
// Pure HTML analysis (most checks)
pub fn analyze_citations(html: &Html) -> Vec<AnalysisResult>

// HTTP header analysis (directives, freshness)
pub fn analyze_ai_header_directives(headers: &reqwest::header::HeaderMap) -> AnalysisResult

// Remote resource analysis (llms.txt)
pub fn analyze_llms_txt(body: &str) -> AnalysisResult

// Combined HTML + headers (freshness)
pub fn analyze_freshness(html: &Html, headers: &reqwest::header::HeaderMap) -> AnalysisResult
```

### Anti-Patterns to Avoid
- **Modifying existing check IDs:** D-08 is explicit -- new checks get new IDs, don't modify `geo-ai-bot-*`.
- **Deep llms.txt parsing:** D-05 says the spec is evolving -- check presence + basic validation only.
- **NLP for entity detection:** D-04 says heuristic-heavy checks use low severity -- capitalization-based proper noun detection is sufficient.
- **Multiple HTTP requests for headers:** Capture headers from the existing page fetch response, don't make a separate HEAD request.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| robots.txt parsing | Custom parser | `robotstxt` crate | Already in deps, Google-compatible |
| HTML parsing | Custom tokenizer | `scraper` crate | Already in deps, browser-grade |
| JSON-LD extraction | Custom JSON walker | `serde_json::Value` | Already used in content.rs and geo.rs |
| Regex compilation | Inline patterns | `regex::Regex` with lazy compilation | Already in deps, guaranteed linear time |

**Key insight:** This phase is pure analysis logic with no infrastructure changes. All tools are already available.

## Common Pitfalls

### Pitfall 1: Header Consumption Before Body Read
**What goes wrong:** reqwest Response is consumed by `.text()` -- headers must be captured first.
**Why it happens:** The current code does `resp.text().await` immediately without saving headers.
**How to avoid:** Clone headers before consuming the response body: `let headers = resp.headers().clone();`
**Warning signs:** Compiler error about moved value.

### Pitfall 2: llms.txt Check Repeated Per Page
**What goes wrong:** Fetching llms.txt for every page in a multi-page crawl wastes HTTP requests.
**Why it happens:** The check is site-wide, not page-specific.
**How to avoid:** Fetch once before the page loop (like robots.txt), pass the body string to the analyzer.
**Warning signs:** N HTTP requests to /llms.txt for N pages.

### Pitfall 3: Regex Compilation in Hot Loop
**What goes wrong:** Compiling regex patterns inside per-page analyzer functions causes unnecessary overhead.
**Why it happens:** Regex::new() is called every time the function runs.
**How to avoid:** Use `lazy_static!` or `std::sync::OnceLock` for regex patterns, or compile once and pass as parameter. The existing `analyze_listicle` compiles regexes per-call -- acceptable for now but worth noting.
**Warning signs:** Profile shows regex compilation time.

### Pitfall 4: False Positive Proper Noun Detection
**What goes wrong:** Detecting capitalized words at sentence starts as proper nouns.
**Why it happens:** English sentences start with capital letters regardless of proper nouns.
**How to avoid:** Skip first word of sentences. Look for mid-sentence capitalized words. Multiple consecutive capitalized words (organization names) are stronger signals.
**Warning signs:** Every page "passes" proper noun detection.

### Pitfall 5: severity_points Default Catch-All
**What goes wrong:** New check IDs fall through to the `_ => 5` default in severity_points().
**Why it happens:** Forgetting to add explicit entries for all 18 new check IDs.
**How to avoid:** Add all 18 check IDs to the match block. The `_ => 5` default is a safety net, not a design choice.
**Warning signs:** Tests pass but severity values are wrong for info-level (2pt) checks.

### Pitfall 6: X-Robots-Tag Case Sensitivity
**What goes wrong:** Missing directives because header values are case-sensitive.
**Why it happens:** HTTP header names are case-insensitive but header values may have varying casing.
**How to avoid:** Use case-insensitive matching for directive values (noai, noimageai, nosnippet).
**Warning signs:** Test with "NoAI" fails but "noai" passes.

## Code Examples

### llms.txt Fetch and Validation (D-05, D-06)
```rust
// Source: llmstxt.org spec + project patterns
pub async fn fetch_llms_txt(client: &reqwest::Client, base_url: &Url) -> String {
    let mut llms_url = base_url.clone();
    llms_url.set_path("/llms.txt");
    llms_url.set_query(None);
    llms_url.set_fragment(None);

    match client.get(llms_url.as_str()).send().await {
        Ok(resp) if resp.status().is_success() => resp.text().await.unwrap_or_default(),
        _ => String::new(),
    }
}

pub fn analyze_llms_txt(body: &str) -> AnalysisResult {
    if body.trim().is_empty() {
        return AnalysisResult {
            check: "geo-llms-txt",
            status: Status::Fail,
            message: "No /llms.txt file found. This file helps AI systems understand your site structure.".to_string(),
            recommendation: "Create a /llms.txt file at your site root following the llms.txt specification (https://llmstxt.org). Include an H1 with your site name, a summary blockquote, and links to key content sections.".to_string(),
        };
    }
    // Basic validation: must have H1 (required by spec), reasonable length
    let has_h1 = body.lines().any(|l| l.starts_with("# "));
    let reasonable_length = body.len() >= 50; // Minimum viable content
    if !has_h1 {
        return AnalysisResult {
            check: "geo-llms-txt",
            status: Status::Warn,
            message: "/llms.txt exists but missing required H1 heading.".to_string(),
            recommendation: "Add an H1 heading (# Your Site Name) as the first line of llms.txt -- this is the only required element per the specification.".to_string(),
        };
    }
    if !reasonable_length {
        return AnalysisResult {
            check: "geo-llms-txt",
            status: Status::Warn,
            message: "/llms.txt exists but appears too short to be useful.".to_string(),
            recommendation: "Expand your llms.txt with a summary blockquote and links to key content sections. AI systems use this to navigate your site efficiently.".to_string(),
        };
    }
    AnalysisResult {
        check: "geo-llms-txt",
        status: Status::Pass,
        message: format!("/llms.txt found ({} bytes) with valid structure.", body.len()),
        recommendation: String::new(),
    }
}
```

### AI Meta Tag Directive Detection (D-07)
```rust
// Source: Google Search Central docs, DeviantArt noai spec
pub fn analyze_ai_meta_directives(html: &Html) -> AnalysisResult {
    let meta_sel = Selector::parse(r#"meta[name="robots"]"#).expect("valid selector");
    let mut ai_directives: Vec<String> = Vec::new();

    for meta in html.select(&meta_sel) {
        if let Some(content) = meta.value().attr("content") {
            let lower = content.to_lowercase();
            // Check for AI-specific directives
            for directive in ["noai", "noimageai", "nosnippet"] {
                if lower.contains(directive) {
                    ai_directives.push(directive.to_string());
                }
            }
        }
    }

    if ai_directives.is_empty() {
        AnalysisResult {
            check: "geo-ai-meta-directives",
            status: Status::Pass,
            message: "No AI-blocking meta robot directives found. AI crawlers can process this page.".to_string(),
            recommendation: String::new(),
        }
    } else {
        AnalysisResult {
            check: "geo-ai-meta-directives",
            status: Status::Warn,
            message: format!("AI-blocking meta directives detected: {}. These may prevent AI systems from using your content in search results.", ai_directives.join(", ")),
            recommendation: "Review your meta robots directives. If you want AI search visibility, remove noai/noimageai directives. If intentional, this is expected.".to_string(),
        }
    }
}
```

### X-Robots-Tag Header Check (D-07)
```rust
// Source: MDN X-Robots-Tag docs
pub fn analyze_ai_header_directives(headers: &reqwest::header::HeaderMap) -> AnalysisResult {
    let mut ai_directives: Vec<String> = Vec::new();

    if let Some(val) = headers.get("x-robots-tag") {
        if let Ok(s) = val.to_str() {
            let lower = s.to_lowercase();
            for directive in ["noai", "noimageai", "nosnippet"] {
                if lower.contains(directive) {
                    ai_directives.push(directive.to_string());
                }
            }
        }
    }

    if ai_directives.is_empty() {
        AnalysisResult {
            check: "geo-ai-header-directives",
            status: Status::Pass,
            message: "No AI-blocking X-Robots-Tag headers found.".to_string(),
            recommendation: String::new(),
        }
    } else {
        AnalysisResult {
            check: "geo-ai-header-directives",
            status: Status::Warn,
            message: format!("AI-blocking X-Robots-Tag directives detected: {}.", ai_directives.join(", ")),
            recommendation: "Review your X-Robots-Tag HTTP headers. noai/noimageai directives may prevent AI search engines from including your content in results.".to_string(),
        }
    }
}
```

### Citation Signal Detection (D-09)
```rust
// Source: GEO research on citation patterns
use regex::Regex;

pub fn analyze_citations(html: &Html) -> Vec<AnalysisResult> {
    let body_sel = Selector::parse("body").expect("valid selector");
    let body_text: String = html.select(&body_sel)
        .flat_map(|el| el.text())
        .collect::<Vec<_>>()
        .join(" ");

    let mut results = Vec::new();

    // geo-citation-stats: numbers with context (percentages, dollar amounts, fractions)
    let stats_re = Regex::new(r"(?i)(\d+\.?\d*\s*%|\$\d[\d,]*\.?\d*|\d+\s+out\s+of\s+\d+|\d+x\s)").expect("valid regex");
    let has_stats = stats_re.is_match(&body_text);
    results.push(AnalysisResult {
        check: "geo-citation-stats",
        status: if has_stats { Status::Pass } else { Status::Warn },
        message: if has_stats {
            "Statistics/numerical data found in content.".to_string()
        } else {
            "No statistics or numerical data found in content.".to_string()
        },
        recommendation: if has_stats { String::new() } else {
            "Add specific statistics, percentages, or numerical data to strengthen content credibility. AI engines prefer content with verifiable data points.".to_string()
        },
    });

    // geo-citation-sources: "according to", "a study by", "research from"
    let sources_re = Regex::new(r"(?i)(according\s+to|a\s+study\s+by|research\s+(from|by|shows)|data\s+from|report\s+by|survey\s+by|published\s+(in|by))").expect("valid regex");
    let has_sources = sources_re.is_match(&body_text);
    results.push(AnalysisResult {
        check: "geo-citation-sources",
        status: if has_sources { Status::Pass } else { Status::Warn },
        message: if has_sources {
            "Source attribution patterns found in content.".to_string()
        } else {
            "No source attributions found in content.".to_string()
        },
        recommendation: if has_sources { String::new() } else {
            "Add source attributions (e.g., 'according to [Source]', 'a study by [Organization] found'). External authority citations boost AI visibility up to 40%.".to_string()
        },
    });

    // geo-citation-quotes: blockquotes or quotation patterns
    let bq_sel = Selector::parse("blockquote").expect("valid selector");
    let has_blockquote = html.select(&bq_sel).next().is_some();
    let quote_re = Regex::new(r#"(?i)(as\s+\w+\s+said|"\s*[A-Z])"#).expect("valid regex");
    let has_quotes = has_blockquote || quote_re.is_match(&body_text);
    results.push(AnalysisResult {
        check: "geo-citation-quotes",
        status: if has_quotes { Status::Pass } else { Status::Warn },
        message: if has_quotes {
            "Quotation/blockquote content found.".to_string()
        } else {
            "No blockquotes or quotation patterns found.".to_string()
        },
        recommendation: if has_quotes { String::new() } else {
            "Add expert quotes using <blockquote> elements or attribution patterns ('As [Expert] said...'). Quotes add credibility signals for AI citation.".to_string()
        },
    });

    // geo-citation-references: reference/bibliography sections
    let heading_sel = Selector::parse("h2, h3, h4").expect("valid selector");
    let ref_re = Regex::new(r"(?i)^(references|sources|bibliography|citations|further\s+reading|works\s+cited)$").expect("valid regex");
    let has_refs = html.select(&heading_sel).any(|el| {
        let text: String = el.text().collect();
        ref_re.is_match(text.trim())
    });
    results.push(AnalysisResult {
        check: "geo-citation-references",
        status: if has_refs { Status::Pass } else { Status::Warn },
        message: if has_refs {
            "Reference/bibliography section found.".to_string()
        } else {
            "No reference or bibliography section detected.".to_string()
        },
        recommendation: if has_refs { String::new() } else {
            "Add a 'References' or 'Sources' section with links to cited material. AI systems use reference sections as credibility signals.".to_string()
        },
    });

    results
}
```

### Entity Coverage Detection (D-11)
```rust
// Source: Schema.org types + project JSON-LD patterns
pub fn analyze_entities(html: &Html) -> Vec<AnalysisResult> {
    let ld_sel = Selector::parse(r#"script[type="application/ld+json"]"#).expect("valid selector");
    let mut all_json: Vec<Value> = Vec::new();
    let mut found_types: HashSet<String> = HashSet::new();

    for block in html.select(&ld_sel) {
        let text: String = block.text().collect();
        if let Ok(val) = serde_json::from_str::<Value>(&text) {
            extract_types(&val, &mut found_types); // reuse from geo.rs
            all_json.push(val);
        }
    }

    let mut results = Vec::new();

    // geo-entity-schema: Person or Organization types present
    let has_person = found_types.contains("Person");
    let has_org = found_types.contains("Organization");
    let has_entity_schema = has_person || has_org;
    results.push(AnalysisResult {
        check: "geo-entity-schema",
        status: if has_entity_schema { Status::Pass } else { Status::Warn },
        // ... messages ...
    });

    // geo-entity-about: check for "about" or "mentions" properties in JSON-LD
    let has_about = all_json.iter().any(|v| v.get("about").is_some() || v.get("mentions").is_some());
    results.push(AnalysisResult {
        check: "geo-entity-about",
        status: if has_about { Status::Pass } else { Status::Warn },
        // ... messages ...
    });

    // geo-entity-proper-nouns: mid-sentence capitalized words
    let body_text = extract_body_text(html);
    let proper_noun_re = Regex::new(r"(?m)\b[a-z]+\s+([A-Z][a-z]{2,})").expect("valid regex");
    let proper_noun_count = proper_noun_re.find_iter(&body_text).count();
    let has_proper_nouns = proper_noun_count >= 3; // threshold: at least 3 proper nouns
    results.push(AnalysisResult {
        check: "geo-entity-proper-nouns",
        status: if has_proper_nouns { Status::Pass } else { Status::Warn },
        // ... messages ...
    });

    // geo-entity-author: author byline detection
    let author_meta = Selector::parse(r#"meta[name="author"]"#).expect("valid selector");
    let has_author_meta = html.select(&author_meta).any(|el| {
        el.value().attr("content").map_or(false, |c| !c.trim().is_empty())
    });
    let has_person_schema = has_person;
    let byline_re = Regex::new(r"(?i)\bby\s+[A-Z][a-z]+\s+[A-Z]").expect("valid regex");
    let has_byline = byline_re.is_match(&body_text);
    let has_author = has_author_meta || has_person_schema || has_byline;
    results.push(AnalysisResult {
        check: "geo-entity-author",
        status: if has_author { Status::Pass } else { Status::Warn },
        // ... messages ...
    });

    results
}
```

### Freshness Signal Detection (D-15)
```rust
// Source: Schema.org dateModified, HTTP Last-Modified header
pub fn analyze_freshness(html: &Html, headers: &reqwest::header::HeaderMap) -> AnalysisResult {
    let mut signals: Vec<&str> = Vec::new();

    // Check JSON-LD for dateModified
    let ld_sel = Selector::parse(r#"script[type="application/ld+json"]"#).expect("valid selector");
    for block in html.select(&ld_sel) {
        let text: String = block.text().collect();
        if let Ok(val) = serde_json::from_str::<Value>(&text) {
            if val.get("dateModified").is_some() {
                signals.push("dateModified in JSON-LD");
                break;
            }
        }
    }

    // Check HTTP Last-Modified header
    if headers.get("last-modified").is_some() {
        signals.push("Last-Modified HTTP header");
    }

    // Check meta tag equivalents
    let meta_sel = Selector::parse(r#"meta[http-equiv="last-modified"], meta[property="article:modified_time"]"#).expect("valid selector");
    if html.select(&meta_sel).next().is_some() {
        signals.push("last-modified meta tag");
    }

    if signals.is_empty() {
        AnalysisResult {
            check: "geo-freshness",
            status: Status::Fail,
            message: "No content freshness signals found.".to_string(),
            recommendation: "Add dateModified to your JSON-LD schema and configure Last-Modified HTTP headers. Content without freshness signals loses citation priority in AI search results.".to_string(),
        }
    } else {
        AnalysisResult {
            check: "geo-freshness",
            status: Status::Pass,
            message: format!("Freshness signals found: {}.", signals.join(", ")),
            recommendation: String::new(),
        }
    }
}
```

### HowTo Schema Validation (D-17)
```rust
pub fn analyze_howto_schema(html: &Html) -> AnalysisResult {
    let ld_sel = Selector::parse(r#"script[type="application/ld+json"]"#).expect("valid selector");
    let mut found_types: HashSet<String> = HashSet::new();
    let mut howto_val: Option<Value> = None;

    for block in html.select(&ld_sel) {
        let text: String = block.text().collect();
        if let Ok(val) = serde_json::from_str::<Value>(&text) {
            extract_types(&val, &mut found_types);
            if found_types.contains("HowTo") && howto_val.is_none() {
                howto_val = Some(val);
            }
        }
    }

    if !found_types.contains("HowTo") {
        return AnalysisResult {
            check: "geo-howto-schema",
            status: Status::Warn,
            message: "No HowTo schema found. Consider adding if this page contains step-by-step instructions.".to_string(),
            recommendation: "Add HowTo JSON-LD schema for instructional content. HowTo schema enables rich results and improves AI search visibility for how-to queries.".to_string(),
        };
    }

    // Structural validation: check for "step" property
    if let Some(val) = howto_val {
        let has_steps = val.get("step").is_some();
        if has_steps {
            AnalysisResult {
                check: "geo-howto-schema",
                status: Status::Pass,
                message: "HowTo schema found with step definitions.".to_string(),
                recommendation: String::new(),
            }
        } else {
            AnalysisResult {
                check: "geo-howto-schema",
                status: Status::Warn,
                message: "HowTo schema found but missing 'step' property.".to_string(),
                recommendation: "Add 'step' property to your HowTo schema with individual HowToStep entries for each instruction step.".to_string(),
            }
        }
    } else {
        // Shouldn't reach here but safety fallback
        AnalysisResult {
            check: "geo-howto-schema",
            status: Status::Warn,
            message: "HowTo schema detected but could not validate structure.".to_string(),
            recommendation: String::new(),
        }
    }
}
```

### scoring.rs Severity Points Update
```rust
// Add to severity_points() match block:
"geo-llms-txt" | "geo-freshness" => 10,                           // critical
"geo-ai-meta-directives" | "geo-ai-header-directives"
| "geo-citation-stats" | "geo-citation-sources"
| "geo-citation-quotes" | "geo-citation-references"
| "geo-entity-schema" | "geo-entity-author"
| "geo-query-qa-patterns" | "geo-query-summary"
| "geo-query-faq" | "geo-howto-schema" => 5,                      // warning
"geo-entity-about" | "geo-entity-proper-nouns"
| "geo-query-snippet" | "geo-faq-quality" => 2,                   // info
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| robots.txt only for AI control | robots.txt + meta tags + X-Robots-Tag + llms.txt | 2024-2025 | Multiple layers of AI crawler control now exist |
| No llms.txt standard | llms.txt at ~10% adoption (SE Ranking survey) | 2024 (proposed by Jeremy Howard) | Emerging standard, still evolving |
| noai/noimageai informal | Growing W3C standardization momentum | 2022 (DeviantArt) - present | Not yet formal standard but widely respected |
| Schema.org for traditional SEO | Schema.org as primary GEO signal | 2024-2025 | JSON-LD stacking is now a GEO best practice |
| Freshness via HTTP headers only | dateModified in JSON-LD + HTTP headers + meta tags | 2025+ | Multi-signal freshness detection improves AI citation |

**Deprecated/outdated:**
- Google-Extended was the original AI-training-specific user agent; now AI crawlers use many distinct user agents (GPTBot, ClaudeBot, etc.)
- `noai` is not yet an official standard -- it works but has no formal specification body

## Open Questions

1. **llms.txt "reasonable length" threshold**
   - What we know: The spec only requires an H1. Real implementations range from 50 bytes to several KB.
   - What's unclear: What constitutes "too short to be useful" vs. minimal compliance.
   - Recommendation: Use 50 bytes minimum (H1 + summary blockquote would exceed this). Warn below, pass above.

2. **FAQ quality scoring granularity (D-13)**
   - What we know: Optimal FAQ answer length is 40-60 words.
   - What's unclear: Should we score per-answer (each FAQ answer independently) or aggregate (average across all answers)?
   - Recommendation: Per-answer with aggregate reporting. Check each FAQ answer word count, report how many fall in optimal range.

3. **Proper noun density threshold**
   - What we know: Named entities strengthen AI citation.
   - What's unclear: How many proper nouns constitute "sufficient" entity coverage.
   - Recommendation: Threshold of 3+ mid-sentence capitalized words. This is deliberately low to avoid false negatives on short pages.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in #[cfg(test)] + integration tests |
| Config file | Cargo.toml [dev-dependencies] |
| Quick run command | `cargo test --lib` |
| Full suite command | `cargo test` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| D-05 | llms.txt presence check | unit | `cargo test --lib geo_llms` | Wave 0 |
| D-06 | llms.txt critical severity | unit | `cargo test --lib geo_llms` | Wave 0 |
| D-07 | AI meta + header directives | unit | `cargo test --lib geo_directives` | Wave 0 |
| D-09 | 4 citation signal checks | unit | `cargo test --lib geo_citations` | Wave 0 |
| D-11 | 4 entity coverage checks | unit | `cargo test --lib geo_entities` | Wave 0 |
| D-12 | 4 query optimization checks | unit | `cargo test --lib geo_query` | Wave 0 |
| D-13 | FAQ quality scoring | unit | `cargo test --lib geo_citations` | Wave 0 |
| D-15 | Freshness signals | unit | `cargo test --lib geo_freshness` | Wave 0 |
| D-17 | HowTo schema validation | unit | `cargo test --lib geo_freshness` | Wave 0 |
| D-04 | Severity model correct | unit | `cargo test --lib scoring` | Wave 0 |
| D-03 | All checks route to geo category | unit | `cargo test --lib scoring` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test --lib`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `src/analyzers/geo_llms.rs` -- unit tests for llms.txt analysis
- [ ] `src/analyzers/geo_directives.rs` -- unit tests for meta + header directive checks
- [ ] `src/analyzers/geo_citations.rs` -- unit tests for 4 citation checks + FAQ quality
- [ ] `src/analyzers/geo_entities.rs` -- unit tests for 4 entity checks
- [ ] `src/analyzers/geo_query.rs` -- unit tests for 4 query optimization checks
- [ ] `src/analyzers/geo_freshness.rs` -- unit tests for freshness + HowTo schema
- [ ] `src/scoring.rs` -- additional tests for 18 new check ID severity values

## Project Constraints (from CLAUDE.md)

- **Language:** Rust -- single binary distribution
- **Output:** JSON-only for v1 (CLI outputs JSON to stdout)
- **Stack:** Use existing deps only: scraper, serde_json, regex, reqwest, robotstxt, anyhow, tracing
- **Patterns:** AnalysisResult return type, kebab-case check IDs with category prefix, CSS selectors via scraper
- **Testing:** Unit tests with #[cfg(test)], integration tests with mockito + assert_cmd
- **Logging:** tracing to stderr only (stdout is reserved for JSON output)
- **Error handling:** anyhow for application code

## Sources

### Primary (HIGH confidence)
- [llmstxt.org](https://llmstxt.org/) -- Official llms.txt specification
- [Google Search Central - Robots Meta Tags](https://developers.google.com/search/docs/crawling-indexing/robots-meta-tag) -- Official Google docs on meta robots directives
- [MDN - X-Robots-Tag](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/X-Robots-Tag) -- Official X-Robots-Tag header documentation
- Existing codebase: src/analyzers/geo.rs, src/scoring.rs, src/lib.rs -- verified patterns

### Secondary (MEDIUM confidence)
- [GEO Best Practices 2026 - SearchEngineLand](https://searchengineland.com/mastering-generative-engine-optimization-in-2026-full-guide-469142) -- Citation signal patterns, entity coverage
- [NoAI Meta Tags - Am I Cited](https://www.amicited.com/blog/noai-meta-tags-controlling-ai-access/) -- noai/noimageai directive details
- [The State of llms.txt in 2026](https://www.aeo.press/ai/the-state-of-llms-txt-in-2026) -- Adoption data (10.13% rate)

### Tertiary (LOW confidence)
- Citation signal regex patterns -- derived from GEO research articles, not formally standardized
- Proper noun detection heuristics -- capitalization-based approach is simple but imperfect

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all deps already in Cargo.toml, no new crates needed
- Architecture: HIGH -- follows established patterns from Phase 3, clear module structure
- Pitfalls: HIGH -- derived from direct codebase analysis (header consumption, regex compilation)
- Detection patterns: MEDIUM -- citation/entity regexes are heuristic-based, thresholds are educated guesses

**Research date:** 2026-03-29
**Valid until:** 2026-04-28 (stable domain, llms.txt spec may evolve)
