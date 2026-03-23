# Features Research: GEO Analysis Tools

**Domain:** SEO/GEO analysis tools (Lighthouse, Screaming Frog, Ahrefs Site Audit, etc.)
**Researched:** 2026-03-23
**Overall confidence:** HIGH

## Table Stakes

Features users expect from any SEO/GEO analysis tool. Missing these = users leave.

### Technical SEO Metrics

| Feature | Why Table Stakes | Complexity | Dependencies | Notes |
|---------|------------------|------------|--------------|-------|
| **Broken links (404s) detection** | Fundamental crawl health signal. All tools provide this. | Low | Full site crawl | Must report both 404s and source URLs where broken links appear. [Screaming Frog](https://www.screamingfrog.co.uk/seo-spider/) instantly identifies these. |
| **Redirect chain detection** | Critical for crawl efficiency and link equity. | Low | Full site crawl | Identify temporary (302) vs permanent (301) redirects, loops, and chains. [Ahrefs](https://ahrefs.com/site-audit) provides distribution analysis. |
| **Meta tag analysis (title + description)** | Basic on-page SEO. Expected by all users. | Low | HTML parsing | Check presence, length (title: 50-60 chars/580-600px, description: 120-158 chars/680-920px), uniqueness. [Optimization guide](https://www.straightnorth.com/blog/title-tags-and-meta-descriptions-how-to-write-and-optimize-them-in-2026/). |
| **Heading hierarchy validation** | Semantic structure is critical for AI parsing. | Low | HTML parsing | Check for single H1, logical nesting (no H2→H4 jumps), keyword relevance. [H1 is page-level descriptor](https://designindc.com/blog/why-header-structure-still-matters-in-2026/). |
| **Mobile responsiveness check** | 62.5% of global traffic is mobile (2025). | Low | Viewport meta tag detection | Verify `<meta name="viewport">` presence, test tap target size (≥48px). [Mobile standards](https://bug0.com/blog/how-to-make-a-website-mobile-friendly-in-2026). |
| **robots.txt validation** | Controls crawl access. Misconfiguration = invisible site. | Low | HTTP request | Verify syntax, check for production `Disallow: /` mistake, validate sitemap reference. [Critical for 2026](https://searchengineland.com/robots-txt-seo-453779). |
| **Sitemap.xml analysis** | Crawl prioritization signal. 50K URL limit. | Low | HTTP request + XML parsing | Check existence, validate format, detect URLs blocked by robots.txt (contradictory signal). [Best practices](https://www.straightnorth.com/blog/xml-sitemaps-and-robots-txt-how-to-guide-search-engines-effectively/). |
| **Page load speed (Core Web Vitals)** | Google ranking factor since 2021. Still critical 2026. | Medium | Headless browser or CrUX API | Measure LCP (≤2.5s), INP (≤200ms, replaced FID in 2024), CLS (≤0.1). [75th percentile threshold](https://www.corewebvitals.io/core-web-vitals). |
| **HTTPS/SSL validation** | Security baseline. Non-HTTPS sites are penalized. | Low | URL protocol check | Flag HTTP URLs, detect mixed content issues. |
| **Canonical tag detection** | Prevents duplicate content indexation. | Low | HTML parsing | Verify presence, check self-referencing canonicals, detect conflicts. |

### Content Structure Analysis

| Feature | Why Table Stakes | Complexity | Dependencies | Notes |
|---------|------------------|------------|--------------|-------|
| **Semantic HTML validation** | AI engines rely on semantic markup to parse content. | Low | HTML parsing | Verify proper use of `<article>`, `<main>`, `<nav>`, `<section>` vs `<div>` soup. [Semantic HTML critical for AI](https://searchatlas.com/blog/semantic-html/). |
| **Schema markup detection (JSON-LD)** | 2.5x higher AI citation rate with schema. | Low | HTML parsing for `<script type="application/ld+json">` | Detect presence, validate JSON-LD syntax, identify types (Article, FAQPage, HowTo, BreadcrumbList, Organization, LocalBusiness, Product). [JSON-LD is Google-recommended format](https://technicalseo.com/tools/schema-markup-generator/). |
| **Image alt text check** | Accessibility + SEO basic. All tools provide. | Low | HTML parsing | Flag missing alt attributes on `<img>` tags. |
| **Internal linking analysis** | Link equity distribution and navigation. | Medium | Full site crawl + graph analysis | Identify orphan pages (no internal links), measure internal PageRank distribution. [Ahrefs provides context](https://ahrefs.com/site-audit). |

### CLI Tool Essentials

| Feature | Why Table Stakes | Complexity | Dependencies | Notes |
|---------|------------------|------------|--------------|-------|
| **JSON output format** | Required for CI/CD integration, programmatic use. | Low | JSON serialization | All modern CLI SEO tools provide this: [aiseo-audit](https://github.com/agencyenterprise/aiseo-audit), [vercel-seo-audit](https://yusufhan.dev/projects/vercel-seo-audit). |
| **Exit codes for CI/CD** | Pipeline integration needs pass/fail signal. | Low | Exit code 0 (success) vs 1 (failure) | Provide `--fail-under [threshold]` flag. [Standard pattern](https://github.com/agencyenterprise/aiseo-audit). |
| **Progress indicators** | Responsiveness > speed. Show something in <100ms. | Low | stdout/stderr | Spinner or progress bar for long-running scans. [CLI UX principle](https://clig.dev/). |
| **Help documentation** | Discoverability starts with `--help`. | Low | Argument parser | Clear, scannable help text. Keep instructions ≤75 chars/paragraph. [Design principle](https://www.atlassian.com/blog/it-teams/10-design-principles-for-delightful-clis). |
| **Sensible defaults** | Don't require flags for common use cases. | Low | Configuration | Default to JSON output, reasonable timeout, follow redirects. [Best practice](https://zapier.com/engineering/how-to-cli/). |

## Differentiators

Features that would set geodaddy apart from competitors. Not expected, but high value.

### GEO-Specific Analysis (AI Search Optimization)

| Feature | Value Proposition | Complexity | Why Differentiating | Notes |
|---------|-------------------|------------|---------------------|-------|
| **Listicle format detection** | 74.2% of AI citations come from "Top N" structured content. | Low | No existing tool specifically checks for this GEO pattern. | Detect numbered/bulleted lists, heading structure like "Top 10...", "Best 5...". [Citation data](https://www.gen-optima.com/blog/generative-engine-optimization-best-practices-complete-2026-playbook/). |
| **FAQ schema quality scoring** | FAQ schema improves AI citation rate by 30%, with 41% vs 15% citation rate difference. | Medium | Most tools detect schema presence but don't score quality. | Check answer length (40-60 word sweet spot), question format, JSON-LD validity. [Performance metrics](https://www.getpassionfruit.com/blog/faq-schema-for-ai-answers). |
| **HowTo schema validation** | Step-by-step content gets preferential AI treatment. | Medium | Underutilized schema type. Few tools validate quality. | Verify numbered steps, 1-2 sentence brevity, tool/supply lists. [AI optimization guide](https://www.stackmatix.com/blog/structured-data-ai-search). |
| **Triple schema stacking detection** | Article + ItemList + FAQPage on same page = 2-3x AI citation rate. | Low | Novel GEO pattern (2026). Not measured by existing tools. | Detect multiple JSON-LD blocks on single page. [GEO best practice](https://www.gen-optima.com/blog/generative-engine-optimization-best-practices/). |
| **Quick answer block detection** | AI engines prefer extractable summaries above fold. | Medium | Content structure analysis + positioning. | Detect TL;DR sections, summary blocks in first viewport, paragraph snippets. [TL;DR for AI](https://www.docommunication.io/blog/tldr-section-in-seo). |
| **Content freshness signals** | Content >14 days old shows 23% decline in AI citations without freshness updates. | Low | Meta tag + content analysis. | Check last-modified header, article:modified_time meta tag, dateModified in schema. [Freshness requirement](https://www.gen-optima.com/blog/generative-engine-optimization-best-practices-complete-2026-playbook/). |
| **Citation/statistic density** | Citations improve AI visibility by 40% (Princeton research). | Medium | Content parsing for citations, footnotes, references. | Detect inline citations, reference sections, data points with sources. [GEO ranking factor](https://searchengineland.com/mastering-generative-engine-optimization-in-2026-full-guide-469142). |
| **Answer length optimization** | 40-60 words per answer is optimal for AI extraction. | Low | Text node word counting. | Flag FAQ/HowTo answers outside optimal range. [Sweet spot data](https://www.getpassionfruit.com/blog/faq-schema-for-ai-answers). |
| **BreadcrumbList schema + navigation** | Critical for AI to understand site hierarchy. | Low | Most tools check breadcrumbs but not GEO implications. | Validate 3-5 level depth, check JSON-LD + visual breadcrumbs. [2026 GEO importance](https://www.yocreativ.com/blog/breadcrumbs-seo-in-2026/). |

### Developer Experience

| Feature | Value Proposition | Complexity | Why Differentiating | Notes |
|---------|-------------------|------------|---------------------|-------|
| **Localhost URL support** | Test local dev sites before deployment. | Low | Lighthouse has it, but many cloud tools don't. | Validate `http://localhost:*` and `http://127.0.0.1:*` URLs. Geodaddy explicitly calls this out in PROJECT.md. |
| **Diff mode for regressions** | Detect what changed between scans. | Medium | Advanced feature. [vercel-seo-audit has it](https://yusufhan.dev/projects/vercel-seo-audit). | Compare previous JSON report with current, highlight new issues/fixes. |
| **Actionable fix recommendations** | "Here's exactly how to fix it" vs generic scores. | High | Geodaddy's core value proposition. | Provide code snippets, specific attribute values, before/after examples. [Value differentiator](https://www.straightnorth.com/blog/title-tags-and-meta-descriptions-how-to-write-and-optimize-them-in-2026/). |
| **Per-URL detail level** | Site-wide scans with per-page granularity. | Medium | Screaming Frog provides this, but CLI tools often don't. | JSON output includes URL-level issues, not just site-level rollup. |
| **Parallel analysis** | Fast site-wide scans via concurrent checks. | Medium | Performance optimization. | [vercel-seo-audit runs 11 modules in parallel](https://yusufhan.dev/projects/vercel-seo-audit). |

### Completeness

| Feature | Value Proposition | Complexity | Why Differentiating | Notes |
|---------|-------------------|------------|---------------------|-------|
| **Optional JavaScript rendering** | Modern sites need JS execution for full analysis. | High | Major complexity add. Make opt-in. | Headless browser (Playwright/Puppeteer) for SPA/React sites. [Screaming Frog has this](https://www.screamingfrog.co.uk/seo-spider/). |
| **AI bot management audit** | 2026-specific: Check robots.txt for GPTBot, PerplexityBot, ClaudeBot. | Low | Emerging requirement. Few tools check AI bots. | Flag missing AI bot directives in robots.txt. [2026 best practice](https://searchengineland.com/robots-txt-seo-453779). |

## Anti-Features

Things to deliberately NOT build, and why.

| Anti-Feature | Why Avoid | What to Do Instead | Confidence |
|--------------|-----------|-------------------|------------|
| **Citation tracking / AI mention monitoring** | Requires continuous monitoring of ChatGPT/Perplexity/Gemini APIs. Outside core value of one-time site analysis. | Focus on structural analysis that *enables* citations. Let tools like [Otterly.ai](https://www.evertune.ai/resources/insights-on-ai/the-10-best-ai-visibility-tools-for-2026), [AIclicks](https://aiclicks.io/blog/best-ai-optimization-tools) handle monitoring. | HIGH |
| **E-E-A-T scoring (author credibility)** | Subjective, requires NLP, external validation of author credentials. Out of scope for v1 per PROJECT.md. | Flag presence/absence of author bylines, author schema. Defer qualitative scoring. | HIGH |
| **Competitive benchmarking** | Requires crawling competitor sites, storing historical data, complex comparisons. Out of scope per PROJECT.md. | Provide absolute scores. Let users run tool on competitors themselves. | HIGH |
| **HTML/PDF report generation** | Visual output deferred to v2 (web UI). PROJECT.md explicitly lists as out of scope. | JSON-only for v1. Web UI can render later. | HIGH |
| **Content quality scoring** | Readability, keyword density, sentiment = subjective NLP. Hard to make actionable. | Focus on structural signals (headings, lists, schema) not content quality. | MEDIUM |
| **Backlink analysis** | Requires massive crawl infrastructure (Ahrefs/Moz domain). Not feasible for local CLI. | Ignore off-page SEO. Focus on on-page + technical. | HIGH |
| **Keyword research** | Different domain. Requires search volume data, SERP analysis. | Analyze existing page content structure, not keyword targeting. | HIGH |
| **Real-time monitoring / scheduled scans** | Requires persistence layer, scheduler, notifications. PROJECT.md: "not planned". | One-shot analysis on demand. Users can wrap in cron/CI. | HIGH |
| **Web scraping for competitive analysis** | Legal/ethical concerns, rate limiting, IP blocking. | Only analyze URLs user provides. | HIGH |
| **Visual/screenshot analysis** | Image processing, layout shift detection beyond CLS metric. High complexity, low ROI. | Stick to DOM/HTML analysis. | MEDIUM |

## Feature Dependencies

```
Sitemap-first crawling
  ↓
Full site URL list
  ↓
Per-URL analysis (parallel)
    ↓
    ├─→ Technical metrics (meta tags, redirects, speed)
    ├─→ Content structure (headings, schema, semantic HTML)
    └─→ GEO-specific (listicles, FAQ schema, quick answers)
  ↓
Three-level scoring
  ↓
JSON output
  ↓
(Optional) Diff mode / regression detection
```

**Critical path:** Crawling → URL list → HTML fetch → Analysis → JSON output

**Optional enhancement:** JavaScript rendering (expensive, opt-in)

## MVP Recommendation

### Must-Have (Table Stakes)

**Technical SEO (8 checks):**
1. Broken links (404s + source URLs)
2. Redirect chains (301/302, loops)
3. Meta tags (title 50-60 chars, description 120-158 chars)
4. Heading hierarchy (H1 uniqueness, logical nesting)
5. Mobile viewport tag
6. robots.txt validation (syntax, sitemap reference, production Disallow check)
7. Sitemap.xml (format, URL limit, robots.txt conflicts)
8. HTTPS/SSL

**Content Structure (4 checks):**
1. Semantic HTML (article, main, nav, section)
2. Schema markup presence (JSON-LD detection)
3. Image alt text
4. Heading structure

**CLI Essentials (3 features):**
1. JSON output
2. Exit codes for CI/CD (`--fail-under`)
3. Progress indicators

**Total: 15 checks + 3 CLI features**

### High-Value Differentiators (GEO-Specific)

**Priority 1 (Low complexity, high impact):**
1. Listicle format detection (74.2% of AI citations)
2. Triple schema stacking detection (2-3x citation rate)
3. Content freshness signals (23% citation decline without)
4. AI bot management audit (robots.txt for GPTBot, etc.)

**Priority 2 (Medium complexity, proven impact):**
1. FAQ schema quality scoring (41% vs 15% citation rate)
2. Quick answer block detection (above-fold TL;DR)
3. Citation/statistic density

**Priority 3 (Polish):**
1. HowTo schema validation
2. Answer length optimization (40-60 words)

### Defer to v2

- E-E-A-T signals (author credibility) — per PROJECT.md
- Answer format metrics (Q&A structure) — per PROJECT.md
- HTML/PDF reports — per PROJECT.md
- JavaScript rendering — make opt-in flag if v1 has time
- Diff mode — nice-to-have, not critical path

## Complexity Assessment

| Feature Category | Implementation Effort | Rationale |
|------------------|----------------------|-----------|
| **Basic crawling (sitemap + links)** | Medium | URL frontier management, robots.txt respect, rate limiting. Well-understood problem. Rust crates available (scraper, reqwest). |
| **HTML parsing & DOM analysis** | Low-Medium | Use scraper or html5ever crate. Extract meta tags, headings, schema = straightforward. |
| **Schema validation (JSON-LD)** | Low | Parse JSON in `<script>` tag, validate against schema.org types. Use serde_json. |
| **Core Web Vitals (LCP, INP, CLS)** | High | Requires headless browser (Chrome DevTools Protocol). Use Playwright/Puppeteer equivalent in Rust. Significant complexity. |
| **JavaScript rendering** | High | Headless browser orchestration. Heavy dependency. Make opt-in. |
| **GEO-specific content analysis** | Low-Medium | Regex/pattern matching for listicles, position detection for quick answers, word counting for optimal lengths. Mostly text processing. |
| **JSON output + scoring** | Low | Serde JSON serialization. Scoring = weighted sum of checks. |
| **Actionable recommendations** | Medium-High | Requires recommendation templates for each check, context-aware suggestions. High effort for good quality. |

**Recommendation:** Start with low/medium complexity features. Core Web Vitals and JS rendering are high-value but high-complexity — consider lighthouse CLI integration or defer to v2.

## Feature Comparison Matrix

| Feature | Lighthouse | Screaming Frog | Ahrefs Site Audit | geodaddy (proposed) |
|---------|-----------|----------------|-------------------|---------------------|
| **Crawling** | Single-page | Site-wide | Site-wide | Site-wide (sitemap-first) |
| **JS Rendering** | Yes (built-in) | Yes (opt-in) | Yes | Opt-in flag |
| **Core Web Vitals** | Yes (primary focus) | Via Lighthouse integration | Yes | Consider integration or v2 |
| **Schema Detection** | Basic | Advanced (extraction) | Advanced | Advanced (JSON-LD focus) |
| **GEO-Specific Analysis** | No | No | No | **YES** (differentiator) |
| **FAQ Schema Scoring** | No | Detection only | Detection only | **Quality scoring** |
| **Listicle Detection** | No | No | No | **YES** |
| **AI Bot Audit** | No | No | No | **YES** (2026-specific) |
| **CLI-First** | Yes | No (GUI-first) | No (web-first) | **YES** |
| **Local Operation** | Yes | Yes | No (cloud) | **YES** |
| **JSON Output** | Yes | CSV/JSON export | API/JSON | **Yes** |
| **Actionable Fixes** | Moderate | Flags issues | Flags issues | **Detailed recommendations** (core value) |
| **Localhost Support** | Yes | Yes | No | **YES** |
| **CI/CD Integration** | Lighthouse CI | API/scripts | API | **Built-in** (exit codes, JSON) |

**geodaddy's niche:** CLI-first + local operation + GEO-specific analysis + actionable recommendations for AI search optimization.

## Sources

### Technical SEO & Traditional Tools
- [Lighthouse Overview - Chrome for Developers](https://developer.chrome.com/docs/lighthouse/overview/)
- [Screaming Frog SEO Spider](https://www.screamingfrog.co.uk/seo-spider/)
- [Ahrefs Site Audit Features](https://ahrefs.com/site-audit)
- [Core Web Vitals Explained (2026)](https://www.corewebvitals.io/core-web-vitals)
- [Robots.txt and SEO 2026 - Search Engine Land](https://searchengineland.com/robots-txt-seo-453779)
- [XML Sitemaps & Robots.txt Guide - Straight North](https://www.straightnorth.com/blog/xml-sitemaps-and-robots-txt-how-to-guide-search-engines-effectively/)
- [Meta Title & Description Optimization 2026 - Straight North](https://www.straightnorth.com/blog/title-tags-and-meta-descriptions-how-to-write-and-optimize-them-in-2026/)
- [Header Structure SEO 2026](https://designindc.com/blog/why-header-structure-still-matters-in-2026/)
- [404 Errors and SEO Impact 2026](https://seofreegenius.com/blog/fix-404-errors-redirects-ux-seo/)

### GEO & AI Search Optimization
- [GEO Best Practices 2026 Playbook - GenOptima](https://www.gen-optima.com/blog/generative-engine-optimization-best-practices-complete-2026-playbook/)
- [Mastering GEO 2026 - Search Engine Land](https://searchengineland.com/mastering-generative-engine-optimization-in-2026-full-guide-469142)
- [Generative Search Ranking Factors 2026 - TrySight](https://www.trysight.ai/blog/generative-search-ranking-factors)
- [AI Search Ranking Factors - TrySight](https://www.trysight.ai/blog/ai-search-ranking-factors)
- [FAQ Schema for AI Answers - Get Passionfruit](https://www.getpassionfruit.com/blog/faq-schema-for-ai-answers)
- [Structured Data AI Search Guide 2026 - Stackmatix](https://www.stackmatix.com/blog/structured-data-ai-search)
- [E-E-A-T for AI Search 2026 - Revved Digital](https://revved.digital/eeat-ai-search-ranking-signals-2026/)
- [Listicle Optimization for AI - eseospace](https://eseospace.com/blog/ai-loves-lists-bullets/)
- [TL;DR Sections in SEO - DoCommunication](https://www.docommunication.io/blog/tldr-section-in-seo)
- [Breadcrumbs SEO 2026 - YoCreativ](https://www.yocreativ.com/blog/breadcrumbs-seo-in-2026/)

### Schema Markup & Structured Data
- [JSON-LD Schema Markup Guide - TechnicalSEO](https://technicalseo.com/tools/schema-markup-generator/)
- [Semantic HTML Guide - Search Atlas](https://searchatlas.com/blog/semantic-html/)

### CLI Tool Design
- [Command Line Interface Guidelines - clig.dev](https://clig.dev/)
- [10 Design Principles for Delightful CLIs - Atlassian](https://www.atlassian.com/blog/it-teams/10-design-principles-for-delightful-clis)
- [Best Practices for CLI Tools - Zapier Engineering](https://zapier.com/engineering/how-to-cli/)

### Example CLI SEO Tools
- [aiseo-audit - GitHub](https://github.com/agencyenterprise/aiseo-audit)
- [vercel-seo-audit - Yusufhan Dev](https://yusufhan.dev/projects/vercel-seo-audit)
- [site-audit-seo - GitHub](https://github.com/viasite/site-audit-seo)

### AI Visibility Tools (Anti-Features Reference)
- [10 Best AI Visibility Tools 2026 - Evertune](https://www.evertune.ai/resources/insights-on-ai/the-10-best-ai-visibility-tools-for-2026)
- [13 Best AI Search Visibility Tools - AIclicks](https://aiclicks.io/blog/best-ai-optimization-tools)

### Mobile & Performance
- [Mobile-Friendly Testing 2026 - Bug0](https://bug0.com/blog/how-to-make-a-website-mobile-friendly-in-2026)
