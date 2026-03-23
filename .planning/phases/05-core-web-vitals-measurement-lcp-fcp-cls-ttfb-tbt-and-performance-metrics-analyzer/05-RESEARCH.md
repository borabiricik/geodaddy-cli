# Phase 5: Core Web Vitals Measurement - Research

**Researched:** 2026-03-23
**Domain:** Chrome DevTools Protocol, Core Web Vitals, chromiumoxide Rust API
**Confidence:** HIGH

## Summary

Phase 5 adds a `--vitals` flag that launches a chromiumoxide headless browser to measure five performance metrics per crawled page: LCP, FCP, CLS, TTFB, and TBT. The implementation is a new `src/analyzers/performance.rs` module that accepts a `&chromiumoxide::Page` and returns `Vec<AnalysisResult>`. chromiumoxide 0.9 is already in `Cargo.toml` with all required features — no new dependencies needed.

The critical technical finding is that Core Web Vitals **cannot** be retrieved from CDP's `Performance.getMetrics` domain alone. LCP, CLS, and TBT require JavaScript injection via `PerformanceObserver` evaluated in the page context. FCP uses `window.performance.getEntriesByName('first-contentful-paint')`. TTFB uses the `navigation` PerformanceTiming entry's `responseStart`. This "evaluate JS in page" approach is the established lab-measurement pattern used by Playwright, Puppeteer, and k6 Browser.

The scoring integration is straightforward: `CategoryScores` gains a `performance: Option<f64>` field (nullable when `--vitals` not passed), and `calculate_score()` adds `perf_earned`/`perf_max` accumulators that only participate in the overall average when performance checks are present — matching the existing geo score pattern.

**Primary recommendation:** Use `page.evaluate()` with JavaScript PerformanceObserver scripts injected after page navigation completes. One page, one `Browser::launch`, one `page.goto()`, five metric evaluations, return `Vec<AnalysisResult>`.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** New `--vitals` CLI flag (opt-in). Default behavior (no flag) stays fast — reqwest-only, no chromiumoxide launched.
- **D-02:** `--vitals` and `--enable-js` are independent flags. Neither implies the other. Both can be combined, but each does its own thing.
- **D-03:** When `--vitals` is active in a multi-page crawl, every crawled page gets its own independent CWV measurement. The aggregate `performance` score = average across all crawled pages.
- **D-04:** New `performance` category added to `CategoryScores`. Overall score becomes a 4-way average: `(tech + cont + geo + perf) / 4.0`.
- **D-05:** When `--vitals` is NOT passed, `performance` is `null` in JSON output (not omitted, not defaulted to 100.0).
- **D-06:** `calculate_score()` only includes performance in the overall average when performance checks are present — must not penalize users who didn't pass `--vitals`.
- **D-07:** LCP — `perf-lcp` check ID — critical severity (10 pts)
- **D-08:** FCP — `perf-fcp` check ID — warning severity (5 pts)
- **D-09:** CLS — `perf-cls` check ID — warning severity (5 pts)
- **D-10:** TTFB — `perf-ttfb` check ID — warning severity (5 pts)
- **D-11:** TBT — `perf-tbt` check ID — warning severity (5 pts)
- **D-12:** `perf-lcp`: pass ≤2.5s, warn ≤4s, fail >4s
- **D-13:** `perf-fcp`: pass ≤1.8s, warn ≤3s, fail >3s
- **D-14:** `perf-cls`: pass ≤0.1, warn ≤0.25, fail >0.25
- **D-15:** `perf-ttfb`: pass ≤800ms, warn ≤1800ms, fail >1800ms
- **D-16:** `perf-tbt`: pass ≤200ms, warn ≤600ms, fail >600ms

### Claude's Discretion

