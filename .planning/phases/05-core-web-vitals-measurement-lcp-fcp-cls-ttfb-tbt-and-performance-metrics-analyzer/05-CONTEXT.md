# Phase 5: Core Web Vitals Measurement - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Add a `--vitals` flag that triggers headless browser measurement of Core Web Vitals (LCP, FCP, CLS, TTFB, TBT) for every crawled page. Results surface as scored `AnalysisResult` entries in a new `performance` scoring category. The phase adds one new CLI flag, one new analyzer module, and extends `CategoryScores` + `calculate_score()`.

Does NOT add: new output formats, changes to existing analyzer behavior, or modifications to the --enable-js JS rendering flow.

</domain>

<decisions>
## Implementation Decisions

### Triggering Mechanism
- **D-01:** New `--vitals` CLI flag (opt-in). Default behavior (no flag) stays fast — reqwest-only, no chromiumoxide launched.
- **D-02:** `--vitals` and `--enable-js` are **independent** flags. Neither implies the other. Both can be combined (e.g., `--vitals --enable-js`), but each does its own thing: `--enable-js` controls JS fallback rendering for HTML extraction; `--vitals` controls CDP performance measurement.
- **D-03:** When `--vitals` is active in a multi-page crawl, every crawled page gets its own independent CWV measurement. The aggregate `performance` score = average across all crawled pages.

### Scoring Category
- **D-04:** New `performance` category added to `CategoryScores`. Overall score becomes a 4-way average: `(tech + cont + geo + perf) / 4.0`.
- **D-05:** When `--vitals` is NOT passed, `performance` is `null` in JSON output (not omitted, not defaulted to 100.0). This keeps the schema shape consistent while clearly signaling "not measured."
- **D-06:** `calculate_score()` only includes performance in the overall average when performance checks are present — it must not penalize users who didn't pass `--vitals`.

### Metrics Included
All five Core Web Vitals are included:
- **D-07:** LCP (Largest Contentful Paint) — `perf-lcp` check ID — **critical severity (10 pts)**
- **D-08:** FCP (First Contentful Paint) — `perf-fcp` check ID — warning severity (5 pts)
- **D-09:** CLS (Cumulative Layout Shift) — `perf-cls` check ID — warning severity (5 pts)
- **D-10:** TTFB (Time to First Byte) — `perf-ttfb` check ID — warning severity (5 pts)
- **D-11:** TBT (Total Blocking Time) — `perf-tbt` check ID — warning severity (5 pts)

### Thresholds (Google's official CWV thresholds)
- **D-12:** `perf-lcp`: pass ≤2.5s, warn ≤4s, fail >4s
- **D-13:** `perf-fcp`: pass ≤1.8s, warn ≤3s, fail >3s
- **D-14:** `perf-cls`: pass ≤0.1, warn ≤0.25, fail >0.25
- **D-15:** `perf-ttfb`: pass ≤800ms, warn ≤1800ms, fail >1800ms
- **D-16:** `perf-tbt`: pass ≤200ms, warn ≤600ms, fail >600ms

### Claude's Discretion
- How to extract TBT from CDP — calculate from `PerformanceTiming` events or use `Performance.getEntriesByType("longtask")`. Claude decides based on what chromiumoxide exposes.
- Whether to reuse the existing chromiumoxide Browser instance (if `--enable-js` also active) or spawn a dedicated instance for vitals measurement.
- How to handle measurement failures (page timeout, CDP error) — emit a `fail` result with an error message, or skip the metric.
- Whether to run vitals measurement before or after the HTML extraction pass per page.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Context
- `.planning/REQUIREMENTS.md` — TBD requirements for Phase 5 (not yet written)
- `.planning/ROADMAP.md` — Phase 5 goal and success criteria
- `CLAUDE.md` — Tech stack. chromiumoxide 0.9 (fetcher+zip8+rustls), tokio async runtime, tracing for logging

### Existing Code (integration points)
- `src/main.rs` — `Cli` struct (add `--vitals: bool`), `Report` struct (add `performance: Option<f64>` to categories), crawl loop (add vitals measurement call per page)
- `src/scoring.rs` — `CategoryScores` struct (add `performance: Option<f64>`), `severity_points()` (add `perf-*` entries), `calculate_score()` (4-way average when perf present)
- `src/analyzers/mod.rs` — Add `pub mod performance;`
- `Cargo.toml` — chromiumoxide already present with fetcher+zip8+rustls features; no new deps expected

### Phase 4 Context (chromiumoxide integration patterns)
- `.planning/phases/04-site-wide-crawling-polish/04-CONTEXT.md` — D-09 through D-11: how chromiumoxide is launched, JS detection thresholds, --enable-js behavior

No external specs beyond Google's CWV thresholds — all captured in D-12 through D-16 above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `chromiumoxide` already in `Cargo.toml` with full feature set — no new dependencies needed
- `Browser` + `BrowserConfig` already imported in `main.rs` from Phase 4 `--enable-js` work
- `calculate_score()` in `scoring.rs` already handles optional categories (geo defaults to 100 when no geo checks) — same pattern applies to performance when `--vitals` not passed

### Established Patterns
- Flat module structure: new file `src/analyzers/performance.rs` with public functions returning `Vec<AnalysisResult>`
- Check IDs: `perf-lcp`, `perf-fcp`, `perf-cls`, `perf-ttfb`, `perf-tbt`
- Severity routing: check `starts_with("perf-")` in `calculate_score()` to route to `perf_earned`/`perf_max`
- All analyzers called sequentially per page in crawl loop in `main.rs`
- Progress to stderr; JSON to stdout (no mixing)

### Integration Points
- `CategoryScores`: add `performance: Option<f64>` field (nullable for when `--vitals` not used)
- `Cli`: add `--vitals: bool` flag
- `calculate_score()`: add `perf_earned`/`perf_max` accumulators; include perf in overall average only when perf checks present
- `severity_points()`: add `"perf-lcp" => 10`, `"perf-fcp" | "perf-cls" | "perf-ttfb" | "perf-tbt" => 5`
- Per-page crawl loop: when `cli.vitals`, call performance analyzer after HTML extraction

</code_context>

<specifics>
## Specific Ideas

- `--vitals` note in `--help` should mirror the chromiumoxide note: "Note: --vitals uses Chromium for measurement (~150MB download on first use)"
- `performance: null` in JSON makes it easy for consumers to check: `if report.categories.performance is None { /* not measured */ }`
- The 4-way average should only activate when performance checks exist in the results — this ensures backward compatibility for existing single-page runs without `--vitals`

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 05-core-web-vitals-measurement-lcp-fcp-cls-ttfb-tbt-and-performance-metrics-analyzer*
*Context gathered: 2026-03-23*
