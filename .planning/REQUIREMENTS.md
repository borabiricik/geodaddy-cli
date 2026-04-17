# Requirements: geodaddy

**Defined:** 2026-03-23
**Core Value:** Surface actionable GEO issues with specific fix recommendations

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Crawling

- [x] **CRAWL-01**: CLI can crawl entire site starting from sitemap.xml
- [x] **CRAWL-02**: CLI falls back to link-following if sitemap unavailable
- [x] **CRAWL-03**: CLI supports localhost URLs (http://localhost:*, http://127.0.0.1:*)
- [x] **CRAWL-04**: CLI has optional JavaScript rendering via headless browser flag
- [x] **CRAWL-05**: CLI respects robots.txt crawl directives and crawl-delay

### Technical SEO

- [x] **TECH-01**: Analyzer detects broken links (404s) and reports source URLs
- [x] **TECH-02**: Analyzer detects redirect chains (301/302, loops, excessive hops)
- [x] **TECH-03**: Analyzer validates meta tags (title 50-60 chars, description 120-158 chars)
- [x] **TECH-04**: Analyzer validates heading hierarchy (single H1, logical nesting)
- [x] **TECH-05**: Analyzer checks mobile viewport meta tag presence
- [x] **TECH-06**: Analyzer validates robots.txt (syntax, sitemap ref, production Disallow check)
- [x] **TECH-07**: Analyzer validates sitemap.xml (format, URL limit, robots.txt conflicts)
- [x] **TECH-08**: Analyzer checks HTTPS/SSL and flags mixed content

### Content Structure

- [x] **CONT-01**: Analyzer validates heading structure (H1-H6 hierarchy, no skipped levels)
- [x] **CONT-02**: Analyzer detects and validates JSON-LD schema markup
- [x] **CONT-03**: Analyzer checks semantic HTML usage (article, main, nav, section vs div soup)
- [x] **CONT-04**: Analyzer flags images missing alt text

### GEO-Specific

- [x] **GEO-01**: Analyzer detects listicle format ("Top N", numbered lists, structured comparisons)
- [x] **GEO-02**: Analyzer audits robots.txt for AI bot directives (GPTBot, PerplexityBot, ClaudeBot)
- [x] **GEO-03**: Analyzer detects triple schema stacking (Article + ItemList + FAQPage on same page)

### Scoring & Output

- [x] **SCORE-01**: CLI outputs overall site score (0-100)
- [x] **SCORE-02**: CLI outputs per-category scores (0-100 for Technical, Content, GEO)
- [x] **SCORE-03**: CLI outputs per-metric pass/fail/warn status
- [x] **SCORE-04**: Each issue includes actionable fix recommendation with specific guidance

### CLI Experience

- [x] **CLI-01**: CLI outputs JSON format to stdout
- [x] **CLI-02**: CLI returns proper exit codes (0=pass, 1=fail) with --fail-under threshold
- [x] **CLI-03**: CLI shows progress indicator during site crawl
- [x] **CLI-04**: CLI has --help with clear usage documentation

### MCP Server Integration

- [ ] **MCP-01**: MCP server written in TypeScript using official @modelcontextprotocol/sdk (D-01)
- [ ] **MCP-02**: MCP server uses stdio transport for local LLM client communication (D-02)
- [ ] **MCP-03**: Single analyze_url tool registered with all CLI flags as parameters (D-04, D-05)
- [ ] **MCP-04**: Raw JSON output passed through as MCP tool result content (D-07)
- [ ] **MCP-05**: Errors return MCP error response with isError:true and stderr message (D-08)
- [ ] **MCP-06**: geodaddy binary bundled via postinstall download from GitHub releases (D-03)
- [ ] **MCP-07**: Published to npm, invokable via npx (D-09, D-10)

### Competitor Comparison

- [x] **COMP-01**: CLI has `compare` subcommand accepting ≥ 2 URLs: `geodaddy compare <url1> <url2> [url3...]`
- [x] **COMP-02**: `compare` reuses `analyze()` per URL sequentially, sharing `reqwest::Client` and optional browsers
- [x] **COMP-03**: Existing flags `--enable-js`, `--vitals`, `--max-pages`, `--beauty`, `--fail-under` work under the `compare` subcommand with semantics preserved (`--max-pages` per-target, `--fail-under` first-URL only)
- [x] **COMP-04**: JSON output: stable `CompareReport` schema with `schema_version`, `compared_at`, `sites`, `winners`, `check_diff`, `errors`
- [x] **COMP-05**: Per-category winner detection with 0.1-point tie epsilon (winner = None when tied or category absent)
- [x] **COMP-06**: Per-check diff table: one row per unique check ID, one cell per site (status "pass"/"warn"/"fail" or null when absent)
- [x] **COMP-07**: Beauty mode: side-by-side colored terminal table, variable column count (2-10 sites), uses existing `colored` crate (no new dependencies)
- [x] **COMP-08**: `--fail-under` applies to first URL only in compare mode (CI pattern: "your site" vs competitors)
- [x] **COMP-09**: Per-URL analysis failures do NOT abort the run; failed sites surface in `errors` array; overall exit code 2 only if the first URL fails
- [x] **COMP-10**: Duplicate URL handling: dedupe via `normalize_url()` with warning logged to stderr

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### GEO-Specific (Deferred)

- **GEO-04**: FAQ schema quality scoring (answer length 40-60 words optimal)
- **GEO-05**: Quick answer block detection (TL;DR, above-fold summaries)
- **GEO-06**: Content freshness signals (last-modified, dateModified schema)
- **GEO-07**: Citation/statistic density analysis
- **GEO-08**: HowTo schema validation

### Source Credibility (Deferred)

- **CRED-01**: Author byline detection
- **CRED-02**: Author schema (Person) validation
- **CRED-03**: Reference/citation section detection

### Output Formats (Deferred)

- **OUT-01**: HTML report generation
- **OUT-02**: Terminal rich output (colors, formatting)
- **OUT-03**: Diff mode for regression detection

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Citation tracking / AI mention monitoring | Requires continuous API monitoring. Let Otterly.ai, AIclicks handle this. |
| E-E-A-T scoring (author credibility) | Subjective NLP required. v2 can add presence detection. |
| Competitive benchmarking | Requires crawling competitor sites, storage. Users can run tool themselves. |
| Content quality scoring | Readability, sentiment = subjective. Focus on structure. |
| Backlink analysis | Requires massive crawl infrastructure. Different domain. |
| Keyword research | Different domain. Requires SERP data. |
| Real-time monitoring / scheduled scans | Requires persistence, scheduler. Users can wrap in cron/CI. |
| Core Web Vitals (LCP, INP, CLS) | High complexity, requires full headless browser. Consider Lighthouse integration later. |
| PDF report generation | Web UI can handle this post-v1. |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| CRAWL-01 | Phase 4 | Complete |
| CRAWL-02 | Phase 4 | Complete |
| CRAWL-03 | Phase 1 | Complete |
| CRAWL-04 | Phase 4 | Complete |
| CRAWL-05 | Phase 1 | Complete |
| TECH-01 | Phase 2 | Complete |
| TECH-02 | Phase 2 | Complete |
| TECH-03 | Phase 2 | Complete |
| TECH-04 | Phase 2 | Complete |
| TECH-05 | Phase 2 | Complete |
| TECH-06 | Phase 2 | Complete |
| TECH-07 | Phase 2 | Complete |
| TECH-08 | Phase 2 | Complete |
| CONT-01 | Phase 2 | Complete |
| CONT-02 | Phase 2 | Complete |
| CONT-03 | Phase 2 | Complete |
| CONT-04 | Phase 2 | Complete |
| GEO-01 | Phase 3 | Complete |
| GEO-02 | Phase 3 | Complete |
| GEO-03 | Phase 3 | Complete |
| SCORE-01 | Phase 2 | Complete |
| SCORE-02 | Phase 2 | Complete |
| SCORE-03 | Phase 2 | Complete |
| SCORE-04 | Phase 2 | Complete |
| CLI-01 | Phase 1 | Complete |
| CLI-02 | Phase 1 | Complete |
| CLI-03 | Phase 4 | Complete |
| CLI-04 | Phase 1 | Complete |
| MCP-01 | Phase 6 | Planned |
| MCP-02 | Phase 6 | Planned |
| MCP-03 | Phase 6 | Planned |
| MCP-04 | Phase 6 | Planned |
| MCP-05 | Phase 6 | Planned |
| MCP-06 | Phase 6 | Planned |
| MCP-07 | Phase 6 | Planned |
| COMP-01 | Phase 8 | Planned |
| COMP-02 | Phase 8 | Planned |
| COMP-03 | Phase 8 | Planned |
| COMP-04 | Phase 8 | Planned |
| COMP-05 | Phase 8 | Planned |
| COMP-06 | Phase 8 | Planned |
| COMP-07 | Phase 8 | Planned |
| COMP-08 | Phase 8 | Planned |
| COMP-09 | Phase 8 | Planned |
| COMP-10 | Phase 8 | Planned |

**Coverage:**
- v1 requirements: 45 total
- Mapped to phases: 45
- Unmapped: 0

---
*Requirements defined: 2026-03-23*
*Last updated: 2026-04-16 after Phase 8 planning*
