# Architecture Research: GEO Analysis Tools

**Project:** geodaddy
**Researched:** 2026-03-23
**Confidence:** HIGH

## Executive Summary

Website analysis CLI tools follow a pipeline architecture with four major stages: **Crawl → Parse → Analyze → Report**. Modern implementations use async Rust with Tokio for crawling, trait-based plugin systems for extensible metrics, and monorepo workspace patterns for CLI + future web UI code sharing.

The geodaddy architecture should prioritize:
1. **Modularity** - Clear component boundaries enable isolated testing and future extension
2. **Async-first** - Tokio-based concurrency for efficient crawling without blocking
3. **Extensibility** - Trait-based analyzer system allows adding metrics without core changes
4. **Monorepo** - Cargo workspace structure enables future web UI to share analysis engine

## Core Components

### 1. Crawler Engine

**Responsibility:** Fetch HTML from URLs with politeness controls

**Inputs:**
- Seed URLs (from CLI arguments or sitemap)
- Crawl configuration (max depth, rate limits, user-agent)
- Optional headless browser flag (for JS-rendered sites)

**Outputs:**
- Raw HTML content
- Fetch metadata (status code, timing, final URL after redirects)

**Key Architecture Patterns:**

**Queue Management:**
- Use Tokio `mpsc::channel` for URL distribution across workers
- Separate tracking: `HashSet<Url>` for visited URLs (prevent duplicates), atomic counter for in-flight tasks
- Front queue: prioritization by sitemap priority or depth
- Back queue: per-domain organization for politeness enforcement

**Rate Limiting:**
- Token bucket algorithm via `governor` crate (50 RPS global default)
- Per-domain rate limiting: separate governors per hostname
- Respect `robots.txt` crawl-delay directive (parse once, cache per domain)
- Implement exponential backoff for 429/5xx responses

**Concurrency Model:**
- Fixed worker pool (100 tasks) using `tokio::spawn`
- Shared state via `Arc<CrawlerState>` containing HTTP client, visited tracker (DashSet), and rate limiter
- Work-stealing from channel - workers poll until channel closes AND in-flight counter reaches zero

**Politeness Implementation:**
```
Per-domain queue:
1. Check robots.txt (cache for 24h)
2. Verify crawl-delay compliance (min 1 second if specified)
3. Apply rate limiter token before request
4. Connection pooling via reqwest (reuse TCP connections)
```

**Headless Browser (Optional):**
- Lazy initialization: only spawn Chrome if `--render-js` flag set
- Use `rust-headless-chrome` (Puppeteer equivalent via Chrome DevTools Protocol)
- Fallback pattern: attempt static fetch first, retry with browser on JS detection
- Pool management: 5 browser instances max, queue requests if pool exhausted

**Dependencies:**
- `reqwest` - HTTP client with connection pooling
- `tokio` - Async runtime
- `governor` - Token bucket rate limiting
- `dashmap` - Lock-free concurrent HashMap for visited URLs
- `rust-headless-chrome` - Optional headless Chrome integration
- `robots_txt` - robots.txt parsing

**Confidence:** HIGH (Tokio patterns widely documented, Rust crawler implementations mature)

### 2. Content Parser

**Responsibility:** Extract structured data and metrics from raw HTML

**Inputs:**
- Raw HTML string
- Page URL (for resolving relative links)
- Parse configuration (which elements to extract)

**Outputs:**
- Parsed document structure:
  - DOM tree (via `scraper` crate)
  - Extracted links (absolute URLs)
  - Heading hierarchy (H1-H6 with nesting depth)
  - Schema.org markup (JSON-LD, Microdata, RDFa)
  - Meta tags (title, description, Open Graph)
  - List structures (ol/ul with nesting)
  - Performance hints (resource sizes, external requests)

**Key Architecture Patterns:**

**Parser Pipeline:**
```
HTML → scraper::Html → Selector queries → Extract metrics
                    ↓
            Link discovery → URL normalization → Crawl queue
```

**Schema Extraction:**
- JSON-LD: Parse `<script type="application/ld+json">` via `serde_json`
- Microdata: Walk DOM for `itemscope`/`itemprop` attributes
- RDFa: Parse `typeof`/`property` attributes
- Validation: Use schema.org vocabulary for type checking

