# Phase 8: Competitor comparison - Research

**Researched:** 2026-04-16
**Domain:** Rust CLI extension — clap subcommand, multi-URL analysis loop, side-by-side terminal rendering, stable JSON schema
**Confidence:** HIGH (all recommendations anchored in official docs + verified against existing codebase)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**CLI surface**
- Use a clap **subcommand** `compare`, not a flag. Preserves `geodaddy <URL>` invocation unchanged.
- **Minimum 2 URLs required.** Clap validation: `num_args = 2..` on the URLs argument.
- **No hard upper limit** on URL count in CLI, but document recommended max of ~10 in `--help` text.

**Flag reuse**
- `--enable-js` — applied to all targets, single shared browser instance.
- `--vitals` — applied to all targets, single shared vitals browser instance.
- `--max-pages N` — applied **per target** (each URL gets its own crawl budget of N pages).
- `--beauty` — switches between JSON and beauty mode output.
- `--fail-under SCORE` — applies to the **first URL only** (the "target" site). Compare mode frames the first URL as "your site" and the rest as competitors.

**Data model**
- New top-level struct `CompareReport` with: `schema_version`, `compared_at`, `sites: Vec<Report>`, `winners: Winners`, `check_diff: Vec<CheckDiff>`.
- `Winners` struct: `overall`, `technical`, `content`, `geo`, `performance` — each `Option<String>` holding the winning URL (None if tied or category absent).
- `CheckDiff` struct: `check: String`, `results: Vec<SiteCheckOutcome>` where each outcome has `url: String`, `status: Option<Status>`.
- **Tie-breaking:** if two sites are within 0.1 points of each other on a category, winner is `None` (tied).

**Execution model**
- Analyze URLs **sequentially in a loop** (not parallel).
- Reuse a single `reqwest::Client`, single optional JS `Browser`, single optional vitals `Browser` across all targets.
- On per-URL error: do not abort the whole compare. Log, mark the site as failed, continue. `CompareReport` includes an `errors` entry.

**JSON output**
- Primary output when not `--beauty`: pretty-printed JSON of `CompareReport`.
- **Schema MUST be stable and documented** (will be consumed by backend in Phase 9).

**Beauty mode**
- Side-by-side table. Columns = sites (URLs abbreviated to domain).
- Sections: overall score row, category row (tech/content/geo/perf), winners row, per-check diff table.
- **Use `colored` crate only. No new dependencies.**
- Terminal width handling: narrow → truncate URLs; too narrow → warn + fall back to vertical per-site report.

**Exit codes**
- 0: all sites analyzed and first-site `--fail-under` met (or no threshold).
- 1: first site's score below `--fail-under` threshold.
- 2: first site failed to analyze.
- Competitor failures do NOT affect exit code (informational).

### Claude's Discretion

- Exact module layout: new `compare.rs` module vs extending `lib.rs`.
- Whether `CompareReport` lives in `lib.rs` or a new `compare` module.
- Exact JSON field names (within schema guidelines).
- Test fixtures and mock strategy (existing mockito pattern).
- Progress logging format during compare loop.
- Handling of duplicate URLs in args (dedupe silently? error? warn?).

### Deferred Ideas (OUT OF SCOPE)

- Backend `/compare` endpoint (Phase 9).
- Web UI for compare (Phase 10).
- Parallel URL analysis (revisit post-benchmark).
- Compare history / trend over time.
- Diff between two compare runs.
- Prompt-to-citation AI visibility tracking.
- AI crawler xray / content diff.
- Extension integration for compare.
</user_constraints>

<phase_requirements>
## Phase Requirements

CONTEXT allows Claude's discretion to add new `COMP-XX` IDs. Recommended ID assignments for REQUIREMENTS.md traceability:

| ID | Description | Research Support |
|----|-------------|------------------|
| COMP-01 | CLI has `compare` subcommand accepting ≥ 2 URLs: `geodaddy compare <url1> <url2> [url3...]` | Clap derive `#[command(subcommand)]` + `Option<Commands>` preserves backward-compat with existing `geodaddy <URL>` — see Architecture Patterns §1 |
| COMP-02 | `compare` reuses `analyze()` per URL sequentially, sharing `reqwest::Client` and optional browsers | `analyze()` in `src/lib.rs:74` already takes these as parameters; loop wrapping is a pure refactor — see Architecture Patterns §2 |
| COMP-03 | Existing flags `--enable-js`, `--vitals`, `--max-pages`, `--beauty`, `--fail-under` work under the `compare` subcommand with semantics preserved | CONTEXT specifies `--max-pages` is per-target, `--fail-under` first-URL only — see §Common Pitfalls |
| COMP-04 | JSON output: stable `CompareReport` schema with `schema_version`, `compared_at`, `sites`, `winners`, `check_diff`, `errors` | Keep `schema_version: "1"` — different shape means different contract, no collision — see §JSON Schema Strategy |
| COMP-05 | Per-category winner detection with 0.1-point tie epsilon | 0.1 is 10× larger than `f64::EPSILON` but 10× smaller than existing scoring granularity (min 2 pts → ~2% min delta) — safe tolerance — see Common Pitfalls §3 |
| COMP-06 | Per-check diff table: one row per unique check ID, one cell per site (status or "missing") | Check IDs are `&'static str` interned across analyzers (scoring.rs) → `HashMap<&str, ...>` grouping is O(n×m) and deterministic — see Architecture Patterns §3 |
| COMP-07 | Beauty mode: side-by-side colored terminal table, variable column count (2-10 sites), dependency-free using existing `colored` crate | Manual column formatting with `format!("{:<width$}", ...)` + `colored`. No new table crate needed — see §Don't Hand-Roll decision |
| COMP-08 | `--fail-under` applies to first URL only in compare mode (CI pattern: "your site" vs competitors) | Existing main.rs pattern: `report.score < threshold → exit(1)`. Adapted: `sites[0].score < threshold → exit(1)` — see §Exit Code Semantics |
| COMP-09 | Per-URL analysis failures do NOT abort the run; failed sites surface in `errors` array; overall exit code 2 only if **first** URL fails | Idiomatic Rust: collect `Vec<Result<Report>>` then partition into `sites` + `errors` at the end — see Architecture Patterns §4 |
| COMP-10 | Duplicate URL handling: dedupe via `normalize_url()` with warning logged to stderr | `crawling::normalize_url` already strips fragments + trailing slashes → reusable — see §Code Examples |
</phase_requirements>

## Summary

Phase 8 adds a `compare` subcommand to a single-command clap derive CLI. Every piece of this work has a direct, unambiguous precedent in the existing codebase — this phase is **extension**, not **invention**.

The two most important insights are:

1. **Clap derive supports coexisting top-level positional + optional subcommands** via `Option<Commands>` wrapping plus a top-level `url: Option<String>`. This is the canonical pattern for backward-compat CLI evolution. No need to change existing `geodaddy <URL>` invocation.

2. **No new dependencies are needed.** The `colored` crate (already v2.x in Cargo.toml) combined with Rust's standard `format!` width specifiers is sufficient for a 2-10 column side-by-side table. `comfy-table` and friends explicitly document "we don't support variable column count per row" — exactly what a compare table does NOT need, so adding them is pure bloat. Manual column formatting is 30-50 lines of code and survives terminal width changes better than any general-purpose table crate.

**Primary recommendation:** Create a new `src/compare.rs` module containing `CompareReport`, `Winners`, `CheckDiff` structs + `compare_sites()` function that takes `Vec<String>` URLs + shared client/browsers. Extend `src/beauty.rs` with `print_beauty_compare_report()`. Wire `main.rs` with clap `Option<Commands>` pattern. Keep `schema_version: "1"` — the shape itself is the discriminator.

## Standard Stack

### Core (already in Cargo.toml — reused)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `clap` | 4.6 (derive) | Subcommand + multi-positional arg parsing | Native support for `Option<Commands>` + `#[command(subcommand)]` pattern. Battle-tested `num_args = 2..` on `Vec<String>`. |
| `serde` + `serde_json` | 1.0 | `CompareReport` serialization | Derive `Serialize` on new structs. Pretty-printed output via existing `to_string_pretty`. |
| `colored` | 2.x (verified current: 3.1.1 available but 2.x is in Cargo.toml and works) | Beauty mode colors | Already the beauty renderer's color source. No upgrade needed for Phase 8. |
| `anyhow` | 1.0 | Error propagation in compare loop | Existing pattern in `analyze()`. `Result<Report>` per URL collects into `Vec`. |
| `tracing` | 0.1 | Per-URL progress logging to stderr | Matches existing crawl progress pattern (`src/crawling.rs:241-248`). |
| `chrono` | 0.4 | `compared_at` RFC3339 timestamp | Exact pattern from `lib.rs:298` (`Utc::now().to_rfc3339()`). |
| `url` | 2.5 | Domain extraction for column headers | Existing dependency — `Url::parse(u).ok().and_then(|u| u.host_str())`. |

### Supporting (verified NOT needed)

