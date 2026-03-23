# Phase 4: Site-Wide Crawling & Polish - Research

**Researched:** 2026-03-23
**Domain:** Rust async web crawling, sitemap parsing, headless browser integration, robots.txt crawl-delay
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Add a top-level `score: f64` and `categories: CategoryScores` to the `Report` struct — simple average across all crawled pages. Per-page scores remain in `pages[]`.
- **D-02:** Top-level `url` field represents the **base URL / site root** (e.g., `https://example.com`) — the starting point of the crawl, not the sitemap URL.
- **D-03:** When sitemap.xml is present, crawl **all URLs listed** by default — no implicit cap.
- **D-04:** Add `--max-pages <N>` CLI flag (optional). When provided, stop after N pages (applies to both sitemap-driven and link-following crawls).
- **D-05:** When sitemap unavailable, fall back to link-following at **depth 2** from the start URL, capped by `--max-pages` if set.
- **D-06:** Deduplicate URLs using a `HashSet<String>` of normalized URLs — skip any URL already visited.
- **D-07:** Print progress to **stderr** — stdout stays pure JSON.
- **D-08:** Format: `[N/TOTAL] <url>` when total is known (sitemap-driven). Format: `Crawling page N... <url>` when total is unknown (link-following). Both use `eprintln!`.
- **D-09:** Detection-based JS rendering: use reqwest first; if rendered page has fewer than 3 headings and no `<p>` elements, treat as JS-rendered and re-fetch via chromiumoxide.
- **D-10:** Detection only activates when `--enable-js` is passed. Without the flag, always use reqwest.
- **D-11:** chromiumoxide auto-downloads Chromium on first run. Document in `--help` text so users aren't surprised.

### Claude's Discretion

- Concurrency model: sequential vs parallel page fetching — Claude decides based on memory constraints and implementation simplicity.
- Rate limiting between requests — implement a sensible default delay (e.g., 1s) or use robots.txt `crawl-delay` if present.
- How sitemap priority ordering is handled (ROADMAP says "priority-based ordering") — Claude implements.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CRAWL-01 | CLI can crawl entire site starting from sitemap.xml | Sitemap URL extraction pattern via quick-xml; `UrlSet`/`UrlEntry` structs already exist in `analyze_sitemap()` — extend to return URL list |
| CRAWL-02 | CLI falls back to link-following if sitemap unavailable | scraper CSS selectors for `a[href]` extraction; depth-BFS loop with `HashSet` dedup (D-05, D-06) |
| CRAWL-04 | CLI has optional JavaScript rendering via headless browser flag | chromiumoxide 0.9.1 available on crates.io; `--enable-js` flag wires into detection logic (D-09, D-10, D-11) |
| CLI-03 | CLI shows progress indicator during site crawl | `eprintln!` to stderr with format per D-07/D-08; stdout remains pure JSON |
</phase_requirements>

---

## Summary

Phase 4 transforms geodaddy from a single-page analyzer into a site-wide crawler. The foundation is already strong: `reqwest::Client` is configured with user-agent and timeouts, `check_robots()` extracts the raw robots.txt body including `crawl-delay`, and `analyze_sitemap()` already parses `UrlSet`/`UrlEntry` structs via `quick-xml` — it just needs to expose those URLs rather than discard them. No new heavyweight dependencies are required for the core crawl loop.

The main new dependency is `chromiumoxide 0.9.1` for the `--enable-js` headless rendering path. This crate is async/tokio-native and auto-downloads Chromium on first use (~150MB). The detection heuristic (fewer than 3 headings AND no `<p>` elements) is simple enough to implement directly against the already-parsed `scraper::Html` document.