**Heading Hierarchy Analysis:**
```rust
struct HeadingNode {
    level: u8,        // 1-6
    text: String,
    children: Vec<HeadingNode>,
    position: usize   // document order
}
```
- Detect hierarchy violations (H1 → H3 skip)
- Track multiple H1s (flag as issue)
- Calculate max nesting depth

**Dependencies:**
- `scraper` - HTML parsing (uses `html5ever` internally)
- `selectors` - CSS selector engine
- `serde_json` - JSON-LD parsing
- `url` - URL normalization

**Confidence:** HIGH (Standard Rust HTML parsing patterns, schema.org spec stable)

### 3. Analysis Engine (Trait-Based Plugin System)

**Responsibility:** Run metric analyzers against parsed content, calculate scores

**Inputs:**
- Parsed document structure
- Page metadata (URL, fetch timing)
- Site-wide context (all pages, sitemap data)

**Outputs:**
- Metric results (pass/fail + severity)
- Category scores (0-100 per category)
- Overall score (weighted average)
- Actionable recommendations per issue

**Key Architecture Patterns:**

**Core Trait:**
```rust
trait Analyzer: Send + Sync {
    fn name(&self) -> &str;
    fn category(&self) -> Category;  // Technical, Content, Authority
    fn weight(&self) -> f32;         // Score weighting

    async fn analyze(&self, doc: &ParsedDocument) -> AnalysisResult;
}

struct AnalysisResult {
    passed: bool,
    score: f32,              // 0.0-1.0
    severity: Severity,      // Critical, High, Medium, Low
    issues: Vec<Issue>,
    recommendations: Vec<String>
}
```

**Registry Pattern:**
```rust
struct AnalyzerRegistry {
    analyzers: HashMap<String, Box<dyn Analyzer>>,
    categories: HashMap<Category, Vec<String>>  // category -> analyzer IDs
}

impl AnalyzerRegistry {
    fn register(&mut self, analyzer: Box<dyn Analyzer>) {
        let id = analyzer.name().to_string();
        self.analyzers.insert(id.clone(), analyzer);
        self.categories.entry(category).or_default().push(id);
    }

    async fn run_all(&self, doc: &ParsedDocument) -> CategoryScores {
        // Run analyzers in parallel, aggregate by category
    }
}
```

**Plugin Discovery (v1 - Static Compilation):**
- All analyzers compiled into binary
- Registration happens in `main()` via macro or manual calls
- No dynamic loading (avoid libloading complexity for v1)

**Plugin Discovery (v2 - Dynamic Loading):**
- Use `libloading` for runtime .so/.dylib/.dll loading
- Plugin ABI: C-compatible FFI layer
- Registration function: `extern "C" fn register(registry: *mut AnalyzerRegistry)`
- Challenge: Rust lacks stable ABI - require exact compiler version match

**Scoring System:**
```
Category Score = (Σ analyzer_score * weight) / Σ weights
Overall Score = (Σ category_score * category_weight) / Σ category_weights

Category weights for v1:
- Technical: 0.6 (60%)
- Content Structure: 0.4 (40%)
```

**Built-in Analyzers (v1):**

**Technical Category:**
- `MobileViewportAnalyzer` - Checks meta viewport tag
- `PageSpeedAnalyzer` - Parse timing, resource sizes
- `CrawlabilityAnalyzer` - robots.txt compliance, canonical tags
- `SchemaMarkupAnalyzer` - Presence and validity of structured data

**Content Structure Category:**
- `HeadingHierarchyAnalyzer` - H1-H6 structure validation
- `ListStructureAnalyzer` - Ordered/unordered list usage
- `SummaryBlockAnalyzer` - Detect intro paragraphs, TL;DR sections
- `ContentLengthAnalyzer` - Word count, readability metrics

**Dependencies:**
- No external dependencies (trait-based, uses parsed data)
- Future: `libloading` for dynamic plugins (v2)

**Confidence:** HIGH (Trait pattern standard Rust, static plugin system well-understood)

### 4. Report Generator

**Responsibility:** Format analysis results as machine-readable JSON

**Inputs:**
- Analysis results (all pages)
- Overall/category scores
- Issues with recommendations
- Site metadata (crawl timestamp, tool version)

**Outputs:**
- JSON report (stdout or file)
- Exit code (0 if no critical issues, 1 if failures)

**Key Architecture Patterns:**

