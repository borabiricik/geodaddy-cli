# geodaddy

Open-source GEO (Generative Engine Optimization) analysis tool. Analyzes websites to help your content rank in AI-powered search engines — ChatGPT, Perplexity, Google AI Overviews, and similar generative engines.

- Runs **completely locally** — no accounts, no API keys, no cloud
- Outputs **machine-readable JSON** for CI/CD pipelines
- Gives **actionable fix recommendations**, not just scores
- Ships as a **single binary** — no runtime dependencies

---

## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage](#usage)
  - [Flags & Options](#flags--options)
  - [Crawling Modes](#crawling-modes)
  - [Output Modes](#output-modes)
- [Scoring](#scoring)
  - [Categories](#categories)
  - [How Scores Are Calculated](#how-scores-are-calculated)
- [Checks Reference](#checks-reference)
  - [Technical](#technical-checks)
  - [Content](#content-checks)
  - [GEO](#geo-checks)
  - [Performance (Core Web Vitals)](#performance-checks)
- [JSON Report Schema](#json-report-schema)
- [CI/CD Integration](#cicd-integration)
- [MCP Server (AI Tool Integration)](#mcp-server-ai-tool-integration)
- [Contributing](#contributing)
- [License](#license)

---

## Installation

### macOS / Linux (one-liner)

```bash
curl -fsSL https://raw.githubusercontent.com/borabiricik/geodaddy-cli/main/install.sh | sh
```

Works on macOS (Intel & Apple Silicon), Linux (x86_64 & arm64), and WSL.

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/borabiricik/geodaddy-cli/main/install.ps1 | iex
```

### Windows (CMD)

```cmd
curl -fsSL https://raw.githubusercontent.com/borabiricik/geodaddy-cli/main/install.cmd -o install.cmd && install.cmd
```

### From source (requires Rust 1.83+)

```bash
git clone https://github.com/borabiricik/geodaddy-cli.git
cd geodaddy-cli
cargo build --release
sudo cp target/release/geodaddy /usr/local/bin/
```

### Verify installation

```bash
geodaddy --version
```

---

## Quick Start

Analyze a single URL and print a human-readable report:

```bash
geodaddy https://example.com --beauty
```

Get a machine-readable JSON report:

```bash
geodaddy https://example.com
```

Crawl an entire site (up to 50 pages):

```bash
geodaddy https://example.com --max-pages 50 --beauty
```

---

## Usage

```
geodaddy <URL> [OPTIONS]
```

### Flags & Options

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `<URL>` | required | — | The URL to analyze. Supports `http://localhost` and `http://127.0.0.1` for local dev. |
| `--max-pages <N>` | optional | — | **Enable crawling** and stop after N pages. Without this flag, only the given URL is analyzed. |
| `--enable-js` | boolean | false | Enable JavaScript rendering for pages detected as JS-heavy. Downloads Chromium (~150 MB) on first use. |
| `--vitals` | boolean | false | Measure Core Web Vitals (LCP, FCP, CLS, TTFB, TBT) via headless browser. Downloads Chromium (~150 MB) on first use. |
| `--beauty` | boolean | false | Output a colored, human-readable report instead of JSON. |
| `--fail-under <SCORE>` | optional | — | Exit with code `1` if the overall score is below this threshold (0–100). Useful for CI gates. |

### Crawling Modes

**Single URL mode** (default — no `--max-pages`):

Analyzes only the URL you provide. No crawling is performed.

```bash
geodaddy https://example.com
```

**Site-wide crawl mode** (`--max-pages` required):

Discovers and analyzes up to N pages. Geodaddy uses a **sitemap-first** strategy:

1. Fetches `/sitemap.xml` and sorts URLs by priority (highest first)
2. Falls back to breadth-first search (depth 2) if no sitemap is found
3. Filters out non-HTML resources (media, CSS, JS, feeds)
4. Deduplicates URLs and strips fragments
5. Respects `Crawl-delay` from `robots.txt` (defaults to 1 second between pages)

Progress is written to **stderr** so the JSON report on stdout stays clean:

```
[3/42] https://example.com/about
[4/42] https://example.com/blog
```

### Output Modes

**JSON (default)** — machine-readable, piped to stdout:

```bash
geodaddy https://example.com > report.json
```

**Beauty mode** — colored, human-readable, written to stdout:

```bash
geodaddy https://example.com --beauty
```

Example beauty output:

```
geodaddy — GEO Analysis Report
URL: https://example.com
Crawled: 2026-03-23T15:30:00Z
─────────────────────────────────────────────────

Overall Score: 75.5/100
Technical: 80.0  Content: 70.0  GEO: 65.0  Performance: N/A

━━━ Page 1/1: https://example.com/ ━━━

  [PASS]  tech-meta-title     Title is 55 chars (optimal range 50-60)
  [PASS]  tech-https          Page served over HTTPS with no mixed content
  [WARN]  geo-listicle        No listicle format detected on this page
      -> Consider restructuring content as a numbered list or 'Top N' format
  [FAIL]  tech-heading-h1     No H1 heading found
      -> Add exactly one <h1> tag per page with your primary keyword
```

---

## Scoring

### Categories

Results are grouped into 4 categories:

| Category | Prefix | What it covers |
|----------|--------|----------------|
| **Technical** | `tech-` | Meta tags, headings, HTTPS, redirects, robots.txt, sitemap |
| **Content** | `cont-` | Heading structure, alt text, JSON-LD schema, semantic HTML |
| **GEO** | `geo-` | Listicle detection, AI bot access, schema stacking |
| **Performance** | `perf-` | Core Web Vitals — only populated when `--vitals` is used |

### How Scores Are Calculated

Each check has a **severity** that determines its point value:

| Severity | Points | Examples |
|----------|--------|---------|
| Critical | 10 | H1 presence, HTTPS, mobile viewport, JSON-LD, AI bot access |
| Medium | 5 | Meta description, redirect chains, alt text, listicle, Core Web Vitals |
| Minor | 2 | Sitemap presence, semantic HTML elements |

Statuses map to points earned:

- **Pass** → full points
- **Warn** → half points
- **Fail** → 0 points

Category score = `(points earned / max possible points) × 100`

Overall score:
- Without `--vitals`: `(technical + content + geo) / 3`
- With `--vitals`: `(technical + content + geo + performance) / 4`

A category with no applicable checks defaults to `100.0`.

---

## Checks Reference

### Technical Checks

| Check ID | Severity | Pass condition |
|----------|----------|----------------|
| `tech-meta-title` | Critical | Title tag present, 50–60 characters |
| `tech-meta-description` | Medium | Meta description present, 120–158 characters |
| `tech-heading-h1` | Critical | Exactly one `<h1>` tag |
| `tech-mobile-viewport` | Critical | `<meta name="viewport" content="width=device-width...">` present |
| `tech-https` | Critical | Served over HTTPS with no mixed-content resources |
| `tech-robots-txt` | Medium | `robots.txt` exists and includes a `Sitemap:` directive |
| `tech-sitemap-xml` | Minor | Valid XML sitemap with ≤50,000 URLs |
| `tech-redirect-chains` | Medium | No chains longer than 2 hops |
| `tech-broken-links` | Medium | Requires `--max-pages` crawl mode to detect |

### Content Checks

| Check ID | Severity | Pass condition |
|----------|----------|----------------|
| `cont-heading-structure` | Medium | No skipped heading levels (e.g., H1 → H3 without H2) |
| `cont-json-ld` | Critical | At least one valid JSON-LD block with `@type` and `schema.org` context |
| `cont-semantic-html` | Minor | At least one semantic element: `<article>`, `<main>`, `<nav>`, `<section>`, `<aside>`, `<header>`, `<footer>` |
| `cont-alt-text` | Medium | All `<img>` elements have a non-empty `alt` attribute |

### GEO Checks

These checks are specific to AI search engine optimization — helping your content get cited by generative AI models.

| Check ID | Severity | Pass condition |
|----------|----------|----------------|
| `geo-listicle` | Medium | Page uses listicle formatting: numbered headings, "Top N" patterns, ordered lists, or comparison tables |
| `geo-schema-stacking` | Medium | All three GEO schema types present: `Article`, `ItemList`, `FAQPage` |
| `geo-ai-bot-gptbot` | Critical | GPTBot (ChatGPT) not blocked in `robots.txt` |
| `geo-ai-bot-claudebot` | Critical | ClaudeBot (Claude) not blocked in `robots.txt` |
| `geo-ai-bot-perplexitybot` | Critical | PerplexityBot (Perplexity) not blocked in `robots.txt` |
| `geo-ai-bot-googleother` | Critical | GoogleOther (Google AI) not blocked in `robots.txt` |
| `geo-ai-bot-bytespider` | Critical | Bytespider (ByteDance AI) not blocked in `robots.txt` |
| `geo-ai-bot-ccbot` | Critical | CCBot (Common Crawl) not blocked in `robots.txt` |

> **Why AI bot checks?** Many sites inadvertently block AI crawlers through catch-all `User-agent: *` directives. If a bot can't read your page, it can't cite it.

### Performance Checks

Only populated when `--vitals` flag is used. Geodaddy launches a headless Chromium instance to measure real browser performance.

| Check ID | Severity | Pass | Warn | Fail |
|----------|----------|------|------|------|
| `perf-lcp` | Critical | ≤2.5s | 2.5–4s | >4s |
| `perf-fcp` | Medium | ≤1.8s | 1.8–3s | >3s |
| `perf-cls` | Medium | ≤0.10 | 0.10–0.25 | >0.25 |
| `perf-ttfb` | Medium | ≤800ms | 800ms–1.8s | >1.8s |
| `perf-tbt` | Medium | ≤200ms | 200–600ms | >600ms |

Thresholds follow [Google's Web Vitals standards](https://web.dev/vitals/).

> **First run note:** `--vitals` and `--enable-js` both require Chromium. geodaddy will automatically download it (~150 MB) on first use via chromiumoxide's built-in fetcher.

---

## JSON Report Schema

```json
{
  "schema_version": "1",
  "url": "https://example.com",
  "crawled_at": "2026-03-23T15:42:30.123Z",
  "score": 82.5,
  "categories": {
    "technical": 85.0,
    "content": 80.0,
    "geo": 75.0,
    "performance": null
  },
  "pages": [
    {
      "url": "https://example.com/",
      "robots_blocked": false,
      "score": 82.5,
      "categories": {
        "technical": 85.0,
        "content": 80.0,
        "geo": 75.0,
        "performance": null
      },
      "results": [
        {
          "check": "tech-meta-title",
          "status": "pass",
          "message": "Title is 55 chars (optimal range 50-60)",
          "recommendation": ""
        },
        {
          "check": "tech-https",
          "status": "fail",
          "message": "Page is served over HTTP, not HTTPS",
          "recommendation": "Migrate to HTTPS — search engines and AI crawlers penalize insecure pages; obtain a TLS certificate from Let's Encrypt (free)"
        }
      ]
    }
  ]
}
```

Key notes:

- `categories.performance` is `null` when `--vitals` is not used
- `status` values are lowercase: `"pass"`, `"warn"`, `"fail"`
- `recommendation` is an empty string `""` for passing checks
- `robots_blocked: true` means the page was skipped per `robots.txt`; results will be empty for that page

---

## CI/CD Integration

### Fail on score threshold

```bash
geodaddy https://example.com --fail-under 80
echo $?  # 1 if score < 80, 0 if score >= 80
```

### GitHub Actions example

```yaml
- name: GEO Analysis
  run: |
    curl -L https://github.com/borabiricik/geodaddy-cli/releases/latest/download/geodaddy-linux-x86_64 -o geodaddy
    chmod +x geodaddy
    ./geodaddy ${{ env.SITE_URL }} --max-pages 20 --fail-under 70
```

### Parse JSON with jq

```bash
# Get overall score
geodaddy https://example.com | jq '.score'

# List all failing checks
geodaddy https://example.com | jq '[.pages[].results[] | select(.status == "fail")]'

# Check if all AI bots are allowed
geodaddy https://example.com | jq '[.pages[].results[] | select(.check | startswith("geo-ai-bot")) | select(.status == "fail")] | length'
```

### Save report to file

```bash
geodaddy https://example.com --max-pages 50 > geo-report-$(date +%Y%m%d).json
```

---

## MCP Server (AI Tool Integration)

geodaddy ships an [MCP](https://modelcontextprotocol.io/) server that lets AI assistants (Claude Desktop, Claude Code, Cursor, etc.) run GEO/SEO analysis via tool calls.

### Setup

#### Claude Desktop

Add to your Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "geodaddy": {
      "command": "npx",
      "args": ["-y", "geodaddy-mcp"]
    }
  }
}
```

#### Claude Code

```bash
claude mcp add geodaddy -- npx -y geodaddy-mcp
```

#### Cursor

Go to **Settings > MCP Servers > Add**, then set:
- **Command:** `npx -y geodaddy-mcp`

#### From source (development)

```bash
cd mcp
npm install
npm run build
```

The MCP server looks for the geodaddy binary in two locations:
1. `mcp/bin/geodaddy` (auto-downloaded via postinstall from GitHub releases)
2. `target/release/geodaddy` (local dev via `cargo build --release`)

### Available Tool

| Tool | Description |
|------|-------------|
| `analyze_url` | Run geodaddy GEO/SEO analysis on a URL. Returns JSON report with scores and fix recommendations. |

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `url` | string | yes | URL to analyze (supports `http://localhost`) |
| `max_pages` | number | no | Enable site crawling, stop after N pages |
| `enable_js` | boolean | no | Enable JavaScript rendering |
| `vitals` | boolean | no | Measure Core Web Vitals |
| `fail_under` | number | no | Error if overall score is below threshold (0-100) |
| `beauty` | boolean | no | Return human-readable output instead of JSON |

### Example Prompt

Once configured, ask your AI assistant:

> "Analyze https://example.com for GEO optimization and suggest improvements"

The assistant will call the `analyze_url` tool and interpret the results for you.

---

## Contributing

geodaddy is open source and welcomes contributions.

### Development Setup

```bash
git clone https://github.com/borabiricik/geodaddy-cli.git
cd geodaddy-cli

# Build
cargo build

# Run tests
cargo test

# Run with local site
cargo run -- http://localhost:3000 --beauty
```

Requirements:
- Rust 1.83 or later (due to `jsonschema` MSRV)
- No other system dependencies for the base build
- Chromium is auto-downloaded only when `--vitals` or `--enable-js` is used

### Project Structure

```
src/
├── main.rs              # CLI entry point, crawl orchestration, report assembly
├── crawling.rs          # Sitemap fetching, BFS link discovery, URL normalization
├── scoring.rs           # AnalysisResult, Status, severity points, score calculation
├── beauty.rs            # Colored terminal output
└── analyzers/
    ├── mod.rs           # Analyzer module exports
    ├── technical.rs     # tech-* checks
    ├── content.rs       # cont-* checks
    ├── geo.rs           # geo-* checks
    └── performance.rs   # perf-* checks (Core Web Vitals)

mcp/                     # MCP server (TypeScript)
├── src/
│   ├── index.ts         # MCP server entry point, analyze_url tool
│   ├── binary.ts        # Binary resolution, arg building, subprocess runner
│   └── install.ts       # Postinstall binary download from GitHub releases
└── tests/
    └── tool.test.ts     # Unit tests for buildArgs and getBinaryPath
```

### Adding a New Check

1. **Pick a check ID** following the naming convention: `{category}-{name}` (e.g., `cont-readability`)

2. **Add the analyzer function** in the appropriate `src/analyzers/*.rs` file:

```rust
pub fn analyze_readability(html: &Html) -> AnalysisResult {
    AnalysisResult {
        check: "cont-readability",
        status: Status::Warn,
        message: "No readability issues detected".to_string(),
        recommendation: "".to_string(),
    }
}
```

3. **Register severity** in `src/scoring.rs` inside `severity_points()`:

```rust
"cont-readability" => 5,
```

4. **Wire it up** in `src/main.rs` where analyzers are called per page.

5. **Write tests** — at minimum, a unit test for the Pass and Fail paths:

```rust
#[test]
fn test_readability_pass() {
    let html = Html::parse_document("<html><body><p>...</p></body></html>");
    let result = analyze_readability(&html);
    assert_eq!(result.status, Status::Pass);
}
```

### Submitting a Pull Request

1. Fork the repository
2. Create a feature branch: `git checkout -b feat/my-new-check`
3. Make your changes with tests
4. Run `cargo test` — all tests must pass
5. Run `cargo clippy` — no warnings
6. Submit a PR with a description of what the check detects and why it matters for GEO

### Reporting Issues

- **Bug reports**: Include the URL (if public), the command you ran, and the output
- **False positives**: Include the expected vs. actual check result
- **Feature requests**: Explain the GEO/SEO signal and how it should be scored

Open an issue at: https://github.com/borabiricik/geodaddy-cli/issues

### Design Principles

When contributing, please keep these in mind:

- **Actionable over informational** — every Warn/Fail result must include a concrete fix recommendation
- **Local-first** — no network calls beyond the target site (no external APIs, no telemetry)
- **JSON-stable** — the `schema_version` field exists so consumers can handle format changes; avoid breaking changes to existing fields
- **Opt-in complexity** — features requiring Chromium (JS rendering, vitals) stay behind explicit flags

---

## License

MIT — see [LICENSE](LICENSE).

---

*geodaddy is not affiliated with GoDaddy, Inc.*
