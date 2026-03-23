# Phase 4: Site-Wide Crawling & Polish - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Multi-page site crawling with sitemap-first strategy and link-following fallback. Adds a `--max-pages` cap flag, progress reporting to stderr, and optional `--enable-js` headless rendering for JS-heavy pages. Produces one report with site-level aggregate score plus individual per-page entries.

Does NOT add: new analyzers, new output formats, web UI, or persistent storage.

</domain>

<decisions>
## Implementation Decisions

### Aggregate Scoring
- **D-01:** Add a top-level `score: f64` and `categories: CategoryScores` to the `Report` struct — simple average across all crawled pages. Per-page scores remain in `pages[]`.
- **D-02:** Top-level `url` field represents the **base URL / site root** (e.g., `https://example.com`) — the starting point of the crawl, not the sitemap URL.

### Crawl Limits & Scope
- **D-03:** When sitemap.xml is present, crawl **all URLs listed** by default — no implicit cap.
- **D-04:** Add `--max-pages <N>` CLI flag (optional). When provided, stop after N pages (applies to both sitemap-driven and link-following crawls).
- **D-05:** When sitemap unavailable, fall back to link-following at **depth 2** from the start URL, capped by `--max-pages` if set.
- **D-06:** Deduplicate URLs using a `HashSet<String>` of normalized URLs — skip any URL already visited.

### Progress Indicator
- **D-07:** Print progress to **stderr** — stdout stays pure JSON.
- **D-08:** Format: `[N/TOTAL] <url>` when total is known (sitemap-driven). Format: `Crawling page N... <url>` when total is unknown (link-following). Both use `eprintln!`.

### JS Rendering (--enable-js)
- **D-09:** Detection-based: use reqwest to fetch each page first; if the rendered page has **fewer than 3 headings and no `<p>` elements**, treat it as JS-rendered and re-fetch via chromiumoxide.
- **D-10:** Detection only activates when `--enable-js` is passed. Without the flag, always use reqwest (no headless browser).
- **D-11:** chromiumoxide auto-downloads Chromium on first run. Document this in `--help` text so users aren't surprised.

### Claude's Discretion
- Concurrency model: sequential vs parallel page fetching — Claude decides based on memory constraints and implementation simplicity.
- Rate limiting between requests — implement a sensible default delay (e.g., 1s) or use robots.txt `crawl-delay` if present.
- How sitemap priority ordering is handled (ROADMAP says "priority-based ordering") — Claude implements.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Context
- `.planning/REQUIREMENTS.md` — CRAWL-01, CRAWL-02, CRAWL-04, CLI-03 are the Phase 4 requirements
- `.planning/ROADMAP.md` — Phase 4 success criteria (5 items)
- `CLAUDE.md` — Tech stack choices, reqwest/tokio/chromiumoxide guidance, robots.txt crate

### Existing Code (reuse patterns from these files)
- `src/main.rs` — Current HTTP client setup, `check_robots()`, `Report`/`PageResult` structs
- `Cargo.toml` — Existing deps (reqwest, tokio, quick-xml already present)

No external specs — requirements fully captured in decisions above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `reqwest::Client` built in `main()` with user_agent, timeout, connect_timeout — reuse as-is for all page fetches
- `check_robots()` returns `(bool, String)` — already extracts crawl-delay info from robots.txt body
- `analyze_sitemap()` in `analyzers/technical.rs` — already fetches and validates sitemap.xml; Phase 4 needs to extend this to extract URL list
- `quick-xml` already in `Cargo.toml` — sitemap URL extraction doesn't need new deps

### Established Patterns
- All analyzer results collected into `Vec<AnalysisResult>`, then scored via `calculate_score()` — same pattern applies per crawled page
- `tracing::warn!` / `tracing::info!` used for diagnostics; `eprintln!` fits for user-facing progress
- JSON to stdout is strict — no mixing with progress output

### Integration Points
- `Report` struct needs `score: f64` and `categories: CategoryScores` added at the top level
- `Cli` struct needs `--max-pages: Option<usize>` and `--enable-js: bool` added
- The single-page flow in `main()` becomes a loop over discovered URLs

</code_context>

<specifics>
## Specific Ideas

- Progress format confirmed: `[1/42] https://example.com/about` for sitemap-driven, `Crawling page 3... https://example.com/about` for link-following
- JS detection threshold: fewer than 3 headings AND no `<p>` elements → trigger headless re-fetch
- chromiumoxide note in --help: "Note: --enable-js downloads Chromium (~150MB) on first use"

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 04-site-wide-crawling-polish*
*Context gathered: 2026-03-23*