**JSON Schema (v1):**
```json
{
  "version": "1.0.0",
  "timestamp": "2026-03-23T10:15:00Z",
  "url": "https://example.com",
  "summary": {
    "overall_score": 78,
    "category_scores": {
      "technical": 85,
      "content_structure": 70
    },
    "page_count": 42,
    "issues_by_severity": {
      "critical": 2,
      "high": 5,
      "medium": 12,
      "low": 8
    }
  },
  "categories": [
    {
      "name": "technical",
      "score": 85,
      "weight": 0.6,
      "metrics": [
        {
          "name": "mobile_viewport",
          "passed": true,
          "score": 100,
          "weight": 0.2,
          "pages_affected": []
        },
        {
          "name": "page_speed",
          "passed": false,
          "score": 45,
          "weight": 0.3,
          "pages_affected": ["/blog/post-1", "/about"],
          "issues": [
            {
              "severity": "high",
              "message": "Load time exceeds 3 seconds",
              "affected_pages": ["/blog/post-1"],
              "recommendation": "Compress images, minify JS/CSS"
            }
          ]
        }
      ]
    }
  ],
  "pages": [
    {
      "url": "https://example.com/",
      "score": 92,
      "issues": [],
      "fetch_time_ms": 450
    }
  ]
}
```

**Design Principles:**
- Machine-readable first (CI/CD integration)
- Hierarchical: summary → category → metric → page
- Actionable: every issue includes specific recommendation
- Parseable: future web UI can render from this structure
- Versioned: schema evolution via `version` field

**Output Modes:**
```
--format=json (default)    → Structured JSON to stdout
--format=json-pretty       → Human-readable JSON with indentation
--output=report.json       → Write to file instead of stdout
```

**Exit Codes:**
```
0 - No critical/high issues (or all issues below threshold)
1 - Critical issues found
2 - Crawl error (network, parsing failures)
```

**Dependencies:**
- `serde` - Serialization framework
- `serde_json` - JSON output

**Confidence:** HIGH (JSON schema design well-documented, serde patterns standard)

### 5. CLI Interface

**Responsibility:** Parse arguments, orchestrate components, handle errors

**Inputs:**
- Command-line arguments
- Configuration file (optional, future)

**Outputs:**
- Orchestrates full pipeline
- Progress indicators (stderr, if not JSON-only)
- Final report (stdout)

**Key Architecture Patterns:**

**Command Structure:**
```bash
geodaddy [URL] [OPTIONS]

geodaddy https://example.com
geodaddy https://example.com --max-depth=2 --render-js
geodaddy https://example.com --output=report.json
geodaddy https://example.com --category=technical  # Run only one category
```

**Argument Parsing:**
- Use `clap` (derive API for type-safe parsing)
- Validation: URL format, depth limits, file paths

**Pipeline Orchestration:**
```rust
async fn run(config: CliConfig) -> Result<Report> {
    // 1. Initialize crawler
    let crawler = Crawler::new(config.crawl_opts);

    // 2. Discover URLs (sitemap-first, fallback to seed)
    let urls = discover_urls(&config.url).await?;

    // 3. Crawl pages (async concurrency)
    let pages = crawler.crawl_all(urls).await?;

    // 4. Parse HTML (parallel via rayon or tokio)
    let parsed: Vec<ParsedDocument> = pages
        .into_iter()
        .map(|p| Parser::parse(p))
        .collect();

    // 5. Run analyzers (async, all analyzers in parallel)
    let registry = build_analyzer_registry();
    let results = registry.run_all(&parsed).await?;

    // 6. Generate report
    let report = ReportGenerator::generate(results);

    Ok(report)
}
```

**Error Handling:**
- Network errors: retry with backoff, skip page on repeated failure
- Parse errors: log warning, skip page analysis
- Analysis errors: mark metric as skipped, continue with others
- Fatal errors: invalid URL, no pages crawled → exit code 2

**Progress Reporting (v1 - Minimal):**
```
[stderr] Crawling: 15/42 pages complete...
[stderr] Analyzing: 42 pages...
[stdout] {JSON report}
```

**Future (v2):**
- Interactive TUI via `ratatui` (progress bars, live metrics)
- Configuration file support (YAML/TOML)

**Dependencies:**
- `clap` - CLI argument parsing
- `tokio` - Async runtime
- `anyhow` - Error handling

**Confidence:** HIGH (Clap patterns standard, orchestration straightforward)

## Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                        CLI Interface                            │
│  (Parse args, validate, orchestrate pipeline, output report)    │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Crawler Engine                             │
│  Input:  Seed URL, crawl config                                 │
│  Tasks:  1. Parse robots.txt                                    │
│          2. Fetch sitemap.xml (priority-based queue)            │
│          3. Spawn worker pool (tokio tasks)                     │
│          4. Crawl with rate limiting + politeness               │
│  Output: Vec<RawPage> (HTML + metadata)                         │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Content Parser                              │
│  Input:  Vec<RawPage>                                           │
│  Tasks:  1. Parse HTML → DOM (scraper)                          │
│          2. Extract links → Normalize URLs                      │
│          3. Extract headings → Build hierarchy tree             │
│          4. Parse schema markup → Validate types                │
│          5. Extract meta tags, lists, images                    │
│  Output: Vec<ParsedDocument>                                    │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Analysis Engine                              │
│  Input:  Vec<ParsedDocument>                                    │
│  Tasks:  1. Load analyzer registry (trait objects)              │
│          2. Run analyzers in parallel (tokio::spawn)            │
│          3. Aggregate results by category                       │
│          4. Calculate weighted scores                           │
│          5. Generate recommendations                            │
│  Output: AnalysisReport (scores + issues + recs)                │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Report Generator                              │
│  Input:  AnalysisReport                                         │
│  Tasks:  1. Build hierarchical JSON structure                   │
│          2. Sort issues by severity                             │
│          3. Format recommendations                              │
│          4. Serialize to JSON (serde)                           │
│  Output: JSON string → stdout or file                           │
└─────────────────────────────────────────────────────────────────┘
```

### Data Structures in Flow

```rust
// Crawler output
struct RawPage {
    url: Url,
    html: String,
    status_code: u16,
    fetch_time_ms: u64,
    final_url: Url,  // After redirects
}

// Parser output
struct ParsedDocument {
    url: Url,
    dom: scraper::Html,
    links: Vec<Url>,
    headings: HeadingTree,
    schema_markup: Vec<SchemaObject>,
    meta: MetaTags,
    timing: PageTiming,
}

// Analyzer output
struct AnalysisReport {
    overall_score: f32,
    category_scores: HashMap<Category, CategoryScore>,
    pages: Vec<PageAnalysis>,
}

struct CategoryScore {
    name: Category,
    score: f32,
    weight: f32,
    metrics: Vec<MetricResult>,
}

struct MetricResult {
    name: String,
    passed: bool,
    score: f32,
    issues: Vec<Issue>,
    recommendations: Vec<String>,
}
```

## Monorepo Structure

Geodaddy uses Cargo workspace for CLI + future web UI code sharing:

```
geodaddy/
├── Cargo.toml                    # Workspace root (virtual manifest)
├── cli/
│   ├── Cargo.toml               # Binary crate
│   └── src/
│       ├── main.rs              # CLI entry point, arg parsing
│       └── commands/            # CLI-specific logic
├── core/
│   ├── Cargo.toml               # Library crate (shared logic)
│   └── src/
│       ├── lib.rs
│       ├── crawler/             # Crawler engine module
│       ├── parser/              # Content parser module
│       ├── analyzer/            # Analysis engine + trait
│       │   ├── mod.rs           # Registry, trait definition
│       │   ├── technical/       # Technical analyzers
│       │   └── content/         # Content analyzers
│       └── report/              # Report generator module
├── web/                          # Future web UI (v2)
│   ├── Cargo.toml               # Web server binary
│   └── src/
│       └── main.rs              # Depends on `core` crate
└── README.md
```

### Workspace Configuration

**Root Cargo.toml:**
```toml
[workspace]
members = ["cli", "core"]
resolver = "2"

[workspace.dependencies]
# Shared dependencies (versions declared once)
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
reqwest = { version = "0.11", features = ["json"] }
```

**cli/Cargo.toml:**
```toml
[package]
name = "geodaddy"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "geodaddy"
path = "src/main.rs"

[dependencies]
geodaddy-core = { path = "../core" }
clap = { version = "4.4", features = ["derive"] }
tokio = { workspace = true }
```

**core/Cargo.toml:**
```toml
[package]
name = "geodaddy-core"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { workspace = true }
reqwest = { workspace = true }
serde = { workspace = true }
scraper = "0.18"
governor = "0.6"
dashmap = "5.5"
```

### Benefits of This Structure

1. **Code Reuse:** Web UI (v2) imports `geodaddy-core` for analysis logic
2. **Single Build Cache:** All workspace members share `target/` directory
3. **Version Consistency:** `Cargo.lock` at workspace root ensures all crates use same dependencies
4. **Independent Testing:** `cargo test -p core` tests only core library
5. **Single Binary Distribution:** CLI compiles to standalone executable (v1)

### Future Web UI Integration (v2)

```rust
// web/src/main.rs (future)
use geodaddy_core::{Crawler, Analyzer, ReportGenerator};
use axum::{Router, Json};

