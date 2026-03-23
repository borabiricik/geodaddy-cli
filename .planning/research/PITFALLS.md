# Pitfalls Research: GEO Analysis Tools

**Domain:** Website analysis CLI tools for GEO/SEO optimization
**Researched:** 2026-03-23
**Overall confidence:** MEDIUM-HIGH

Research focused on domain-specific pitfalls for SEO/website analysis CLI tools, including crawler architecture, scoring systems, Rust implementation, CLI UX, and GEO-specific concerns.

---

## Critical Pitfalls

These mistakes cause rewrites, major issues, or complete project failure.

### 1. URL Deduplication Without Proper Normalization

**What goes wrong:**
Crawlers revisit the same content through different URL variants (http/https, www/non-www, trailing slashes, UTM params, session IDs, pagination params), causing massive duplicate work, memory bloat, and inaccurate reporting. Without normalization, you systematically overcount issues and manufacture phantom signals. A site with 100 unique pages can appear as 500+ URLs.

**Why it happens:**
URLs like `example.com/page`, `example.com/page/`, `example.com/page?utm_source=email`, and `example.com/page?sessionid=abc123` all return identical content but look different to naive string comparison.

**Warning signs:**
- Crawl queue grows exponentially beyond expected site size
- Same content analysis appears multiple times in results
- Memory usage spikes unexpectedly during crawls
- Reports show duplicate issues for what should be single pages
- Hash-based deduplication shows different hashes for identical content

**Prevention:**
1. Implement URL normalization BEFORE adding URLs to crawl queue or visited set
2. Normalize: protocol (https), domain (www), path (trailing slash), query params (sort alphabetically, remove tracking params)
3. Respect canonical link tags - Google may choose different canonical than you indicate, but use it as a hint
4. Use URL hashing with xxHash or bloom filters for memory-efficient deduplication at scale
5. Store checksum/hash instead of full URL strings to save memory
6. Handle both sitemap-declared canonicals AND HTML link rel=canonical

**Phase relevance:** Phase 1 (Core Crawler) - Must be in initial crawler design. Retrofitting is expensive and breaks visited-URL tracking.

**Confidence:** HIGH - Multiple authoritative sources, core crawler design pattern

---

### 2. Unbounded Async Task Queues in Rust/Tokio

**What goes wrong:**
If requests pile up faster than they're processed, unbounded channels (`mpsc::channel` without limits) accept them until memory exhausts. Stuck tasks from stalled dependencies (network timeouts, DNS failures) accumulate indefinitely, holding connections, heap allocations, and preventing graceful shutdown. Application crashes with OOM or becomes unresponsive.

**Why it happens:**
Default async patterns prioritize throughput over backpressure. Easy to spawn thousands of concurrent HTTP requests without considering memory constraints or downstream capacity. Tokio makes it too easy to spawn tasks without lifecycle management.

**Warning signs:**
- Memory usage creeps upward continuously during long-running crawls
- Task count grows unbounded (check with metrics/tracing)
- Application becomes unresponsive but CPU is low
- Graceful shutdown takes forever or hangs
- Error messages about "too many open files" or connection pool exhaustion

**Prevention:**
1. Use **bounded channels** (`tokio::sync::mpsc::channel(capacity)`) for all inter-task communication
2. Set explicit concurrency limits with `futures::stream::buffer_unordered(N)` or semaphores
3. Wrap ALL remote calls in `tokio::time::timeout` - assume networks fail
4. Design tasks to drop quickly when parent future is cancelled
5. Monitor: channel buffer usage, active task count, memory per-task
6. Start with small concurrency (10-20) and increase only if bottlenecked
7. Use `DashSet` with URL hashes, not full strings - memory at scale matters

**Phase relevance:** Phase 1 (Core Crawler) - Async architecture must be designed correctly from start. Impossible to bolt on backpressure later.

**Confidence:** HIGH - Recent 2026 Tokio-specific articles, practical experience reports

---

### 3. Blocking AI Crawlers (GEO-Specific)

**What goes wrong:**
Many sites actively block AI engine crawlers (GPTBot, PerplexityBot, Google-Extended, ClaudeBot) in robots.txt, treating AI as a threat rather than traffic source. Cloudflare changed default configuration to block AI bots automatically. If your tool doesn't detect and warn about this, users optimize content that AI engines cannot even see - wasted effort and invisible in AI search results.

**Why it happens:**
Confusion between blocking content scrapers vs. allowing legitimate AI search indexing. Copy-paste robots.txt from old SEO guides. Cloudflare defaults. Fear of content being "stolen" by AI.

**Warning signs:**
- robots.txt contains `Disallow: /` for User-agent: GPTBot, PerplexityBot, Google-Extended, ClaudeBot, CCBot
- Site uses Cloudflare with default settings
- Site has good traditional SEO but zero AI search visibility
- Headers or middleware block bots with "AI" in user-agent

**Prevention:**
1. **Parse robots.txt specifically for AI crawler user-agents** (not just generic crawlers)
2. Flag as CRITICAL issue: "Your site blocks AI search engines - content invisible to ChatGPT, Perplexity, etc."
3. Provide specific fix: Show exact robots.txt entries to remove or modify
4. Check for Cloudflare Bot Fight Mode / Super Bot Fight Mode
5. Distinguish: blocking content scrapers (good) vs. blocking AI search indexing (bad for GEO)

