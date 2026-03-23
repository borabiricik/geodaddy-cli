# Research Summary: geodaddy

**Project:** geodaddy - GEO/SEO Analysis CLI Tool
**Research Completed:** 2026-03-23
**Overall Confidence:** HIGH

---

## Executive Summary

Geodaddy is a CLI-first website analysis tool focused on Generative Engine Optimization (GEO) - optimizing content for AI search engines like ChatGPT, Perplexity, and Gemini. The research reveals a clear product-market fit: while traditional SEO tools (Lighthouse, Screaming Frog, Ahrefs) focus on Google PageRank, **GEO requires fundamentally different optimization** - structured data quality, natural language answers, FAQ schema, and listicle formats that AI engines prefer for citations.

The Rust ecosystem in 2026 provides production-ready libraries for building this tool. The recommended stack leverages **Tokio** for async crawling, **reqwest** for HTTP client, **scraper** for HTML parsing, and **chromiumoxide** for optional JS rendering. The architecture follows a pipeline pattern: **Crawl → Parse → Analyze → Report**, with a trait-based plugin system for extensibility and monorepo structure for future web UI integration.

**Key differentiators from existing tools:**
1. GEO-specific checks (listicle detection, FAQ schema quality scoring, AI bot management audit)
2. CLI-first with localhost support and CI/CD integration built-in
3. Actionable fix recommendations with code snippets, not just scores
4. JSON-only output for v1 (web UI deferred to v2)

**Critical risks identified:** URL deduplication without normalization (causes exponential crawl bloat), unbounded async task queues (memory exhaustion), blocking AI crawlers in robots.txt (makes optimization invisible), arbitrary scoring weights without validation (meaningless vanity metrics), and headless browser memory leaks (OOM crashes in containers).

---

## Recommended Stack

| Technology | Version | Purpose | Confidence |
|------------|---------|---------|------------|
| **Rust** | 1.83+ | Language (performance, safety, single binary) | HIGH |
| **tokio** | 1.49+ | Async runtime for concurrent crawling | HIGH |
| **reqwest** | 0.13+ | HTTP client with connection pooling, cookies, gzip | HIGH |
| **scraper** | 0.26+ | HTML parsing, CSS selectors (Servo-based) | HIGH |
| **chromiumoxide** | 0.9+ | Optional headless Chrome for JS rendering | HIGH |
| **clap** | 4.6+ | CLI argument parsing (derive API) | HIGH |
| **serde** + **serde_json** | 1.0.149+ | JSON serialization for output | HIGH |
| **quick-xml** | 0.38+ | XML parsing for sitemaps | HIGH |
| **url** | 2.5+ | URL normalization (WHATWG standard) | HIGH |
| **robotstxt** | 0.3+ | robots.txt parsing (Google algorithm) | HIGH |
| **governor** | 0.6+ | Rate limiting (token bucket) | MEDIUM |
| **jsonschema** | 0.45+ | Schema.org validation | HIGH |
| **anyhow** | 1.0+ | Error handling for applications | HIGH |
| **tracing** | 0.1+ | Structured logging and diagnostics | HIGH |

**Rationale for key choices:**
- **Tokio over async-std:** Larger ecosystem, better middleware support, 575M+ downloads
- **chromiumoxide over rust-headless-chrome:** Async API, comprehensive CDP coverage, auto-generated bindings
- **clap with derive macros:** Structopt functionality integrated, cleaner code than builder pattern
- **anyhow over thiserror:** Application error handling (CLI) vs library (defer thiserror to v2)
- **robotstxt over texting_robots:** Google's official algorithm - critical for SEO tool to match Google behavior

---

## Table Stakes Features

Must-have features for v1 - any GEO/SEO tool must provide these:

### Technical SEO (8 checks)
1. **Broken links (404s)** - Report 404s + source URLs where broken links appear
2. **Redirect chain detection** - 301/302, loops, chains
3. **Meta tags** - Title (50-60 chars), description (120-158 chars), uniqueness
4. **Heading hierarchy** - Single H1, logical nesting (no H2→H4 jumps)
5. **Mobile viewport tag** - Presence and correctness
6. **robots.txt validation** - Syntax, sitemap reference, production Disallow check
7. **Sitemap.xml analysis** - Format, 50K URL limit, robots.txt conflicts
8. **HTTPS/SSL** - Protocol validation, mixed content detection

### Content Structure (4 checks)
1. **Semantic HTML** - Proper use of article, main, nav, section vs div soup
2. **Schema markup detection** - JSON-LD presence and types
3. **Image alt text** - Flag missing alt attributes
4. **Canonical tags** - Self-referencing canonicals, conflict detection

### CLI Essentials (3 features)
1. **JSON output format** - Primary output for CI/CD integration
2. **Exit codes** - 0 (success), 1 (critical issues), 2 (crawl error)
3. **Progress indicators** - Show something in <100ms (responsiveness > speed)

**Total MVP: 15 checks + 3 CLI features**

---

## Differentiating Features (GEO-Specific)

What makes geodaddy unique - these are NOT covered by traditional SEO tools:

### Priority 1 (Low complexity, proven high impact)
1. **Listicle format detection** - 74.2% of AI citations come from "Top N" structured content
2. **Triple schema stacking detection** - Article + ItemList + FAQPage = 2-3x citation rate
3. **Content freshness signals** - Last-modified headers, dateModified schema (23% citation decline without)
4. **AI bot management audit** - Check robots.txt for GPTBot, PerplexityBot, ClaudeBot, Google-Extended (2026-specific)

### Priority 2 (Medium complexity, proven impact)
1. **FAQ schema quality scoring** - 40-60 word answers, question format, not just presence (41% vs 15% citation rate)
2. **Quick answer block detection** - TL;DR sections, summary blocks in first viewport
3. **Citation/statistic density** - Detect inline citations, references, data points with sources (40% visibility improvement)

### Priority 3 (Polish features)
1. **HowTo schema validation** - Step-by-step structure, tool/supply lists
2. **Answer length optimization** - Flag FAQ/HowTo answers outside 40-60 word sweet spot
3. **BreadcrumbList schema** - Validate 3-5 level depth, JSON-LD + visual breadcrumbs

### Developer Experience Differentiators
1. **Localhost URL support** - Test local dev sites before deployment (explicit PROJECT.md requirement)
2. **Actionable fix recommendations** - "Here's exactly how to fix it" with code snippets
3. **Per-URL detail level** - Site-wide scans with per-page granularity in JSON output
4. **Parallel analysis** - Fast scans via concurrent checks

---

## Architecture Highlights

### Pipeline Pattern: Crawl → Parse → Analyze → Report

**Component Boundaries:**
- **Crawler Engine** - Fetches HTML with politeness (rate limiting, robots.txt respect)
- **Content Parser** - Extracts structured data (headings, schema, links, meta tags)
- **Analysis Engine** - Trait-based plugin system for metrics
- **Report Generator** - JSON output with versioned schema

### Key Architectural Decisions

**1. Monorepo Structure (Cargo Workspace)**
```
geodaddy/
├── core/        # Library crate (shared analysis engine)
├── cli/         # Binary crate (v1)
└── web/         # Future web UI (v2, imports core)
```
**Rationale:** Code reuse for future web UI, single build cache, version consistency

**2. Trait-Based Analyzer Plugin System**
```rust
trait Analyzer: Send + Sync {
    fn name(&self) -> &str;
    fn category(&self) -> Category;
    fn weight(&self) -> f32;
    async fn analyze(&self, doc: &ParsedDocument) -> AnalysisResult;
}
```
**Rationale:** Extensibility without core changes, static compilation for v1 (Rust lacks stable ABI for dynamic loading)