async fn analyze_url(url: String) -> Json<Report> {
    let crawler = Crawler::new(Config::default());
    let pages = crawler.crawl_all(vec![url]).await;
    let parsed = Parser::parse_all(pages);
    let results = AnalyzerRegistry::default().run_all(&parsed).await;
    Json(ReportGenerator::generate(results))
}
```

**Confidence:** HIGH (Cargo workspace patterns well-documented, standard Rust monorepo approach)

## Component Boundaries

### Strict Boundaries (Enforced)

| Component | Public API | Private Details |
|-----------|-----------|----------------|
| **Crawler** | `crawl_all(urls) -> Vec<RawPage>` | Queue management, rate limiting, robots.txt cache |
| **Parser** | `parse(html) -> ParsedDocument` | DOM traversal, selector queries |
| **Analyzer** | `Analyzer` trait, `run_all() -> Report` | Individual analyzer implementations |
| **Reporter** | `generate(results) -> String` | JSON formatting logic |

### Communication Patterns

- **Crawler → Parser:** Owned data transfer (no shared state)
- **Parser → Analyzer:** Shared read-only access via `&ParsedDocument`
- **Analyzer → Reporter:** Owned `AnalysisReport` struct
- **All components → Logger:** Shared `tracing` subscriber

### No Direct Dependencies

```
Crawler ✗→ Analyzer     (crawler doesn't know about analyzers)
Parser  ✗→ Reporter     (parser doesn't know about reporting)
Analyzer ✗→ Crawler     (analyzers operate on parsed data only)
```

This isolation enables:
- Unit testing with mocked dependencies
- Swapping implementations (e.g., different parsers)
- Future web UI to import only needed components

## Suggested Build Order

### Phase 1: Foundation (Week 1-2)

**Goal:** Basic pipeline working end-to-end

1. **Core Data Structures**
   - Define `RawPage`, `ParsedDocument`, `AnalysisReport` structs
   - Implement serialization (`serde` derives)
   - **Why first:** All components depend on these types

2. **Simple Crawler (No Politeness Yet)**
   - Fetch single URL with `reqwest`
   - Basic error handling
   - **Why second:** Needed to get HTML for parser development

3. **Content Parser**
   - Parse HTML → extract headings, links, meta tags
   - Ignore schema markup initially
   - **Why third:** Enables analyzer development with real data

4. **CLI Scaffolding**
   - Arg parsing with `clap`
   - Orchestration: crawl → parse → print debug output
   - **Why fourth:** Integration testing of pipeline

**Deliverable:** `geodaddy https://example.com` fetches and parses one page

### Phase 2: Analysis (Week 3-4)

**Goal:** Scoring system working

5. **Analyzer Trait + Registry**
   - Define `Analyzer` trait
   - Implement registry with static registration
   - **Why first:** Framework for all analyzers

6. **2-3 Simple Analyzers**
   - `HeadingHierarchyAnalyzer` (easy - just walk tree)
   - `MobileViewportAnalyzer` (easy - check meta tag)
   - **Why second:** Validate trait design with real implementations

7. **Scoring System**
   - Weighted category scores
   - Overall score calculation
   - **Why third:** Integrate analyzer results into scores

8. **Report Generator**
   - JSON output with schema from architecture doc
   - **Why fourth:** Completes analysis → output pipeline

**Deliverable:** `geodaddy https://example.com --format=json` outputs scored report

### Phase 3: Crawling (Week 5-6)

**Goal:** Multi-page crawling with politeness

9. **URL Queue + Deduplication**
   - Tokio channel-based queue
   - HashSet for visited tracking
   - **Why first:** Foundation for multi-page crawling

10. **Worker Pool**
    - Fixed pool of async workers
    - In-flight task tracking
    - **Why second:** Concurrent crawling without unbounded spawning

11. **Rate Limiting**
    - Global rate limiter via `governor`
    - Per-domain limiters
    - **Why third:** Politeness critical before crawling multiple pages

12. **Robots.txt Parsing**
    - Fetch and cache robots.txt per domain
    - Crawl-delay directive handling
    - **Why fourth:** Complete politeness implementation

**Deliverable:** `geodaddy https://example.com --max-depth=2` crawls site with politeness

### Phase 4: Advanced Features (Week 7-8)

**Goal:** Sitemap, schema, remaining analyzers

13. **Sitemap Parser**
    - Fetch sitemap.xml
    - Priority-based URL queue seeding
    - **Why first:** Better crawl coverage, uses existing crawler

14. **Schema Markup Extraction**
    - JSON-LD parser
    - Microdata/RDFa support
    - **Why second:** Unlocks schema-related analyzers

15. **Remaining Analyzers**
    - `SchemaMarkupAnalyzer`
    - `PageSpeedAnalyzer`
    - `ListStructureAnalyzer`
    - **Why third:** Complete v1 analyzer set

16. **Headless Browser (Optional)**
    - `rust-headless-chrome` integration
    - JS rendering flag
    - **Why last:** Complex, optional feature

**Deliverable:** Full v1 feature set, ready for testing

### Dependencies Between Phases

```
Phase 1 (Foundation)
    ↓
Phase 2 (Analysis) ← requires parsed data from Phase 1
    ↓
Phase 3 (Crawling) ← needs analysis to validate multi-page results
    ↓
Phase 4 (Advanced) ← builds on all previous phases
```

**Critical Path:** Foundation → Analysis (blockers for all other work)
**Parallel Work Possible:** Once Phase 2 complete, individual analyzers (Phase 4 step 15) can be built independently

## Anti-Patterns to Avoid

### 1. Unbounded Async Spawning

**Bad:**
```rust
for url in urls {
    tokio::spawn(crawl(url));  // Spawns millions of tasks
}
```

**Good:**
```rust
let semaphore = Arc::new(Semaphore::new(100));
for url in urls {
    let permit = semaphore.acquire().await;
    tokio::spawn(async move {
        let _permit = permit;  // Held until task completes
        crawl(url).await;
    });
}
```

**Why:** Prevents memory exhaustion, controls concurrency

### 2. Ignoring Politeness

**Bad:**
```rust
// Blast 1000 requests/sec at a single domain
```

**Good:**
```rust
// Per-domain rate limiter, respect crawl-delay, exponential backoff
```

**Why:** Ethical crawling, avoid IP bans, respect server resources

### 3. Tight Coupling Between Components

**Bad:**
```rust
// Crawler directly calls analyzer.run()
impl Crawler {
    fn crawl(&self) -> AnalysisReport {  // Crawler knows about analysis
        let html = self.fetch();
        analyze(html)  // Tight coupling
    }
}
```

**Good:**
```rust
// Crawler returns data, orchestrator connects components
let pages = crawler.crawl_all(urls).await;
let parsed = parser.parse_all(pages);
let report = analyzer.run_all(parsed);
```

**Why:** Testability, modularity, future web UI can reuse components

### 4. Synchronous Blocking in Async Context

**Bad:**
```rust
async fn crawl(url: Url) {
    let html = reqwest::blocking::get(url);  // Blocks Tokio thread
}
```

**Good:**
```rust
async fn crawl(url: Url) {
    let html = reqwest::get(url).await;  // Async, yields to executor
}
```

**Why:** Blocking ties up Tokio worker thread, kills concurrency

### 5. Dynamic Plugin Loading in v1

**Bad:**
```rust
// Load .so files at runtime - complex, ABI instability
```

**Good:**
```rust
// Static compilation, trait-based registry - simple, reliable
registry.register(Box::new(HeadingAnalyzer));
```

**Why:** Rust lacks stable ABI, dynamic loading requires exact compiler match - defer to v2

### 6. Unversioned JSON Schema

**Bad:**
```json
{
  "score": 85,
  "issues": [...]
}
```

**Good:**
```json
{
  "version": "1.0.0",
  "score": 85,
  "issues": [...]
}
```

**Why:** Future web UI needs schema evolution, breaking changes detectable

## Scalability Considerations

| Concern | At 10 Pages | At 1,000 Pages | At 100,000 Pages |
|---------|------------|----------------|------------------|
| **Memory** | Load all in RAM | Load all in RAM | Stream processing, SQLite cache |
| **Crawl Time** | Seconds | Minutes | Hours - chunked processing |
| **Rate Limiting** | Global limiter | Per-domain limiters | IP rotation, distributed crawling |
| **Storage** | In-memory report | JSON file | Database for results |
| **Concurrency** | 100 workers | 100 workers | 500 workers, multiple machines |

**v1 Target:** 1,000 pages (most small-medium sites)
**v2 Scope:** 100,000+ pages (requires architectural changes)

## Technology Stack Summary

| Category | Technology | Version | Purpose |
|----------|-----------|---------|---------|
| **Language** | Rust | 1.75+ | Performance, safety, single binary |
| **Async Runtime** | Tokio | 1.35+ | Concurrent crawling |
| **HTTP Client** | reqwest | 0.11+ | Fetch HTML, connection pooling |
| **HTML Parser** | scraper | 0.18+ | DOM traversal, CSS selectors |
| **Rate Limiting** | governor | 0.6+ | Token bucket algorithm |
| **Concurrent HashMap** | dashmap | 5.5+ | Lock-free visited URL tracking |
| **CLI Parsing** | clap | 4.4+ | Argument parsing (derive API) |
| **Serialization** | serde, serde_json | 1.0+ | JSON report output |
| **Error Handling** | anyhow | 1.0+ | Ergonomic error propagation |
| **Headless Browser** | rust-headless-chrome | 0.12+ | Optional JS rendering |
| **Robots.txt** | robots_txt | 0.4+ | Parse robots.txt files |
| **URL Parsing** | url | 2.5+ | URL normalization |

**Future (v2):**
- `libloading` - Dynamic plugin loading
- `ratatui` - Interactive TUI
- `axum` - Web server for web UI

## Key Architectural Decisions

### Decision 1: Trait-Based vs Dynamic Plugin System

**Chosen:** Trait-based static compilation for v1

**Rationale:**
- Rust lacks stable ABI - dynamic loading requires exact compiler version
- Static plugins simpler to test and debug
- No runtime overhead from dynamic dispatch (trait objects optimize well)
- Future v2 can add dynamic loading if needed

**Tradeoff:** Users can't add analyzers without recompiling (acceptable for v1)

### Decision 2: Tokio vs Async-std vs Rayon

**Chosen:** Tokio for async I/O, rayon for CPU-bound parsing (if needed)

**Rationale:**
- Tokio ecosystem mature, wide library support (reqwest, governor)
- Async ideal for I/O-bound crawling (network waiting)
- Rayon better for parallel HTML parsing (CPU-bound)

**Tradeoff:** Mixing runtimes adds complexity - defer rayon until proven necessary

### Decision 3: Single Binary vs Library + Binary

**Chosen:** Workspace with `core` library + `cli` binary

**Rationale:**
- Future web UI reuses `core` logic
- Library enables integration tests without running full CLI
- Monorepo simplifies dependency management

**Tradeoff:** Slightly more complex initial setup (workspace config)

### Decision 4: JSON-only Output (v1) vs Rich Terminal Output

**Chosen:** JSON-only for v1

**Rationale:**
- CI/CD integration primary use case
- Web UI (v2) will consume JSON
- TUI adds complexity without MVP value

**Tradeoff:** Less user-friendly for manual testing - mitigated by pretty-print option

## Research Gaps & Future Investigation

### Items Requiring Phase-Specific Research

1. **JS Rendering Detection** (Phase 4)
   - How to detect if page requires JS rendering?
   - Heuristics: empty body, "loading..." text, framework detection?
   - Confidence: MEDIUM - need experimentation

2. **Schema Validation** (Phase 4)
   - Full schema.org vocabulary validation complex
   - Adobe library exists but JavaScript - port or call via Node?
   - Confidence: LOW - need prototyping

3. **Scoring Weights** (Phase 2)
   - Current weights (60/40 technical/content) are guesses
   - Need user feedback to validate importance
   - Confidence: LOW - requires testing with real sites

4. **Crawl Performance Benchmarks** (Phase 3)
   - Optimal worker pool size?
   - Connection pool limits?
   - Confidence: MEDIUM - need profiling with real workloads

### Known Unknowns

- **Headless Chrome Stability:** rust-headless-chrome less mature than Puppeteer - may have edge cases
- **Sitemap Variations:** sitemap index files, gzipped sitemaps, sitemap_index.xml - need handling
- **robots.txt Edge Cases:** Multiple User-agent blocks, wildcard patterns, non-standard directives

## Sources

### Crawler Architecture
- [DELine - Practical Guide to Go Distributed Crawlers](https://www.de-line.net/2026/03/go-distributed-crawlers-high-performance-proxy-pool/)
- [Best Open-Source Web Crawlers in 2026](https://www.firecrawl.dev/blog/best-open-source-web-crawler)
- [Design a Web Crawler | Hello Interview](https://www.hellointerview.com/learn/system-design/problem-breakdowns/web-crawler)
- [How to Build a High-Performance Web Crawler with Async Rust](https://oneuptime.com/blog/post/2026-01-25-high-performance-web-crawler-async-rust/view)
- [Build A Tiny Web Crawler With Rust and Tokio](https://www.buildwithrs.dev/blog/build-a-tiny-web-crawler-with-rust-and-tokio)
- [Building a web crawler with Rust: Part 1](https://lukemcauley.dev/2025/04/17/rust-web-crawler.html)

### Rate Limiting & Politeness
- [Crawling rules - GitBook](https://codepr.github.io/webcrawler-from-scratch/chapter1/crawling-rules.html)
- [Respecting Robots Exclusion Protocol at Scale](https://medium.com/gumgum-tech/respecting-robots-exclusion-protocol-or-robots-txt-at-scale-60ee57dc1295)
- [What is crawl delay? | Firecrawl Glossary](https://www.firecrawl.dev/glossary/web-crawling-apis/what-is-crawl-delay)

### Sitemap Strategy
- [Sitemap Priority: How to prioritize crawling](https://seo-galaxy.com/en/blog/sitemap-priority)
- [XML Sitemap Priority & Sitemap Change Frequency](https://slickplan.com/blog/xml-sitemap-priority-changefreq)
- [XML Sitemap Best Practices 2025](https://www.trysight.ai/blog/xml-sitemap-best-practices)

### Analysis Pipeline & Scoring
- [How to Monitor Data Pipelines in Rust Using OpenTelemetry](https://www.shuttle.dev/blog/2025/09/23/monitor-data-pipelines-in-rust)
- [Lighthouse Metrics](https://lighthouse-metrics.com/)
- [Measure And Optimize The Lighthouse Performance Score](https://www.debugbear.com/docs/metrics/lighthouse-performance)
- [SEO Audit: Scoring Systems](https://seomator.com/free-seo-audit-tool)

### Plugin System
- [Telegraf Plugin System Architecture](https://deepwiki.com/influxdata/telegraf/2.4-metric-collection-and-processing)
- [Plugins in Rust](https://adventures.michaelfbryan.com/posts/plugins-in-rust/)
- [Plugins in Rust: Diving into Dynamic Loading](https://nullderef.com/blog/plugin-dynload/)
- [How to build a plugin system in Rust](https://www.arroyo.dev/blog/rust-plugin-systems/)

### Monorepo Structure
- [The Ultimate Guide to Building a Monorepo in 2026](https://medium.com/@sanjaytomar717/the-ultimate-guide-to-building-a-monorepo-in-2025-sharing-code-like-the-pros-ee4d6d56abaa)
- [Monorepo with Turborepo 2026](https://www.askantech.com/monorepo-with-turborepo-enterprise-code-management-guide-2026/)
- [Cargo Workspaces - The Rust Programming Language](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- [Monorepos with Cargo Workspace and Crates](https://earthly.dev/blog/cargo-workspace-crates/)
- [Building a Monorepo with Rust](https://earthly.dev/blog/rust-monorepo/)
- [Structuring Rust Projects With Multiple Binaries](https://www.justanotherdot.com/posts/structuring-rust-projects-with-multiple-binaries)

### Headless Browser
- [rust-headless-chrome GitHub](https://github.com/rust-headless-chrome/rust-headless-chrome)
- [headless_chrome crate](https://docs.rs/headless_chrome/latest/headless_chrome/)

### Schema Validation
- [Schema.org Markup Validator](https://schema.org/docs/validator.html)
- [Adobe Structured Data Validator](https://github.com/adobe/structured-data-validator)

### GEO & Modern SEO
- [Strategic SEO Architecture 2026](https://www.clickrank.ai/strategic-seo-architecture/)
- [The 2026 SEO Roadmap: 4-Layer Framework](https://growth-engines.com/insights/branding/the-2026-seo-roadmap-mastering-the-4-layer-framework-for-modern-visibility)
