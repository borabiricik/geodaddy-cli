# Phase 2: Core Analysis Engine - Research

**Researched:** 2026-03-23
**Domain:** Rust HTML parsing, JSON-LD validation, XML sitemap parsing, HTTP redirect detection, SEO threshold values
**Confidence:** HIGH

## Summary

Phase 2 transforms the Phase 1 scaffold into a working analysis engine by adding 12 analyzers across Technical SEO (TECH-01 through TECH-08) and Content Structure (CONT-01 through CONT-04) categories, plus a severity-weighted scoring system. The codebase is Rust with tokio + reqwest already in place; new dependencies are `scraper` (HTML parsing), `jsonschema` (JSON-LD validation), and `quick-xml` (sitemap parsing).

All locked decisions from CONTEXT.md constrain this phase: `AnalysisResult` shape, scoring formula, severity assignments, flat module structure, and the TECH-01 stub are non-negotiable. Research focuses on the concrete API calls and threshold values the planner needs to specify in tasks.

The key architectural insight is that reqwest's `redirect::Policy::custom()` is the correct approach for redirect chain detection — it gives access to `attempt.previous().len()` inside the policy closure, allowing the client to abort and return an error after exceeding 3 hops. All HTML analysis is done through `scraper::Html::parse_document()` + CSS selector queries.

**Primary recommendation:** Use `scraper` CSS selectors for all HTML checks (meta tags, headings, images, links, semantic elements). Use `quick_xml::de::from_str()` with serde-derived structs for sitemap parsing. Use a dedicated `reqwest::Client` configured with `Policy::custom()` for redirect chain detection only.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-05 (Result Shape):** Every analyzer produces `AnalysisResult` with fields `check: &'static str`, `status: Status` (Pass/Fail/Warn), `message: String`, `recommendation: String`. No `details` object. Serializes to `{ "check", "status", "message", "recommendation" }`.

- **D-06 (Scoring Formula):** Severity-weighted. Critical=10pts, Warning=5pts, Info=2pts. Fail=full loss, Warn=half loss, Pass=0 loss. Score = `(earned / max_possible) * 100` clamped to [0,100]. Per-category scores for Technical (TECH-*) and Content (CONT-*) only — no GEO in phase 2. Overall = equal-weight average of Technical and Content (50/50).

  Canonical severity assignments (must not change):
  | Check ID | Severity |
  |----------|----------|
  | tech-broken-links | warning (stub) |
  | tech-redirect-chains | warning |
  | tech-meta-title | critical |
  | tech-meta-description | warning |
  | tech-heading-h1 | critical |
  | tech-heading-hierarchy | warning |
  | tech-mobile-viewport | critical |
  | tech-robots-txt | warning |
  | tech-sitemap-xml | info |
  | tech-https | critical |
  | cont-heading-structure | warning |
  | cont-json-ld | critical |
  | cont-semantic-html | info |
  | cont-alt-text | warning |

- **D-07 (Module Structure):** Flat modules. No trait abstraction.
  ```
  cli/src/
  ├── main.rs
  ├── analyzers/
  │   ├── mod.rs
  │   ├── technical.rs
  │   └── content.rs
  └── scoring.rs
  ```
  Analyzer function signatures: `fn analyze_<name>(html: &scraper::Html, url: &Url) -> Vec<AnalysisResult>` or `-> AnalysisResult`.
  `main.rs` collects all results into `Vec<AnalysisResult>`, serializes, then scores.

- **D-08 (TECH-01 Stub):** `tech-broken-links` always emits a single `warn` with fixed message. No HTTP requests. No link fetching.

- **Carried-forward constraints:**
  - Tracing always to stderr, JSON always to stdout.
  - `process::exit(1)` called AFTER `println!()`.
  - `anyhow::Result<()>` in main, `?` for error propagation.
  - `PageResult.results` changes from `Vec<serde_json::Value>` to `Vec<AnalysisResult>`.
  - Score fields added to `PageResult`: `score: f64`, `categories: CategoryScores`.
  - `reqwest::Client` built in `main.rs` — pass as reference to analyzers that need HTTP.

- **JSON schema frozen** (from Phase 1): `schema_version/url/crawled_at/pages[]` — phases 2-4 only add to `results[]`.

### Claude's Discretion

- Exact redirect chain detection logic — threshold: 3+ hops = excessive.
- robots.txt validation heuristics — what counts as syntax error vs. warning.
- Sitemap.xml URL count threshold for "too large" warning.
- Mixed content detection pattern — scan for `http://` in `src`/`href` attributes on HTTPS pages.

### Deferred Ideas (OUT OF SCOPE)

