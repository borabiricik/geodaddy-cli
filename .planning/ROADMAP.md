# Roadmap: geodaddy

## Overview

Geodaddy delivers GEO analysis through four phases: establish Rust CLI foundation with single-URL crawling, build the analysis engine with technical and content metrics, add GEO-specific differentiators that distinguish us from traditional SEO tools, then scale to site-wide crawling with sitemap support and politeness controls.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Foundation & CLI Setup** - Rust project scaffold, single-URL crawler, JSON output (completed 2026-03-23)
- [x] **Phase 2: Core Analysis Engine** - Technical and content structure analyzers with scoring system (completed 2026-03-23)
- [ ] **Phase 3: GEO Differentiators** - Listicle detection, AI bot audit, schema stacking, FAQ quality
- [ ] **Phase 4: Site-Wide Crawling & Polish** - Sitemap parser, multi-page crawling, rate limiting, final integration

## Phase Details

### Phase 1: Foundation & CLI Setup
**Goal**: CLI can analyze single URL and output JSON report
**Depends on**: Nothing (first phase)
**Requirements**: CRAWL-03, CRAWL-05, CLI-01, CLI-02, CLI-04
**Success Criteria** (what must be TRUE):
  1. User can run geodaddy on localhost URL and receive JSON report
  2. CLI returns proper exit codes (0=success, 1=fail with --fail-under)
  3. CLI shows helpful usage documentation via --help
  4. URL normalization prevents duplicate crawl entries
  5. JSON output includes schema version field for future compatibility
**Plans**: 1 plan

Plans:
- [x] 01-01-PLAN.md — Rust project scaffold, CLI implementation, and integration tests

### Phase 2: Core Analysis Engine
**Goal**: Technical SEO and content structure analysis working with pass/fail scoring
**Depends on**: Phase 1
**Requirements**: TECH-01, TECH-02, TECH-03, TECH-04, TECH-05, TECH-06, TECH-07, TECH-08, CONT-01, CONT-02, CONT-03, CONT-04, SCORE-01, SCORE-02, SCORE-03, SCORE-04
**Success Criteria** (what must be TRUE):
  1. User receives overall site score (0-100) and per-category scores
  2. User sees actionable fix recommendations for each detected issue
  3. Each metric reports pass/fail/warn status with specific guidance
  4. Technical checks detect broken links, redirects, meta tag issues, heading problems
  5. Content checks validate schema markup, semantic HTML, alt text, heading hierarchy
**Plans**: 4 plans

Plans:
- [x] 02-01-PLAN.md — Cargo dependencies + scoring.rs shared types + analyzers/ module scaffold
- [x] 02-02-PLAN.md — Technical SEO analyzers (TECH-01 through TECH-08) in analyzers/technical.rs
- [x] 02-03-PLAN.md — Content structure analyzers (CONT-01 through CONT-04) in analyzers/content.rs
- [x] 02-04-PLAN.md — main.rs orchestration: HTML fetch, all analyzer calls, scoring integration

### Phase 3: GEO Differentiators
**Goal**: GEO-specific analysis features distinguish geodaddy from traditional SEO tools
**Depends on**: Phase 2
**Requirements**: GEO-01, GEO-02, GEO-03
**Success Criteria** (what must be TRUE):
  1. User is warned if robots.txt blocks AI search engines (GPTBot, PerplexityBot, ClaudeBot)
  2. User sees listicle format detection (Top N lists, numbered structures)
  3. User is notified when triple schema stacking detected (Article + ItemList + FAQPage)
  4. Recommendations explain GEO impact (e.g., "74.2% of AI citations come from listicles")
**Plans**: 2 plans

Plans:
- [x] 03-01-PLAN.md — GEO analyzer functions (listicle, AI bot audit, schema stacking) in analyzers/geo.rs
- [x] 03-02-PLAN.md — Scoring integration (GEO category, 3-way average) and main.rs orchestration

### Phase 4: Site-Wide Crawling & Polish
**Goal**: Multi-page crawling with politeness controls and sitemap-first strategy
**Depends on**: Phase 3
**Requirements**: CRAWL-01, CRAWL-02, CRAWL-04, CLI-03
**Success Criteria** (what must be TRUE):
  1. User can crawl entire site via sitemap.xml with priority-based ordering
  2. CLI falls back to link-following if sitemap unavailable
  3. Crawler respects robots.txt directives and crawl-delay settings
  4. User sees progress indicator during site crawl
  5. Optional --enable-js flag enables headless browser for JavaScript rendering
**Plans**: 2 plans

Plans:
- [x] 04-01-PLAN.md — crawling.rs module: sitemap URL extraction, BFS link-following, JS detection, URL normalization, aggregate scoring helpers
- [x] 04-02-PLAN.md — main.rs crawl loop wiring: --max-pages, --enable-js, progress to stderr, aggregate Report fields

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation & CLI Setup | 0/1 | Complete    | 2026-03-23 |
| 2. Core Analysis Engine | 4/4 | Complete | 2026-03-23 |
| 3. GEO Differentiators | 0/2 | In progress | - |
| 4. Site-Wide Crawling & Polish | 0/0 | Not started | - |

### Phase 5: Core Web Vitals measurement: LCP, FCP, CLS, TTFB, TBT and performance metrics analyzer

**Goal:** Add `--vitals` flag that measures Core Web Vitals (LCP, FCP, CLS, TTFB, TBT) via chromiumoxide headless browser per crawled page, surfacing results as scored AnalysisResult entries in a new `performance` scoring category with a 4-way overall average
**Requirements**: PERF-01, PERF-02, PERF-03, PERF-04, PERF-05, PERF-06, PERF-07, PERF-08
**Depends on:** Phase 4
**Plans:** 3/3 plans complete

Plans:
- [x] 05-01-PLAN.md — scoring.rs: CategoryScores + performance field, severity_points perf entries, calculate_score 4-way average; analyzers/mod.rs: pub mod performance
- [x] 05-02-PLAN.md — analyzers/performance.rs: analyze_vitals + 5 classify_* functions + JS constants + unit tests
- [x] 05-03-PLAN.md — main.rs wiring: --vitals flag, vitals_browser launch, per-page analyze_vitals call; crawling.rs: aggregate_scores performance averaging; integration tests