**Phase relevance:** Phase 1 (Technical Metrics) - Core to GEO analysis. Traditional SEO tools miss this entirely.

**Confidence:** HIGH - Multiple 2026 GEO sources, documented Cloudflare behavior change

---

### 4. Arbitrary Scoring Weights Without Validation

**What goes wrong:**
Creating proprietary scores like "GEO Score 0-100" with made-up weights (e.g., "schema markup = 30%, headings = 20%, mobile = 50%") produces meaningless numbers. Users optimize for scores that don't correlate with actual AI search visibility. Becomes "Domain Authority for GEO" - vanity metric that misleads strategy.

**Why it happens:**
Pressure to provide a single number for easy reporting. Looking at how existing SEO tools work (Moz DA, Ahrefs DR) and copying the pattern. Assumption that weighted scores = sophisticated analysis.

**Warning signs:**
- No validation that score correlates with actual outcomes
- Weights chosen by "feels right" not data
- Users ask "how do I improve my score?" instead of "how do I rank in AI search?"
- Score changes without meaningful content changes
- Marketing team sets goals like "improve GEO score from 42 to 60"

**Prevention:**
1. **Start with pass/fail per metric, not scores** - each check either passes, warns, or fails
2. Provide category-level summaries (Technical: 3 pass, 1 warn, 2 fail) not numeric scores
3. If you must score: validate against real AI search visibility data before launch
4. Make scoring optional/secondary - actionable fixes are primary
5. Document exactly what the score means and what it doesn't predict
6. Avoid comparisons to competitors - without validation it's noise

**Phase relevance:** Phase 1 (Scoring System) - Design decision before first metric. Changing scoring later breaks historical data.

**Confidence:** HIGH - Well-documented SEO industry pitfall, 2026 sources on arbitrary metrics

---

### 5. Headless Browser Memory Leaks

**What goes wrong:**
Chrome headless defaults assume desktop workstation with ample resources. Each browser instance spawns multiple processes (browser, GPU, renderer per tab). Launching new instance per request without proper cleanup causes /tmp file buildup, zombie processes, and exhausted file descriptors. In Docker/containers, /dev/shm limits (default 64MB) cause crashes. Memory consumption spirals until OOM killer or system freeze.

**Why it happens:**
Chrome is multi-process by design. Shared memory (/dev/shm) fills up in constrained environments. Failing to close contexts/pages between requests. Not monitoring heap usage. Documentation examples show single-page usage, not long-running crawlers.

**Warning signs:**
- Memory usage increases linearly with pages crawled (should be flat)
- `/tmp` directory fills with Chrome profile data
- "Out of memory" errors from Chrome, not Rust
- Process count grows unbounded
- Crashes in Docker but works on laptop
- Error: "Failed to allocate shared memory" or "cannot create temp directory"

**Prevention:**
1. **Reuse browser contexts** - launching the binary is the bottleneck, create once
2. Close pages and contexts explicitly after each crawl
3. Use `--disable-dev-shm-usage` flag in Docker (critical for containerized deployment)
4. Block unnecessary content aggressively: images, fonts, media if not needed for analysis
5. Monitor heap usage - let V8 crash a single request rather than whole server
6. Set resource limits per page (memory, CPU time)
7. Implement page-level timeouts - zombie tabs holding resources
8. Clean /tmp periodically or use ephemeral directories
9. Make JS rendering **opt-in** not default - most sites don't need it

**Phase relevance:** Phase 1 (Optional JS Rendering) - Architecture must handle lifecycle correctly from start or you build technical debt.

**Confidence:** HIGH - Recent March 2026 article, Chromium bug reports, production deployment guides

---

### 6. JSON Output Schema Without Versioning

**What goes wrong:**
When you inevitably add fields, change field names, or restructure output, existing CI/CD pipelines that parse your JSON break. No way for consumers to handle multiple schema versions gracefully. Users pin to old version forever or update breaks their automation. Creates fragmentation: some users on v1 schema, others on v2, support nightmare.

**Why it happens:**
First version doesn't include schema version field. Assumption that JSON is "flexible" so changes won't break things. Underestimating how many users build automation around output structure.

**Warning signs:**
- Bug reports: "Update broke our CI pipeline"
- Users request "don't change JSON format"
- No migration path when adding/removing fields
- Multiple documentation versions for different output formats
- Parser code has version-detection hacks

**Prevention:**
1. **Include `schema_version` field in root of JSON from day 1** (e.g., "1.0.0")
2. Use semantic versioning: major.minor.patch
3. Major bump = breaking changes (field renamed/removed, type changed)
4. Minor bump = additive changes (new optional fields)
5. Patch bump = no schema change (bug fixes in analysis logic)
6. Document schema explicitly (JSON Schema format recommended)
7. Test output against schema validator in CI
8. Provide migration guide when schema changes
9. Consider: support reading previous schema versions for 1-2 major releases

**Phase relevance:** Phase 1 (JSON Output) - Must be in v1 output or can never add it without breaking change.

**Confidence:** MEDIUM - Combined CI/CD and JSON schema sources, GSoC 2026 project on schema compatibility

---