- How to extract TBT from CDP — calculate from `PerformanceObserver longtask` entries (subtracting 50ms threshold per task) or use `Performance.getEntriesByType("longtask")`.
- Whether to reuse the existing chromiumoxide Browser instance (if `--enable-js` also active) or spawn a dedicated instance for vitals measurement.
- How to handle measurement failures (page timeout, CDP error) — emit a `fail` result with an error message, or skip the metric.
- Whether to run vitals measurement before or after the HTML extraction pass per page.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PERF-01 (new) | `--vitals` flag triggers CWV measurement via chromiumoxide | D-01, D-02: chromiumoxide already in Cargo.toml; `BrowserConfig::builder()` + `Browser::launch()` pattern established in Phase 4 |
| PERF-02 (new) | LCP measured per page, critical severity (10 pts) | D-07, D-12: PerformanceObserver `largest-contentful-paint` type, buffered; last entry `.startTime` ms value |
| PERF-03 (new) | FCP measured per page, warning severity (5 pts) | D-08, D-13: `window.performance.getEntriesByName('first-contentful-paint')[0].startTime` |
| PERF-04 (new) | CLS measured per page, warning severity (5 pts) | D-09, D-14: PerformanceObserver `layout-shift`, sum `entry.value` where `!entry.hadRecentInput` |
| PERF-05 (new) | TTFB measured per page, warning severity (5 pts) | D-10, D-15: Navigation Timing API `performance.getEntriesByType('navigation')[0].responseStart` ms |
| PERF-06 (new) | TBT measured per page, warning severity (5 pts) | D-11, D-16: PerformanceObserver `longtask`, sum `(task.duration - 50)` for each task >50ms |
| PERF-07 (new) | `performance: null` in JSON when `--vitals` not used | D-05: `Option<f64>` + `#[serde(skip_serializing_if = ...)]` NOT used — field present as JSON null |
| PERF-08 (new) | 4-way average score when performance present | D-04, D-06: `perf_max > 0` guard mirrors existing geo score pattern in `calculate_score()` |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| chromiumoxide | 0.9 (already in Cargo.toml) | Headless Chrome CDP automation | Already the project standard; async tokio-based; auto-generated CDP types; fetcher feature handles Chromium download |
| tokio | 1.50 (already in Cargo.toml) | Async runtime | Already the project runtime; `tokio::time::timeout` for measurement timeouts |
| serde_json | 1.0 (already in Cargo.toml) | Parse JS evaluate results | Already used; `EvaluationResult::into_value::<f64>()` for typed numeric extraction |

### No New Dependencies

chromiumoxide 0.9 with `fetcher + zip8 + rustls` features already in `Cargo.toml`. All five metrics can be measured using `page.evaluate()` with injected JavaScript. No additional crates needed.

**Version verification:** All versions confirmed from existing `Cargo.toml` — no changes required.

## Architecture Patterns

### Recommended Project Structure

```
src/
├── analyzers/
│   ├── mod.rs           # Add: pub mod performance;
│   ├── performance.rs   # New: analyze_vitals(page: &Page) -> Vec<AnalysisResult>
│   ├── technical.rs
│   ├── content.rs
│   └── geo.rs
├── scoring.rs           # Extend: CategoryScores + performance field, calculate_score()
├── crawling.rs
└── main.rs              # Extend: Cli + --vitals, crawl loop, aggregate_scores()
```

### Pattern 1: Analyzer Module Structure

**What:** `src/analyzers/performance.rs` exposes one public async function that takes a chromiumoxide `Page` reference and returns a flat `Vec<AnalysisResult>`. This mirrors all existing analyzer modules.

**When to use:** Same call site as all other analyzers — inside the per-page crawl loop in `main.rs`, after HTML extraction.

```rust
// Source: established pattern from src/analyzers/geo.rs, technical.rs
use chromiumoxide::Page;
use crate::scoring::{AnalysisResult, Status};

pub async fn analyze_vitals(page: &Page) -> Vec<AnalysisResult> {
    let mut results = Vec::new();
    results.push(measure_lcp(page).await);
    results.push(measure_fcp(page).await);
    results.push(measure_cls(page).await);
    results.push(measure_ttfb(page).await);
    results.push(measure_tbt(page).await);
    results
}
```

### Pattern 2: JS Evaluation for PerformanceObserver-based Metrics (LCP, CLS, TBT)

**What:** Navigate page with chromiumoxide, then inject JavaScript that uses PerformanceObserver with `buffered: true` to read already-collected entries. The `buffered: true` flag is critical — it allows reading entries captured before the observer was attached, which is the only reliable approach in a post-load evaluation context.

**When to use:** LCP, CLS, and TBT — all require PerformanceObserver with specific entry types.

```javascript
// LCP — Source: Checkly Playwright docs (verified pattern)
() => {
    return new Promise((resolve) => {
        new PerformanceObserver((l) => {
            const entries = l.getEntries();
            const last = entries[entries.length - 1];
            resolve(last ? (last.renderTime || last.loadTime) : -1);
        }).observe({ type: 'largest-contentful-paint', buffered: true });
        // Fallback timeout if no LCP entry found
        setTimeout(() => resolve(-1), 3000);
    });
}
```