- Full broken link HTTP checking — Phase 4.
- GEO category scoring — Phase 3.
- Trait-based Analyzer abstraction — not planned for v1.

</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| TECH-01 | Analyzer detects broken links (404s) and reports source URLs | Stubbed — always emits warn per D-08. No HTTP requests. |
| TECH-02 | Analyzer detects redirect chains (301/302, loops, excessive hops) | reqwest `Policy::custom()` with `attempt.previous().len()` threshold; separate client needed. |
| TECH-03 | Analyzer validates meta tags (title 50-60 chars, description 120-158 chars) | scraper CSS selectors `meta[name="description"]`, `title`; thresholds confirmed by 2026 SEO sources. |
| TECH-04 | Analyzer validates heading hierarchy (single H1, logical nesting) | scraper selectors `h1`, `h2`, `h3` etc.; count H1 elements, check sequential nesting order. |
| TECH-05 | Analyzer checks mobile viewport meta tag presence | scraper selector `meta[name="viewport"]`; check `content` contains `width=device-width`. |
| TECH-06 | Analyzer validates robots.txt (syntax, sitemap ref, production Disallow check) | Move existing `check_robots()` to `analyzers/technical.rs`; add syntax + Sitemap directive checks. |
| TECH-07 | Analyzer validates sitemap.xml (format, URL limit, robots.txt conflicts) | quick-xml serde deserialization of `<urlset>`; Google limit = 50,000 URLs per file. |
| TECH-08 | Analyzer checks HTTPS/SSL and flags mixed content | URL scheme check + scraper scan for `http://` in src/href on img/script/link/iframe. |
| CONT-01 | Analyzer validates heading structure (H1-H6 hierarchy, no skipped levels) | scraper selects all heading elements in DOM order; check for level skips (H1→H3 = skip). |
| CONT-02 | Analyzer detects and validates JSON-LD schema markup | scraper finds `<script type="application/ld+json">`; `jsonschema::validator_for()` to validate. |
| CONT-03 | Analyzer checks semantic HTML usage (article, main, nav, section vs div soup) | scraper counts `article`, `main`, `nav`, `section`, `aside`, `header`, `footer`; warn if absent. |
| CONT-04 | Analyzer flags images missing alt text | scraper selector `img`; check each for `alt` attribute presence and non-empty value. |
| SCORE-01 | CLI outputs overall site score (0-100) | Implemented in `scoring.rs`: weighted average of Technical + Content category scores. |
| SCORE-02 | CLI outputs per-category scores (0-100 for Technical, Content) | Implemented in `scoring.rs`: separate earned/max accumulation per category prefix. |
| SCORE-03 | CLI outputs per-metric pass/fail/warn status | `AnalysisResult.status` field (D-05); serialized in each result item. |
| SCORE-04 | Each issue includes actionable fix recommendation with specific guidance | `AnalysisResult.recommendation` field (D-05); each analyzer must populate with specific text. |

</phase_requirements>

---

## Standard Stack

### New Dependencies for Phase 2

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| scraper | 0.26.0 | HTML parsing + CSS selector queries | Verified current version on crates.io. Browser-grade HTML5 parsing via Servo's html5ever. CSS selectors match browser behavior. |
| jsonschema | 0.45.0 | JSON-LD schema validation | Verified current version on crates.io. 75-645x faster than valico. Supports Draft 2020-12 through Draft 4. |
| quick-xml | 0.39.2 | Sitemap.xml parsing | CLAUDE.md specifies 0.38+; crates.io current is 0.39.2. Use `"serialize"` feature for serde deserialization. |

**Note on quick-xml version:** CLAUDE.md cites 0.38+; actual current version on crates.io is 0.39.2 (verified 2026-03-23). Use `0.39` in Cargo.toml to get the latest patch.

**Installation (add to cli/Cargo.toml `[dependencies]`):**
```toml
scraper = "0.26"
jsonschema = "0.45"
quick-xml = { version = "0.39", features = ["serialize"] }
```

**Existing dependencies already covering phase 2 needs:**
- `reqwest 0.13` — redirect detection (Policy::custom), robots.txt fetch, sitemap fetch
- `url 2.5` — URL scheme inspection for HTTPS check
- `serde + serde_json` — AnalysisResult serialization, JSON-LD parsing
- `anyhow` — error propagation in analyzer functions

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| scraper | html5ever directly | html5ever lacks CSS selector support; scraper wraps it ergonomically |
| quick-xml + serde | Event-based quick-xml Reader | Serde deserialization is cleaner for structured sitemap; event API for streaming huge files (not needed at 50K URL limit) |
| jsonschema | Custom JSON-LD type checking | JSON-LD can embed arbitrary nested schemas; library validation covers edge cases custom code would miss |