## Medium Pitfalls

Significant issues but not project-killers. Cause user frustration, bad UX, or require refactoring.

### 7. Crawler Infinite Loops (Classic Trap)

**What goes wrong:**
Circular links (A→B→A), dynamic URL generation, calendar pages with infinite next/previous, paginated listings without end detection cause crawler to run forever, exhaust memory, or timeout without completing analysis.

**Why it happens:**
Not tracking visited URLs. Depth-first crawling gets stuck in one site section. Malformed HTML generates new URLs infinitely. Missing loop detection.

**Warning signs:**
- Crawl never completes on medium-sized sites
- URL queue size grows without bound
- Same paths appear repeatedly with different params
- Crawl depth exceeds reasonable limits (>10 levels for typical site)

**Prevention:**
1. Track visited URLs with in-memory set (use URL hash for memory efficiency)
2. Set maximum crawl depth (default 5-7 levels)
3. Set maximum pages per domain (default 1000-5000)
4. Use breadth-first crawling, not depth-first (prevents getting stuck in deep sections)
5. Detect pagination patterns and set page limits
6. Timeout entire crawl operation (e.g., 10 minutes for single site)
7. URL-seen test BEFORE adding to frontier queue

**Phase relevance:** Phase 1 (Core Crawler) - Basic crawler hygiene

**Confidence:** HIGH - Classic web crawler design pattern, multiple sources

---

### 8. Ignoring robots.txt Politeness Rules

**What goes wrong:**
Hammering servers with hundreds of parallel requests triggers rate limiting, IP bans, or Cloudflare challenges. Violating crawl-delay directives gets your tool blocked. Crawling disallowed paths wastes time and violates site preferences. Users complain tool "doesn't work" on their site.

**Why it happens:**
Maximizing speed over politeness. Not implementing robots.txt parser. Misunderstanding that being a "tool" not "search engine" doesn't exempt you from REP (Robots Exclusion Protocol).

**Warning signs:**
- Frequent 429 (Too Many Requests) errors
- 403 (Forbidden) errors during crawling
- Cloudflare challenges or CAPTCHAs
- User reports of IP bans during testing
- Complaints from site admins about aggressive crawling

**Prevention:**
1. **Respect robots.txt Disallow directives** - parse and honor before crawling
2. Implement Crawl-delay if specified (typically 10-15 seconds between requests)
3. Default to polite rate: 1 request per second per domain
4. Use exponential backoff on errors (429, 503)
5. Identify with clear User-Agent including contact URL/email
6. Stop crawling if repeated 403 errors (site is blocking you)
7. Allow users to set custom rate limits (slower for fragile sites)
8. Divide crawl into small batches instead of parallel flood

**Phase relevance:** Phase 1 (Core Crawler) - Required for responsible tool behavior

**Confidence:** HIGH - IETF draft, AWS best practices, multiple 2026 sources

---

### 9. Generic Error Messages Without Context

**What goes wrong:**
Errors like "Failed to crawl", "Invalid URL", "Analysis error" give users no actionable information. They don't know if it's their fault, the site's fault, or a bug. Can't debug or fix the issue. File support tickets instead of self-serving.

**Why it happens:**
Bubbling up raw error messages from libraries. Assuming technical details confuse users. Not distinguishing between error types (user error vs. system error vs. site issue).

**Warning signs:**
- Support requests: "What does this error mean?"
- Users can't determine if issue is fixable
- Bug reports that are actually user mistakes
- Error messages with stack traces or library internals exposed

