# Phase 4: Site-Wide Crawling & Polish - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-23
**Phase:** 04-site-wide-crawling-polish
**Areas discussed:** Aggregate scoring, Crawl limits & scope, Progress indicator style, JS rendering scope

---

## Aggregate Scoring

| Option | Description | Selected |
|--------|-------------|----------|
| Site-level score + per-page scores | Add top-level score/categories averaged across all pages. Per-page scores remain. | ✓ |
| Per-page only — no aggregation | Keep current structure, callers aggregate themselves. | |
| Weighted by page importance | Use sitemap `<priority>` for weighted site score. | |

**User's choice:** Site-level score + per-page scores
**Notes:** Top-level `url` field = base URL / site root (the starting point of the crawl).

---

## Crawl Limits & Scope

| Option | Description | Selected |
|--------|-------------|----------|
| All pages in sitemap | Crawl every URL listed by default. | ✓ |
| Default cap of 50 pages | Crawl up to 50 unless --max-pages overrides. | |
| Require --max-pages explicitly | No crawl without explicit limit. | |

**Crawl flags:**
- `--max-pages` flag: Yes, add it (optional cap, applies to both sitemap and link-following)
- Link-following depth: Depth 2 with `--max-pages` cap
- Deduplication: Yes — HashSet of normalized URLs

---

## Progress Indicator Style

| Option | Description | Selected |
|--------|-------------|----------|
| Simple URL log to stderr | Print each URL: `[1/42] https://example.com/about` | ✓ |
| Silent | No progress output. | |
| Counter only | `Crawling page 3/42...` without URL. | |

**Format decision:** Total when known (`[N/TOTAL] url`), running count when unknown (`Crawling page N... url`).

---

## JS Rendering Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Apply to ALL pages | Every page via headless Chrome. | |
| Detect JS-heavy pages automatically | Use reqwest first; if thin HTML, switch to headless. | ✓ |
| You decide | Claude picks approach. | |

**Detection threshold chosen:** Fewer than 3 headings AND no `<p>` elements → trigger headless re-fetch.
**Notes:** Only activates with `--enable-js` flag. chromiumoxide downloads Chromium on first run — document in --help.

---

## Claude's Discretion

- Concurrency model (sequential vs parallel page fetching)
- Rate limiting default delay between requests
- Sitemap priority ordering implementation

## Deferred Ideas

None raised during discussion.
