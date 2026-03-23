# geodaddy

## What This Is

Open source GEO (Generative Engine Optimization) analysis tool. Analyzes websites for AI-powered search engine optimization — helping content rank in ChatGPT, Perplexity, Google AI Overviews, and similar generative search engines. CLI-first, runs completely locally, outputs machine-readable JSON reports.

## Core Value

Surface actionable GEO issues with specific fix recommendations — not just scores, but "here's what's wrong and exactly how to fix it."

## Requirements

### Validated

- [x] CLI analyzes any URL (including localhost) — Validated in Phase 1: Foundation & CLI Setup
- [x] JSON output format for CI/CD integration — Validated in Phase 1: Foundation & CLI Setup
- [x] Technical metrics: crawlability signals, mobile compatibility, technical SEO — Validated in Phase 2: Core Analysis Engine
- [x] Content structure metrics: heading hierarchy, schema markup, semantic HTML, alt text — Validated in Phase 2: Core Analysis Engine
- [x] Three-level scoring: overall 0-100, category scores, per-metric pass/fail — Validated in Phase 2: Core Analysis Engine
- [x] Actionable recommendations for each issue found — Validated in Phase 2: Core Analysis Engine
- [x] GEO-specific analyzers: listicle detection, AI bot robots.txt audit, triple schema stacking — Validated in Phase 3: GEO Differentiators
- [x] GEO category scoring with 3-way average (technical + content + geo) — Validated in Phase 3: GEO Differentiators

- [x] Core Web Vitals measurement (LCP, FCP, CLS, TTFB, TBT via --vitals flag) — Validated in Phase 5: Core Web Vitals Measurement
- [x] Performance scoring category (4-way average: technical + content + geo + performance) — Validated in Phase 5: Core Web Vitals Measurement

### Active

- [ ] Site-wide crawling (sitemap-first, link-following fallback)
- [ ] Optional JavaScript rendering via headless browser
- [ ] Completely local operation (no external API dependencies)

### Out of Scope

- Web UI — deferred to post-v1 (will live in `web/` directory)
- Source credibility metrics (citations, E-E-A-T) — v2
- Answer format metrics (FAQ, Q&A structure) — v2
- HTML report output — v2
- Terminal rich output — v2, JSON-only for v1
- Comparison with competitors/benchmarks — v2
- Real-time monitoring / scheduled scans — not planned

## Context

**GEO vs SEO**: Traditional SEO optimizes for Google's ranking algorithm. GEO optimizes for AI-powered search engines that synthesize answers from multiple sources. Key differences:
- AI engines prefer structured, citable content
- Direct answers and summaries get featured
- Schema markup helps AI understand content semantics
- Source credibility signals matter more

**Monorepo Structure**:
- `cli/` — v1 Rust CLI application
- `web/` — Future web interface (post-v1)

**Target Users**: Developers and SEO professionals who want to optimize content for generative search. Open source, free, runs locally.

## Constraints

- **Language**: Rust — single binary distribution, performance for crawling
- **Distribution**: Local CLI first, no cloud dependencies in v1
- **Output**: JSON-only for v1 (enables CI/CD integration, web UI can parse later)
- **Crawling**: Must handle localhost URLs for local development testing

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust over Node/Python | Single binary, performance for crawling, no runtime deps | — Pending |
| JSON-only output for v1 | Simpler MVP, enables CI/CD, web UI parses later | — Pending |
| Three categories for v1 (Technical + Content + GEO) | GEO differentiators are core value — 3-way scoring | Phase 3 |
| Sitemap-first crawling with link fallback | Best coverage strategy for diverse sites | — Pending |
| Optional JS rendering | Some sites need it, but adds complexity — make it opt-in | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-03-23 after Phase 5: Core Web Vitals Measurement*