```javascript
// CLS — Source: Addyosmani Puppeteer recipes (verified pattern)
() => {
    return new Promise((resolve) => {
        let cls = 0;
        new PerformanceObserver((list) => {
            for (const entry of list.getEntries()) {
                if (!entry.hadRecentInput) { cls += entry.value; }
            }
            resolve(cls);
        }).observe({ type: 'layout-shift', buffered: true });
        setTimeout(() => resolve(cls), 2000);
    });
}
```

```javascript
// TBT — Source: Checkly Playwright docs (verified pattern)
// Long tasks are tasks >50ms; TBT = sum of (task.duration - 50) for each long task
() => {
    return new Promise((resolve) => {
        let tbt = 0;
        new PerformanceObserver((list) => {
            for (const entry of list.getEntries()) {
                tbt += entry.duration - 50;
            }
            resolve(tbt);
        }).observe({ type: 'longtask', buffered: true });
        setTimeout(() => resolve(tbt), 2000);
    });
}
```

### Pattern 3: Navigation Timing for FCP and TTFB

**What:** FCP is available from the Paint Timing API via `performance.getEntriesByName()`. TTFB uses the Navigation Timing API `responseStart` property. Both are synchronous reads available immediately after page load.

**When to use:** FCP and TTFB — synchronous reads, no PerformanceObserver needed.

```javascript
// FCP
() => {
    const entries = performance.getEntriesByName('first-contentful-paint');
    return entries.length > 0 ? entries[0].startTime : -1;
}

// TTFB
() => {
    const nav = performance.getEntriesByType('navigation');
    return nav.length > 0 ? nav[0].responseStart : -1;
}
```

### Pattern 4: chromiumoxide Page Navigation for Vitals

**What:** For the vitals analyzer, navigate fresh to the target URL using the chromiumoxide `Page` API. The page must fully load before metrics can be read.

**When to use:** Each page that needs vitals measurement.

```rust
// Source: chromiumoxide docs.rs + existing Phase 4 pattern in main.rs
let page = browser.new_page(url.as_str()).await?;
// page.goto() is used for already-created pages; new_page() already navigates
// Wait for network idle would be ideal but wait_for_navigation() is sufficient
let page = page.wait_for_navigation().await?;

// Then evaluate JS for each metric
let result = page.evaluate(lcp_js).await?;
let lcp_ms: f64 = result.into_value()?;
```

### Pattern 5: Browser Instance Strategy

**What:** When `--vitals` is active, spawn a dedicated `Browser` instance for performance measurement, separate from the `--enable-js` browser instance. This avoids cross-contamination of page state between JS rendering (which modifies DOM) and vitals measurement (which needs clean navigation).

**When to use:** Whenever both `--vitals` and `--enable-js` are active simultaneously.

```rust
// In main.rs crawl setup
let vitals_browser: Option<Browser> = if cli.vitals {
    let config = BrowserConfig::builder()
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build vitals BrowserConfig: {}", e))?;
    let (b, mut handler) = Browser::launch(config).await?;
    tokio::spawn(async move { while let Some(_) = handler.next().await {} });
    Some(b)
} else {
    None
};
```

### Pattern 6: CategoryScores Extension with Nullable Performance

**What:** Add `performance: Option<f64>` to `CategoryScores`. When `None`, serde serializes to JSON `null` (default behavior — no `skip_serializing_if` annotation). This satisfies D-05 (null in JSON, not omitted).

```rust
// Source: src/scoring.rs pattern
#[derive(Serialize, Debug, Clone)]
pub struct CategoryScores {
    pub technical: f64,
    pub content: f64,
    pub geo: f64,
    pub performance: Option<f64>,  // null when --vitals not passed
}
```

**Note:** `aggregate_scores()` in `crawling.rs` constructs `CategoryScores` inline — this function needs updating to include the `performance` field.

### Pattern 7: calculate_score() 4-Way Average Guard

**What:** Add `perf_earned`/`perf_max` accumulators in `calculate_score()`. Only include performance in the overall average divisor when `perf_max > 0`, matching the existing geo score pattern for defaulting to 100 when no checks are present.

