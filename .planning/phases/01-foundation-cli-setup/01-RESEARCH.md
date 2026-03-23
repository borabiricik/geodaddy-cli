# Phase 1: Foundation & CLI Setup - Research

**Researched:** 2026-03-23
**Domain:** Rust CLI application scaffolding — clap, reqwest, serde_json, url, robotstxt, tokio, tracing
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Flat command structure — `geodaddy <url>` with all flags directly on the root command. No subcommands. Flags at this phase: `--fail-under <score>`, `--help`.
- **D-02:** Page-centric JSON structure from day one. Top-level fields: `schema_version` (string "1"), `url`, `crawled_at` (ISO 8601), `pages` (array). Each page object: `url` (normalized), `robots_blocked` (bool), `results` (empty array in phase 1).
- **D-03:** Soft warn mode for robots.txt. Crawl always proceeds. `robots_blocked: true` at page level when disallowed. Missing robots.txt = allow all. No exit code impact from robots.txt.
- **D-04:** Single Rust crate in `cli/` directory. Plain `cli/Cargo.toml` — no Cargo workspace at root. `web/` will be a separate non-Rust project.

### Claude's Discretion

- Internal HTTP client configuration (timeouts, user-agent string, connection pooling settings) — use sensible defaults from CLAUDE.md tech stack.
- URL normalization implementation details — standard WHATWG normalization via the `url` crate.
- Error message formatting to stderr — keep it simple, human-readable.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CRAWL-03 | CLI supports localhost URLs (http://localhost:*, http://127.0.0.1:*) | reqwest ClientBuilder does not restrict by scheme/host; url crate parses localhost correctly; robots.txt fetch must also handle localhost (404 = allow all) |
| CRAWL-05 | CLI respects robots.txt crawl directives and crawl-delay | robotstxt 0.3.0 crate: `DefaultMatcher::one_agent_allowed_by_robots()` API; soft-warn mode means result sets `robots_blocked` field only |
| CLI-01 | CLI outputs JSON format to stdout | serde + serde_json with `#[derive(Serialize)]`; `serde_json::to_string_pretty()` to stdout |
| CLI-02 | CLI returns proper exit codes (0=pass, 1=fail) with --fail-under threshold | `std::process::exit(1)` after printing JSON; clap derive handles parse errors automatically; --fail-under is f64 optional flag with no threshold = always exit 0 |
| CLI-04 | CLI has --help with clear usage documentation | clap `#[command(about, long_about)]` and `#[arg(help)]` attributes generate --help automatically |
</phase_requirements>

---

## Summary

Phase 1 creates the scaffold for geodaddy: a Rust binary in `cli/` that accepts a URL, fetches robots.txt for the target origin, normalizes the URL, and outputs a foundational JSON report. No content analysis happens in this phase — the output schema is established so later phases add to `results[]` without changing the top-level shape.

The entire stack is pre-decided in CLAUDE.md. Research confirmed all library versions from crates.io (tokio 1.50.0, clap 4.6.0, reqwest 0.13.2, serde_json 1.0.149, url 2.5.8, robotstxt 0.3.0, anyhow 1.0.102, tracing 0.1.44, tracing-subscriber 0.3.23, chrono 0.4.44). Rust 1.93.1 is installed on this machine — all MSRV requirements are satisfied (highest MSRV in this phase's dependencies is jsonschema 0.45 at 1.83, but jsonschema is not needed in phase 1; the highest here is likely reqwest at 1.70).

The critical implementation detail is the robots.txt soft-warn behavior: fetch `<origin>/robots.txt`, pass the body to `DefaultMatcher::one_agent_allowed_by_robots()`, set `robots_blocked` in the page struct if disallowed, then proceed with the crawl unconditionally. A 404 or network error on robots.txt fetch = allow all (do not fail the run).

**Primary recommendation:** Implement as a single `cli/src/main.rs` with inline structs for now — no module splitting needed at this scale. Use `#[tokio::main]` entry point, clap derive for CLI, reqwest for HTTP, robotstxt for compliance check, serde_json for output, std::process::exit for exit codes.

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tokio | 1.50.0 | Async runtime | Industry standard; required by reqwest async |
| clap | 4.6.0 | CLI argument parsing | Derive macros; structopt functionality included |
| reqwest | 0.13.2 | HTTP client | Async, connection pooling, ergonomic API |
| serde | 1.0.x | Serialization framework | De facto Rust serialization |
| serde_json | 1.0.149 | JSON output | Strongly typed JSON serialization |
| url | 2.5.8 | URL parsing/normalization | WHATWG standard, Servo project |
| robotstxt | 0.3.0 | robots.txt compliance | Google algorithm port, zero dependencies |
| anyhow | 1.0.102 | Error propagation | Standard for CLI applications |
| tracing | 0.1.44 | Structured logging | Tokio ecosystem standard |
| tracing-subscriber | 0.3.23 | Tracing output backend | Required to actually emit tracing output |
| chrono | 0.4.44 | ISO 8601 timestamps | Standard Rust date/time library |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| chrono with serde feature | 0.4.44 | Serialize DateTime as ISO 8601 | `crawled_at` field in report |

### Alternatives Considered

All alternatives are pre-decided in CLAUDE.md. No alternatives researched per locked decisions.

**Installation:**

```toml
[package]
name = "geodaddy"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "geodaddy"
path = "src/main.rs"

[dependencies]
tokio = { version = "1.50", features = ["full"] }
clap = { version = "4.6", features = ["derive"] }
reqwest = { version = "0.13", features = ["rustls-tls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
url = { version = "2.5", features = ["serde"] }
robotstxt = "0.3"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
chrono = { version = "0.4", features = ["serde"] }
```

**Version verification:** All versions above confirmed against crates.io registry on 2026-03-23 via `cargo search`.

---

## Architecture Patterns

### Recommended Project Structure

```
cli/
├── Cargo.toml
└── src/
    └── main.rs        # Single file for phase 1 — all logic here
```

No module splitting in phase 1. Phases 2-4 will extract analyzers into modules. Starting flat keeps the scaffold simple.

### Pattern 1: Clap Derive for Flat CLI

**What:** Use `#[derive(Parser)]` with a positional `url: String` field and an optional `--fail-under` flag.
**When to use:** Phase 1 and beyond — D-01 mandates no subcommands.

```rust
// Source: https://docs.rs/clap/latest/clap/_derive/index.html
use clap::Parser;

#[derive(Parser)]
#[command(name = "geodaddy")]
#[command(about = "GEO analysis tool — surface actionable AI search optimization issues")]
#[command(version)]
struct Cli {
    /// URL to analyze (supports localhost)
    url: String,

    /// Exit with code 1 if overall score is below this threshold (0-100)
    #[arg(long, value_name = "SCORE")]
    fail_under: Option<f64>,
}
```

clap automatically generates `--help` from doc comments (`///`). `--version` from `#[command(version)]` uses the Cargo.toml version.

### Pattern 2: tokio::main Entry Point

**What:** Single async entry point with error propagation to main.
**When to use:** All async Rust CLI applications.

```rust
// Source: https://docs.rs/tokio/latest/tokio/attr.main.html
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tracing init
    // cli parse
    // run
    Ok(())
}
```

Note: `anyhow::Result<()>` return from main prints the error chain to stderr and exits with code 1. For exit code control (the `--fail-under` case), call `std::process::exit(1)` explicitly after printing JSON to stdout.

### Pattern 3: reqwest Client Setup

**What:** Single shared client instance with sensible defaults.
**When to use:** All HTTP fetches in geodaddy.

```rust
// Source: https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html
use std::time::Duration;

let client = reqwest::Client::builder()
    .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
    .timeout(Duration::from_secs(30))
    .connect_timeout(Duration::from_secs(10))
    .build()?;
```

`env!("CARGO_PKG_NAME")` and `env!("CARGO_PKG_VERSION")` give a user-agent like `geodaddy/0.1.0` automatically from Cargo.toml.

### Pattern 4: robots.txt Soft-Warn Check

**What:** Fetch robots.txt for the target origin, check if URL is allowed, set flag — never block.
**When to use:** Every page fetch in geodaddy (D-03).

```rust
// Source: https://docs.rs/robotstxt/0.3.0/robotstxt/
use robotstxt::DefaultMatcher;

async fn check_robots(
    client: &reqwest::Client,
    page_url: &url::Url,
    user_agent: &str,
) -> bool {
    // Build robots.txt URL from origin
    let mut robots_url = page_url.clone();
    robots_url.set_path("/robots.txt");
    robots_url.set_query(None);
    robots_url.set_fragment(None);

    let robots_body = match client.get(robots_url.as_str()).send().await {
        Ok(resp) if resp.status().is_success() => {
            resp.text().await.unwrap_or_default()
        }
        _ => String::new(), // 404, network error, etc. = allow all
    };

    let mut matcher = DefaultMatcher::default();
    !matcher.one_agent_allowed_by_robots(&robots_body, user_agent, page_url.as_str())
    // returns true if blocked
}
```

### Pattern 5: URL Normalization

**What:** Parse input URL string with the `url` crate; use the parsed URL as the canonical form.
**When to use:** Normalize the CLI input before all operations.

```rust
// Source: https://docs.rs/url/latest/url/
use url::Url;

let normalized = Url::parse(&cli.url)
    .map_err(|e| anyhow::anyhow!("Invalid URL '{}': {}", cli.url, e))?;
// normalized.as_str() is the WHATWG-normalized form
```

WHATWG normalization handles: lowercase scheme/host, default port removal, path percent-encoding normalization. Covers the phase 1 URL normalization success criterion.

### Pattern 6: JSON Report Structs

**What:** Strongly-typed structs for the JSON output shape defined in D-02.

```rust
// Source: derived from D-02 in CONTEXT.md
use serde::Serialize;
use chrono::{DateTime, Utc};

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    url: String,
    crawled_at: DateTime<Utc>,
    pages: Vec<PageResult>,
}

#[derive(Serialize)]
struct PageResult {
    url: String,
    robots_blocked: bool,
    results: Vec<serde_json::Value>, // empty in phase 1, typed in phases 2-4
}
```

`chrono::DateTime<Utc>` with the `serde` feature serializes to ISO 8601 automatically. `schema_version` is `&'static str` with value `"1"` (D-02 specifies string type, not integer).

### Pattern 7: Exit Code Control

**What:** Print JSON to stdout, then call `std::process::exit()` based on --fail-under threshold.
**When to use:** After report generation.

```rust
// Print JSON to stdout first (always)
let json = serde_json::to_string_pretty(&report)?;
println!("{}", json);

// Then apply exit code logic
if let Some(threshold) = cli.fail_under {
    let score = compute_score(&report); // returns 0.0 in phase 1 (no analyzers)
    if score < threshold {
        std::process::exit(1);
    }
}
// Implicit exit(0)
```

In phase 1 there are no analyzers, so score is always 0.0. If a user passes `--fail-under 0.1`, they will always get exit code 1. This is expected and correct — score fields are populated starting in phase 2.

### Pattern 8: Tracing Initialization

**What:** Initialize tracing-subscriber to write diagnostics to stderr, controlled by RUST_LOG.

```rust
// Source: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/
use tracing_subscriber::{fmt, filter::EnvFilter};

fmt()
    .with_writer(std::io::stderr)
    .with_env_filter(EnvFilter::from_default_env())
    .init();
```

JSON output to stdout is clean (no tracing noise). Tracing goes to stderr. RUST_LOG controls verbosity (e.g., `RUST_LOG=debug geodaddy http://localhost:3000`).

### Anti-Patterns to Avoid

- **Mixing JSON output and tracing on stdout:** All tracing/logging MUST go to stderr. Stdout is reserved for the JSON report. Mixing breaks CI/CD pipelines that parse `geodaddy ... | jq`.
- **Returning early on robots.txt 404:** A 404 or network error on `/robots.txt` is normal (especially localhost). Treat it as an empty file = allow all. Do not fail the run.
- **Calling `std::process::exit()` before printing JSON:** Always print the report first, then exit. The report should be emitted even when --fail-under triggers exit code 1.
- **Blocking in async context:** Never use `std::thread::sleep` or synchronous I/O inside `async fn`. All I/O goes through tokio + reqwest.
- **Constructing robots.txt URL by string concatenation:** Use `url::Url` manipulation (`.set_path()`) to correctly handle ports, trailing slashes, and path components.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| URL parsing/normalization | Custom regex or string manipulation | `url` crate | WHATWG compliance, percent-encoding, port handling edge cases |
| robots.txt parsing | Custom string parser | `robotstxt` crate | Google's exact algorithm — precedence rules, wildcard handling, typo tolerance are subtle |
| CLI argument parsing | Manual `std::env::args()` iteration | `clap` derive | --help generation, error messages, type coercion, validation |
| JSON serialization | Manual string formatting | `serde_json` | Escaping, Unicode, pretty printing |
| Async HTTP | Blocking `std::net::TcpStream` | `reqwest` + tokio | Connection pooling, TLS, redirects, timeouts |
| ISO 8601 timestamps | Manual date formatting | `chrono` with serde feature | Timezone handling, RFC 3339 compliance |

**Key insight:** robots.txt parsing has significant edge cases (wildcard `*`, `$` anchors, `Allow` vs `Disallow` precedence, `crawl-delay`, typo tolerance). Google's algorithm differs from naive implementations in ways that matter for an SEO tool. Use the robotstxt crate.

---

## Common Pitfalls

### Pitfall 1: robots.txt URL Construction for Non-Root Paths

**What goes wrong:** If the user provides `http://localhost:3000/blog/post/1`, the robots.txt lives at `http://localhost:3000/robots.txt`, not `http://localhost:3000/blog/robots.txt`.
**Why it happens:** Naively appending "/robots.txt" to the input URL path.
**How to avoid:** Clone the parsed `Url`, call `.set_path("/robots.txt")`, clear query and fragment.
**Warning signs:** robots.txt returning 404 for valid sites, or fetching the wrong file.

### Pitfall 2: Writing Tracing Output to stdout

**What goes wrong:** `tracing-subscriber` defaults to stdout. The JSON report is also on stdout. Anything printed before the JSON (e.g., a tracing span) corrupts the output.
**Why it happens:** Forgetting `.with_writer(std::io::stderr)` in subscriber init.
**How to avoid:** Always initialize fmt subscriber with `.with_writer(std::io::stderr)`.
**Warning signs:** `jq` parse errors when running `geodaddy ... | jq`.

### Pitfall 3: Exit Before Printing Report

**What goes wrong:** `anyhow::Result<()>` returned from main exits with code 1 on error. But if the error occurs mid-execution, the partial or complete report is never printed.
**Why it happens:** Propagating errors with `?` before the `println!("{}", json)` call.
**How to avoid:** Separate the "build report" step from the "output and exit" step. Print report unconditionally, then apply threshold logic.
**Warning signs:** `geodaddy` exits with code 1 but no output.

### Pitfall 4: reqwest Default TLS vs Localhost

**What goes wrong:** `http://` (non-TLS) localhost URLs work fine. The issue is when `reqwest` is built with only `rustls-tls` and a test server uses self-signed certificates on `https://localhost`.
**Why it happens:** rustls rejects self-signed certs by default.
**How to avoid:** For v1, document that self-signed localhost HTTPS requires `--danger-accept-invalid-certs` if we add that flag, or rely on HTTP for local dev. Phase 1 does not add that flag — HTTP localhost is the primary use case (CRAWL-03).
**Warning signs:** TLS errors on localhost HTTPS. This is expected behavior in phase 1.

### Pitfall 5: score Field Absent in Phase 1 Report

**What goes wrong:** `--fail-under` compares against a score field that doesn't exist yet (analyzers not implemented until phase 2).
**Why it happens:** Phase 1 establishes the CLI flag but the score is 0.0.
**How to avoid:** Document this behavior clearly. `--fail-under 0` will always exit 0 (0 >= 0), `--fail-under 1` will always exit 1 (0 < 1). This is correct and intentional — the flag is wired but scoring is empty.
**Warning signs:** None — expected behavior. Add a note to help text.

### Pitfall 6: Cargo.toml at Wrong Level

**What goes wrong:** Placing `Cargo.toml` at repo root creates a workspace (D-04 prohibits this). If `[workspace]` or `[package]` is at root, the `web/` directory will conflict.
**Why it happens:** Cargo auto-discovers workspace members.
**How to avoid:** Create `cli/Cargo.toml` only. Keep repo root clean (no Cargo.toml, no Cargo.lock at root). Each project manages its own manifest.
**Warning signs:** `cargo build` from repo root succeeds but produces unexpected artifacts.

---

## Code Examples

### Complete Cargo.toml

```toml
# Source: CLAUDE.md tech stack + verified crate versions
[package]
name = "geodaddy"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "geodaddy"
path = "src/main.rs"

[dependencies]
tokio = { version = "1.50", features = ["full"] }
clap = { version = "4.6", features = ["derive"] }
reqwest = { version = "0.13", features = ["rustls-tls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
url = { version = "2.5", features = ["serde"] }
robotstxt = "0.3"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
chrono = { version = "0.4", features = ["serde"] }
```

### Minimal main.rs Skeleton

```rust
// Source: synthesized from CONTEXT.md decisions + verified crate APIs
use anyhow::Result;
use clap::Parser;
use chrono::Utc;
use serde::Serialize;
use url::Url;

#[derive(Parser)]
#[command(name = "geodaddy")]
#[command(about = "GEO analysis tool — surface actionable AI search optimization issues")]
#[command(version)]
struct Cli {
    /// URL to analyze (supports http://localhost and http://127.0.0.1)
    url: String,

    /// Exit with code 1 if overall score is below this threshold (0-100)
    #[arg(long, value_name = "SCORE")]
    fail_under: Option<f64>,
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    url: String,
    crawled_at: String,           // ISO 8601 via chrono
    pages: Vec<PageResult>,
}

#[derive(Serialize)]
struct PageResult {
    url: String,
    robots_blocked: bool,
    results: Vec<serde_json::Value>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Init tracing — stderr only
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::filter::EnvFilter::from_default_env()
        )
        .init();

    // 2. Parse CLI args
    let cli = Cli::parse();

    // 3. Normalize URL
    let normalized_url = Url::parse(&cli.url)
        .map_err(|e| anyhow::anyhow!("Invalid URL '{}': {}", cli.url, e))?;

    // 4. Build HTTP client
    let client = reqwest::Client::builder()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")
        ))
        .timeout(std::time::Duration::from_secs(30))
        .connect_timeout(std::time::Duration::from_secs(10))
        .build()?;

    // 5. Check robots.txt (soft-warn)
    let robots_blocked = check_robots(&client, &normalized_url).await;

    // 6. Build report
    let report = Report {
        schema_version: "1",
        url: cli.url.clone(),
        crawled_at: Utc::now().to_rfc3339(),
        pages: vec![PageResult {
            url: normalized_url.to_string(),
            robots_blocked,
            results: vec![],
        }],
    };

    // 7. Output JSON to stdout
    println!("{}", serde_json::to_string_pretty(&report)?);

    // 8. Apply exit code threshold
    if let Some(threshold) = cli.fail_under {
        // Phase 1: no score yet — score is 0.0
        // fail_under > 0 will always exit 1 until analyzers land in phase 2
        if 0.0_f64 < threshold {
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn check_robots(client: &reqwest::Client, page_url: &Url) -> bool {
    let mut robots_url = page_url.clone();
    robots_url.set_path("/robots.txt");
    robots_url.set_query(None);
    robots_url.set_fragment(None);

    let body = match client.get(robots_url.as_str()).send().await {
        Ok(resp) if resp.status().is_success() => {
            resp.text().await.unwrap_or_default()
        }
        _ => String::new(), // 404, connection error, etc. = allow all
    };

    let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    let mut matcher = robotstxt::DefaultMatcher::default();
    !matcher.one_agent_allowed_by_robots(&body, user_agent, page_url.as_str())
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| structopt for CLI derive | clap 4 with integrated derive | clap v3 (2021) | structopt is maintenance-only; clap 4.6 is current |
| `log` crate + env_logger | `tracing` + tracing-subscriber | ~2020, dominant by 2023 | Better async context, spans, OpenTelemetry integration |
| `failure` crate | `anyhow` / `thiserror` | 2020 | failure is unmaintained; anyhow is current standard |

**Deprecated/outdated:**
- `structopt`: Maintenance mode. Do not use — clap 4 derive is the replacement.
- `log` + `env_logger` standalone: Works but lacks span context for async code. Use tracing.
- `failure` crate: Unmaintained. Use anyhow for applications.

---

## Open Questions

1. **chrono vs time crate for timestamps**
   - What we know: `chrono 0.4.44` is the dominant date/time library; CLAUDE.md does not specify one.
   - What's unclear: `time` crate is the newer alternative with fewer footguns (chrono had a soundness issue fixed in 0.4.20+). Both are actively maintained.
   - Recommendation: Use `chrono` — it has broader ecosystem adoption and the `.to_rfc3339()` method is familiar. The soundness issue was fixed years ago.

2. **`results` field type in PageResult**
   - What we know: Empty array in phase 1. Phase 2-4 populate with analyzer results.
   - What's unclear: Should `results` be `Vec<serde_json::Value>` (flexible) or `Vec<AnalysisResult>` (typed) now?
   - Recommendation: Use `Vec<serde_json::Value>` in phase 1. Phase 2 introduces the typed `AnalysisResult` struct and swaps the type. This avoids defining a struct whose shape we haven't designed yet.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust/cargo | All compilation | Yes | 1.93.1 (stable) | — |
| curl | robots.txt / URL inspection | Yes | 8.7.1 | — |
| Internet access (crates.io) | `cargo build` dependency download | Assumed yes | — | vendor crates if offline |

Rust 1.93.1 satisfies all MSRV requirements. The highest MSRV in phase 1 dependencies is reqwest 0.13 at 1.70 (Rust 1.93.1 >> 1.70). No blockers.

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:** None for phase 1.

---

## Sources

### Primary (HIGH confidence)

- crates.io registry (`cargo search`) — versions for all phase 1 dependencies verified 2026-03-23
- https://docs.rs/robotstxt/0.3.0/robotstxt/ — `DefaultMatcher::one_agent_allowed_by_robots()` API confirmed
- https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html — ClientBuilder API confirmed
- https://docs.rs/tracing-subscriber/latest/tracing_subscriber/ — fmt subscriber + EnvFilter pattern confirmed
- https://docs.rs/serde_json/latest/serde_json/ — `to_string_pretty()` and derive pattern confirmed
- CLAUDE.md — all technology choices, versions, and conventions (highest-authority source for this project)
- CONTEXT.md — locked implementation decisions (D-01 through D-04)

### Secondary (MEDIUM confidence)

- https://docs.rs/url/latest/url/ — URL crate WHATWG compliance and `Url::parse()` confirmed
- https://docs.rs/clap/latest/clap/ — derive Parser pattern confirmed (tutorial chapter 0 returned 404, main page confirmed derive approach)
- WebSearch results for clap 4 derive — cross-verified with official docs

### Tertiary (LOW confidence)

- None — all critical claims verified against official docs or registry.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all versions verified against crates.io registry 2026-03-23
- Architecture: HIGH — all patterns verified against official docs
- Pitfalls: HIGH (robots.txt construction, stdout/stderr separation) / MEDIUM (TLS self-signed, exit code behavior) — based on Rust ecosystem experience + verified API behavior
- Code examples: HIGH — synthesized from verified API documentation

**Research date:** 2026-03-23
**Valid until:** 2026-06-23 (stable libraries — 90 day validity)
