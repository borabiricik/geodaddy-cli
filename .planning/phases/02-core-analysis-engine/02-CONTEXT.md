# Phase 2: Core Analysis Engine - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Add 12 analyzers across Technical SEO and Content Structure categories, a severity-weighted scoring system, and a typed `AnalysisResult` struct — all feeding into the existing `results[]` array from phase 1. Phase 2 transforms the scaffold into a working analysis engine.

Requirements: TECH-01, TECH-02, TECH-03, TECH-04, TECH-05, TECH-06, TECH-07, TECH-08, CONT-01, CONT-02, CONT-03, CONT-04, SCORE-01, SCORE-02, SCORE-03, SCORE-04

</domain>

<decisions>
## Implementation Decisions

### Result Item Shape (D-05)
- **D-05:** Every analyzer produces `AnalysisResult` structs that serialize to:
  ```json
  {
    "check": "tech-meta-title",
    "status": "fail",
    "message": "Title is 72 chars (max 60)",
    "recommendation": "Shorten title to 50-60 chars for optimal display in search results"
  }
  ```
  Fields: `check: &'static str`, `status: Status` (enum: Pass/Fail/Warn), `message: String`, `recommendation: String`.
  No `details` object — v1 keeps results human-readable strings only.

### Scoring Formula (D-06)
- **D-06:** Severity-weighted scoring. Each check has a severity: `critical | warning | info`.
  - `fail` on a check → lose **full** severity points
  - `warn` on a check → lose **half** severity points
  - `pass` on a check → lose **0** points

  Suggested point values (planner can adjust, but must assign one of these tiers):
  - `critical`: 10 points
  - `warning`: 5 points
  - `info`: 2 points

  Score = `(earned_points / max_possible_points) * 100`, clamped to [0, 100].

  **Per-category scores** (SCORE-02): calculate separately for Technical (TECH-*) and Content (CONT-*). GEO category is absent in phase 2 — omit from output, do not penalize.

  **Overall score** (SCORE-01): weighted average of present categories. Equal weight in phase 2 (50/50 Technical/Content). GEO added in phase 3.

  **Severity assignments** (canonical list — planner must use exactly these):

  | Check ID | Requirement | Severity |
  |----------|-------------|----------|
  | tech-broken-links | TECH-01 | warning (stub in phase 2) |
  | tech-redirect-chains | TECH-02 | warning |
  | tech-meta-title | TECH-03 | critical |
  | tech-meta-description | TECH-03 | warning |
  | tech-heading-h1 | TECH-04 | critical |
  | tech-heading-hierarchy | TECH-04 | warning |
  | tech-mobile-viewport | TECH-05 | critical |
  | tech-robots-txt | TECH-06 | warning |
  | tech-sitemap-xml | TECH-07 | info |
  | tech-https | TECH-08 | critical |
  | cont-heading-structure | CONT-01 | warning |
  | cont-json-ld | CONT-02 | critical |
  | cont-semantic-html | CONT-03 | info |
  | cont-alt-text | CONT-04 | warning |

### Code Architecture (D-07)
- **D-07:** Flat module structure. No trait abstraction. `main.rs` stays thin.

  ```
  cli/src/
  ├── main.rs              — CLI, HTTP fetch, orchestrate analyzers, output JSON
  ├── analyzers/
  │   ├── mod.rs           — re-exports
  │   ├── technical.rs     — TECH-01 through TECH-08 analyzer functions
  │   └── content.rs       — CONT-01 through CONT-04 analyzer functions
  └── scoring.rs           — AnalysisResult struct, Status enum, score calculation
  ```

  Each analyzer function signature: `fn analyze_<name>(html: &scraper::Html, url: &Url) -> Vec<AnalysisResult>`
  (or `-> AnalysisResult` for single-result checks).

  `main.rs` collects all results into a `Vec<AnalysisResult>`, serializes to `results[]`, then passes to scorer.