```rust
// Source: existing pattern in src/scoring.rs — geo_max == 0 defaults to 100.0
let perf_score: Option<f64> = if perf_max == 0 {
    None  // No perf checks present — don't include in average
} else {
    Some((perf_earned as f64 / perf_max as f64 * 100.0).clamp(0.0, 100.0))
};

// Overall: only divide by 4 when perf is present
let overall = if let Some(p) = perf_score {
    ((tech_score + cont_score + geo_score + p) / 4.0).clamp(0.0, 100.0)
} else {
    ((tech_score + cont_score + geo_score) / 3.0).clamp(0.0, 100.0)
};
```

### Pattern 8: Error Handling for Metric Failures

**What:** When a metric evaluation fails (CDP error, JS exception, timeout, value -1 sentinel), emit a `fail` result with an explanatory message rather than propagating the error. This keeps the analyzer non-fatal.

**When to use:** Wrap each individual metric measurement in error handling. -1 sentinel from JS means "no data collected."

```rust
async fn measure_lcp(page: &Page) -> AnalysisResult {
    let lcp_ms = match page.evaluate(LCP_JS).await
        .and_then(|r| r.into_value::<f64>().map_err(Into::into)) {
        Ok(v) if v >= 0.0 => v,
        Ok(_) | Err(_) => {
            return AnalysisResult {
                check: "perf-lcp",
                status: Status::Fail,
                message: "LCP could not be measured (page may not have loaded)".to_string(),
                recommendation: "Ensure the page loads a visible content element...".to_string(),
            };
        }
    };
    classify_lcp(lcp_ms)
}
```

### Pattern 9: Measurement Timing — After HTML Extraction

**What:** Run vitals measurement after the reqwest HTML extraction pass. This ensures the page has already been requested by reqwest (TTFB reflects a real request), and the chromiumoxide measurement is a separate fresh navigation for accurate metrics.