The concurrency question (Claude's discretion) should resolve to **sequential crawl with a configurable inter-request delay**. For a CLI tool with no explicit parallelism requirement and a polite-crawling mandate (robots.txt `crawl-delay`), sequential is simpler, more predictable in memory usage, and avoids the risk of overwhelming target servers. Parallel fetching would require a semaphore/channel pattern that adds complexity without a clear user benefit at typical site scales (< 10,000 pages).

**Primary recommendation:** Sequential crawl loop in `main()`, sitemap-first strategy extracting URLs from the existing `UrlSet` deserialization, link-following BFS fallback via scraper `a[href]` selection, `eprintln!` progress to stderr, and chromiumoxide behind the `--enable-js` flag with JS-detection heuristic.

---

## Standard Stack

### Core (all already in Cargo.toml)

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| tokio | 1.50 | Async runtime for sequential async loop | Already present, `full` features |
| reqwest | 0.13 | HTTP fetches for each crawled page | Already present, connection-pooled |
| scraper | 0.26 | `a[href]` extraction for link-following | Already present, CSS selector API |
| quick-xml | 0.39 | Sitemap URL extraction | Already present, `UrlEntry.loc` deserialization |
| robotstxt | 0.3 | `crawl-delay` extraction, per-URL block check | Already present |
| url | 2.5 | URL normalization for dedup `HashSet` | Already present |

### New Dependency (for --enable-js)

| Library | Version | Purpose | Why |
|---------|---------|---------|-----|
| chromiumoxide | 0.9.1 | Headless Chrome for JS-rendered pages | Async/tokio-native, auto-downloads Chromium, comprehensive CDP coverage per CLAUDE.md |

**Installation (only new dep):**
```bash
cargo add chromiumoxide --features fetcher
```

Cargo.toml addition:
```toml
chromiumoxide = { version = "0.9", features = ["fetcher"] }
```

**Note on chromiumoxide feature flags:** The `fetcher` feature enables the auto-download of Chromium. Without it, users must supply their own Chrome/Chromium binary path. Per D-11, the fetcher is the correct choice for zero-setup UX.

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Sequential crawl | tokio::spawn parallel | Parallel is harder to bound memory, adds complexity, violates polite-crawl spirit |
| scraper a[href] extraction | custom regex | scraper uses Servo's standards-compliant parser; regex is fragile on real HTML |
| chromiumoxide fetcher | User-supplied Chrome path | Fetcher avoids "chrome not found" errors; increases first-run time by ~150MB download |

---

## Architecture Patterns

### Recommended Project Structure (Phase 4 additions)

```
src/
├── main.rs              # Cli struct gains --max-pages + --enable-js; main() becomes crawl loop
├── crawler.rs           # NEW: crawl() fn, sitemap extraction, link-following BFS, JS detection
├── scoring.rs           # No change
└── analyzers/
    ├── mod.rs           # No change
    ├── technical.rs     # extract_sitemap_urls() extracted from analyze_sitemap()
    ├── content.rs       # No change
    └── geo.rs           # No change
```

Alternatively, all crawl logic can stay in `main.rs` if the file stays manageable — the codebase is currently 198 lines. Given phase 4 adds ~150-200 lines of crawl logic, splitting into `crawler.rs` is recommended to keep main.rs readable.

### Pattern 1: Sitemap URL Extraction

The existing `analyze_sitemap()` already deserializes `UrlSet { urls: Vec<UrlEntry> }`. Phase 4 needs a companion function that returns the URL list rather than an `AnalysisResult`. Extract the XML deserialization into a shared helper:

```rust
// In analyzers/technical.rs — new pub fn alongside analyze_sitemap()
pub async fn fetch_sitemap_urls(client: &reqwest::Client, base_url: &Url) -> Option<Vec<String>> {
    let mut sitemap_url = base_url.clone();
    sitemap_url.set_path("/sitemap.xml");
    sitemap_url.set_query(None);
    sitemap_url.set_fragment(None);

    let body = client.get(sitemap_url.as_str()).send().await.ok()?
        .text().await.ok()?;

    let url_set: UrlSet = xml_from_str(&body).ok()?;
    Some(url_set.urls.into_iter().map(|e| e.loc).collect())
}
```

The `UrlSet` and `UrlEntry` structs are currently local to `analyze_sitemap()`. They must be promoted to module-level (or duplicated) so both functions can use them.

### Pattern 2: Sitemap Priority Ordering

The Sitemaps protocol allows optional `<priority>` (0.0–1.0) and `<changefreq>` on each `<url>` entry. Per Claude's discretion, sort by priority descending before crawling:

```rust
#[derive(Deserialize, Debug)]
struct UrlEntry {
    loc: String,
    #[serde(default = "default_priority")]
    priority: f64,
}

fn default_priority() -> f64 { 0.5 }

// After fetching url_set.urls, sort:
urls.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal));
```

Confidence: HIGH — `<priority>` is a standard Sitemaps protocol field; quick-xml serde deserialization handles optional fields with `#[serde(default)]`.

### Pattern 3: Link-Following BFS (depth 2 fallback)

```rust
// Pseudocode — implement in crawler.rs or main.rs
async fn collect_links_bfs(client: &reqwest::Client, start: &Url, max_depth: u8) -> Vec<String> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, u8)> = VecDeque::new();
    let mut result: Vec<String> = Vec::new();

    queue.push_back((start.to_string(), 0));

    while let Some((url_str, depth)) = queue.pop_front() {
        if visited.contains(&url_str) || depth > max_depth { continue; }
        visited.insert(url_str.clone());
        result.push(url_str.clone());

        if depth < max_depth {
            // fetch HTML, extract <a href>, filter to same origin, push to queue
            let links = extract_links_from_page(client, &url_str, start).await;
            for link in links {
                if !visited.contains(&link) {
                    queue.push_back((link, depth + 1));
                }
            }
        }
    }
    result
}
```

Key constraint: filter extracted links to same origin only (compare `Url::origin()`) to prevent off-site crawling.

### Pattern 4: Link Extraction via scraper

```rust
// Source: scraper crate CSS selector API (already used throughout codebase)
fn extract_same_origin_links(html: &Html, base: &Url) -> Vec<String> {
    let sel = Selector::parse("a[href]").expect("valid selector");
    html.select(&sel)
        .filter_map(|el| el.value().attr("href"))
        .filter_map(|href| base.join(href).ok())                    // resolve relative URLs
        .filter(|u| u.origin() == base.origin())                    // same-origin only
        .map(|u| {
            let mut norm = u.clone();
            norm.set_fragment(None);                                  // strip #anchors
            norm.to_string()
        })
        .collect()
}
```

### Pattern 5: crawl-delay Extraction

The existing `check_robots()` returns the raw robots.txt body as `String`. Parse `crawl-delay` from it:

```rust
fn extract_crawl_delay(robots_body: &str) -> Option<u64> {
    for line in robots_body.lines() {
        let lower = line.to_lowercase();
        if lower.starts_with("crawl-delay:") {
            if let Some(val) = lower.split(':').nth(1) {
                return val.trim().parse::<u64>().ok();
            }
        }
    }
    None
}
```

Default delay when not specified: 1 second (Claude's discretion per D-39 spirit, polite-crawl default).

### Pattern 6: JS Detection & chromiumoxide Re-fetch

The detection check runs against the already-parsed `scraper::Html` from the initial reqwest fetch:

```rust
fn needs_js_rendering(html: &Html) -> bool {
    let heading_sel = Selector::parse("h1,h2,h3,h4,h5,h6").expect("valid");
    let p_sel = Selector::parse("p").expect("valid");
    let heading_count = html.select(&heading_sel).count();
    let p_count = html.select(&p_sel).count();
    heading_count < 3 && p_count == 0
}
```

chromiumoxide headless fetch pattern (tokio async):

```rust
// Source: chromiumoxide README / crates.io docs
use chromiumoxide::{Browser, BrowserConfig};

async fn fetch_with_headless(url: &str) -> anyhow::Result<String> {
    let (browser, mut handler) = Browser::launch(
        BrowserConfig::builder().build()?
    ).await?;

    let _handler_task = tokio::spawn(async move {
        while let Some(_event) = handler.next().await {}
    });

    let page = browser.new_page(url).await?;
    // Wait for network idle — simple approach: wait for content
    let content = page.content().await?;
    Ok(content)
}
```

**Critical: `handler` must be driven in a separate tokio task.** chromiumoxide requires its event handler to be polled continuously; blocking on it is a common pitfall. The `tokio::spawn` pattern above is the standard approach.

### Pattern 7: Aggregate Score Computation (D-01)

After all pages are crawled and scored:

```rust
// Add to Report struct
#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    url: String,
    crawled_at: String,
    score: f64,             // NEW: average of per-page scores
    categories: CategoryScores,  // NEW: average category scores across pages
    pages: Vec<PageResult>,
}

fn aggregate_scores(pages: &[PageResult]) -> (f64, CategoryScores) {
    if pages.is_empty() {
        return (100.0, CategoryScores { technical: 100.0, content: 100.0, geo: 100.0 });
    }
    let n = pages.len() as f64;
    let score = pages.iter().map(|p| p.score).sum::<f64>() / n;
    let tech = pages.iter().map(|p| p.categories.technical).sum::<f64>() / n;
    let cont = pages.iter().map(|p| p.categories.content).sum::<f64>() / n;
    let geo  = pages.iter().map(|p| p.categories.geo).sum::<f64>() / n;
    (score, CategoryScores { technical: tech, content: cont, geo: geo })
}
```

The `--fail-under` check in `main()` should compare against the **top-level** `score`, not per-page scores.

### Anti-Patterns to Avoid

- **Tokio blocking in async context:** Never call `std::thread::sleep()` in an async function — use `tokio::time::sleep(Duration::from_secs(1)).await` for the crawl delay.
- **chromiumoxide handler not spawned:** If the event handler task is not polled, all CDP commands will hang indefinitely. Always `tokio::spawn` the handler loop.
- **Deduplication by raw string only:** URLs like `https://example.com/page` and `https://example.com/page/` are different strings but the same page. Normalize by parsing with `Url`, stripping trailing slash from path, and removing fragments before inserting into `HashSet`.
- **Mixing stdout progress with JSON:** All user-facing crawl progress MUST go to `eprintln!` (stderr), never `println!`. The JSON report is emitted at the end after all pages are crawled.
- **Re-fetching robots.txt per page:** Fetch robots.txt once at crawl start for the origin, cache the body string, then check each discovered URL against the cached matcher.
- **Off-site link following:** Always filter discovered links to `url.origin() == base_url.origin()`. Without this, depth-2 BFS will crawl the entire web.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Relative URL resolution | String concatenation | `base_url.join(href)` from `url` crate | Handles `../`, `//`, protocol-relative, absolute URLs correctly |
| robots.txt crawl-delay parsing | Custom line parser | Simple split on `:` (single field, no ambiguity) | robots.txt crawl-delay format is trivial; no need for full RFC parser |
| HTML anchor extraction | Regex on raw HTML | `scraper` CSS `a[href]` selector | scraper handles malformed HTML, nested tags, and HTML entities |
| Sitemap XML parsing | String search for `<loc>` | `quick-xml` serde deserialization | Already in Cargo.toml; handles namespaces, encoding, CDATA |
| JS-detection threshold | Complex heuristics | Heading count < 3 AND no `<p>` (per D-09) | Simple, fast, already decided. Resist over-engineering. |
| Concurrency throttle | Custom semaphore | tokio::time::sleep between requests (sequential) | Sequential is sufficient for phase 4 scope; governor crate needed only if parallelizing |

**Key insight:** The hardest parts (URL parsing, HTML parsing, XML parsing, robots.txt) are already handled by crates already in `Cargo.toml`. Phase 4 is primarily wiring existing tools together into a loop.

---

## Runtime State Inventory

Step 2.5 SKIPPED — this is a greenfield feature addition (site-wide crawl), not a rename/refactor/migration phase. No stored data, live service config, or OS-registered state is affected.

---

## Environment Availability Audit

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | All compilation | ✓ | rustc 1.93.1 (exceeds MSRV 1.83 for jsonschema) | — |
| cargo | Build system | ✓ | 1.93.1 | — |
| chromiumoxide | --enable-js flag | ✓ (crates.io) | 0.9.1 | Feature is opt-in; without --enable-js, Chromium never downloads |
| Chromium browser | --enable-js at runtime | Downloads on demand | ~150MB first-run | Documented in --help per D-11 |

**Missing dependencies with no fallback:** None — chromiumoxide is behind an opt-in flag.

**Note on Chromium auto-download:** chromiumoxide's `fetcher` feature downloads a compatible Chromium revision on first `Browser::launch()`. This is fine for the CLI use case but adds ~150MB to the user's home directory (`~/.cache/chromiumoxide/` or similar). The `--help` text must warn users per D-11.

---

## Common Pitfalls

### Pitfall 1: chromiumoxide Handler Not Driven

**What goes wrong:** All `page.content()`, `browser.new_page()`, and other CDP calls hang indefinitely.
**Why it happens:** chromiumoxide's `Browser::launch()` returns a `(Browser, BrowserHandler)` tuple. `BrowserHandler` is a future that must be polled to process CDP events. If it's never awaited or spawned, the event loop stops and all commands stall.
**How to avoid:** Always `tokio::spawn(async move { while let Some(_) = handler.next().await {} })` immediately after `Browser::launch()`.
**Warning signs:** `page.content().await` never returns; program appears frozen.

### Pitfall 2: Using std::thread::sleep in Async Code

**What goes wrong:** The entire tokio runtime thread is blocked during the crawl delay, preventing all other async tasks from progressing.
**Why it happens:** `std::thread::sleep` is a blocking call that doesn't yield to the tokio executor.
**How to avoid:** Use `tokio::time::sleep(Duration::from_secs(delay)).await` for inter-request delays.
**Warning signs:** Performance degrades significantly when --enable-js is active and chromiumoxide tasks are spawned alongside the sleep delay.

### Pitfall 3: URL Deduplication by Raw String

**What goes wrong:** The same page is crawled multiple times because `https://example.com/page` and `https://example.com/page/` (trailing slash) are different `HashSet` entries.
**Why it happens:** String-based comparison is exact; URL semantics are not captured.
**How to avoid:** Normalize URLs before insertion: parse with `Url::parse()`, strip trailing slash from path if path is longer than `/`, remove fragment, then convert back to `String` for the `HashSet`.
**Warning signs:** Duplicate `url` values in `pages[]` array.

### Pitfall 4: Sitemap UrlSet/UrlEntry Structs Are Currently Private

**What goes wrong:** Attempting to call `fetch_sitemap_urls()` from `main.rs` fails to compile because `UrlSet` and `UrlEntry` are defined as local structs inside `analyze_sitemap()`.
**Why it happens:** The current implementation scopes these structs to the function body since they were only needed for validation, not URL extraction.
**How to avoid:** Promote `UrlSet` and `UrlEntry` to module-level in `technical.rs` before extracting the URL-fetching helper function.
**Warning signs:** Compiler error "cannot find type `UrlSet` in this scope".

### Pitfall 5: Off-Site Link Following

**What goes wrong:** BFS link-following crawls third-party domains, massively expanding the crawl scope.
**Why it happens:** Raw `href` extraction without origin filtering includes `https://twitter.com/...`, `https://fonts.googleapis.com/...`, etc.
**How to avoid:** Filter all extracted links: `url.origin() == base_url.origin()`. The `url` crate's `Origin` type implements `PartialEq`.
**Warning signs:** `pages[]` array contains URLs from domains other than the input URL.

### Pitfall 6: --fail-under Applied to Wrong Score

**What goes wrong:** `--fail-under` exits 1 even when the overall site score passes, because it's compared against `pages[0].score` instead of the top-level `report.score`.
**Why it happens:** The current code checks `overall_score` which was the single-page score; the variable must be updated to use the new aggregate score.
**How to avoid:** Compute aggregate score after all pages are crawled; store in `report.score`; compare that value against `cli.fail_under`.

### Pitfall 7: robots.txt Per-Page Check vs. Per-Origin

**What goes wrong:** robots.txt is fetched for every crawled page, making N HTTP requests to `/robots.txt` for N pages.
**Why it happens:** Naively calling `check_robots()` inside the page loop.
**How to avoid:** Call `check_robots()` once before the crawl loop (already done in the current single-page flow). Cache `(robots_blocked_for_base, robots_body)`. For each discovered URL, check it against the same `DefaultMatcher` and cached body. The `robotstxt` crate's `one_agent_allowed_by_robots()` takes the body string and URL — no re-fetch needed.

---

## Code Examples

### Sitemap URL Fetch + Priority Sort

```rust
// Source: quick-xml serde + sitemaps.org protocol
#[derive(serde::Deserialize, Debug)]
struct UrlSet {
    #[serde(default, rename = "url")]
    urls: Vec<UrlEntry>,
}

#[derive(serde::Deserialize, Debug)]
struct UrlEntry {
    loc: String,
    #[serde(default = "default_priority")]
    priority: f64,
}

fn default_priority() -> f64 { 0.5 }

// Sort descending by priority before crawling
url_set.urls.sort_by(|a, b| {
    b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal)
});
```

### Inter-Request Delay (async, polite crawling)

```rust
// Source: tokio::time — correct pattern for async sleep
use std::time::Duration;
use tokio::time::sleep;

// Inside crawl loop, after each page fetch:
let delay_secs = crawl_delay_from_robots.unwrap_or(1);
sleep(Duration::from_secs(delay_secs)).await;
```

### Same-Origin Link Extraction

```rust
// Source: scraper CSS selector API + url crate join/origin
use scraper::{Html, Selector};
use url::Url;

fn extract_same_origin_links(html: &Html, base: &Url) -> Vec<String> {
    let sel = Selector::parse("a[href]").expect("valid selector");
    html.select(&sel)
        .filter_map(|el| el.value().attr("href"))
        .filter_map(|href| base.join(href).ok())
        .filter(|u| u.origin() == base.origin())
        .map(|mut u| { u.set_fragment(None); u.to_string() })
        .collect()
}
```

### chromiumoxide Browser Launch (correct handler spawning)

```rust
// Source: chromiumoxide crate README pattern
use chromiumoxide::{Browser, BrowserConfig};
use futures::StreamExt;  // for handler.next()

async fn launch_browser() -> anyhow::Result<Browser> {
    let (browser, mut handler) = Browser::launch(
        BrowserConfig::builder().build()?
    ).await?;

    // CRITICAL: handler MUST be spawned, not awaited inline
    tokio::spawn(async move {
        while let Some(_event) = handler.next().await {}
    });

    Ok(browser)
}

async fn fetch_html_headless(browser: &Browser, url: &str) -> anyhow::Result<String> {
    let page = browser.new_page(url).await?;
    let content = page.content().await?;
    Ok(content)
}
```

Note: `futures::StreamExt` is needed for `.next()` on `BrowserHandler`. Add `futures = "0.3"` to Cargo.toml.

### Progress Output to stderr

```rust
// Sitemap-driven (total known):
eprintln!("[{}/{}] {}", current_index, total_count, url);

// Link-following (total unknown):
eprintln!("Crawling page {}... {}", current_index, url);
```

### URL Normalization for HashSet Dedup

```rust
fn normalize_url(url_str: &str) -> Option<String> {
    let mut u = Url::parse(url_str).ok()?;
    u.set_fragment(None);
    // Strip trailing slash from non-root paths
    let path = u.path().to_string();
    if path.len() > 1 && path.ends_with('/') {
        u.set_path(path.trim_end_matches('/'));
    }
    Some(u.to_string())
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| reqwest-middleware for rate limiting | tokio::time::sleep (sequential) | Phase 4 decision | Sequential is simpler for single-threaded crawl; reqwest-middleware only needed for parallel |
| phantomjs/casperjs headless | chromiumoxide (CDP-based) | ~2020 | CDP gives full async API; phantomjs is abandoned |
| WebDriver (selenium/fantoccini) | chromiumoxide | 2022-2023 | CDP avoids WebDriver server overhead; chromiumoxide is tokio-native |

**Deprecated/outdated in this context:**
- `structopt`: Merged into clap; already using clap with derive macros.
- Synchronous HTTP clients (`ureq`, `attohttpc`): Incompatible with the tokio async crawl loop.

---

## Open Questions

1. **chromiumoxide fetcher download location**
   - What we know: `chromiumoxide_fetcher` downloads Chromium automatically on `Browser::launch()`.
   - What's unclear: The exact cache directory path (may vary by OS). Likely `~/.cache/chromiumoxide` on Linux/macOS.
   - Recommendation: Accept the default; document in `--help` that Chromium is auto-downloaded to a local cache directory.

2. **Sitemap index files (sitemapindex)**
   - What we know: Large sites use `<sitemapindex>` XML with `<sitemap><loc>...</loc></sitemap>` entries pointing to child sitemaps.
   - What's unclear: Phase 4 scope — D-03 says "crawl all URLs listed in sitemap.xml" without specifying sitemap index handling.
   - Recommendation: Handle single-level sitemap only (parse `<urlset>`). If the root `/sitemap.xml` is a `<sitemapindex>`, log a warning and fall through to link-following. Sitemap index support can be v2.

3. **`futures` crate as new dependency**
   - What we know: chromiumoxide's `BrowserHandler` implements `Stream`, requiring `futures::StreamExt` for `.next()`.
   - What's unclear: Whether `futures` is already transitively available without being in Cargo.toml.
   - Recommendation: Add `futures = "0.3"` explicitly to Cargo.toml to avoid relying on transitive availability.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + `#[tokio::test]` for async |
| Config file | `Cargo.toml` (no separate config needed) |
| Quick run command | `cargo test` |
| Full suite command | `cargo test --all` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CRAWL-01 | `fetch_sitemap_urls()` returns parsed URL list | unit | `cargo test test_fetch_sitemap_urls` | ❌ Wave 0 |
| CRAWL-01 | Priority sort: high priority URLs come first | unit | `cargo test test_sitemap_priority_sort` | ❌ Wave 0 |
| CRAWL-02 | Link extractor returns same-origin `a[href]` only | unit | `cargo test test_extract_same_origin_links` | ❌ Wave 0 |
| CRAWL-02 | Off-site links are filtered out | unit | `cargo test test_offsite_links_filtered` | ❌ Wave 0 |
| CRAWL-02 | Relative links are resolved against base URL | unit | `cargo test test_relative_link_resolution` | ❌ Wave 0 |
| CRAWL-04 | `needs_js_rendering()` returns true for sparse HTML | unit | `cargo test test_js_detection_sparse_html` | ❌ Wave 0 |
| CRAWL-04 | `needs_js_rendering()` returns false for content-rich HTML | unit | `cargo test test_js_detection_rich_html` | ❌ Wave 0 |
| CLI-03 | Progress prints to stderr, not stdout | smoke | `cargo test` + manual verify with `2>/dev/null` pipe | ❌ Wave 0 |
| D-06 | URL dedup via `normalize_url()` strips fragments and trailing slashes | unit | `cargo test test_url_normalization` | ❌ Wave 0 |
| D-01 | Aggregate score is average of per-page scores | unit | `cargo test test_aggregate_score` | ❌ Wave 0 |

**Note on CRAWL-04 (chromiumoxide integration):** The actual headless browser launch cannot be unit-tested without Chromium available. Unit tests cover only the `needs_js_rendering()` detection logic (pure HTML analysis). The chromiumoxide fetch path is tested manually during integration testing.

### Sampling Rate

- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test --all`
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps

- [ ] `src/analyzers/technical.rs` — add `#[cfg(test)]` block with `test_fetch_sitemap_urls`, `test_sitemap_priority_sort`
- [ ] `src/crawler.rs` (if extracted) OR `src/main.rs` — add tests for `extract_same_origin_links`, `test_relative_link_resolution`, `test_offsite_links_filtered`, `needs_js_rendering`, `normalize_url`, `aggregate_score`
- [ ] No new test infrastructure needed — existing `#[test]` + `#[tokio::test]` pattern is sufficient

---

## Project Constraints (from CLAUDE.md)

All directives from `CLAUDE.md` that the planner must verify compliance with:

| Directive | Constraint |
|-----------|------------|
| Language | Rust only — no shell scripts, Python helpers, or Node.js tooling for implementation |
| Distribution | Local CLI, no cloud dependencies — chromiumoxide's auto-download is local (acceptable) |
| Output | JSON to stdout only — all progress must go to stderr via `eprintln!` |
| Crawling | Must handle localhost URLs — `Url::parse()` already handles `http://localhost:*` |
| HTTP client | Use reqwest (async) — no ureq, no custom socket code |
| Async runtime | Tokio only — no async-std |
| HTML parsing | scraper only — no direct html5ever, no regex-on-HTML |
| XML parsing | quick-xml only — no xml-rs |
| Headless browser | chromiumoxide only — no rust-headless-chrome (sync), no fantoccini (WebDriver) |
| CLI parsing | clap with derive macros — no structopt |
| Error handling | anyhow for application code |
| Logging | tracing + tracing-subscriber — no bare `log` crate |
| robots.txt | robotstxt crate (Google algorithm port) — matches Googlebot behavior |
| GSD Workflow | All implementation must go through a GSD command; no direct repo edits outside GSD workflow |

---

## Sources

### Primary (HIGH confidence)

- Existing `src/main.rs`, `src/analyzers/technical.rs` — direct inspection of `UrlSet`/`UrlEntry` struct definitions, `check_robots()` return signature, reqwest client configuration
- `Cargo.toml` — confirmed present: tokio 1.50, reqwest 0.13, scraper 0.26, quick-xml 0.39, url 2.5, robotstxt 0.3
- `CLAUDE.md` — authoritative stack decisions for this project
- `04-CONTEXT.md` — locked implementation decisions D-01 through D-11
- cargo search output — confirmed chromiumoxide 0.9.1 on crates.io (2026-03-23)
- Rust toolchain — confirmed rustc 1.93.1 installed (exceeds all MSRV requirements)

### Secondary (MEDIUM confidence)

- chromiumoxide README pattern for Browser::launch + handler spawning — documented pattern across multiple community examples; handler spawn requirement is documented in chromiumoxide crate
- Sitemaps.org protocol for `<priority>` field — standard protocol, `<priority>` is optional with default 0.5

### Tertiary (LOW confidence)

- chromiumoxide cache directory location (`~/.cache/chromiumoxide`) — inferred from chromiumoxide_fetcher crate behavior; exact path not verified against current version docs

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all core libraries verified in Cargo.toml; chromiumoxide version confirmed on crates.io
- Architecture: HIGH — patterns derived from existing codebase conventions + locked decisions in CONTEXT.md
- Pitfalls: HIGH — derived from direct code inspection (private struct issue, existing `check_robots` return signature) and well-known tokio/chromiumoxide gotchas
- Test map: HIGH — unit-testable logic clearly identified; chromiumoxide integration correctly scoped to manual testing

**Research date:** 2026-03-23
**Valid until:** 2026-04-23 (stable libraries; chromiumoxide API stable since 0.9.x)