**Prevention:**
1. Categorize errors: **User Error** (bad input), **Site Issue** (site is broken/unreachable), **System Error** (bug in tool)
2. Error format: "What happened" + "Why" + "What to do"
3. Example: ❌ "Invalid URL" → ✅ "Cannot crawl localhost without protocol. Try: http://localhost:3000"
4. Example: ❌ "Connection failed" → ✅ "Cannot reach example.com - check if site is online or try again later"
5. Preserve user input so they can correct errors (don't make them retype URL)
6. Include error codes for debugging (e.g., E001: robots.txt blocked)
7. Never blame the user even if it is user error
8. For verbose details: use `--verbose` flag, not default output

**Phase relevance:** Phase 1 (CLI Output) - Error handling architecture

**Confidence:** HIGH - CLI UX guidelines (clig.dev), multiple 2026 sources

---

### 10. Missing Self-Signed Certificate Handling for Localhost

**What goes wrong:**
Local development sites (localhost:3000, 127.0.0.1:8080) often use self-signed HTTPS certificates. Default HTTPS clients reject these with certificate errors. Tool fails to analyze local sites, frustrating developer users who want to test before deployment. "Works on production but can't test locally."

**Why it happens:**
HTTPS clients enforce certificate validation by default (correct for production). Not considering local development use case. Assumption that HTTP localhost is sufficient.

**Warning signs:**
- Error: "Certificate verify failed" or "SSL certificate problem"
- Error: "NET::ERR_CERT_AUTHORITY_INVALID" equivalent
- Tool works on production sites but fails on localhost HTTPS
- Users report issues testing local development servers

**Prevention:**
1. Add `--insecure` or `--allow-self-signed` flag for development use
2. Clear warning: "⚠️  Using --insecure disables certificate validation. Only use for local testing."
3. Document: recommend mkcert for trusted local certificates (adds to system trust store)
4. Detect localhost/127.0.0.1 automatically and prompt user to allow
5. Never allow insecure connections by default for non-localhost
6. Handle different error types: self-signed vs. expired vs. wrong domain
7. Firefox note: doesn't use system cert store, may need separate config

**Phase relevance:** Phase 1 (HTTP Client) - Required for "analyze localhost" requirement

**Confidence:** MEDIUM - Multiple sources on self-signed certs, browser-specific quirks

---

### 11. Treating GEO as Traditional SEO

**What goes wrong:**
Using traditional SEO tactics (keyword density, exact-match keywords, backlink counting) for GEO optimization. These don't work for AI search. AI engines prefer natural language, comprehensive answers, and semantic understanding. Users follow advice that actively hurts AI search visibility.

**Why it happens:**
Copying existing SEO tool patterns. Assuming SEO and GEO are the same thing. Not understanding how LLMs process and cite content differently than PageRank.

**Warning signs:**
- Recommendations focus on keyword frequency
- Scoring based on keyword placement (H1, title, first 100 words)
- No consideration of answer completeness or natural language
- Ignoring author attribution and source credibility
- Missing checks for AI-specific signals (schema, FAQ structure, citations)

**Prevention:**
1. GEO checks should focus on: **Answer Completeness**, **Natural Language**, **Structured Data**, **Author Attribution**, **Source Credibility**
2. Flag keyword stuffing as BAD for GEO (not neutral)
3. Check: Does content directly answer questions in first 100-150 words?
4. Check: Are follow-up questions anticipated and addressed?
5. Check: Is author attribution clear (not "content team" or anonymous)?
6. Warn: "Anonymous content = GEO penalty for AI engines"
7. Measure: content comprehensiveness, not keyword count

**Phase relevance:** Phase 2 (Content Structure Metrics) - Distinguish from traditional SEO approach

**Confidence:** HIGH - Multiple 2026 GEO guides, documented differences from SEO

---

### 12. Not Testing Core Web Vitals on Real Devices

**What goes wrong:**
Lab data (Lighthouse in headless Chrome) shows good scores, but real users on mobile devices experience poor performance. Field data (real user metrics) tells different story. Optimizing for lab metrics that don't reflect actual user experience. Particularly bad on mid-tier Android devices with slower processors and throttled connections.

**Why it happens:**
Lab testing is convenient and reproducible. Real device testing is slow and requires hardware. Assumption that lab data approximates reality. Most traffic is mobile but most testing is desktop.

**Warning signs:**
- Lab scores (Lighthouse) are good but Search Console shows poor Core Web Vitals
- Mobile users report slow site but desktop testing shows fast
- CLS score differs dramatically between lab and field
- INP (Interaction to Next Paint) data missing because not enough real users yet

**Prevention:**
1. Document limitation: "Lab metrics shown - may differ from real user experience"
2. Recommend: Test on actual mobile devices (at least one iOS, one mid-tier Android)
3. Recommend: Use throttled connections (3G) for realistic mobile testing
4. Check all page types, not just homepage (product pages, blog posts, etc.)
5. If INP missing: recommend monitoring TBT (Total Blocking Time) as proxy
6. Warn: Field data is authoritative, lab data is estimate
7. For v1: acknowledge limitation. For v2: consider field data integration if feasible

**Phase relevance:** Phase 1 (Technical Metrics) - Set expectations correctly

**Confidence:** HIGH - Multiple 2026 Core Web Vitals sources, Google documentation

---

### 13. Sitemap XML Parsing Without Error Handling

**What goes wrong:**
Malformed XML, contradictory signals (sitemap includes URL, robots.txt blocks it), missing sitemaps despite robots.txt reference, huge sitemaps (>50MB) causing memory issues. Parser crashes or silently fails, crawler falls back to link-following without informing user of sitemap problems.

**Why it happens:**
Assuming sitemaps are well-formed. Not validating sitemap against robots.txt rules. Not handling sitemap indexes (sitemaps pointing to other sitemaps). Memory-loading entire XML without streaming.

**Warning signs:**
- Parser crashes on specific sites
- Silently ignoring sitemap errors
- Not detecting sitemap/robots.txt conflicts
- Memory spike when loading large sitemaps
- Missing pages that are in sitemap but blocked by robots.txt

**Prevention:**
1. Use streaming XML parser for large sitemaps (don't load entire file to memory)
2. Validate: sitemap URLs against robots.txt Disallow rules
3. Flag contradiction: "Sitemap includes URL that robots.txt blocks - conflicting signals"
4. Handle sitemap indexes (sitemap files pointing to other sitemap files)
5. Respect sitemap size limits (50,000 URLs per file, 50MB uncompressed)
6. Parse errors gracefully: "Sitemap XML malformed at line X - falling back to link crawling"
7. Check: robots.txt references sitemap but sitemap doesn't exist
8. Inform user when falling back from sitemap to link-following

**Phase relevance:** Phase 1 (Sitemap Crawling) - Core crawler feature

**Confidence:** MEDIUM - Google documentation, SEO audit tools

---

## Minor Pitfalls

Quality-of-life issues. Won't break the project but harm adoption or polish.

### 14. Rust Binary Size Bloat

**What goes wrong:**
Including heavy dependencies (clap for CLI parsing, tokio with all features, headless browser binaries) produces 50-100MB+ binaries. Debug symbols in release builds. Users complain about download size, slow CI builds, Docker image bloat.

**Why it happens:**
Default Cargo settings include debug info. Enabling all Tokio features when only need subset. Not optimizing for size. Dependencies transitively pulling in large libs.

**Warning signs:**
- Release binary >50MB for basic CLI tool
- Long compile times for dependencies
- Docker image unnecessarily large
- Users on metered connections complain about download size

**Prevention:**
1. Strip debug symbols: `strip = true` in Cargo.toml release profile
2. Optimize for size: `opt-level = "z"` (smaller than "s")
3. Use `cargo-bloat` to identify which dependencies contribute most to size
4. Feature flags: only enable needed Tokio features (e.g., `tokio = { version = "1", features = ["rt-multi-thread", "net"] }`)
5. Separate library deps from binary-only deps (use `[target.'cfg(not(target_env = "msvc"))'.dependencies]` pattern)
6. LTO (Link Time Optimization): `lto = true` for further size reduction
7. Consider: `upx` compression for final binary (not officially supported but works)
8. Document: expected binary size per platform

**Phase relevance:** Phase 1 (CLI Binary) - Affects distribution from first release

**Confidence:** MEDIUM-HIGH - Rust-specific guides, cargo-bloat tool

---

### 15. Schema Markup Validation: Duplicate Detection

**What goes wrong:**
WordPress themes + SEO plugins (Yoast, RankMath) both inject schema markup, creating duplicates. Multiple H1 tags each with Article schema. Reporting "schema present" when actually duplicate/conflicting schemas exist. False positives in validation.

**Why it happens:**
Not checking for duplicates, only presence. Parsing JSON-LD but not checking if multiple blocks define same entity. Different sources (theme, plugin, manual) don't coordinate.

**Warning signs:**
- Schema validation passes but Google Rich Results Test shows issues
- Multiple JSON-LD blocks with same @type and @id
- Conflicting values for same property (e.g., two different author names)

**Prevention:**
1. Parse ALL JSON-LD blocks on page, not just first
2. Check for duplicates: same @type with overlapping properties
3. Warn: "Multiple Article schemas detected - likely conflict between theme and plugin"
4. Validate: properties within same type don't contradict (e.g., two different datePublished)
5. Check: each required property present in at least one valid schema block
6. Provide fix: "Remove duplicate schema - likely from theme or SEO plugin settings"

**Phase relevance:** Phase 2 (Content Structure) - When implementing schema checks

**Confidence:** HIGH - Multiple 2026 schema validation sources, common WordPress issue

---

### 16. Heading Hierarchy: Skipped Levels Not Detected

**What goes wrong:**
Page jumps H2→H4, skipping H3. Breaks document outline, confuses screen readers, reduces AI understanding of content structure. Multiple H1s dilute topic focus. Developers use headings for styling (big text) not semantic structure. Tool reports "headings present" but misses structural problems.

**Why it happens:**
Checking heading presence, not hierarchy validity. CSS makes semantic errors invisible visually. Developers prioritize design over document structure.

**Warning signs:**
- Accessibility audits fail (Lighthouse, axe, WAVE)
- Screen reader users report confusing navigation
- AI-generated answers don't cite content despite good keywords
- Headings used for styling (e.g., H4 for small text callout)

**Prevention:**
1. Validate heading order: must not skip levels (H1→H2→H3, not H1→H3)
2. Check: exactly one H1 per page (multiple H1s dilutes focus)
3. Check: headings form logical outline (each level represents subsection of parent)
4. Warn: "Skipped from H2 to H4 without H3 - breaks document structure"
5. Flag: headings used for styling (detected by checking CSS classes or surrounding content)
6. For GEO: Flag question-based subheadings as POSITIVE (AI citation signal)
7. Provide fix: "Use H3 between H2 and H4 to maintain hierarchy"

**Phase relevance:** Phase 2 (Content Structure) - When implementing heading analysis

**Confidence:** HIGH - Accessibility guidelines, 2026 SEO heading hierarchy sources

---

### 17. Viewport Meta Tag: Incorrect or Missing

**What goes wrong:**
Missing viewport tag makes site display as zoomed-out desktop on mobile. Fixed-width viewport (width=1024) defeats responsive design. Disabling zoom (user-scalable=no) breaks accessibility. Site fails Google mobile-friendly test.

**Why it happens:**
Forgetting to add meta tag. Copy-paste from old tutorials with bad examples. Trying to force specific layout across all devices.

**Warning signs:**
- Site fails Google Mobile-Friendly Test
- Mobile users report having to pinch-zoom to read text
- Responsive CSS doesn't work on mobile
- Content wider than screen causing horizontal scroll

**Prevention:**
1. Check: viewport meta tag present in <head>
2. Validate: `<meta name="viewport" content="width=device-width, initial-scale=1">`
3. Flag as ERROR: fixed pixel width (e.g., width=1024)
4. Flag as ACCESSIBILITY ISSUE: user-scalable=no or maximum-scale=1 (prevents zoom)
5. Check: even with correct viewport, no elements exceed viewport width
6. Test simulation: flag if likely to fail on 375px (iPhone SE) width
7. Provide fix: exact tag to add/replace

**Phase relevance:** Phase 1 (Technical/Mobile) - Basic mobile compatibility

**Confidence:** HIGH - Google documentation, 2026 mobile best practices

---

### 18. CLI Output Too Verbose by Default

**What goes wrong:**
Printing every URL crawled, every check performed, progress updates floods terminal. Users can't see actual issues among noise. Hard to pipe output or use in scripts. Makes JSON output less useful if wrapped in progress text.

**Why it happens:**
Helpful during development to see progress. Assumption users want detailed feedback. Not distinguishing between interactive human use vs. CI/CD automation.

**Warning signs:**
- Users pipe output to file instead of viewing directly
- Complaints: "too much output", "can't find the actual errors"
- JSON output contaminated with progress text
- Hard to use in scripts because of formatting

**Prevention:**
1. Default: quiet mode - only show summary and errors
2. `--verbose` flag for detailed progress
3. `--json` flag: ONLY output JSON, no human text
4. Progress updates: use stderr so stdout is clean for piping
5. Interactive mode: detect TTY and show progress bar (not text spam)
6. Summary at end: "Crawled 47 pages, found 12 issues (3 errors, 9 warnings)"
7. Errors: show immediately, warnings: collect for summary

**Phase relevance:** Phase 1 (CLI Output) - User experience from first use

**Confidence:** HIGH - CLI guidelines (clig.dev), UX patterns

---

### 19. Forgetting Cross-Platform Compatibility (Rust)

**What goes wrong:**
Code works on macOS dev machine but fails on Linux CI or Windows users. Path separators hard-coded (/). Dependencies not available on all platforms. Binary distribution only for one platform.

**Why it happens:**
Developing on single platform. Not testing cross-platform. Using platform-specific APIs without feature gates.

**Warning signs:**
- CI fails on different platform than dev machine
- User reports: "doesn't work on Windows"
- Path errors on different OS
- Dependency compilation failures on specific platforms

**Prevention:**
1. Use `std::path::Path` and `PathBuf`, never string path manipulation
2. Test on all target platforms: Linux, macOS, Windows minimum
3. CI: test on matrix of platforms
4. Check dependencies support all platforms (Cargo.toml platform-specific deps)
5. Binary releases: provide for Linux (x64), macOS (x64, ARM), Windows (x64)
6. Document: minimum supported OS versions
7. Use `#[cfg(target_os = "...")]` for platform-specific code

**Phase relevance:** Phase 1 (CLI Binary) - Architecture decision, hard to retrofit

**Confidence:** MEDIUM - Standard Rust cross-platform advice

---

### 20. Not Handling Redirects in Crawling

**What goes wrong:**
URL redirects (301, 302, 307, 308) not followed, missing content. Redirect chains (A→B→C→D) waste time and may hit limits. Infinite redirect loops (A→B→A). Not preserving redirect information for SEO analysis (301 permanent vs. 302 temporary matters).

**Why it happens:**
Using HTTP client without redirect following enabled. Not limiting redirect depth. Not tracking redirect chains.

**Warning signs:**
- Sitemap URLs return empty content (they redirect but not followed)
- Missing important pages that are behind redirects
- Crawler hangs on redirect loops
- Not reporting SEO issues with redirect chains

**Prevention:**
1. Enable redirect following in HTTP client
2. Limit redirect depth (max 5-7 redirects)
3. Detect redirect loops: if URL appears twice in chain, abort
4. Track redirect chains for SEO analysis
5. Report: "Page X has redirect chain of 4 hops - simplify to direct redirect"
6. Distinguish: 301 (permanent - update links) vs. 302 (temporary - OK)
7. Warn: redirect chains slow crawling and hurt SEO

**Phase relevance:** Phase 1 (Core Crawler) - HTTP client configuration

**Confidence:** HIGH - Standard crawler behavior

---

## Phase-Specific Warnings

Pitfalls likely to emerge in specific development phases.

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| **Phase 1: Core Crawler** | Unbounded queues, no URL normalization, missing rate limiting | Design with backpressure, visited-set, politeness from day 1 |
| **Phase 1: JSON Output** | No schema versioning | Include schema_version field in v1 or never possible |
| **Phase 1: JS Rendering** | Memory leaks from Chrome | Reuse contexts, --disable-dev-shm-usage, make opt-in |
| **Phase 1: CLI** | Verbose output, poor errors | Default quiet, errors with context, --verbose flag |
| **Phase 2: Scoring** | Arbitrary weights | Start pass/fail/warn, avoid vanity metrics |
| **Phase 2: Content Structure** | Missing schema duplicates, heading hierarchy | Validate structure not just presence |
| **Phase 2: GEO Checks** | Using SEO tactics for GEO | Focus on answer completeness, natural language, attribution |
| **CI/CD Integration** | Breaking changes to JSON | Semantic versioning, migration guides |
| **Production Deployment** | Headless Chrome in containers | --disable-dev-shm-usage mandatory in Docker |

---

## Sources

### Crawler Architecture
- [How to avoid getting into infinite loops when designing a web crawler | KodeKnight](https://k2code.blogspot.com/2014/04/how-to-avoid-getting-into-infinite.html)
- [Crawler Traps, how to prevent and fix them | Marketing Tracer](https://www.marketingtracer.com/seo/crawler-traps)
- [Web Scraping: What It Is, How It Works, and Best Practices | Browserless](https://www.browserless.io/blog/web-scraping-guide)
- [URL Normalization for De-duplication of Web Pages | Cornell](https://www.cs.cornell.edu/~hema/papers/sp0955-agarwalATS.pdf)
- [What is URL normalization in web crawling? | Firecrawl Glossary](https://www.firecrawl.dev/glossary/web-crawling-apis/url-normalization-web-crawling)
- [Deduplication & Canonicalization | Potent Pages](https://potentpages.com/web-crawler-development/web-crawlers-and-hedge-funds/deduplication-canonicalization-preventing-double-counts-and-phantom-signals)

### Rate Limiting & Politeness
- [What is polite crawling? | Firecrawl Glossary](https://www.firecrawl.dev/glossary/web-crawling-apis/what-is-polite-crawling)
- [Best practices for ethical web crawlers | AWS Prescriptive Guidance](https://docs.aws.amazon.com/prescriptive-guidance/latest/web-crawling-system-esg-data/best-practices.html)
- [Crawler best practices | IETF Draft](https://www.ietf.org/archive/id/draft-illyes-aipref-cbcp-00.html)
- [Robots.txt Scraping: Rules, Ethics, and Policy Explained | PromptCloud](https://www.promptcloud.com/blog/robots-txt-scraping-compliance-guide/)

### Sitemap & Robots.txt
- [How Google Interprets the robots.txt Specification | Google Developers](https://developers.google.com/search/docs/crawling-indexing/robots/robots_txt)
- [XML Sitemaps & Robots.txt: Guide to Better SEO Crawling | Straight North](https://www.straightnorth.com/blog/xml-sitemaps-and-robots-txt-how-to-guide-search-engines-effectively/)
- [What is URL Canonicalization | Google Search Central](https://developers.google.com/search/docs/crawling-indexing/canonicalization)

### Rust Async & Tokio
- [How to Build a High-Performance Web Crawler with Async Rust | OneUptime](https://oneuptime.com/blog/post/2026-01-25-high-performance-web-crawler-async-rust/view)
- [Top 5 Tokio Runtime Mistakes That Quietly Kill Your Async Rust | techbuddies.io](https://www.techbuddies.io/2026/03/21/top-5-tokio-runtime-mistakes-that-quietly-kill-your-async-rust/)
- [Slow memory creep in long running Tokio process | Rust Forum](https://users.rust-lang.org/t/slow-memory-creep-in-long-running-tokio-process/44115)

### Rust Binary Size
- [How to minimize Rust binary size | GitHub - johnthagen/min-sized-rust](https://github.com/johnthagen/min-sized-rust)
- [I Finally Figured Out Why Rust Binaries Are Massive (and How to Fix It) | Medium](https://medium.com/@neerupujari5/i-finally-figured-out-why-rust-binaries-are-massive-and-how-to-fix-it-b15346c0347d)
- [Binary Size Optimization | Rust Project Primer](https://rustprojectprimer.com/building/size.html)

### Headless Browser
- [Headless Chrome: Configuring and Optimizing Server Memory Consumption | Medium](https://medium.com/@onlineproxypmm/headless-chrome-configuring-and-optimizing-server-memory-consumption-eb8c9a1bc63b)
- [What are the best practices for managing memory usage in Headless Chromium? | WebScraping.AI](https://webscraping.ai/faq/headless-chromium/what-are-the-best-practices-for-managing-memory-usage-in-headless-chromium)
- [The Hidden Cost of Headless Browsers: A Puppeteer Memory Leak Journey | Medium](https://medium.com/@matveev.dina/the-hidden-cost-of-headless-browsers-a-puppeteer-memory-leak-journey-027e41291367)

### CLI UX & Errors
- [Command Line Interface Guidelines | clig.dev](https://clig.dev/)
- [UX patterns for CLI tools | Lucas F. Costa](https://www.lucasfcosta.com/blog/ux-patterns-cli-tools)
- [Error Handling in CLI Tools: A Practical Pattern | Medium](https://medium.com/@czhoudev/error-handling-in-cli-tools-a-practical-pattern-thats-worked-for-me-6c658a9141a9)
- [Error Message UX, Handling & Feedback | Pencil & Paper](https://www.pencilandpaper.io/articles/ux-pattern-analysis-error-feedback)

### SEO Scoring Pitfalls
- [9 SEO Metrics That Will Derail Your 2026 Strategy | ALM Corp](https://almcorp.com/blog/seo-metrics-to-stop-tracking-2026/)
- [SEO Metrics That Matter in 2026 | Minty Digital](https://www.mintydigital.com/blog/seo-metrics-that-matter-in-2026/)

### GEO (Generative Engine Optimization)
- [Mastering generative engine optimization in 2026: Full guide | Search Engine Land](https://searchengineland.com/mastering-generative-engine-optimization-in-2026-full-guide-469142)
- [12 Common GEO Mistakes to Avoid in 2026 | Genixly](https://genixly.io/blogs/common-geo-mistakes-to-avoid-ai-ecommerce)
- [GEO Guide 2026: Generative Engine Optimization Explained | Digital Applied](https://www.digitalapplied.com/blog/geo-guide-generative-engine-optimization-2026)
- [Generative Engine Optimization (GEO): The 2026 Guide | LLMrefs](https://llmrefs.com/generative-engine-optimization)

### Schema Markup
- [Common Schema Markup Errors That Kill Your SEO Rankings | Medium](https://robertcelt95.medium.com/common-schema-markup-errors-that-kill-your-seo-rankings-cc64a83480af)
- [How to Fix Schema Validation Errors | Neil Patel](https://neilpatel.com/blog/schema-errors/)
- [Schema Markup Validator: Improve SEO & Fix Common Errors | WiRe Innovation](https://wireinnovation.com/schema-markup-validation/)

### Heading Hierarchy
- [Header Structure SEO in 2026 | Design in DC](https://designindc.com/blog/why-header-structure-still-matters-in-2026/)
- [HTML Heading Hierarchy (H1–H6): SEO, AI & Accessibility Guide | DevTrios](https://devtrios.com/blog/html-heading-hierarchy/)
- [H1 Tag Best Practices 2026: AI, SEO & Accessibility | DevTrios](https://devtrios.com/blog/h1-tag-best-practices/)

### Core Web Vitals
- [Core Web Vitals Optimization Guide 2026 | Sky SEO Digital](https://skyseodigital.com/core-web-vitals-optimization-complete-guide-for-2026/)
- [Understanding Core Web Vitals and Google search results | Google Developers](https://developers.google.com/search/docs/appearance/core-web-vitals)
- [Front-End Performance in 2026: What Core Web Vitals Actually Mean | Vofox Solutions](https://vofoxsolutions.com/front-end-performance-in-2026)

### Mobile Compatibility
- [Viewport Meta Tag Explained: How It Works on Mobile | Mobile Viewer](https://blog.mobileviewer.io/blog/viewport-meta-tag-explained-how-it-works-on-mobile/)
- [How to Make a Website Mobile Friendly in 2026 | Bug0](https://bug0.com/blog/how-to-make-a-website-mobile-friendly-in-2026)
- [Best Practices for Viewport Meta Tag Setup | OneNine](https://onenine.com/best-practices-for-viewport-meta-tag-setup/)

### Localhost Testing
- [Testing Secure Sites Using Self-Signed Certificates | BrowserStack](https://www.browserstack.com/docs/live/self-signed-certificates)
- [Certificates for localhost | Let's Encrypt](https://letsencrypt.org/docs/certificates-for-localhost/)
- [Understanding Self-signed Certificates | BrowserStack](https://www.browserstack.com/guide/self-signed-certificate)

### JSON Schema & Versioning
- [GSoC 2026: JSON Schema Compatibility Checker | GitHub Issue](https://github.com/json-schema-org/community/issues/984)
- [Designing a Robust Configuration Versioning System with JSON Schema Validation | Medium](https://medium.com/@ansujain/designing-a-robust-configuration-versioning-system-with-json-schema-validation-0f24d7ac53d3)
- [jsonschema-tools | Wikimedia GitHub](https://github.com/wikimedia/jsonschema-tools)

### CI/CD & Versioning
- [Best CI/CD practices matters in 2026 | Kellton](https://www.kellton.com/kellton-tech-blog/continuous-integration-deployment-best-practices-2025)
- [CI/CD Pipelines 2026 Complete Guide | Calmops](https://calmops.com/devops/cicd-pipelines-2026/)

### Actionable Recommendations
- [SEO in 2026: 17 Expert Tips & Predictions | Sitebulb](https://sitebulb.com/resources/guides/seo-in-2026-17-expert-tips-predictions/)
- [47 SEO Best Practices That Drive Results in 2026 | ALM Corp](https://almcorp.com/blog/seo-best-practices-complete-guide-2026/)
- [Actionable SEO Tips for Small Businesses in 2026 | Thryv](https://www.thryv.com/blog/actionable-seo-tips-for-small-businesses/)

---

**Confidence Assessment:**

| Category | Confidence | Notes |
|----------|------------|-------|
| Crawler Architecture | HIGH | Classic patterns + 2026 verification |
| Rust/Tokio Async | HIGH | Recent March 2026 Tokio-specific sources |
| GEO-Specific | HIGH | Multiple comprehensive 2026 GEO guides |
| CLI UX | HIGH | Authoritative clig.dev + multiple sources |
| Headless Browser | HIGH | Recent 2026 article + production experience |
| Scoring Systems | HIGH | Well-documented SEO industry pattern |
| Schema/Content | MEDIUM-HIGH | Current sources, common validation issues |
| Mobile/Viewport | HIGH | Google documentation + 2026 guides |
| Binary Size | MEDIUM-HIGH | Rust-specific tooling and guides |
| JSON Versioning | MEDIUM | CI/CD patterns + GSoC 2026 project |

**Overall:** MEDIUM-HIGH - Core pitfalls have strong 2026 verification, Rust-specific areas have good tooling coverage, GEO domain has comprehensive recent guides. Some areas (JSON versioning) inferred from CI/CD best practices rather than tool-specific sources.