**Decision (Claude's Discretion):** Measure after HTML extraction, using a fresh `page.new_page(url).await?` in the vitals analyzer. This keeps the two measurements independent and avoids state pollution.

### Anti-Patterns to Avoid

- **Reusing the `--enable-js` page for vitals measurement:** The `--enable-js` code modifies `html_doc` in-place but doesn't retain the `Page` handle. Additionally, `--enable-js` only creates a page when `needs_js_rendering()` is true, while `--vitals` must measure every page. Use a separate browser instance.
- **Calling `Performance.getMetrics` for LCP/CLS/TBT:** CDP's Performance domain does NOT expose LCP, CLS, or TBT. These require JavaScript PerformanceObserver injection. Using `GetMetricsParams` alone will miss these metrics.
- **Missing `buffered: true` on PerformanceObserver:** Without `buffered: true`, the observer will miss entries already collected before the script runs. This causes LCP, CLS, and longtask entries to appear empty.
- **Blocking on Promise with no timeout fallback:** LCP and TBT Promises can hang if no entries are ever emitted (e.g., a page with no LCP candidate). Always include a `setTimeout(() => resolve(-1), N)` fallback.
- **Using `window.performance.timing` (deprecated):** The `PerformanceTiming` legacy API is deprecated. Use `performance.getEntriesByType('navigation')[0].responseStart` instead.
- **Mixing progress output to stdout:** All `eprintln!` / `tracing` output goes to stderr. Vitals measurement failures must use `tracing::warn!`, not `println!`.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Browser launch/CDP | Custom WebSocket CDP client | `chromiumoxide::Browser::launch()` | Already in Cargo.toml; handles Chromium process lifecycle, handler spawning, CDP framing |
| Chromium download | Manual binary fetching | `BrowserConfig::builder().build()` (fetcher feature) | Auto-detects existing Chrome; falls back to fetching; already configured |
| JS injection for LCP | Custom CDP trace analysis | `page.evaluate()` + PerformanceObserver JS | Established lab-measurement pattern; avoids trace parsing complexity |
| Timeout management | Custom async timeout logic | `tokio::time::timeout()` wrapping per-metric evaluation | tokio already in deps; clean cancellation |
| CLS accumulation | Reading CDP layout-shift trace events | `page.evaluate()` + PerformanceObserver `layout-shift` | Same result, far simpler than trace event parsing |

**Key insight:** All five metrics can be extracted via JavaScript evaluation in the page context. Attempting to use CDP trace-level analysis (Lighthouse approach) would require parsing megabytes of trace JSON — overkill for a single-metric extraction.

## Common Pitfalls

### Pitfall 1: PerformanceObserver Without `buffered: true` Returns No Data

**What goes wrong:** LCP, CLS, and longtask observers return 0 entries because the page already loaded by the time the JavaScript runs.

**Why it happens:** PerformanceObserver by default only fires for new entries after `observe()` is called. Page load entries (LCP, paint, navigation) are buffered by the browser before the observer attaches.

**How to avoid:** Always pass `{ type: '...', buffered: true }` to `.observe()`. The `buffered: true` option delivers previously recorded entries immediately.

**Warning signs:** LCP consistently measures as -1 or 0ms despite pages clearly having content.

### Pitfall 2: Missing Handler Spawn Causes Browser Hang

**What goes wrong:** `Browser::launch()` returns a handler that MUST be spawned as a separate task. Without this, all CDP communication hangs indefinitely.

**Why it happens:** The handler processes all incoming CDP messages. Without polling it, the browser cannot respond to any command.

**How to avoid:** Pattern from Phase 4 (already in codebase):
```rust
tokio::spawn(async move { while let Some(_) = handler.next().await {} });
```

**Warning signs:** First `page.evaluate()` call hangs forever with no error.

### Pitfall 3: `aggregate_scores()` Breaks When `CategoryScores` Gains New Field

**What goes wrong:** `aggregate_scores()` in `crawling.rs` constructs `CategoryScores` structs inline. When `performance: Option<f64>` is added to the struct, all call sites that construct `CategoryScores` with positional or named fields fail to compile unless updated.

**Why it happens:** Rust struct initialization requires all fields to be specified (no default unless `Default` is derived).

**How to avoid:** When adding `performance` field, search all constructors — there are at least two: the `calculate_score()` return in `scoring.rs` and the explicit construction in `main.rs` inside `page_score_tuples` map, plus `aggregate_scores()` in `crawling.rs`.

**Warning signs:** Compile error `missing field 'performance' in initializer of CategoryScores`.

### Pitfall 4: TBT Long Task Subtraction Produces Negative Values

**What goes wrong:** If `longtask` entries have duration exactly equal to 50ms (rare but possible), `duration - 50 = 0`, which is fine. But if the duration is measured slightly below 50ms due to floating-point imprecision, a negative contribution is added to TBT.

**Why it happens:** Long tasks are defined as >50ms by spec. However, floating-point arithmetic near the boundary can produce `entry.duration = 49.999...`.

**How to avoid:** Use `f64::max(0.0, entry.duration - 50.0)` instead of bare subtraction in the TBT accumulation.

**Warning signs:** Negative TBT values in output.

### Pitfall 5: LCP Promise Hangs on Pages With No LCP Candidate

**What goes wrong:** On pages with no visible content elements, the PerformanceObserver for `largest-contentful-paint` never fires. The Promise never resolves, and the `page.evaluate()` call hangs until it times out.

**Why it happens:** LCP only fires when a qualifying element (image, text block, video) is rendered. Empty or error pages have no LCP candidate.

**How to avoid:** Include a `setTimeout(() => resolve(-1), 3000)` fallback inside the LCP Promise. Treat -1 as "LCP not measurable" and emit a fail result.

**Warning signs:** Vitals measurement hangs for several seconds on error pages.

### Pitfall 6: Both `--vitals` and `--enable-js` Launch Chromium Independently

**What goes wrong:** Two `Browser::launch()` calls happen in the same run, which is fine for correctness but doubles the Chromium process overhead.

**Why it happens:** The flags are independent by design (D-02). Each spawns its own browser.

**How to avoid:** This is acceptable per the design. Document in code comments that the two browser instances are intentionally separate. Do not attempt to share state between them.

**Warning signs:** Not a bug — just understand it's expected behavior under `--vitals --enable-js`.

### Pitfall 7: Serde Serializes `None` as Field Omission Unless `Option<f64>` is Used Carefully

**What goes wrong:** If `#[serde(skip_serializing_if = "Option::is_none")]` is accidentally added, `performance` is omitted from JSON instead of being `null` when `--vitals` is not used. This breaks D-05.

**Why it happens:** Common Rust serde pattern for optional fields. But D-05 explicitly requires `null` in JSON, not field omission.

**How to avoid:** Do NOT add `skip_serializing_if` to the `performance` field in `CategoryScores`. The default `Option<f64>` serialization already produces `null` for `None`.

**Warning signs:** `categories` object missing `performance` key instead of showing `"performance": null`.

## Code Examples

### Verified LCP Measurement Pattern (Rust)

```rust
// Source: page.evaluate() API from chromiumoxide docs.rs + JS from Checkly Playwright docs
const LCP_JS: &str = r#"() => {
    return new Promise((resolve) => {
        new PerformanceObserver((l) => {
            const entries = l.getEntries();
            if (entries.length > 0) {
                const last = entries[entries.length - 1];
                resolve(last.renderTime || last.loadTime);
            }
        }).observe({ type: 'largest-contentful-paint', buffered: true });
        setTimeout(() => resolve(-1), 5000);
    });
}"#;

async fn measure_lcp(page: &Page) -> AnalysisResult {
    let lcp_ms = match page.evaluate(LCP_JS).await
        .and_then(|r| r.into_value::<f64>().map_err(Into::into)) {
        Ok(v) if v >= 0.0 => v,
        _ => return fail_result("perf-lcp", "LCP could not be measured"),
    };
    let lcp_s = lcp_ms / 1000.0;
    let (status, msg, rec) = if lcp_s <= 2.5 {
        (Status::Pass, format!("LCP {:.2}s (good, ≤2.5s)", lcp_s), "No action needed.".into())
    } else if lcp_s <= 4.0 {
        (Status::Warn, format!("LCP {:.2}s (needs improvement, 2.5–4s)", lcp_s),
         "Optimize server response time and eliminate render-blocking resources.".into())
    } else {
        (Status::Fail, format!("LCP {:.2}s (poor, >4s)", lcp_s),
         "Critical: reduce LCP by optimizing images, server response, and critical rendering path.".into())
    };
    AnalysisResult { check: "perf-lcp", status, message: msg, recommendation: rec }
}
```

### Verified TTFB Measurement Pattern (JavaScript)

```javascript
// Source: web.dev TTFB article + Playwright/Checkly patterns
() => {
    const nav = performance.getEntriesByType('navigation');
    if (!nav || nav.length === 0) return -1;
    return nav[0].responseStart;
}
```

### Verified CLS Measurement Pattern (JavaScript)

```javascript
// Source: Addyosmani Puppeteer recipes
() => {
    return new Promise((resolve) => {
        let cls = 0;
        new PerformanceObserver((list) => {
            for (const entry of list.getEntries()) {
                if (!entry.hadRecentInput) { cls += entry.value; }
            }
            resolve(cls);
        }).observe({ type: 'layout-shift', buffered: true });
        setTimeout(() => resolve(cls), 2000);
    });
}
```

### Verified TBT Measurement Pattern (JavaScript)

```javascript
// Source: Checkly Playwright docs
// Long task threshold: 50ms. TBT = sum of blocking portions.
() => {
    return new Promise((resolve) => {
        let tbt = 0;
        new PerformanceObserver((list) => {
            for (const entry of list.getEntries()) {
                tbt += Math.max(0, entry.duration - 50);
            }
            resolve(tbt);
        }).observe({ type: 'longtask', buffered: true });
        setTimeout(() => resolve(tbt), 5000);
    });
}
```

### Integration in main.rs Crawl Loop

```rust
// Source: existing pattern in main.rs, adapted for --vitals
// After HTML extraction and analysis:
if cli.vitals {
    if let Some(ref vb) = vitals_browser {
        match vb.new_page(page_url.as_str()).await {
            Ok(vp) => {
                let vitals = analyze_vitals(&vp).await;
                results.extend(vitals);
            }
            Err(e) => tracing::warn!("Vitals measurement failed for {}: {}", page_url, e),
        }
    }
}
```

### calculate_score() Extension

```rust
// Source: existing pattern in src/scoring.rs
// Add to severity_points():
"perf-lcp" => 10,
"perf-fcp" | "perf-cls" | "perf-ttfb" | "perf-tbt" => 5,

// Add in the routing loop:
} else if result.check.starts_with("perf-") {
    perf_earned += earned;
    perf_max += pts;
}

// Add score computation:
let perf_score: Option<f64> = if perf_max == 0 {
    None
} else {
    Some((perf_earned as f64 / perf_max as f64 * 100.0).clamp(0.0, 100.0))
};

// Overall average:
let overall = match perf_score {
    Some(p) => ((tech_score + cont_score + geo_score + p) / 4.0).clamp(0.0, 100.0),
    None    => ((tech_score + cont_score + geo_score) / 3.0).clamp(0.0, 100.0),
};

// Return:
(overall, CategoryScores { technical: tech_score, content: cont_score, geo: geo_score, performance: perf_score })
```

## Project Constraints (from CLAUDE.md)

- **Language:** Rust only — no external scripts, no Node.js helpers for measurement
- **No cloud dependencies:** All measurement runs locally via headless browser
- **JSON-only output:** Performance results as `AnalysisResult` entries with check/status/message/recommendation fields
- **chromiumoxide 0.9** with `fetcher + zip8 + rustls` features — already in Cargo.toml, no version bump needed
- **tokio** async runtime — all async code uses tokio; `tokio::time::timeout` for measurement guards
- **tracing** for diagnostics — vitals failures go to `tracing::warn!` on stderr
- **anyhow** for error handling — analyzer functions return `AnalysisResult` (not `Result`); errors convert to fail-status results internally
- **`AnalysisResult.check` is `&'static str`** — check IDs (`perf-lcp`, etc.) must be string literals, not `String::leak()`

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| FID (First Input Delay) as Core Web Vital | INP (Interaction to Next Paint) replaced FID | March 2024 | The 5 metrics in this phase (LCP, FCP, CLS, TTFB, TBT) are unaffected — FID/INP is NOT in scope |
| `window.performance.timing` legacy API | `performance.getEntriesByType('navigation')[0]` | Deprecated in modern browsers | Use Navigation Timing Level 2 API for TTFB |
| CDP `Performance.getMetrics` for all metrics | PerformanceObserver JS injection for LCP/CLS/TBT | N/A — never supported | CDP Performance domain does not expose LCP, CLS, or TBT |

**Deprecated/outdated:**
- `window.performance.timing` (legacy PerformanceTiming): works but deprecated. Use `getEntriesByType('navigation')[0]` instead.
- First Input Delay (FID): replaced by INP in March 2024. Not in scope for this phase.

## Open Questions

1. **TBT longtask observer reliability in headless lab context**
   - What we know: `longtask` PerformanceObserver with `buffered: true` captures long tasks. The 2-second setTimeout fallback resolves with accumulated total.
   - What's unclear: Whether headless Chromium populates longtask entries for short-lived page loads that complete in <200ms with no actual blocking. Pages may have TBT=0 legitimately.
   - Recommendation: Treat 0ms TBT as a pass (≤200ms threshold). Emit a pass result, not a "could not measure" fail.

2. **LCP measurement timing vs. navigation completion**
   - What we know: `page.wait_for_navigation()` resolves after the document load event. LCP continues being updated after load until user interaction or `visibilitychange`.
   - What's unclear: Whether LCP entries are finalized when `wait_for_navigation()` resolves, or if additional wait time is needed for LCP to stabilize.
   - Recommendation: The `buffered: true` + 5-second fallback timeout pattern is standard in Playwright/Puppeteer lab tests. Accept that lab LCP may differ from field LCP — this is the documented limitation of all lab-based CWV tools including Lighthouse.

3. **`aggregate_scores()` in crawling.rs needs updating**
   - What we know: `aggregate_scores()` constructs `CategoryScores` from per-page tuples.
   - What's unclear: The averaging logic for `performance: Option<f64>` — when only some pages have performance data (shouldn't happen given the current design, but worth specifying).
   - Recommendation: Average only non-None performance values. If all pages have `performance: None`, the aggregate is also `None`. This matches D-03 and D-05.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | Build | ✓ | 1.93.1 | — |
