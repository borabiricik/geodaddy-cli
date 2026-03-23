<!-- GSD:project-start source:PROJECT.md -->
## Project

**geodaddy**

Open source GEO (Generative Engine Optimization) analysis tool. Analyzes websites for AI-powered search engine optimization — helping content rank in ChatGPT, Perplexity, Google AI Overviews, and similar generative search engines. CLI-first, runs completely locally, outputs machine-readable JSON reports.

**Core Value:** Surface actionable GEO issues with specific fix recommendations — not just scores, but "here's what's wrong and exactly how to fix it."

### Constraints

- **Language**: Rust — single binary distribution, performance for crawling
- **Distribution**: Local CLI first, no cloud dependencies in v1
- **Output**: JSON-only for v1 (enables CI/CD integration, web UI can parse later)
- **Crawling**: Must handle localhost URLs for local development testing
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->
## Technology Stack

## Executive Summary
- **Tokio** for async runtime (industry standard)
- **Reqwest** for HTTP client (11.5k stars, 294k projects)
- **Scraper** for HTML parsing (browser-grade via Servo)
- **Chromiumoxide** for headless browsing (async, comprehensive CDP coverage)
- **Clap** with derive macros for CLI (integrated structopt functionality)
## Core Framework & Runtime
### Async Runtime
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **tokio** | 1.49+ | Async I/O runtime | Industry standard for async Rust. 575M+ downloads. Mature ecosystem. Essential for concurrent HTTP requests. Built-in runtime metrics for detecting blocking operations. Use `full` feature for comprehensive capabilities. |
- De facto async runtime for Rust (2026 consensus)
- Excellent for I/O-bound crawling workloads
- Mature ecosystem with extensive middleware
- Built-in tracing integration
- `async-std`: Less ecosystem support in 2026
- `smol`: Lighter but smaller ecosystem
## HTTP Client & Web Crawling
### HTTP Client
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **reqwest** | 0.13+ | Async HTTP client | Ergonomic, batteries-included HTTP client built on hyper + tokio. 11.5k stars, 294k projects use it. Supports connection pooling, cookies, redirects, JSON serialization. Most mature option for web crawling. |
- Cookie storage (maintains session state)
- Connection pooling (built into async client)
- Customizable redirect handling
- HTTPS via rustls (default) or native-tls
- Timeout configuration (request-level and connection-level)
- Reuse client instances for connection pooling
- Configure timeouts: 30s request, 10s connection
- Enable gzip decompression
- Set custom User-Agent
- `hyper`: Lower-level, more boilerplate
- `surf`: Smaller ecosystem than reqwest
- `ureq`: Synchronous only
### Rate Limiting Middleware
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **reqwest-middleware** + **governor** | Latest | Rate limiting for polite crawling | Enables configurable throttling to avoid overwhelming target servers. Community standard pattern for crawlers. |
## HTML & Data Parsing
### HTML Parsing
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **scraper** | 0.26+ | HTML parsing & CSS selectors | Browser-grade parsing via Servo's html5ever + selectors. 2.3k stars. Standards-compliant HTML5 parsing. CSS selector querying matches browser behavior. Latest release March 18, 2026. |
- HTML5-compliant parsing (same engine as Firefox)
- CSS selector support (standard selectors)
- DOM tree manipulation
- Text and attribute extraction
- Heading hierarchy analysis
- Schema.org markup extraction
- Content structure metrics
- Link discovery for crawling
- `html5ever`: Lower-level, requires more code
- `select.rs`: Less actively maintained
- `kuchiki`: Good but larger API surface
### XML/Sitemap Parsing
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **quick-xml** | 0.38+ | XML parsing for sitemaps | High-performance streaming XML parser. 10-50x faster than xml-rs. Event-based API for large files (e.g., Wikipedia dumps 1GB-100GB). Serde support for structured parsing. |
- Sitemap.xml parsing (sitemap-first strategy)
- Schema.org microdata extraction
- RSS/Atom feed discovery
- sitemapo adds complexity with auto-detection
- quick-xml provides more control for custom parsing
- Broader ecosystem usage (better maintained)
## Headless Browser Integration
### Chrome DevTools Protocol
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **chromiumoxide** | 0.9+ | Headless browser for JS rendering | Async/await support (tokio-based). Auto-generated type bindings from CDP spec (60k LOC generated). More comprehensive protocol coverage than rust-headless-chrome. Automatic Chromium download. Latest release Feb 25, 2026. |
- Full CDP protocol coverage (auto-generated from spec)
- Async API (integrates with tokio)
- Browser binary fetcher (downloads Chromium automatically)
- Type-safe command/event handling
- Sites with client-side rendering
- JavaScript-heavy SPAs
- Dynamic content loading
- Opt-in flag for geodaddy (adds complexity)
- `rust-headless-chrome`: Synchronous API, incomplete CDP coverage, but nicer hand-written API. Version 1.0.21 (Feb 3, 2026) actively maintained but lacks async.
- `fantoccini`: WebDriver-based (works with Firefox/Chrome) but heavier protocol overhead.
## CLI Framework
### Command-Line Parsing
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **clap** | 4.6+ | CLI argument parsing | Structopt functionality integrated into clap v3+. Derive macros (#[derive(Parser)]) for declarative CLI definitions. Industry standard. Well-documented with extensive examples. |
#[derive(Parser)]
#[command(name = "geodaddy")]
#[command(about = "GEO/SEO analysis CLI tool")]
## JSON & Schema Handling
### JSON Serialization
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **serde** + **serde_json** | 1.0.149+ | JSON output format | De facto serialization framework. serde_json v1.0.149 (Jan 6, 2026). Strongly typed, compile-time checked serialization. |
- JSON report output (primary geodaddy output format)
- Schema.org JSON-LD parsing
- API response handling
### JSON Schema Validation
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **jsonschema** | 0.45+ | Schema.org validation | High-performance validator (75-645x faster than valico). Supports Draft 2020-12, 2019-09, 7, 6, 4. Custom format validators. Latest release March 8, 2026. CLI tool available (jsonschema-cli). |
- Validate Schema.org markup
- Structured data compliance checks
- JSON-LD validation
- `valico`: 75-645x slower
- `jsonschema-valid`: Simpler but less feature-complete
- `serde_valid`: Validation via derive macros (different use case)
## Utility Libraries
### URL Parsing
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **url** | 2.5+ | URL parsing & manipulation | Servo's URL parser. WHATWG URL Standard compliant. Type-safe parsing. Version 2.5.7 (Aug 27, 2025). Serde support available. |
- URL normalization
- Query parameter parsing
- Relative URL resolution
- Sitemap URL extraction
### robots.txt Parsing
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **robotstxt** | 0.3+ | robots.txt compliance | Native Rust port of Google's C++ parser. Zero unsafe code. No dependencies. 100% Google original test passed. Matches Googlebot behavior. |
- Check crawl permissions before fetching
- Respect robots.txt directives
- Sitemap discovery (Sitemap: directive)
- robotstxt is Google's official algorithm
- texting_robots is more permissive (54M test corpus)
- For SEO tool, matching Google's behavior is critical
### Regular Expressions
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **regex** | 1.12+ | Pattern matching | Rust's official regex crate. Guaranteed linear time matching (no ReDoS). Unicode support. Latest version 1.12.3. MSRV 1.65. |
- Content pattern extraction
- Schema markup pattern matching
- URL pattern validation
## Error Handling & Logging
### Error Handling
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **anyhow** | Latest | Application error handling | Standard for applications (vs thiserror for libraries). Ergonomic error propagation with context. Clean error messages for CLI output. |
- Use `anyhow` in main application code
- Use `thiserror` if creating library crates (future web/ code)
- CLI tools care about error messages, not matchable types
- `anyhow::Context` adds breadcrumbs for debugging
- Reduces boilerplate in application code
### Logging & Tracing
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| **tracing** | Latest | Structured logging & diagnostics | Tokio ecosystem standard. Structured, event-based logging. Span-based tracing (temporality + causality). OpenTelemetry integration. Actively maintained (2026 updates). |
- CLI execution tracing (--verbose mode)
- Performance span tracking (crawl timing)
- Structured JSON logs for CI/CD
- `log` is facade-only (requires backend)
- `tracing` provides spans (not just log levels)
- Better for async code (tokio integration)
## What NOT to Use
| Technology | Why Avoid | Use Instead |
|------------|-----------|-------------|
| **structopt** | Maintenance mode since clap v3. Functionality merged into clap. | **clap** with derive macros |
| **async-std** | Smaller ecosystem than tokio in 2026. Less middleware support. | **tokio** |
| **ureq** | Synchronous only. Blocking I/O inefficient for concurrent crawling. | **reqwest** (async) |
| **html5ever** (directly) | Low-level, requires boilerplate. No CSS selector support. | **scraper** (wraps html5ever) |
| **xml-rs** | 10-50x slower than quick-xml. Not streaming. | **quick-xml** |
| **rust-headless-chrome** | Synchronous API doesn't integrate with tokio runtime. Incomplete CDP coverage. | **chromiumoxide** (async, complete CDP) |
| **log crate** (alone) | Facade only, needs backend. Less ergonomic than tracing for async. | **tracing** + tracing-subscriber |
| **valico** | 75-645x slower than jsonschema. | **jsonschema** |
| **sitemapo** | Over-engineered for geodaddy needs. Quick-xml sufficient. | **quick-xml** |
## Installation Example
# Core async runtime
# HTTP client & crawling
# Parsing
# Headless browser (optional)
# JSON handling
# CLI framework
# Error handling & logging
## Performance Considerations
### Concurrency Strategy
- **tokio** runtime handles concurrent HTTP requests efficiently
- Use `tokio::spawn` for parallel page fetching
- Connection pooling in reqwest (automatic)
- Rate limiting via governor to respect servers
### Memory Management
- **quick-xml** streaming parser prevents loading entire XML into memory
- **scraper** creates DOM tree (acceptable for single-page analysis)
- For large crawls: process pages sequentially to bound memory
### Binary Size Optimization (if needed)
- Use `regex-lite` instead of `regex` (smaller binary, no Unicode)
- Strip unused tokio features (replace `full` with specific features)
- Use `rustls` instead of `native-tls` (smaller)
## Minimum Supported Rust Version (MSRV)
| Library | MSRV | Notes |
|---------|------|-------|
| tokio 1.49 | 1.71 | Bumped from 1.70 in v1.48 |
| reqwest 0.13 | 1.70 | Stable MSRV |
| jsonschema 0.45 | 1.83 | Highest requirement |
| regex 1.12 | 1.65 | Conservative MSRV |
## Deployment & Distribution
### Single Binary Distribution
- Rust compiles to single static binary (no runtime dependencies)
- Use `cargo build --release` for optimized build
- Cross-compilation via `cross` crate for Linux/macOS/Windows
### Chromium Binary Handling (for --enable-js flag)
- chromiumoxide auto-downloads Chromium on first run
- Alternative: Bundle Chromium binary (increases distribution size significantly)
- Recommended: Document auto-download behavior, let users know first run fetches browser
## Testing Strategy
### Recommended Testing Libraries
### Test Patterns
- Unit tests for parsers (scraper, quick-xml)
- Integration tests with mockito for HTTP mocking
- CLI tests with assert_cmd
- Avoid hitting real websites in tests (use fixtures)
## Confidence Assessment
| Category | Confidence | Reason |
|----------|------------|--------|
| HTTP Client (reqwest) | **HIGH** | Official GitHub, 294k projects, verified version 0.13.2 |
| Async Runtime (tokio) | **HIGH** | Industry standard, verified version 1.49+, official docs |
| HTML Parsing (scraper) | **HIGH** | Servo lineage, verified version 0.26, March 2026 release |
| Headless Browser (chromiumoxide) | **HIGH** | Verified comparison, async advantage, Feb 2026 release |
| CLI Framework (clap) | **HIGH** | Official docs, structopt integration confirmed, version 4.6 |
| JSON Schema (jsonschema) | **HIGH** | Benchmarks verified, version 0.45, March 2026 release |
| XML Parsing (quick-xml) | **HIGH** | Performance benchmarks verified, version 0.38+ |
| Error Handling (anyhow) | **HIGH** | 2026 best practices, multiple authoritative sources |
| Logging (tracing) | **HIGH** | Tokio ecosystem standard, 2026 active maintenance |
| URL Parsing (url) | **HIGH** | Servo's WHATWG implementation, version 2.5.7 |
| robots.txt (robotstxt) | **HIGH** | Google algorithm port, version 0.3 |
| Regex (regex) | **HIGH** | Official crate, version 1.12.3 verified |
| Rate Limiting | **MEDIUM** | WebSearch-based, community pattern (not official docs) |
## Migration from Node/Python Equivalents
| Rust | Node.js | Python | Purpose |
|------|---------|--------|---------|
| reqwest | axios, node-fetch | requests, httpx | HTTP client |
| scraper | cheerio | BeautifulSoup | HTML parsing |
| chromiumoxide | puppeteer | playwright, selenium | Headless browser |
| clap | commander, yargs | argparse, click | CLI framework |
| serde_json | JSON.parse | json module | JSON handling |
| quick-xml | fast-xml-parser | xml.etree | XML parsing |
| tokio | async/await | asyncio | Async runtime |
| anyhow | - | - | Error handling (Rust-specific) |
| tracing | winston, bunyan | logging | Structured logging |
## Sources
### HTTP & Crawling
- [How to choose the right Rust HTTP client - LogRocket Blog](https://blog.logrocket.com/best-rust-http-client/)
- [GitHub - seanmonstar/reqwest](https://github.com/seanmonstar/reqwest)
- [How to Build HTTP Clients in Rust with Reqwest](https://oneuptime.com/blog/post/2026-01-26-rust-reqwest-http-client/view)
- [Web Scraping With Rust - Complete Guide 2026](https://brightdata.com/blog/how-tos/web-scraping-with-rust)
- [5 Best Rust HTML Parsers for Web Scraping - ZenRows](https://www.zenrows.com/blog/rust-html-parser)
- [Rust Web Scraping in 2026 - ZenRows](https://www.zenrows.com/blog/rust-web-scraping)
### Headless Browsers
- [GitHub - rust-headless-chrome/rust-headless-chrome](https://github.com/rust-headless-chrome/rust-headless-chrome)
- [GitHub - mattsse/chromiumoxide](https://github.com/mattsse/chromiumoxide)
- [chromiumoxide vs rust-headless-chrome - LibHunt](https://www.libhunt.com/compare-chromiumoxide-vs-rust-headless-chrome)
### CLI & Parsing
- [Clap and Structopt Crafting Intuitive Rust CLIs | Leapcell](https://leapcell.io/blog/clap-and-structopt-crafting-intuitive-rust-clis)
- [clap::_faq - Rust](https://docs.rs/clap/latest/clap/_faq/index.html)
- [GitHub - Stranger6667/jsonschema](https://github.com/Stranger6667/jsonschema)
- [GitHub - tafia/quick-xml](https://github.com/tafia/quick-xml)
### Async & Runtime
- [The Evolution of Async Rust: From Tokio to High-Level Applications | The RustRover Blog](https://blog.jetbrains.com/rust/2026/02/17/the-evolution-of-async-rust-from-tokio-to-high-level-applications/)
- [How to Use Tokio for Async Runtime in Rust](https://oneuptime.com/blog/post/2026-02-01-rust-tokio-async-runtime/view)
- [Top 5 Tokio Runtime Mistakes That Quietly Kill Your Async Rust](https://www.techbuddies.io/2026/03/21/top-5-tokio-runtime-mistakes-that-quietly-kill-your-async-rust/)
### Error Handling & Logging
- [How to Design Error Types with thiserror and anyhow in Rust](https://oneuptime.com/blog/post/2026-01-25-error-types-thiserror-anyhow-rust/view)
- [Rust Error Handling: thiserror, anyhow, and When to Use Each | Momori Nakano](https://momori.dev/posts/rust-error-handling-thiserror-anyhow/)
- [Getting started with Tracing | Tokio](https://tokio.rs/tokio/topics/tracing)
- [How to Structure Logs Properly in Rust with tracing and OpenTelemetry](https://oneuptime.com/blog/post/2026-01-07-rust-tracing-structured-logs/view)
### Utilities
- [GitHub - servo/rust-url](https://github.com/servo/rust-url)
- [GitHub - Folyd/robotstxt](https://github.com/Folyd/robotstxt)
- [GitHub - rust-lang/regex](https://github.com/rust-lang/regex)
### Sitemaps & robots.txt
- [GitHub - Folyd/robotstxt](https://github.com/Folyd/robotstxt)
- [GitHub - Smerity/texting_robots](https://github.com/Smerity/texting_robots)
- [sitemapo — Rust parser // Lib.rs](https://lib.rs/crates/sitemapo)
### Version Verification
- [reqwest 0.13.2 - GitHub Release](https://github.com/seanmonstar/reqwest)
- [tokio 1.49.0 - Docs.rs](https://docs.rs/crate/tokio/latest/source/README.md)
- [scraper v0.26.0 - GitHub Release](https://github.com/causal-agent/scraper)
- [chromiumoxide v0.9.1 - GitHub Release](https://github.com/mattsse/chromiumoxide)
- [clap 4.6.0 - Docs.rs](https://docs.rs/clap/latest/clap/_faq/index.html)
- [jsonschema 0.45.0 - GitHub Release](https://github.com/Stranger6667/jsonschema)
- [quick-xml 0.38.4 - crates.io](https://crates.io/crates/quick-xml)
- [serde_json 1.0.149 - crates.io](https://crates.io/crates/serde_json)
- [url 2.5.7 - GitHub Release](https://github.com/servo/rust-url)
- [robotstxt 0.3.0 - GitHub](https://github.com/Folyd/robotstxt)
- [regex 1.12.3 - Docs.rs](https://docs.rs/crate/regex/latest)
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd:quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd:debug` for investigation and bug fixing
- `/gsd:execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd:profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