**3. Async-First with Tokio**
- Fixed worker pool (100 tasks) using `tokio::spawn`
- Bounded channels with backpressure
- Rate limiting via `governor` (token bucket)
- Per-domain politeness enforcement

**4. Sitemap-First Crawling Strategy**
- Fetch sitemap.xml → priority queue → parallel fetch
- Fallback to link-following if sitemap missing
- Validate sitemap URLs against robots.txt

**5. JSON-Only Output for v1**
- Machine-readable for CI/CD
- Versioned schema (`"version": "1.0.0"`)
- Future web UI can consume same JSON structure
- TUI/HTML reports deferred to v2

### Data Flow
```
CLI Args → Crawler (fetch) → Parser (extract) → Analyzer (score) → Report (JSON)
```

**Strict Component Isolation:**
- Crawler doesn't know about analyzers
- Parser doesn't know about reporting
- Analyzers operate on parsed data only
- Enables unit testing, swappable implementations, future web UI reuse

---

## Critical Pitfalls to Avoid

### 1. URL Deduplication Without Normalization (CRITICAL)
**Risk:** Crawl queue grows exponentially, same content analyzed multiple times, memory bloat
**Prevention:** Normalize URLs (https, www, trailing slash, sort query params) BEFORE adding to visited set. Use canonical link hints. Hash URLs with xxHash for memory efficiency.
**Phase:** Phase 1 (Core Crawler) - Must be in initial design, retrofitting breaks visited-URL tracking

### 2. Unbounded Async Task Queues (CRITICAL)
**Risk:** Memory exhaustion, application hangs, OOM crashes
**Prevention:** Use bounded channels `tokio::sync::mpsc::channel(capacity)`. Wrap all network calls in `tokio::time::timeout`. Start with 10-20 concurrency, monitor memory/task count.
**Phase:** Phase 1 (Core Crawler) - Async architecture must be designed correctly from start

### 3. Blocking AI Crawlers in robots.txt (CRITICAL - GEO-Specific)
**Risk:** Users optimize content that AI engines cannot see - wasted effort, zero AI search visibility
**Prevention:** Parse robots.txt for GPTBot, PerplexityBot, ClaudeBot, Google-Extended. Flag as CRITICAL issue: "Your site blocks AI search engines". Check Cloudflare Bot Fight Mode.
**Phase:** Phase 1 (Technical Metrics) - Core to GEO value proposition

### 4. Arbitrary Scoring Weights Without Validation (CRITICAL)
**Risk:** Meaningless vanity metrics that don't correlate with actual AI search visibility
**Prevention:** Start with pass/fail per metric, not numeric scores. If scoring: validate against real AI search data. Make actionable fixes primary, scores secondary.
**Phase:** Phase 1 (Scoring System) - Design decision before first metric

### 5. Headless Browser Memory Leaks (CRITICAL)
**Risk:** OOM crashes, /dev/shm exhaustion in Docker, zombie processes
**Prevention:** Reuse browser contexts. Close pages explicitly. Use `--disable-dev-shm-usage` in Docker. Make JS rendering opt-in flag. Block unnecessary resources (images, fonts).
**Phase:** Phase 1 (Optional JS Rendering) - Lifecycle management must be correct from start

### 6. JSON Schema Without Versioning (CRITICAL)
**Risk:** Breaking CI/CD pipelines when adding/changing fields, no migration path
**Prevention:** Include `"schema_version": "1.0.0"` in v1 root JSON. Use semantic versioning. Document schema explicitly.
**Phase:** Phase 1 (JSON Output) - Must be in v1 or can never add without breaking change

---

## Roadmap Implications

### Suggested Phase Structure

Based on dependencies identified in architecture research, recommended build order:

#### **Phase 1: Foundation (Week 1-2)**
**Goal:** Basic pipeline working end-to-end

**Deliverables:**
- Core data structures (RawPage, ParsedDocument, AnalysisReport)
- Simple crawler (single URL, no politeness yet)
- Content parser (headings, links, meta tags)
- CLI scaffolding with clap