---

## Architecture Patterns

### Recommended Project Structure

```
cli/src/
├── main.rs              -- CLI, HTTP fetch, analyzer orchestration, JSON output
├── analyzers/
│   ├── mod.rs           -- re-exports: pub use technical::*; pub use content::*;
│   ├── technical.rs     -- TECH-01 through TECH-08 analyzer functions
│   └── content.rs       -- CONT-01 through CONT-04 analyzer functions
└── scoring.rs           -- AnalysisResult, Status, CategoryScores, score calculation
```

### Pattern 1: AnalysisResult and Status Types (scoring.rs)

**What:** Shared types consumed by all analyzer modules. Lives in `scoring.rs`, not in analyzer modules.

```rust
// Source: CONTEXT.md D-05, D-06
use serde::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status { Pass, Fail, Warn }

#[derive(Serialize, Debug, Clone)]
pub struct AnalysisResult {
    pub check: &'static str,
    pub status: Status,
    pub message: String,
    pub recommendation: String,
}

#[derive(Serialize, Debug)]
pub struct CategoryScores {
    pub technical: f64,
    pub content: f64,
}
```

### Pattern 2: scraper HTML Parsing

**What:** Parse fetched HTML body string into a `scraper::Html` document, then query with CSS selectors.

```rust
// Source: https://docs.rs/scraper/latest/scraper/
use scraper::{Html, Selector};

// Parse once in main.rs, pass &Html to all analyzers
let document = Html::parse_document(&html_body);

// Select single element
let title_sel = Selector::parse("title").unwrap();
let title_text = document.select(&title_sel)
    .next()
    .map(|el| el.text().collect::<String>());

// Select multiple elements
let img_sel = Selector::parse("img").unwrap();
for img in document.select(&img_sel) {
    let alt = img.value().attr("alt");  // Option<&str>
}

// Attribute query selector (e.g., meta[name="viewport"])
let viewport_sel = Selector::parse(r#"meta[name="viewport"]"#).unwrap();
let has_viewport = document.select(&viewport_sel).next().is_some();
```

**Key scraper behaviors:**
- `Selector::parse()` returns `Result` — use `.unwrap()` for compile-time-known selectors, or `.expect("msg")`
- `element.text()` returns an iterator over descendent text nodes — collect to `String` to get all text
- `element.value().attr("name")` returns `Option<&str>` — `None` if attribute absent
- `Html::parse_document()` is infallible — never panics on malformed HTML (uses HTML5 error recovery)

### Pattern 3: Redirect Chain Detection (TECH-02)

**What:** Detect HTTP redirect chains using reqwest's `Policy::custom()`. The policy closure receives an `Attempt` with `previous()` returning all prior URLs in the chain.

**Critical:** The main `reqwest::Client` in `main.rs` should NOT use this policy (it must follow redirects normally for fetching page HTML). Create a separate client for redirect checking only.

```rust
// Source: https://docs.rs/reqwest/latest/reqwest/redirect/struct.Policy.html
use reqwest::redirect;

fn build_redirect_check_client() -> reqwest::Client {
    reqwest::Client::builder()
        .redirect(redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 3 {
                // Stop — 3+ hops is excessive
                attempt.error("too many redirects")
            } else {
                attempt.follow()
            }
        }))
        .timeout(Duration::from_secs(10))
        .build()
        .expect("failed to build redirect client")
    // Note: attempt.stop() would silently stop; attempt.error() returns Err for detection
}

// In analyzer: compare original URL to final response URL
// response.url() returns the final URL after all redirects
// If response.url() != original_url, at least one redirect occurred
```