### Broken Link Check (D-08)
- **D-08:** TECH-01 is a phase 2 stub. No HTTP requests fired. Emits a single `warn` result:
  ```json
  {
    "check": "tech-broken-links",
    "status": "warn",
    "message": "Broken link detection requires site-wide crawl mode",
    "recommendation": "Run geodaddy with site-wide crawling (Phase 4) to detect broken links across all pages"
  }
  ```
  Full implementation moves to phase 4 alongside the multi-page crawler.

### Carried Forward from Phase 1
- Flat CLI — any new flags (none expected in phase 2) go directly on `geodaddy <url>`
- `PageResult.results` transitions from `Vec<serde_json::Value>` to `Vec<AnalysisResult>` where `AnalysisResult: Serialize`
- `reqwest::Client` built in `main.rs` — pass as reference to analyzers that need HTTP (redirect checks)
- Tracing always goes to stderr, JSON always to stdout

### Claude's Discretion
- Exact redirect chain detection logic (how many hops = "excessive" — suggest 3+)
- robots.txt validation heuristics (what counts as a syntax error vs. warning)
- Sitemap.xml URL count threshold for "too large" warning
- Mixed content detection pattern (scan for `http://` in `src` / `href` attributes on HTTPS pages)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Technology Stack
- `CLAUDE.md` — Full tech stack. Phase 2 adds: `scraper` (HTML parsing), `jsonschema` (JSON-LD validation), `quick-xml` (sitemap.xml parsing). All versions pre-decided.

### Phase 1 Foundation
- `cli/src/main.rs` — Existing entry point. Understand current structure before modifying.
- `cli/Cargo.toml` — Add new dependencies here. No root-level Cargo.toml.

### Requirements
- `.planning/REQUIREMENTS.md` — Full descriptions of TECH-01–08, CONT-01–04, SCORE-01–04

### Roadmap
- `.planning/ROADMAP.md` — Phase 2 success criteria (5 items), phase dependencies

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `reqwest::Client` built in `main.rs` — pass by reference to analyzers needing HTTP (TECH-02 redirect detection, TECH-06 robots.txt fetch, TECH-07 sitemap fetch, TECH-08 HTTPS check)
- `Url` parsing already in scope — pass normalized `&Url` to all analyzers
- `check_robots()` in `main.rs` — can be moved to `analyzers/technical.rs` as part of TECH-06

### Established Patterns
- All tracing → stderr via `tracing_subscriber` with stderr writer — do not break this
- `process::exit(1)` called AFTER `println!()` — JSON must print before exit
- `anyhow::Result<()>` in main, `?` propagation for recoverable errors

### Integration Points
- `PageResult.results: Vec<serde_json::Value>` → changes to `Vec<AnalysisResult>` where `AnalysisResult: Serialize`
- Score fields added to `PageResult`: `score: f64`, `categories: CategoryScores`
- `main.rs` gains: HTML fetch step, analyzer orchestration, score calculation before JSON output

### New Dependencies to Add (cli/Cargo.toml)
- `scraper = "0.26"` — HTML parsing for all content/technical checks
- `jsonschema = "0.45"` — JSON-LD schema validation (CONT-02)
- `quick-xml = { version = "0.38", features = ["serialize"] }` — sitemap.xml parsing (TECH-07)

</code_context>

<specifics>
## Specific Ideas

- `tech-broken-links` is a **stub** in phase 2 — always emits `warn`, never makes HTTP requests. This is intentional (D-08). Do not implement actual link fetching.
- Severity table in D-06 is canonical. Planner must use exactly these check IDs and severity assignments — changing them breaks the scoring contract.
- `AnalysisResult` struct lives in `scoring.rs` (not in the analyzer modules) because it's a shared type.
- Phase 2 score has no GEO category — `categories` output should only include `technical` and `content` keys.

</specifics>

<deferred>
## Deferred Ideas

- Full broken link HTTP checking → Phase 4 (multi-page crawl)
- GEO category scoring → Phase 3
- Trait-based Analyzer abstraction → not planned for v1 (flat modules sufficient)

</deferred>

---

*Phase: 02-core-analysis-engine*
*Context gathered: 2026-03-23*