**Why this order:** All components depend on these types. Get integration working before complexity.

**Critical decisions locked in:**
- URL normalization strategy (affects visited-URL tracking forever)
- JSON schema with versioning (breaking change if missing)
- Bounded channels with backpressure (async architecture foundation)

**Must avoid:**
- Deferring URL normalization "until crawling works"
- Missing schema_version field in first JSON output
- Using unbounded channels as placeholder

#### **Phase 2: Analysis System (Week 3-4)**
**Goal:** Scoring and reporting working

**Deliverables:**
- Analyzer trait + registry pattern
- 2-3 simple analyzers (HeadingHierarchyAnalyzer, MobileViewportAnalyzer)
- Pass/fail/warn system (NOT numeric scores initially)
- Report generator with JSON output

**Why this order:** Validates trait design before implementing all analyzers. Provides testable output.

**Critical decisions locked in:**
- Scoring approach (pass/fail vs numeric weights)
- Recommendation format (actionable fixes vs generic advice)
- Category structure (Technical vs Content Structure)

**Must avoid:**
- Creating arbitrary numeric scores without validation
- Generic error messages without context
- Coupling analyzers to crawler (maintain isolation)

#### **Phase 3: Crawling at Scale (Week 5-6)**
**Goal:** Multi-page crawling with politeness

**Deliverables:**
- URL queue with deduplication
- Worker pool (fixed size, not unbounded)
- Rate limiting (global + per-domain via governor)
- robots.txt parsing (respect Disallow, Crawl-delay)

**Why this order:** Builds on Phase 1 foundation, enables site-wide analysis.

**Critical decisions locked in:**
- Concurrency limits (worker pool size)
- Rate limiting strategy (global vs per-domain)
- Politeness defaults (1 req/sec, respect crawl-delay)

**Must avoid:**
- Ignoring robots.txt to ship faster
- Unbounded task spawning for "max performance"
- Missing redirect handling (breaks sitemap-first strategy)

#### **Phase 4: GEO Features & Polish (Week 7-8)**
**Goal:** Differentiating GEO analysis + full v1 feature set

**Deliverables:**
- Sitemap parser (priority-based crawling)
- Schema markup extraction (JSON-LD, Microdata, RDFa)
- GEO-specific analyzers (listicle detection, FAQ schema quality, AI bot audit)
- Remaining technical analyzers (PageSpeedAnalyzer, SchemaMarkupAnalyzer)
- Optional: Headless browser integration (--enable-js flag)

**Why this order:** GEO features are value differentiators, but depend on solid crawling foundation.

**Parallel work possible:** Individual analyzers can be built independently once Phase 2 complete.

**Must avoid:**
- Treating GEO as traditional SEO (keyword density focus)
- Making JS rendering default (memory leaks, slow)
- Missing AI crawler detection (Cloudflare blocks by default)

### Phase Dependencies
```
Phase 1 (Foundation) → REQUIRED for all other phases
    ↓
Phase 2 (Analysis) ← REQUIRED for Phase 3 validation
    ↓
Phase 3 (Crawling) ← Needs analysis to test multi-page results
    ↓
Phase 4 (GEO Features) ← Builds on all previous
```

### Research Flags for Phases

**Needs phase-specific research:**
- Phase 4: JS rendering detection heuristics (how to auto-detect if page needs JS?)
- Phase 4: Full schema.org vocabulary validation (complex, may need external library)
- Phase 3: Crawl performance benchmarks (optimal worker pool size, connection limits)

**Standard patterns (skip research):**
- Phase 1: URL normalization (well-documented)
- Phase 1: CLI argument parsing (clap examples abundant)
- Phase 2: Trait-based plugins (standard Rust pattern)
- Phase 3: Tokio async patterns (mature ecosystem)

---

## Confidence Assessment