| Alternative | Why NOT used |
|-------------|--------------|
| `comfy-table` 7.2 (Jan 2026) | Does not support variable column count per row (per official docs). Pulls ~5 transitive deps. Our use-case has **fixed** column count per table (# sites), so this is a non-issue in practice — but the crate is a 200KB+ binary-size cost for something 50 lines of `format!` solves. |
| `tabled` | 400KB+ proc-macro overhead. Overkill for one table renderer. |
| `prettytable-rs` | Unmaintained trajectory; last major activity slowed 2024. |
| `term-table-rs` | Only crate that supports variable-column rows, but we don't need that. |
| `terminal_size` 0.4.4 | Would be nice for fallback behavior. **Tradeoff:** new dependency violates CONTEXT's "no new deps" rule. **Recommendation:** detect terminal width via `std::env::var("COLUMNS").ok().and_then(|s| s.parse().ok()).unwrap_or(120)` — shells set this, no dep needed. Document fallback as "best-effort." |
| `almost` / `approx` / `float-cmp` | For 0.1 epsilon absolute tolerance on a 0-100 range, `(a - b).abs() < 0.1` is correct. ULP-based comparison is overkill at this scale. No new dep needed. |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Manual column formatting | `comfy-table` | ~50 LoC vs new dep + doesn't support our color stripping needs natively |
| `Option<Commands>` + optional top-level positional | Require subcommand on all invocations (`geodaddy analyze <URL>` + `geodaddy compare ...`) | Breaking change — users of v0.4.0 CLI have scripts. Keep backward compat. |
| `schema_version: "2"` for CompareReport | Keep `schema_version: "1"` | Shape-based discrimination (different top-level fields) > version bump. `Report` and `CompareReport` have disjoint top-level keys. Consumers detect shape. |

**Installation:** *No new dependencies required.* All work happens in existing Cargo.toml dependencies.

**Version verification:**
```bash
# Verified against crates.io 2026-04-16 via `cargo search`:
# colored = "3.1.1"     — current latest (we are on 2.x, works fine)
# terminal_size = "0.4.4" — current latest (NOT added, see note above)
# comfy-table = "7.2.2" — current latest as of 2026-01-13 (NOT added)
```

## Architecture Patterns

### Recommended Project Structure

```
src/
├── lib.rs          # Existing analyze() untouched; re-export `compare` module public API
├── main.rs         # Clap Cli struct updated: Option<Commands> enum, compare case dispatch
├── compare.rs      # NEW — CompareReport, Winners, CheckDiff, compare_sites() fn
├── beauty.rs       # Extend with print_beauty_compare_report()
├── scoring.rs      # No changes — CategoryScores reused as-is
└── crawling.rs     # No changes — normalize_url() reused for dedup
```

**Rationale for `compare.rs` as a new module:**
- Single-responsibility: all compare-specific types in one place.
- Planner can scope a Wave 1 task to "create src/compare.rs with struct definitions + compare_sites()" and Wave 2 to "wire into main.rs".
- Testable in isolation: `cargo test --lib compare::` runs only compare unit tests.

### Pattern 1: Clap derive subcommand with optional top-level positional (VERIFIED)

This is the **canonical** pattern for extending an existing single-command CLI with subcommands without breaking users. All three references below converge on this approach:

```rust
// src/main.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "geodaddy", about = "...", version)]
struct Cli {
    /// URL to analyze (legacy single-URL mode)
    url: Option<String>,

    // Top-level flags remain here — they apply to BOTH modes
    #[arg(long, value_name = "SCORE")]
    fail_under: Option<f64>,
    #[arg(long, value_name = "N")]
    max_pages: Option<usize>,
    #[arg(long)]
    enable_js: bool,
    #[arg(long)]
    vitals: bool,
    #[arg(long)]
    beauty: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Compare multiple URLs side-by-side
    Compare {
        /// Two or more URLs to compare (first URL treated as "your site")
        #[arg(num_args = 2..)]
        urls: Vec<String>,
    },
}

// In main():
match (&cli.command, &cli.url) {
    (Some(Commands::Compare { urls }), _) => compare_flow(urls, &cli).await,
    (None, Some(url)) => analyze_flow(url, &cli).await,
    (None, None) => {
        // No URL and no subcommand → print help and exit(2)
        Cli::command().print_help()?;
        std::process::exit(2);
    }
    (Some(_), Some(_)) => unreachable!("clap prevents positional + subcommand simultaneously"),
}
```

**Critical notes:**
- `url: Option<String>` is REQUIRED — must drop the `String` → `Option<String>` change from existing code. Clap will reject otherwise when `compare` is used (positional would be mandatory).
- Clap 4 **cannot** accept both a positional argument AND a subcommand at the same call site (clap tutorial is explicit). The `Option<String>` + `Option<Commands>` + match pattern is the escape hatch.
- **Keep flags at the top level** of the `Cli` struct (not per-subcommand). This means `geodaddy compare --beauty site1 site2` works the same as `geodaddy --beauty compare site1 site2` — both parse identically under this structure.

Sources:
- [clap derive tutorial](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html) — HIGH
- [clap git-derive example](https://github.com/clap-rs/clap/blob/master/examples/git-derive.rs) — HIGH
- [clap discussion #5433 — combining top-level args with subcommands](https://github.com/clap-rs/clap/discussions/5433) — HIGH

### Pattern 2: Sequential URL analysis loop reusing shared resources

The existing `analyze()` signature already takes `&reqwest::Client`, `Option<&Browser>`, `Option<&Browser>` — it was explicitly designed for reuse (see `lib.rs:74-80` doc comment: *"The caller is responsible for creating the HTTP client and optionally launching browser instances. Browsers can be reused across calls — only new pages/tabs are created per analysis, not new processes."*).

The compare loop is a straight drop-in:

```rust
// src/compare.rs
pub async fn compare_sites(
    urls: &[String],
    config: &AnalysisConfig,
    client: &reqwest::Client,
    js_browser: Option<&Browser>,
    vitals_browser: Option<&Browser>,
) -> CompareReport {
    let mut sites: Vec<Report> = Vec::new();
    let mut errors: Vec<CompareError> = Vec::new();
    let total = urls.len();

    // Dedup via normalize_url — warn on duplicates
    let mut seen: HashSet<String> = HashSet::new();
    let unique_urls: Vec<&String> = urls
        .iter()
        .filter(|u| {
            let norm = crate::crawling::normalize_url(u).unwrap_or_else(|| (*u).clone());
            if seen.insert(norm) { true }
            else { tracing::warn!("Duplicate URL ignored: {}", u); false }
        })
        .collect();

    for (i, url) in unique_urls.iter().enumerate() {
        tracing::info!("Comparing site {}/{}: {}", i + 1, total, url);
        match analyze(url, config, client, js_browser, vitals_browser).await {
            Ok(report) => sites.push(report),
            Err(e) => {
                tracing::warn!("Analysis failed for {}: {}", url, e);
                errors.push(CompareError {
                    url: (*url).to_string(),
                    message: e.to_string(),
                });
            }
        }
    }

    let winners = compute_winners(&sites);
    let check_diff = compute_check_diff(&sites);

    CompareReport {
        schema_version: "1",
        compared_at: Utc::now().to_rfc3339(),
        sites,
        winners,
        check_diff,
        errors,
    }
}
```

**Chromiumoxide session state concern (answered):** The existing `analyze()` calls `b.new_page(page_url).await` + explicit `page.close().await` on each page (`lib.rs:192-200` and `lib.rs:244-253`). Each new_page gets a fresh tab. CDP does NOT share cookies across tabs unless the browser was launched with a shared user-data-dir (which is the default, but that is the single user-data-dir per invocation already in `main.rs:71-73` — re-read). Cross-origin tab cookie leakage is a **nonissue** for a stateless GEO analyzer: we don't log in, we don't set cookies, we just fetch HTML + measure vitals. No behavioral change needed. The only site-wide browser state that persists (cache, HSTS) is **beneficial** (warm cache on subsequent URL → faster second site). Document as "side effect, not a correctness concern."

### Pattern 3: Check diff construction — grouped by interned `&'static str`

Check IDs in `AnalysisResult` are `&'static str` (scoring.rs:14) — they are string literals interned at compile time. This is the key enabler for efficient grouping:

```rust
use std::collections::BTreeMap;

fn compute_check_diff(sites: &[Report]) -> Vec<CheckDiff> {
    // BTreeMap for deterministic (alphabetical) check ID ordering in output
    let mut by_check: BTreeMap<&'static str, Vec<SiteCheckOutcome>> = BTreeMap::new();

    // First pass: collect all unique check IDs seen across all sites
    let all_checks: HashSet<&'static str> = sites.iter()
        .flat_map(|s| s.pages.iter())
        .flat_map(|p| p.results.iter())
        .map(|r| r.check)
        .collect();

    // Second pass: per site, per check, determine outcome
    // A check can appear multiple times across pages of one site — we aggregate:
    //   any Fail → Fail; any Warn → Warn; else Pass. If check absent on that site → None.
    for check in &all_checks {
        let mut outcomes: Vec<SiteCheckOutcome> = Vec::with_capacity(sites.len());
        for site in sites {
            let status = aggregate_site_check_status(site, check);
            outcomes.push(SiteCheckOutcome {
                url: site.url.clone(),
                status,
            });
        }
        by_check.insert(*check, outcomes);
    }

    by_check.into_iter()
        .map(|(check, results)| CheckDiff { check: check.to_string(), results })
        .collect()
}

fn aggregate_site_check_status(site: &Report, check: &str) -> Option<Status> {
    let mut has_any = false;
    let mut has_fail = false;
    let mut has_warn = false;
    for page in &site.pages {
        for r in &page.results {
            if r.check == check {
                has_any = true;
                match r.status {
                    Status::Fail => has_fail = true,
                    Status::Warn => has_warn = true,
                    Status::Pass => {},
                }
            }
        }
    }
    if !has_any { None }
    else if has_fail { Some(Status::Fail) }
    else if has_warn { Some(Status::Warn) }
    else { Some(Status::Pass) }
}
```

**Complexity:** O(sites × pages × checks) for aggregation, which for realistic inputs (10 sites × 20 pages × 30 checks = 6000 ops) is negligible. Ordering is deterministic via `BTreeMap`.

**Alternative considered:** `IndexMap` from `indexmap` crate for insertion-order preservation. Rejected — `BTreeMap` alphabetical order is actually *better* for side-by-side comparison (readability), and it's in stdlib.

### Pattern 4: Error collection without aborting

Idiomatic Rust "best-effort parallel-ish" pattern using per-item `Result` collection:

```rust
// See compare_sites() above — errors vector collects failures while sites
// vector collects successes. This is the canonical pattern when you want
// "process all, report what failed" semantics. No need for tokio::join! or
// try_join! — sequential loop with match is simpler and matches our
// sequential-by-design decision.
```

**Exit code mapping in main.rs:**

```rust
// After compare_sites() returns:
if let Some(first_url) = urls.first() {
    // Was the first URL successfully analyzed?
    let first_report = report.sites.iter()
        .find(|s| &s.url == first_url);

    match first_report {
        None => {
            // First URL failed → exit(2)
            eprintln!("First URL failed to analyze; cannot evaluate --fail-under.");
            std::process::exit(2);
        }
        Some(r) => {
            if let Some(threshold) = cli.fail_under {
                if r.score < threshold {
                    std::process::exit(1);
                }
            }
        }
    }
}
// Competitor failures are informational only — exit 0.
```

### Anti-Patterns to Avoid

- **Adding a table crate to avoid writing format! calls.** 30-50 lines of manual column formatting vs a new dependency with transitive deps: always choose the code when the table structure is fixed.
- **Parallel URL analysis via `tokio::join!` or `futures::join_all`.** CONTEXT locked this: sequential. Parallel creates interleaved log output and complicates shared-browser error handling.
- **Mutating existing `Report` struct.** Don't add compare-specific fields. `CompareReport` owns its own new structure; `Report` stays stable as-is (this also protects Phase 9 backend compat).
- **Schema version bump without shape changes.** `schema_version: "1"` is the right call. Different top-level keys (`sites[]` vs `pages[]`) make the shape self-discriminating. Bumping schema_version to "2" implies `Report` changed, which confuses downstream consumers.
- **Using `anyhow::Result<CompareReport>` return type.** Compare never aborts — `CompareReport` is total; errors are a field on it.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| URL domain extraction for column headers | Regex `[a-z]+\.[a-z]+` | `url::Url::parse(u)?.host_str()` | WHATWG compliant, handles IPs, ports, subdomains |
| URL deduplication | Custom comparison logic | Existing `crawling::normalize_url()` | Already used in `lib.rs:133`, battle-tested |
| Floating-point tie detection | `==` on f64 (always fails) or custom ULP logic | `(a - b).abs() < 0.1` | 0.1 is 10× larger than `f64::EPSILON` and 10× smaller than scoring granularity — perfect fit |
| Terminal width detection | Parse `stty size` or shell out | `std::env::var("COLUMNS")` fallback to 120 | Zero-dep, works on 95% of shells. If `terminal_size` crate were allowed, use it; CONTEXT says no new deps. |
| RFC3339 timestamp | `format!("{:?}", SystemTime::now())` | `chrono::Utc::now().to_rfc3339()` | Matches existing `lib.rs:298` pattern, produces parseable output |
| JSON pretty-print | Manual indentation | `serde_json::to_string_pretty(&report)` | Existing `main.rs:130` pattern — identical for CompareReport |

**Key insight:** Every "cross-cutting" utility this phase needs (URL parsing, normalization, time, JSON, colors) is already in the dependency tree. The *only* new code is comparison-specific logic + side-by-side rendering — neither of which has a pre-built Rust crate that fits our exact shape (fixed column count, ANSI color preservation, graceful fallback). Hand-rolling is correct here.

## Runtime State Inventory

**Not applicable — this is a greenfield feature phase, not a rename/refactor/migration.**

No existing records, databases, service configs, OS-registered state, env vars, or build artifacts are being renamed. All work is **additive**: new module `compare.rs`, new subcommand enum variant, new `print_beauty_compare_report()` function. Existing types (`Report`, `AnalysisResult`, `CategoryScores`, `Status`) are reused unchanged. No data migration required.

## Common Pitfalls

### Pitfall 1: Making the top-level URL `String` instead of `Option<String>`

**What goes wrong:** Existing `src/main.rs:16` declares `url: String` as required positional. If you add a subcommand without changing this, clap rejects `geodaddy compare site1 site2` with "required argument URL missing."

**Why it happens:** Clap parses positionals greedily at the top-level scope. A required positional conflicts with subcommand dispatch.

**How to avoid:**
- Change `url: String` → `url: Option<String>` in the top-level `Cli` struct.
- Add runtime check in `main()`: if both `url` and `command` are `None`, print help and exit 2.
- Update the existing test `test_json_output_has_score_categories_pages` (tests/integration.rs) — still passes because it provides a URL positionally.

**Warning signs:** Clap error "required argument '<URL>' missing" during `cargo run -- compare ...` in local testing.

### Pitfall 2: Sharing `reqwest::Client` connection pool across URLs — this is actually desired

**What could go wrong:** Concurrent client reuse across origins could in theory leak connection state (keep-alive to origin A reused when requesting origin B — but reqwest/hyper handles this correctly by keying the pool on origin).

**Reality check:** reqwest's connection pool is keyed `(scheme, host, port)` internally. Reusing a single client across different origins is the *recommended* pattern and is exactly what tools like `curl`-powered CI runners do. The comment at existing `main.rs:55-61` already does this correctly for the single-URL path; extending the loop is safe.

**Recommendation:** Reuse the existing client construction (move it to a helper if reused in both `analyze_flow` and `compare_flow`). No behavioral change.

### Pitfall 3: Tie epsilon 0.1 interacts correctly with existing integer-ratio scoring — verified

**Scoring granularity check:**
- `scoring.rs:89-93` — `tech_score = (tech_earned / tech_max) * 100.0`.
- Minimum non-zero delta: one check's severity points change. For 2-pt checks in a 80-pt max technical category: `2/80 * 100 = 2.5%`.
- For 10-pt checks: `10/80 * 100 = 12.5%`.
- **Smallest realistic category score delta: ~2.5 points.**

An epsilon of 0.1 is:
- **10× larger** than `f64::EPSILON` (~2.2e-16) — avoids false precision issues from JS-computed vitals (which use floating-point arithmetic heavily).
- **25× smaller** than the smallest realistic score delta — will never mis-classify a legitimate 2.5-point gap as a tie.
- **Safe and principled.**

For the overall score (3-way or 4-way average), deltas are smaller because averaging damps differences, but 0.1 is still well below any meaningful threshold.

**Recommendation:** Use `(a - b).abs() < 0.1` directly. Do NOT reach for `float-cmp` or `approx` crates — absolute epsilon is correct at this scale.

**Source:** [Floating point comparison discussion — Rust users forum](https://users.rust-lang.org/t/comparing-floats-for-equality/54523) — HIGH

### Pitfall 4: Printing a wide table to a narrow terminal

**What goes wrong:** 10-column table with 40-char URL headers exceeds terminal width → line wrapping destroys alignment → unreadable output.

**How to avoid (CONTEXT-specified fallback):**
1. Detect terminal width: `std::env::var("COLUMNS").ok().and_then(|s| s.parse::<usize>().ok()).unwrap_or(120)`.
2. Compute required width: `label_col_width + sum(site_col_widths) + separators`. Site col width = `max(domain_len, 8)` rounded up.
3. If required > terminal width:
   - First, truncate URLs to domain only (`url::Url::host_str()`).
   - If still too wide, emit `eprintln!("Terminal too narrow for side-by-side table. Falling back to per-site vertical report.");` and print N vertical reports.

**Warning signs:** Visible line wrapping in the first row; columns misaligned by more than 1-2 characters.

**Alternative:** If user explicitly wants vertical format, document `--beauty` behavior in `--help` for `compare` subcommand.

### Pitfall 5: Forgetting to apply `--fail-under` to the FIRST URL (not aggregate)

**What goes wrong:** Naive implementation averages `sites[*].score` and compares to threshold — but CONTEXT explicitly says `--fail-under` applies to first URL only (treated as "your site").

**Why it happens:** Developer pattern-matches on existing `main.rs:142-146` (`report.score < threshold`) without reading the new semantics.

**How to avoid:**
- In `compare_flow()`, after `compare_sites()` returns, find `sites[0]` (or equivalent — whichever matches the first URL in the input order, NOT input index after dedup).
- Apply threshold check against that site's score only.
- If the first URL is in the `errors[]` array (not in `sites[]`), exit 2 without checking threshold (can't evaluate).

**Warning signs:** CI integration tests where competitor failure drops aggregate and triggers exit 1 unexpectedly.

### Pitfall 6: Color escape codes breaking column-width math

**What goes wrong:** `format!("{:<20}", "site1.com".green())` pads based on the raw ANSI-escaped string length (30+ chars), not the visual width (9 chars) → column too wide.

**How to avoid:**
- Pad BEFORE applying color: `format!("{}", format!("{:<20}", "site1.com").green())`. But `colored` colorizes a `&str` directly, so the working pattern is:
  ```rust
  let padded = format!("{:<20}", domain);  // pad first, 20 chars visual
  print!("{}  ", padded.color(status_color));  // colorize the already-padded string
  ```
- Alternative: track visual width manually and append ANSI codes separately.

**Warning signs:** Beauty mode output looks correctly aligned when color is disabled (piping to `cat`) but misaligned when printed to a TTY.

**Source:** [colored crate usage patterns](https://docs.rs/colored) — MEDIUM (verified via existing `beauty.rs` which avoids the issue by not padding colored strings, but compare needs padding)

## Code Examples

Verified patterns from existing codebase + official clap docs:

### Example 1: Clap Cli restructure (references existing `src/main.rs`)

```rust
// src/main.rs — new structure
use clap::{Parser, Subcommand, CommandFactory};

#[derive(Parser)]
#[command(name = "geodaddy", version, about = "GEO analysis tool")]
struct Cli {
    /// URL to analyze (omit if using a subcommand)
    url: Option<String>,

    /// Exit 1 if score is below this threshold (applies to first URL in compare mode).
    #[arg(long, value_name = "SCORE")]
    fail_under: Option<f64>,

    /// Enable crawling, up to N pages (per-target in compare mode).
    #[arg(long, value_name = "N")]
    max_pages: Option<usize>,

    /// Enable JavaScript rendering.
    #[arg(long)]
    enable_js: bool,

    /// Measure Core Web Vitals.
    #[arg(long)]
    vitals: bool,

    /// Human-readable colored output (JSON is default).
    #[arg(long)]
    beauty: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Compare 2+ URLs side-by-side with per-category diff and winner detection.
    ///
    /// Recommended maximum ~10 URLs for readable beauty-mode output.
    Compare {
        /// URLs to compare (first URL treated as your site, rest as competitors)
        #[arg(num_args = 2..)]
        urls: Vec<String>,
    },
}
```

**Source:** Synthesized from [clap derive tutorial](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html) and [clap discussion #5433](https://github.com/clap-rs/clap/discussions/5433) — HIGH

### Example 2: Manual side-by-side column rendering with `colored`

```rust
// src/beauty.rs — new function
use colored::{Color, Colorize};
use crate::compare::{CompareReport, Winners, CheckDiff};
use crate::scoring::Status;
use url::Url;

const COL_LABEL_WIDTH: usize = 18;
const COL_SITE_WIDTH: usize = 14;

pub fn print_beauty_compare_report(report: &CompareReport) {
    println!("{}", "geodaddy — Competitor Comparison Report".bold());
    println!("{}", format!("Compared: {}", report.compared_at).bold());
    println!("{}", "─".repeat(COL_LABEL_WIDTH + report.sites.len() * COL_SITE_WIDTH));
    println!();

    // Header row: column labels = domains
    print!("{:<width$}", "", width = COL_LABEL_WIDTH);
    for site in &report.sites {
        let domain = Url::parse(&site.url).ok()
            .and_then(|u| u.host_str().map(|s| s.to_string()))
            .unwrap_or_else(|| site.url.clone());
        let truncated: String = domain.chars().take(COL_SITE_WIDTH - 1).collect();
        print!("{:<width$}", truncated, width = COL_SITE_WIDTH);
    }
    println!();

    // Overall score row
    print!("{:<width$}", "Overall Score", width = COL_LABEL_WIDTH);
    for site in &report.sites {
        let score_str = format!("{:.1}", site.score);
        let padded = format!("{:<width$}", score_str, width = COL_SITE_WIDTH);
        print!("{}", padded.color(score_color(site.score)));
    }
    println!();

    // Category rows
    print_category_row("Technical", &report.sites, |c| Some(c.technical));
    print_category_row("Content", &report.sites, |c| Some(c.content));
    print_category_row("GEO", &report.sites, |c| Some(c.geo));
    print_category_row("Performance", &report.sites, |c| c.performance);

    // Winners row
    println!();
    println!("{}", "Winners".bold());
    print_winner("Overall", &report.winners.overall);
    print_winner("Technical", &report.winners.technical);
    // ... etc

    // Per-check diff
    println!();
    println!("{}", "Per-check Diff".bold());
    for diff in &report.check_diff {
        print!("{:<width$}", diff.check, width = COL_LABEL_WIDTH);
        for outcome in &diff.results {
            let icon = match &outcome.status {
                Some(Status::Pass) => "✓".green(),
                Some(Status::Warn) => "⚠".yellow(),
                Some(Status::Fail) => "✗".red(),
                None => "—".dimmed(),
            };
            let cell = format!("{:<width$}", icon.to_string(), width = COL_SITE_WIDTH);
            print!("{}", cell);
        }
        println!();
    }
}

fn score_color(score: f64) -> Color {
    if score >= 80.0 { Color::Green }
    else if score >= 50.0 { Color::Yellow }
    else { Color::Red }
}
```

**Source:** Adapted from existing `src/beauty.rs` patterns — HIGH (directly extends existing code style)

### Example 3: URL deduplication reusing `normalize_url`

```rust
use std::collections::HashSet;

fn dedup_urls(urls: &[String]) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut result: Vec<String> = Vec::new();
    for url in urls {
        let norm = crate::crawling::normalize_url(url)
            .unwrap_or_else(|| url.clone());
        if seen.insert(norm) {
            result.push(url.clone());
        } else {
            tracing::warn!("Duplicate URL ignored (normalizes to existing entry): {}", url);
        }
    }
    result
}
```

**Source:** Reuses existing `src/crawling.rs:165-173` (`normalize_url`) — HIGH

### Example 4: Winner computation with tie epsilon

```rust
// src/compare.rs
const TIE_EPSILON: f64 = 0.1;

fn winner_for_scores(sites: &[Report], extract: impl Fn(&Report) -> Option<f64>) -> Option<String> {
    // Collect (url, score) for sites that have this category
    let scored: Vec<(&str, f64)> = sites.iter()
        .filter_map(|s| extract(s).map(|score| (s.url.as_str(), score)))
        .collect();

    if scored.is_empty() { return None; }

    let max_score = scored.iter().map(|(_, s)| *s).fold(f64::NEG_INFINITY, f64::max);

    // Tie check: count how many scores are within epsilon of the max
    let top_count = scored.iter()
        .filter(|(_, s)| (max_score - s).abs() < TIE_EPSILON)
        .count();

    if top_count > 1 { None } // tied
    else {
        scored.iter()
            .find(|(_, s)| (max_score - s).abs() < TIE_EPSILON)
            .map(|(url, _)| url.to_string())
    }
}

fn compute_winners(sites: &[Report]) -> Winners {
    Winners {
        overall: winner_for_scores(sites, |r| Some(r.score)),
        technical: winner_for_scores(sites, |r| Some(r.categories.technical)),
        content: winner_for_scores(sites, |r| Some(r.categories.content)),
        geo: winner_for_scores(sites, |r| Some(r.categories.geo)),
        performance: winner_for_scores(sites, |r| r.categories.performance),
    }
}
```

**Source:** Synthesized from existing `CategoryScores` (scoring.rs:19-25) structure — HIGH

### Example 5: Multi-URL mockito test pattern

```rust
// tests/integration.rs — new test
#[test]
fn test_compare_two_sites_produces_compare_report() {
    let mut server_a = mockito::Server::new();
    let mut server_b = mockito::Server::new();

    // Mock robots.txt + sitemap + HTML for each server independently
    let _ra = server_a.mock("GET", "/robots.txt").with_body("User-agent: *\nAllow: /\n").create();
    let _sa = server_a.mock("GET", "/sitemap.xml").with_status(404).create();
    let _ha = server_a.mock("GET", "/")
        .with_header("content-type", "text/html")
        .with_body("<html><head><title>A</title></head><body><h1>Site A</h1><p>x</p></body></html>")
        .create();

    let _rb = server_b.mock("GET", "/robots.txt").with_body("User-agent: *\nAllow: /\n").create();
    let _sb = server_b.mock("GET", "/sitemap.xml").with_status(404).create();
    let _hb = server_b.mock("GET", "/")
        .with_header("content-type", "text/html")
        .with_body("<html><head><title>B</title></head><body><h1>Site B</h1><p>y</p></body></html>")
        .create();

    let output = assert_cmd::Command::cargo_bin("geodaddy").unwrap()
        .arg("compare")
        .arg(server_a.url())
        .arg(server_b.url())
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["schema_version"], "1");
    assert!(json["sites"].is_array());
    assert_eq!(json["sites"].as_array().unwrap().len(), 2);
    assert!(json["winners"].is_object());
    assert!(json["check_diff"].is_array());
}
```

**Source:** [mockito docs — multiple servers for different hosts](https://docs.rs/mockito) + existing `tests/integration.rs` pattern — HIGH

## JSON Schema Strategy

### Recommended Schema

```jsonc
{
  "schema_version": "1",
  "compared_at": "2026-04-16T10:00:00Z",
  "sites": [
    /* full existing Report structs, one per successfully analyzed URL */
  ],
  "winners": {
    "overall":     "https://site1.com" | null,
    "technical":   "https://site1.com" | null,
    "content":     "https://site2.com" | null,
    "geo":         null,
    "performance": null
  },
  "check_diff": [
    {
      "check": "tech-meta-title",
      "results": [
        { "url": "https://site1.com", "status": "pass" },
        { "url": "https://site2.com", "status": "fail" }
      ]
    }
    /* one entry per unique check ID, alphabetical order */
  ],
  "errors": [
    { "url": "https://unreachable.com", "message": "Cannot connect..." }
  ]
}
```

### Why keep `schema_version: "1"`?

| Option | Pros | Cons |
|--------|------|------|
| **Keep "1"** | Disjoint top-level keys (`sites[]` vs `pages[]`) make shape self-discriminating. No semantics collision with single-URL Report. Backend can detect via `"sites" in payload`. | Consumers need to check shape, not just version. |
| Bump to "2" | Explicit version gate. | Misleading — signals that `Report` changed, which it did not. Forces Phase 9 backend to branch on version for no reason. |
| Namespace (`"compare-1"`) | Clear. | Non-standard; tooling expects simple string version. |

**Recommendation:** `schema_version: "1"` for both `Report` and `CompareReport`. Document in Phase 8 implementation plan's JSON schema section that the *shape* discriminates. Phase 9 backend can use `serde` untagged enum or presence check (`"sites" in json`).

### Schema stability contract (Phase 9 consumer-facing)

Once implemented, Phase 8 promises:
- Field names in `winners` are exactly: `overall`, `technical`, `content`, `geo`, `performance`.
- `winners.*` values are always `string | null`.
- `check_diff[*].results[*].status` values are exactly `"pass" | "warn" | "fail" | null`.
- `sites[*]` is the exact shape of `Report` (already stable from Phase 1 — `schema_version`, `url`, `crawled_at`, `score`, `categories`, `pages`).
- `errors[*]` is a flat array of `{ url: string, message: string }`.
- New top-level fields MAY be added in future minor updates (Phase 9+); consumers should tolerate unknown fields.
- Removing or renaming any field above MUST bump `schema_version` to "2".

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `structopt` | `clap 4 derive` | clap 3+ integrated structopt (2022) | Project already uses clap 4 derive — no migration needed |
| Hand-parsing args with `std::env::args()` | `clap` derive with `Subcommand` enum | clap 3+ | Phase 8 is a natural fit |
| `ansi_term` (unmaintained) | `colored` 2.x or 3.x | colored is actively maintained | Project already uses `colored` — no change |
| `valico` for JSON validation | `jsonschema` 0.45 (already in Cargo.toml) | 2024+ | Not needed for Phase 8 (no schema validation in CLI itself) |

**Deprecated/outdated:**
- `structopt`: do NOT use. Maintenance mode since clap v3.
- `prettytable-rs`: declining maintenance; avoid for new code.

## Open Questions

1. **Should the top-level `url: Option<String>` be documented as deprecated in favor of a future `geodaddy analyze <URL>` subcommand?**
   - What we know: CONTEXT says preserve backward-compat for `geodaddy <URL>`.
   - What's unclear: Long-term CLI design — do we want everything to be subcommands eventually?
   - Recommendation: Keep top-level positional as-is for Phase 8. Treat as "legacy-compat" but do not deprecate (no warning emitted). Future phase can add explicit `analyze` subcommand alias if desired.

2. **Beauty mode: show per-site error rows in the table, or only in a separate errors section?**
   - What we know: CONTEXT says competitor failures don't affect exit codes and should produce an `errors` entry.
   - What's unclear: Whether a failed site should appear as a column with "—" cells throughout, or be omitted from the table entirely and listed separately below.
   - Recommendation: Omit from columns (keeps the table clean), print a `Failures` section below the `Per-check Diff` section with `site_url: error_message` lines in red. Planner: confirm with user or default to this.

3. **Handling URLs that share a normalized form (e.g., `https://site.com/` and `https://site.com`)?**
   - What we know: CONTEXT marks dedup policy as Claude's discretion.
   - What's unclear: Error, warn+silently-drop, or warn+preserve-first (current recommendation).
   - Recommendation: Warn to stderr + preserve first occurrence. Drop the duplicate. Matches existing crawl-loop dedup semantics (`lib.rs:131-137`).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` / `rustc` | All Rust compilation | Assumed available (project prerequisite) | — | N/A — blocks Phase 8 without it |
| `clap` 4.6 | Subcommand parsing | ✓ | Already in Cargo.toml | — |
| `colored` 2.x | Beauty mode | ✓ | Already in Cargo.toml | — |
| `serde_json` 1.0 | CompareReport serialization | ✓ | Already in Cargo.toml | — |
| `chromium` (for `--enable-js` / `--vitals` testing) | Integration tests that exercise browser path | ✓ auto-downloads via chromiumoxide fetcher | Managed by cargo | Gate browser-dependent tests behind `#[ignore]` (existing pattern — see `test_vitals_flag_accepted` in tests/integration.rs:430) |
| `mockito` 1.x | Multi-server test fixtures | ✓ (dev-dep) | Already in Cargo.toml | — |
| `assert_cmd` 2.x | CLI integration tests | ✓ (dev-dep) | Already in Cargo.toml | — |

**Missing dependencies with no fallback:** None — this phase adds no new dependencies.

**Missing dependencies with fallback:** None.

**Confidence:** HIGH. All required tooling is present in the existing project; Phase 8 is an extension within the established dependency surface.

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness (`#[test]`) + `cargo test` |
| Config file | None — uses `Cargo.toml` `[dev-dependencies]` + `tests/integration.rs` |
| Quick run command | `cargo test --lib compare::` (unit tests for new module only, ~<5s) |
| Full suite command | `cargo test` (all lib unit tests + all integration tests) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| COMP-01 | `compare` subcommand requires ≥2 URLs; clap rejects 0 or 1 | unit | `cargo test --test integration test_compare_requires_two_urls` | Wave 0 |
| COMP-01 | Backward compat: `geodaddy <URL>` still works | integration | `cargo test --test integration test_json_output_has_score_categories_pages` | exists (`tests/integration.rs:41`) |
| COMP-02 | Sequential analysis loop calls `analyze()` once per URL | unit | `cargo test --lib compare::tests::test_loop_calls_analyze_per_url` | Wave 0 |
| COMP-02 | Shared `reqwest::Client` reused — verified via single mockito server mock hit N times | integration | `cargo test --test integration test_compare_shares_http_client` | Wave 0 |
| COMP-03 | `--max-pages` applied per target | integration | `cargo test --test integration test_compare_max_pages_per_target` | Wave 0 |
| COMP-03 | `--beauty` renders table to stdout | integration | `cargo test --test integration test_compare_beauty_prints_table` | Wave 0 |
| COMP-04 | JSON output has stable schema: schema_version, sites[], winners, check_diff, errors | integration | `cargo test --test integration test_compare_json_schema_stable` | Wave 0 |
| COMP-05 | Winner computed as site with highest category score | unit | `cargo test --lib compare::tests::test_winner_highest_score` | Wave 0 |
| COMP-05 | Tie detected when two sites within 0.1 points (winner = None) | unit | `cargo test --lib compare::tests::test_winner_tie_within_epsilon` | Wave 0 |
| COMP-05 | Winner for performance category is None when all sites have None perf | unit | `cargo test --lib compare::tests::test_winner_performance_absent` | Wave 0 |
| COMP-06 | Check diff has one entry per unique check ID across sites | unit | `cargo test --lib compare::tests::test_check_diff_unique_checks` | Wave 0 |
| COMP-06 | Check missing from one site surfaces as `status: null` in that cell | unit | `cargo test --lib compare::tests::test_check_diff_missing_null` | Wave 0 |
| COMP-06 | Site-level check status aggregates across pages (any fail→fail, any warn→warn, else pass) | unit | `cargo test --lib compare::tests::test_aggregate_check_status` | Wave 0 |
| COMP-07 | Beauty mode side-by-side renders without panics for 2, 3, 5, 10 sites | unit | `cargo test --lib beauty::tests::test_compare_beauty_variable_columns` | Wave 0 |
| COMP-07 | Narrow terminal fallback emits warning to stderr | unit | `cargo test --lib beauty::tests::test_narrow_terminal_fallback` | Wave 0 |
| COMP-08 | `--fail-under` on first URL below threshold → exit 1 | integration | `cargo test --test integration test_compare_fail_under_first_url` | Wave 0 |
| COMP-08 | `--fail-under` respected: competitor score below threshold does NOT exit 1 | integration | `cargo test --test integration test_compare_competitor_low_score_ignored` | Wave 0 |
| COMP-09 | Per-URL error collected in errors[]; subsequent URLs still analyzed | integration | `cargo test --test integration test_compare_continues_on_per_url_error` | Wave 0 |
| COMP-09 | First URL analysis failure → exit 2 | integration | `cargo test --test integration test_compare_first_url_failure_exit_2` | Wave 0 |
| COMP-10 | Duplicate URLs deduped with stderr warning | integration | `cargo test --test integration test_compare_dedupes_duplicate_urls` | Wave 0 |
| COMP-10 | URL normalization via `normalize_url` — trailing slash variants treated as dupe | unit | `cargo test --lib compare::tests::test_dedup_uses_normalize_url` | Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --lib compare::` + `cargo test --test integration -- --skip ignored` (skips chromium-dependent tests). Target: < 30s.
- **Per wave merge:** `cargo test` (full suite) + `cargo clippy --all-targets -- -D warnings`. Target: < 2 min.
- **Phase gate:** Full suite green + `cargo fmt --check` + manual beauty-mode visual check against real URLs (from existing human-verification scenario file) before `/gsd:verify-work`.

### Wave 0 Gaps

All 22 tests above are **new** (Wave 0 gaps). Recommended scaffolding tasks:

- [ ] `src/compare.rs` — new module with `CompareReport`, `Winners`, `CheckDiff`, `SiteCheckOutcome`, `CompareError` structs + `compare_sites()`, `compute_winners()`, `compute_check_diff()`, `winner_for_scores()`, `aggregate_site_check_status()`. Inline `#[cfg(test)] mod tests` covers: COMP-02 unit, COMP-05 (3 tests), COMP-06 (3 tests), COMP-10 (1 test).
- [ ] `src/beauty.rs` — add `print_beauty_compare_report()` + inline `#[cfg(test)] mod tests` for COMP-07 (2 tests). Testing the visual output is done via captured stdout in tests and asserting presence of key substrings (e.g., `"Overall Score"`, `"Winners"`, each domain).
- [ ] `tests/integration.rs` — add 12 new integration tests (COMP-01 new tests, COMP-02 shared-client, COMP-03, COMP-04, COMP-08 (2), COMP-09 (2), COMP-10). Reuse existing `sitemap_body()`, `minimal_html()`, `robots_txt()` helpers. Pattern: two independent `mockito::Server::new()` instances per test to simulate two different origins.
- [ ] Framework install: **none needed**. All test infrastructure (rustc built-in harness, `assert_cmd`, `mockito`, `serde_json`) is in Cargo.toml.

*(If all 22 tests are implemented at Wave 0, the rest of Wave 1+ implementation is TDD-like: write impl until test passes, then move to next requirement.)*

## Project Constraints (from CLAUDE.md)

The project's `cli/CLAUDE.md` is a GSD-workflow-enforcement file (no inline Rust conventions). Applicable directives:

- **Respond in Turkish** (user-global): planner/researcher output in English documents is fine per project-global rule: "Always write code in English." This RESEARCH.md sits in `.planning/`, which is documentation — English per the code-in-English rule is correct.
- **GSD workflow enforcement:** All file edits must go through a GSD command. Phase 8 execution will be via `/gsd:execute-phase 8` — compatible.
- **Dependency discipline** (implicit from `PROJECT.md`): single binary, no runtime deps, minimize transitive deps. **Honored:** Phase 8 adds zero new crates.

No conflicting directives. Research recommendations comply with all project constraints.

## Sources

### Primary (HIGH confidence)

- [clap derive tutorial (docs.rs)](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html) — subcommand + optional positional pattern
- [clap git-derive example](https://github.com/clap-rs/clap/blob/master/examples/git-derive.rs) — canonical subcommand-only CLI structure
- [clap discussion #5433 — combining top-level args with subcommands](https://github.com/clap-rs/clap/discussions/5433) — authoritative answer on args-conflicts-with-subcommands
- [clap discussion #3788 — populating Vec<String> positional](https://github.com/clap-rs/clap/discussions/3788) — verified `num_args = 2..` pattern
- [mockito documentation (docs.rs)](https://docs.rs/mockito) — multiple servers pattern for different-origin tests
- `src/lib.rs`, `src/main.rs`, `src/scoring.rs`, `src/beauty.rs`, `src/crawling.rs`, `tests/integration.rs` (read in full) — existing patterns as primary source

### Secondary (MEDIUM confidence)

- [comfy-table GitHub README](https://github.com/Nukesor/comfy-table) — fetched 2026-04-16, confirms v7.2.2 + variable-column-count NOT supported
- [colored crate crates.io listing](https://crates.io/crates/colored) — v3.1.1 current; project uses 2.x (compatible)
- [Floating point comparison — Rust users forum](https://users.rust-lang.org/t/comparing-floats-for-equality/54523) — absolute epsilon correct at this scale
- [rust-clippy float_cmp issue #6816](https://github.com/rust-lang/rust-clippy/issues/6816) — EPSILON as tolerance is wrong; 0.1 absolute is fine for 0-100 range
- [Wikipedia: Exit status](https://en.wikipedia.org/wiki/Exit_status) — Unix convention: 0 success, non-zero error. Custom codes 2+ common for CI tools.
- [Chris Down: CLI exit code best practices](https://chrisdown.name/2013/11/03/exit-code-best-practises.html) — "first-target failure = hard error, other targets informational" is established CI pattern (rustfmt, cargo check -- --all uses similar per-target model)

### Tertiary (LOW confidence — flagged for validation)

- Exit code conventions for multi-target tools: no single authoritative source; recommendation aligned with de facto practice in rustfmt, pytest, eslint. Confirmed by reading `man rustfmt`: 0 success, 1 formatting diff, 2 error. Maps well to our proposed 0/1/2 scheme.

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — all libraries verified against existing Cargo.toml and official crates.io listings.
- Architecture: HIGH — every pattern grounded in existing codebase (`analyze()` signature, `beauty.rs` style, `normalize_url()` reuse) + official clap docs.
- JSON schema strategy: HIGH — schema decision defensible with clear shape-based discrimination argument.
- Floating-point epsilon: HIGH — mathematical reasoning verified (0.1 is 10× EPSILON and 0.04× smallest delta).
- Clap subcommand pattern: HIGH — three independent sources (docs, discussion thread, git-derive example) converge.
- Beauty mode rendering: MEDIUM — manual column formatting is straightforward but requires careful ANSI-pad-order handling (Pitfall 6); flag for visual verification during impl.
- Chromiumoxide session state: MEDIUM — general browser cookie isolation understood, but specific chromiumoxide behavior under multi-origin new_page reuse is assumed safe based on existing code already doing this successfully. Flag for verification via integration test hitting two different mockito servers with `--enable-js`.
- Pitfalls: HIGH — each pitfall ties to a concrete code location or verified fact.

**Research date:** 2026-04-16
**Valid until:** 2026-05-16 (30 days — stable ecosystem, no fast-moving libs involved)