| chromiumoxide (Chromium) | `--vitals` measurement | Auto-downloads | Will fetch on first use | Google Chrome found at `/Applications/Google Chrome.app` — chromiumoxide will detect it |
| Google Chrome | chromiumoxide launch | ✓ | Found at `/Applications/Google Chrome.app` | chromiumoxide fetcher downloads ~150MB Chromium if Chrome not found |
| tokio | Async runtime | ✓ | 1.50 (in Cargo.toml) | — |

**Missing dependencies with no fallback:** None — Chromium is either found locally or fetched automatically.

**Note:** On macOS, chromiumoxide with `fetcher` feature first searches for installed Chrome/Chromium before downloading. Chrome is installed at `/Applications/Google Chrome.app` on the development machine, so the ~150MB download will NOT occur in development. End users without Chrome will trigger the fetcher on first `--vitals` run.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `cargo test` + `assert_cmd` (2.x) + `mockito` (1.x) |
| Config file | None — standard `cargo test` |
| Quick run command | `cargo test --test integration 2>/dev/null` |
| Full suite command | `cargo test 2>/dev/null` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PERF-07 | `performance: null` when `--vitals` not passed | integration (assert_cmd) | `cargo test --test integration test_no_vitals_performance_null` | ❌ Wave 0 |
| PERF-08 | Overall score is 3-way average without `--vitals` | unit (scoring.rs tests) | `cargo test -p geodaddy test_three_way_average_without_perf` | ❌ Wave 0 |
| PERF-08 | Overall score is 4-way average with `--vitals` | unit (scoring.rs tests) | `cargo test -p geodaddy test_four_way_average_with_perf` | ❌ Wave 0 |
| PERF-02/03/04/05/06 | Threshold classification (pass/warn/fail) | unit (performance.rs) | `cargo test -p geodaddy test_lcp_threshold` | ❌ Wave 0 |
| PERF-01 | `--vitals` flag accepted by CLI | integration (assert_cmd) | `cargo test --test integration test_vitals_flag_accepted` | ❌ Wave 0 |
| backward compat | Existing score is 3-way average (no regression) | unit | `cargo test -p geodaddy test_overall_is_average` | ✅ (exists) |