| Area | Confidence | Source Quality | Gaps |
|------|------------|----------------|------|
| **Stack (Technologies)** | **HIGH** | Official GitHub repos, verified versions, 2026 benchmarks, 294k projects use reqwest | Rate limiting middleware (MEDIUM - community pattern not official) |
| **Features (What to build)** | **HIGH** | Multiple authoritative 2026 GEO sources, traditional SEO tool comparisons, CLI design guidelines | Scoring weights need validation with real AI search data |
| **Architecture (How to build)** | **HIGH** | Standard Rust patterns, Tokio ecosystem docs, crawler architecture from multiple implementations | JS rendering detection heuristics, schema validation library choice |
| **Pitfalls (What to avoid)** | **HIGH** | Recent March 2026 Tokio-specific articles, classic crawler patterns, production deployment guides, GEO-specific 2026 sources | JSON versioning (MEDIUM - inferred from CI/CD best practices) |

**Overall confidence: HIGH** - Core technologies mature, GEO domain well-researched, architecture patterns proven, critical pitfalls documented with 2026-specific sources.

**Known gaps requiring validation:**
1. Scoring weights (60% technical, 40% content) - need real-world testing
2. Optimal worker pool size - need profiling with real workloads
3. JS rendering detection - need experimentation with SPA patterns
4. Schema.org vocabulary validation - Adobe library is JavaScript, need Rust alternative or bridge

---

## Feature Comparison: geodaddy vs Existing Tools

| Feature | Lighthouse | Screaming Frog | Ahrefs Site Audit | **geodaddy** |
|---------|-----------|----------------|-------------------|--------------|
| **Crawling** | Single-page | Site-wide | Site-wide | Site-wide (sitemap-first) |
| **JS Rendering** | Yes (built-in) | Yes (opt-in) | Yes | Opt-in flag |
| **Schema Detection** | Basic | Advanced | Advanced | **JSON-LD focus + quality scoring** |
| **GEO-Specific Analysis** | No | No | No | **YES (core value)** |
| **FAQ Schema Scoring** | No | Detection only | Detection only | **Quality scoring (40-60 words)** |
| **Listicle Detection** | No | No | No | **YES (74.2% of citations)** |
| **AI Bot Audit** | No | No | No | **YES (2026-specific)** |
| **CLI-First** | Yes | No (GUI-first) | No (web-first) | **YES** |
| **Localhost Support** | Yes | Yes | No (cloud) | **YES** |
| **CI/CD Integration** | Lighthouse CI | API/scripts | API | **Built-in (exit codes, JSON)** |
| **Actionable Fixes** | Moderate | Flags issues | Flags issues | **Detailed recommendations** |

**Geodaddy's niche:** CLI-first + local operation + GEO-specific analysis + actionable recommendations for AI search optimization.

---

## Anti-Features (Deliberately NOT Building)

What geodaddy will NOT do, and why:

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Citation tracking** | Requires continuous monitoring of ChatGPT/Perplexity APIs - outside scope of one-time analysis | Focus on structural signals that enable citations. Let tools like Otterly.ai handle monitoring |
| **E-E-A-T scoring** | Subjective, requires NLP, external validation of credentials - out of scope per PROJECT.md | Flag presence/absence of author bylines, defer qualitative scoring to v2 |
| **Competitive benchmarking** | Requires crawling competitors, storing historical data - complex and out of scope | Provide absolute scores. Users can run tool on competitors themselves |
| **HTML/PDF reports** | Visual output deferred to v2 web UI per PROJECT.md | JSON-only for v1, web UI can render later |
| **Content quality scoring** | Readability, keyword density, sentiment = subjective NLP - hard to make actionable | Focus on structural signals (headings, lists, schema) not content quality |
| **Backlink analysis** | Requires massive crawl infrastructure (Ahrefs domain) - not feasible for local CLI | Ignore off-page SEO, focus on on-page + technical |
| **Keyword research** | Different domain, requires search volume data and SERP analysis | Analyze existing page structure, not keyword targeting |
| **Real-time monitoring** | Requires persistence layer, scheduler, notifications - PROJECT.md: "not planned" | One-shot analysis on demand, users can wrap in cron/CI |
| **Core Web Vitals (full)** | Requires headless browser for accurate LCP/INP/CLS - high complexity | Consider Lighthouse CLI integration or defer to v2. Acknowledge lab vs field data limitation |