**Threshold recommendation (Claude's discretion):** 3 or more hops = `fail`. 1-2 hops = `pass` (single canonical redirect is normal). The custom policy aborts at hop 3 — the resulting error signals excessive redirects.

### Pattern 4: Sitemap XML Parsing (TECH-07)

**What:** Use quick-xml's serde integration to deserialize sitemap.xml into typed structs.

```rust
// Source: https://docs.rs/quick-xml/latest/quick_xml/de/index.html
use quick_xml::de::from_str;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct UrlSet {
    #[serde(default, rename = "url")]
    urls: Vec<UrlEntry>,
}

#[derive(Deserialize, Debug)]
struct UrlEntry {
    loc: String,
}

// Usage in analyzer:
let url_set: Result<UrlSet, _> = from_str(&xml_body);
match url_set {
    Ok(set) => {
        let count = set.urls.len();
        if count > 50_000 {
            // Warn: exceeds Google's 50,000 URL limit
        }
    }
    Err(_) => {
        // Fail: malformed XML or missing <urlset> root
    }
}
```

### Pattern 5: JSON-LD Validation (CONT-02)

**What:** Find `<script type="application/ld+json">` elements, parse as JSON, validate against schema.

```rust
// Source: https://docs.rs/jsonschema/latest/jsonschema/
use scraper::{Html, Selector};
use serde_json::{Value, from_str as json_from_str};

let ld_sel = Selector::parse(r#"script[type="application/ld+json"]"#).unwrap();

for script in document.select(&ld_sel) {
    let json_text = script.text().collect::<String>();
    match json_from_str::<Value>(&json_text) {
        Ok(instance) => {
            // instance is valid JSON — check for @type property
            let has_type = instance.get("@type").is_some();
            // Optionally validate against a schema:
            // let validator = jsonschema::validator_for(&schema)?;
            // validator.is_valid(&instance)
        }
        Err(_) => {
            // JSON parse error — the JSON-LD block is malformed
        }
    }
}
```

**Scope for phase 2:** Validate that JSON-LD blocks are (1) valid JSON, (2) have `@type` property, (3) have `@context` containing `schema.org`. Full Schema.org type-specific validation is GEO territory (phase 3).

### Pattern 6: Heading Hierarchy Validation (TECH-04 + CONT-01)

**What:** TECH-04 and CONT-01 both inspect headings — they differ in focus. TECH-04 checks H1 count (must be exactly 1). CONT-01 checks for skipped levels (H1 → H3 skipping H2).

```rust
// Source: scraper docs
let heading_sel = Selector::parse("h1, h2, h3, h4, h5, h6").unwrap();
let headings: Vec<u8> = document.select(&heading_sel)
    .map(|el| el.value().name().chars().last().unwrap().to_digit(10).unwrap() as u8)
    .collect();

// TECH-04: count H1s
let h1_count = headings.iter().filter(|&&n| n == 1).count();

// CONT-01: check for skipped levels
let mut prev_level = 0u8;
let mut has_skip = false;
for &level in &headings {
    if level > prev_level + 1 && prev_level > 0 {
        has_skip = true;
        break;
    }
    prev_level = level;
}
```

### Anti-Patterns to Avoid

- **Static selectors in hot loops:** Create `Selector` once outside the loop, not inside. `Selector::parse()` is moderately expensive.
- **Collecting `text()` via `join("")`:** Use `.collect::<String>()` directly — the iterator yields `&str` slices that concatenate correctly.
- **Using main's reqwest Client for redirect checks:** The main client auto-follows all redirects. Redirect detection needs a separate `Policy::custom()` client.
- **Parsing sitemap with event API when serde works:** quick-xml's serde `from_str` handles the standard `<urlset><url><loc>` structure cleanly. Event API is only needed for streaming 50MB+ files.
- **Blocking async in analyzer functions:** Analyzer functions that need HTTP (TECH-02, TECH-06, TECH-07) must be `async fn` and `await` properly — never use `.block_on()` inside a tokio runtime.
- **Panicking on Selector::parse():** All static CSS selectors are known at compile time; using `.unwrap()` on them is acceptable. But document this clearly in code comments.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTML parsing with regex | Custom regex for meta/title/img | `scraper` CSS selectors | HTML is not regular; regex misses attributes in different orders, multi-line, entities, comments |
| XML parsing with string search | `contains("<loc>")` substring scans | `quick_xml::de::from_str()` | Misses CDATA, namespaces, entity encoding in sitemap URLs |
| JSON-LD validation with serde types | Custom `#[derive(Deserialize)]` structs for Schema.org | `serde_json::Value` + `jsonschema` | Schema.org types have hundreds of properties; `Value` handles arbitrary shapes; library validates structure |
| Redirect counting via multiple requests | Manual `client.head(url)` loop | `redirect::Policy::custom()` | reqwest handles the HTTP state machine; manual loop misses `Location` edge cases and cycles |
| robots.txt parsing | Regex or manual line parsing | `robotstxt` crate (already in Cargo.toml) | Google's algorithm has non-obvious precedence rules (longest match wins); crate is official port |

**Key insight:** The scraper crate's CSS selector engine is the same one Firefox uses. Trust it over any custom parsing for HTML structure questions.

---

## Analyzer-Specific Thresholds and Heuristics

### Meta Tag Thresholds (TECH-03) — HIGH confidence

Confirmed by multiple 2026 SEO sources:

| Tag | Min | Max | Fail condition | Warn condition |
|-----|-----|-----|----------------|----------------|
| `<title>` | 50 chars | 60 chars | Missing entirely | < 50 or > 60 |
| `meta[name="description"]` | 120 chars | 158 chars | Missing entirely | < 120 or > 158 |

Title: 50-60 characters shows fully in ~90% of SERPs. Google truncates at ~600px desktop width (approx 60 chars average font).
Description: 120 chars handles mobile (680px / ~120 chars). 158 chars is desktop max (920px). Recommend reporting both character count and the limit.

### Redirect Chain Threshold (TECH-02) — MEDIUM confidence (Claude's discretion)

- 0 redirects: `pass` (no redirect)
- 1-2 redirects: `pass` with informational message (single canonical redirect is normal HTTP practice)
- 3+ redirects: `fail` — use `Policy::custom()` that errors at `attempt.previous().len() >= 3`

Detection approach: Build a separate `redirect_client` in `main.rs` or in `analyze_redirects()`. Make HEAD request to URL. If error from custom policy fires, it's 3+ hops. If response URL differs from original URL, count = 1 redirect.

### Sitemap XML Thresholds (TECH-07) — HIGH confidence (Google official)

- Missing sitemap: `warn` (info severity — not required, but recommended)
- Present, valid XML with `<urlset>` root: `pass`
- Malformed XML: `fail`
- URL count > 50,000: `warn` (Google's official per-file limit)
- No `<loc>` elements: `warn` (empty sitemap)

### robots.txt Heuristics (TECH-06) — MEDIUM confidence (Claude's discretion)

Valid robots.txt rules:
- Each record starts with `User-agent:` line
- `Disallow:` path must start with `/`
- `Allow:` path must start with `/`
- Lines not starting with recognized directives are ignored (not errors per spec)
- Completely missing robots.txt is `warn`, not `fail` (Google treats missing as allow-all)
- `Disallow: /` (blocks everything) on a non-staging domain is `fail`
- `Disallow:` with empty value (allows all) is valid — `pass`
- Has `Sitemap:` directive referencing valid URL: `pass`; missing `Sitemap:` directive: `warn`

### Mixed Content Detection (TECH-08) — HIGH confidence (MDN + web.dev)

On HTTPS pages, scan these element/attribute combinations for `http://` scheme:
- `img[src]`, `img[srcset]`
- `script[src]`
- `link[href]` (stylesheets)
- `iframe[src]`
- `source[src]`, `source[srcset]`
- `video[src]`, `audio[src]`
- `embed[src]`, `object[data]`

**Exclude `<a href>` links** — anchor links are not mixed content (they are user-navigated, not auto-loaded by browser).

Implementation: URL scheme is checked on the normalized `Url` struct or by `starts_with("http://")` on the raw attribute value. If page URL is HTTPS and any of the above attributes load HTTP resources, emit `fail`.

### Semantic HTML Heuristics (CONT-03) — MEDIUM confidence

Elements that indicate semantic structure (count presence, not quantity):
- Structural: `<main>`, `<article>`, `<section>`, `<aside>`
- Navigation: `<nav>`, `<header>`, `<footer>`

Scoring recommendation:
- 0 of these elements present: `fail` (pure div soup)
- `<main>` absent (but others present): `warn`
- `<main>` + at least 2 others present: `pass`

Rationale: `<main>` is the single most important signal for AI/crawler content identification.

### Alt Text (CONT-04) — HIGH confidence

- No `<img>` elements on page: `pass`
- All `<img>` elements have non-empty `alt` attribute: `pass`
- Any `<img>` missing `alt` attribute entirely: `fail`
- Any `<img>` with `alt=""` (empty string): `warn` — decorative images may use empty alt intentionally, but flag for review

---

## Common Pitfalls

### Pitfall 1: Selector Compiled Inside Loop

**What goes wrong:** `Selector::parse()` is called inside a `for` loop iterating over pages, causing redundant allocation.

**Why it happens:** Developer follows scraper docs' inline example pattern.

**How to avoid:** Compile all `Selector` instances once as `let sel = Selector::parse("...").unwrap()` at the top of each analyzer function. They are cheap to clone but moderately expensive to compile.

**Warning signs:** Clippy won't catch this; look for `Selector::parse()` inside loops in code review.

### Pitfall 2: text() Does Not Trim Whitespace

**What goes wrong:** `element.text().collect::<String>()` returns `"\n  Title Text\n  "` — whitespace-padded — causing character count checks to over-count.

**Why it happens:** `text()` yields raw text nodes including whitespace-only nodes between elements.

**How to avoid:** Always `.trim()` the collected string: `el.text().collect::<String>().trim().to_string()`.

**Warning signs:** Off-by-N character count failures in meta title/description checks.

### Pitfall 3: Multiple reqwest Clients

**What goes wrong:** Creating a new `reqwest::Client` inside each analyzer function, losing connection pooling.

**Why it happens:** Analyzer functions take `&scraper::Html` by default; adding `client` requires changing function signature.

**How to avoid:** Per CONTEXT.md D-07 and existing architecture, `reqwest::Client` is built in `main.rs` and passed as `&reqwest::Client` to analyzer functions that need HTTP (TECH-02, TECH-06, TECH-07, TECH-08). The redirect-checking client is the exception — it's a separate instance with a custom `Policy`.

**Warning signs:** `reqwest::Client::builder().build()` appearing inside analyzer functions.

### Pitfall 4: quick-xml Field Renaming for XML Elements

**What goes wrong:** `#[derive(Deserialize)]` struct field `urls` does not match XML element name `url`, causing empty deserialization.

**Why it happens:** quick-xml's serde integration maps XML element names to struct field names directly — no camelCase conversion.

**How to avoid:** Use `#[serde(rename = "url")]` on the field, or name the field exactly as the XML element: `url: Vec<UrlEntry>`.

**Warning signs:** Sitemap parses successfully (no error) but `url_set.url.len() == 0` on a non-empty sitemap.

### Pitfall 5: Heading Level Extraction from Tag Name

**What goes wrong:** `el.value().name()` returns `"h1"` (lowercase string) — accessing `.chars().last()` works but is fragile.

**Why it happens:** Numeric extraction from tag name string is not explicit.

**How to avoid:** Use explicit match:
```rust
let level: u8 = match el.value().name() {
    "h1" => 1, "h2" => 2, "h3" => 3,
    "h4" => 4, "h5" => 5, "h6" => 6,
    _ => continue,
};
```

**Warning signs:** Heading hierarchy analysis silently misses heading levels or panics on unexpected tag names.

### Pitfall 6: reqwest::Client redirect policy and main fetch conflict

**What goes wrong:** If the main `reqwest::Client` is given a `Policy::custom()` for redirect detection, it will also abort redirects during the HTML fetch step, causing the analyzer to fail on sites that legitimately redirect (e.g., `http://` → `https://`).

**Why it happens:** Single client reused for all HTTP calls.

**How to avoid:** Keep main client with default policy (follows up to 10 redirects). Build a separate `redirect_client` with `Policy::custom()` used only in `analyze_redirect_chains()`.

---

## Code Examples

### Full Analyzer Function Skeleton

```rust
// Source: CONTEXT.md D-07, scraper docs
use scraper::{Html, Selector};
use url::Url;
use crate::scoring::{AnalysisResult, Status};

pub fn analyze_meta_tags(html: &Html, _url: &Url) -> Vec<AnalysisResult> {
    let mut results = Vec::new();

    // Title check
    let title_sel = Selector::parse("title").expect("static selector");
    let title_text = html.select(&title_sel)
        .next()
        .map(|el| el.text().collect::<String>())
        .map(|s| s.trim().to_string());

    results.push(match title_text {
        None => AnalysisResult {
            check: "tech-meta-title",
            status: Status::Fail,
            message: "No <title> element found".to_string(),
            recommendation: "Add a <title> element with 50-60 characters".to_string(),
        },
        Some(t) if t.len() < 50 => AnalysisResult {
            check: "tech-meta-title",
            status: Status::Warn,
            message: format!("Title is {} chars (min 50)", t.len()),
            recommendation: "Expand title to 50-60 characters for optimal SERP display".to_string(),
        },
        Some(t) if t.len() > 60 => AnalysisResult {
            check: "tech-meta-title",
            status: Status::Fail,
            message: format!("Title is {} chars (max 60)", t.len()),
            recommendation: "Shorten title to 50-60 characters to prevent truncation in search results".to_string(),
        },
        Some(_) => AnalysisResult {
            check: "tech-meta-title",
            status: Status::Pass,
            message: "Title length is optimal".to_string(),
            recommendation: String::new(),
        },
    });

    results
}
```

### Scoring Function Skeleton

```rust
// Source: CONTEXT.md D-06
use crate::scoring::{AnalysisResult, Status, CategoryScores};

const SEVERITY_CRITICAL: f64 = 10.0;
const SEVERITY_WARNING: f64 = 5.0;
const SEVERITY_INFO: f64 = 2.0;

fn check_severity(check_id: &str) -> f64 {
    match check_id {
        "tech-meta-title" | "tech-heading-h1" | "tech-mobile-viewport"
        | "tech-https" | "cont-json-ld" => SEVERITY_CRITICAL,
        "tech-broken-links" | "tech-redirect-chains" | "tech-meta-description"
        | "tech-heading-hierarchy" | "tech-robots-txt" | "cont-heading-structure"
        | "cont-alt-text" => SEVERITY_WARNING,
        "tech-sitemap-xml" | "cont-semantic-html" => SEVERITY_INFO,
        _ => SEVERITY_INFO,
    }
}

pub fn calculate_scores(results: &[AnalysisResult]) -> (f64, CategoryScores) {
    // Accumulate per-category
    let (tech_earned, tech_max) = category_score(results, "tech-");
    let (cont_earned, cont_max) = category_score(results, "cont-");
    let technical = if tech_max > 0.0 { (tech_earned / tech_max) * 100.0 } else { 100.0 };
    let content   = if cont_max > 0.0 { (cont_earned / cont_max) * 100.0 } else { 100.0 };
    let overall   = (technical + content) / 2.0;
    (overall.clamp(0.0, 100.0), CategoryScores { technical, content })
}

fn category_score(results: &[AnalysisResult], prefix: &str) -> (f64, f64) {
    results.iter()
        .filter(|r| r.check.starts_with(prefix))
        .fold((0.0_f64, 0.0_f64), |(earned, max), r| {
            let sev = check_severity(r.check);
            let loss = match r.status {
                Status::Fail => sev,
                Status::Warn => sev / 2.0,
                Status::Pass => 0.0,
            };
            (earned + sev - loss, max + sev)
        })
}
```

### Redirect Chain Detection

```rust
// Source: https://docs.rs/reqwest/latest/reqwest/redirect/
use reqwest::redirect;
use url::Url;

pub async fn analyze_redirect_chains(
    client: &reqwest::Client,
    url: &Url,
) -> AnalysisResult {
    // Build a one-off client that errors after 3 hops
    let redirect_client = reqwest::Client::builder()
        .redirect(redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 3 {
                attempt.error("redirect chain exceeds 3 hops")
            } else {
                attempt.follow()
            }
        }))
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap();

    match redirect_client.head(url.as_str()).send().await {
        Err(e) if e.is_redirect() => AnalysisResult {
            check: "tech-redirect-chains",
            status: Status::Fail,
            message: "Redirect chain exceeds 3 hops".to_string(),
            recommendation: "Reduce redirect chain to a single canonical redirect".to_string(),
        },
        Ok(resp) if resp.url() != url => AnalysisResult {
            check: "tech-redirect-chains",
            status: Status::Pass,
            message: format!("Redirects to {}", resp.url()),
            recommendation: String::new(),
        },
        Ok(_) => AnalysisResult {
            check: "tech-redirect-chains",
            status: Status::Pass,
            message: "No redirects detected".to_string(),
            recommendation: String::new(),
        },
        Err(_) => AnalysisResult {
            check: "tech-redirect-chains",
            status: Status::Warn,
            message: "Could not check redirect chain (network error)".to_string(),
            recommendation: "Verify URL is accessible".to_string(),
        },
    }
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| structopt for CLI | clap v4 with derive macros | clap v3 (2021) | structopt is maintenance mode; clap absorbed it |
| html5ever directly | scraper (wraps html5ever) | ~2018 | scraper adds CSS selectors; no reason to use html5ever directly |
| xml-rs for XML | quick-xml | ~2020 | 10-50x performance improvement; streaming API |
| valico for JSON Schema | jsonschema | ~2022 | 75-645x performance improvement; modern draft support |

**Deprecated/outdated:**
- `quick-xml 0.38` (CLAUDE.md reference): Current is 0.39.2 on crates.io as of 2026-03-23. The 0.38 → 0.39 migration is minor API evolution; `features = ["serialize"]` pattern unchanged.

---

## Open Questions

1. **reqwest HEAD vs GET for redirect detection**
   - What we know: HEAD requests are lighter; some servers return 405 Method Not Allowed for HEAD.
   - What's unclear: Whether the target sites in typical geodaddy usage support HEAD.
   - Recommendation: Try HEAD first; fall back to GET with `redirect::Policy::none()` if 405 received. For phase 2, HEAD is sufficient — document the fallback limitation.

2. **JSON-LD @context URL variations**
   - What we know: Valid JSON-LD can use `"@context": "https://schema.org"`, `"https://schema.org/"`, or `"http://schema.org"`.
   - What's unclear: How strict to be in phase 2 validation.
   - Recommendation: Accept all three variants as valid. Check for presence of `@context` string containing `"schema.org"` — do not require exact URL match.

3. **reqwest error type for redirect abort**
   - What we know: `attempt.error()` causes the request to return `Err(reqwest::Error)`. The `Error` type has `.is_redirect()` method.
   - What's unclear: Whether a custom error from `attempt.error("msg")` is classified as `.is_redirect() == true` by reqwest.
   - Recommendation: Test this in Wave 0. Fallback: check `err.to_string().contains("redirect")` if `.is_redirect()` returns false for custom errors.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | All compilation | Yes | rustc 1.93.1 (2026-02-11) | — |
| cargo | Build system | Yes | 1.93.1 | — |
| scraper 0.26 | HTML parsing | Not yet installed | To be added | — |
| jsonschema 0.45 | JSON-LD validation | Not yet installed | To be added | — |
| quick-xml 0.39 | Sitemap parsing | Not yet installed | To be added | — |

**Missing dependencies with no fallback:**
- scraper, jsonschema, quick-xml — must be added to cli/Cargo.toml before implementation.

**Missing dependencies with fallback:** None.

---

## Project Constraints (from CLAUDE.md)

The following directives from `CLAUDE.md` are binding on this phase:

| Directive | Impact on Phase 2 |
|-----------|------------------|
| Language: Rust, single binary | All code must be Rust; no runtime scripting |
| Async runtime: tokio with `full` feature | All async functions must be compatible with `#[tokio::main]` |
| HTTP client: reqwest 0.13+ | Must use `rustls` feature (not `native-tls`); already in Cargo.toml as `features = ["rustls"]` |
| HTML parsing: scraper 0.26+ (not html5ever directly) | Use CSS selectors via scraper, not raw html5ever API |
| XML: quick-xml (not xml-rs) | Streaming + serde deserialization; xml-rs is forbidden |
| JSON schema validation: jsonschema (not valico) | Must use jsonschema crate |
| CLI framework: clap 4.6+ with derive | No new CLI flags expected in phase 2 |
| Error handling: anyhow in application code | Analyzer functions that return Results use `anyhow::Result`; no custom error enums |
| Logging: tracing to stderr | No `println!` for diagnostics; use `tracing::debug!` / `tracing::warn!` for internal state |
| Output: JSON-only to stdout | No human-readable output to stdout; all diagnostics to stderr |
| Distribution: local CLI, no cloud dependencies | No external API calls for analysis; all analysis is local HTML/XML inspection |
| No structopt, async-std, ureq, html5ever directly, xml-rs, log crate alone, valico | These are explicitly forbidden |
| Use `cargo build --release` for optimized build | No phase 2 impact |

---

## Sources

### Primary (HIGH confidence)
- scraper 0.26.0 — https://docs.rs/scraper/latest/scraper/ — Html::parse_document, Selector::parse, ElementRef methods
- jsonschema 0.45.0 — https://docs.rs/jsonschema/latest/jsonschema/ — validator_for(), is_valid(), iter_errors()
- quick-xml latest — https://docs.rs/quick-xml/latest/quick_xml/de/index.html — serde deserialization pattern
- reqwest redirect module — https://docs.rs/reqwest/latest/reqwest/redirect/ — Policy::custom(), Attempt::previous()
- Google Sitemap spec — https://developers.google.com/search/docs/crawling-indexing/sitemaps/build-sitemap — 50,000 URL limit, 50MB limit
- Google robots.txt spec — https://developers.google.com/crawling/docs/robots-txt/robots-txt-spec — syntax rules, 500KiB limit

### Secondary (MEDIUM confidence)
- 2026 meta title best practices — multiple SEO sources agree on 50-60 char range
- 2026 meta description best practices — multiple SEO sources agree on 120-158 char range (120 for mobile, 158 for desktop)
- Mixed content elements list — MDN Web Security docs + web.dev/articles/fixing-mixed-content

### Tertiary (LOW confidence)
- Semantic HTML heuristics for CONT-03 (pass/fail thresholds) — derived from SEO best practice articles, not an official spec. Treat threshold recommendations as Claude's discretion per CONTEXT.md.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — crates.io version verification, official docs
- Architecture: HIGH — based on locked CONTEXT.md decisions + verified API docs
- Threshold values (meta tags, sitemap): HIGH — Google official docs + multiple 2026 SEO sources
- Threshold values (redirects, semantic HTML, robots.txt): MEDIUM — Claude's discretion per CONTEXT.md, reasonable heuristics
- Pitfalls: HIGH — based on direct API inspection and known Rust async patterns

**Research date:** 2026-03-23
**Valid until:** 2026-06-23 (stable ecosystem; quick-xml may have patch releases but API is stable)