**Note on headless tests:** Full vitals measurement tests (actually launching Chromium and measuring real pages) are impractical in unit tests and should be marked `#[ignore]` or skipped in CI without the `--include-ignored` flag. Unit tests should test the threshold classification logic directly (given a measured value, does it produce the right status?), not the JS evaluation plumbing.

### Sampling Rate
- **Per task commit:** `cargo test 2>/dev/null` (unit tests only — fast, no browser launch)
- **Per wave merge:** `cargo test 2>/dev/null` (full suite)
- **Phase gate:** Full suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] `tests/integration.rs` — add `test_no_vitals_performance_null` test
- [ ] `src/scoring.rs` test module — add `test_three_way_average_without_perf` and `test_four_way_average_with_perf`
- [ ] `src/analyzers/performance.rs` — unit tests for each threshold classification function
- [ ] Framework install: None — `cargo test` already works, `assert_cmd`/`mockito` already in `[dev-dependencies]`

## Sources

### Primary (HIGH confidence)
- [chromiumoxide docs.rs](https://docs.rs/chromiumoxide/latest/chromiumoxide/) — Page::evaluate, Page::evaluate_function, Page::execute method signatures
- [chromiumoxide GitHub README](https://github.com/mattsse/chromiumoxide) — BrowserConfig::builder(), chrome_executable, new_page patterns
- [CDPPerformance domain spec](https://chromedevtools.github.io/devtools-protocol/tot/Performance/) — Confirmed Performance.getMetrics does NOT expose LCP/CLS/TBT
- [CDP PerformanceTimeline domain spec](https://chromedevtools.github.io/devtools-protocol/tot/PerformanceTimeline/) — LCP and CLS available via timelineEventAdded events; confirmed entry types

### Secondary (MEDIUM confidence)
- [Checkly Playwright performance docs](https://www.checklyhq.com/docs/learn/playwright/performance/) — Verified LCP, CLS, TBT PerformanceObserver patterns; confirmed `buffered: true` requirement
- [Addyosmani Puppeteer recipes](https://addyosmani.com/blog/puppeteer-recipes/) — FCP, LCP, CLS, TTFB extraction patterns; CLS accumulation algorithm
- [web.dev TTFB article](https://web.dev/articles/ttfb) — Navigation Timing Level 2 `responseStart` for TTFB
- [Google Core Web Vitals thresholds](https://web.dev/articles/vitals) — Official LCP/FCP/CLS/TTFB/TBT thresholds (confirmed match with D-12–D-16)
- [DebugBear Core Web Vitals docs](https://www.debugbear.com/docs/core-web-vitals-metrics) — Threshold values cross-verification

### Tertiary (LOW confidence)
- None — all critical claims verified from official sources.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — chromiumoxide already in Cargo.toml; all versions confirmed from existing project
- CDP metric extraction approach: HIGH — confirmed via CDP spec that Performance.getMetrics does not expose LCP/CLS/TBT; PerformanceObserver JS injection verified via Playwright/Puppeteer official docs
- JavaScript measurement patterns: HIGH — verified from Checkly (Playwright official docs) and Addyosmani (Puppeteer author)
- Architecture patterns: HIGH — follows existing project conventions exactly
- TBT longtask reliability: MEDIUM — longtask availability in headless lab context is known-functional but results depend on page complexity

**Research date:** 2026-03-23
**Valid until:** 2026-06-23 (stable APIs; CWV thresholds change rarely; chromiumoxide API is stable at 0.9)