---

## What Makes This Research Actionable

**Clear technology choices:**
- Tokio, reqwest, scraper, chromiumoxide - all versions verified, rationale documented
- MSRV: Rust 1.83+ (required by jsonschema)

**Specific feature prioritization:**
- MVP: 15 table stakes checks + 3 CLI features
- Priority 1 differentiators: 4 GEO-specific checks (listicle, triple schema, freshness, AI bots)
- Priority 2: 3 medium-complexity GEO checks (FAQ quality, quick answers, citations)

**Phase structure with dependencies:**
- Phase 1 locks in URL normalization, JSON versioning, async architecture
- Phase 2 validates analyzer trait design with 2-3 simple implementations
- Phase 3 adds multi-page crawling with politeness
- Phase 4 adds GEO differentiators (can parallelize individual analyzers)

**Critical pitfalls with prevention:**
- Each pitfall includes: what goes wrong, why it happens, warning signs, prevention steps, phase relevance
- 6 critical, 7 medium, 7 minor pitfalls documented

**Architecture decisions justified:**
- Trait-based plugins (not dynamic loading) - Rust lacks stable ABI, v1 doesn't need runtime extension
- Tokio (not async-std) - larger ecosystem, better middleware
- JSON-only v1 (not TUI) - CI/CD primary use case, web UI v2 consumes JSON
- Monorepo (not single crate) - future web UI reuses core logic

---

## Ready for Requirements Definition

This research provides:
- **Technology stack** with versions and rationale
- **Feature list** prioritized by complexity and impact
- **Architecture patterns** with component boundaries
- **Pitfall prevention** with phase-specific warnings
- **Build order** with phase dependencies

**Next step:** Roadmapper agent can use this to create phase-specific roadmap with:
- Concrete deliverables per phase
- Technology choices already validated
- Known pitfalls to avoid per phase
- Research flags for areas needing deeper investigation

**High confidence areas** (ready to implement):
- Core crawler with URL normalization and rate limiting
- HTML parsing with scraper for headings, meta tags, schema
- Trait-based analyzer system with static compilation
- JSON output with versioned schema

**Medium confidence areas** (need phase-specific validation):
- Scoring weights (start pass/fail, validate before numeric scores)
- JS rendering detection (experiment with SPA patterns)
- Schema.org vocabulary validation (need library research)

---

## Sources Summary

**Stack Research:**
- 23 sources from GitHub, official docs, 2026 benchmarks, Rust ecosystem guides
- Key sources: reqwest GitHub (verified 294k projects), Tokio evolution 2026, chromiumoxide vs rust-headless-chrome comparison

**Features Research:**
- 37 sources from SEO tool documentation, GEO guides, CLI design principles
- Key sources: GenOptima GEO Playbook 2026, Search Engine Land GEO Guide 2026, clig.dev CLI guidelines

**Architecture Research:**
- 31 sources from crawler design, Rust async patterns, monorepo structure guides
- Key sources: Tokio crawler implementations, Cargo workspace docs, plugin system patterns

**Pitfalls Research:**
- 54 sources from production deployment guides, Tokio mistakes 2026, GEO errors, crawler traps
- Key sources: "Top 5 Tokio Mistakes That Quietly Kill Your Async Rust" (March 2026), AWS crawler best practices, IETF crawler draft

**Total:** 145 sources, majority from 2026, high-authority (official docs, GitHub repos, Google developers, IETF drafts).
