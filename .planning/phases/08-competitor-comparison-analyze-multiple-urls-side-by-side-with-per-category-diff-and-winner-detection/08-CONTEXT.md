# Phase 8: Competitor comparison - Context

**Gathered:** 2026-04-16
**Status:** Ready for planning
**Source:** User specification (brainstorm + direct feature request)

<domain>
## Phase Boundary

Phase 8 delivers a **competitor comparison** feature to the geodaddy CLI — a new subcommand that accepts 2+ URLs, analyzes each one using the existing `analyze()` engine, and produces a single side-by-side comparison report (JSON + beauty mode).

**In scope (CLI layer only):**
- New `compare` subcommand: `geodaddy compare <url1> <url2> [url3...]`
- New `CompareReport` struct aggregating `Vec<Report>` plus derived comparison data
- Per-site score snapshot (overall + per-category)
- Per-check diff: for each check ID, which URLs pass/warn/fail
- Winner-per-category: site with highest score in technical / content / geo / performance
- Overall winner: site with highest aggregate score
- JSON output as the primary machine-readable format
- Beauty mode (`--beauty`): side-by-side colored terminal table
- Reuse of existing flags (`--enable-js`, `--vitals`, `--max-pages`) applied per target
- `--fail-under` semantics in compare mode: applies to the **first URL** (treated as "your site"), failing if the first URL's aggregate score falls below threshold (treat subsequent URLs as competitors)

**Out of scope (deferred to future phases):**
- Backend endpoint for compare (Phase 9)
- Web UI for compare (Phase 10)
- Prompt-to-citation tracking (different feature)
- AI crawler xray / content diff (different feature)
- Historical comparison tracking (diff between two runs over time)

</domain>

<decisions>
## Implementation Decisions

### CLI Surface
- **Use a clap subcommand `compare`**, not a flag. Rationale: user spec explicitly
  shows `geodaddy compare site1.com site2.com site3.com`. Preserves existing
  single-URL invocation semantics unchanged.
- **Minimum 2 URLs required.** Clap validation: `num_args = 2..` on the URLs argument.
- **No hard upper limit** on URL count in CLI, but document recommended max of ~10
  in `--help` text (beyond which beauty-mode table becomes unreadable).

### Flag reuse
- `--enable-js` — applied to all targets, single shared browser instance
- `--vitals` — applied to all targets, single shared vitals browser instance
- `--max-pages N` — applied **per target** (each URL gets its own crawl budget of N pages)
- `--beauty` — switches between JSON and beauty mode output
- `--fail-under SCORE` — applies to the **first URL only** (the "target" site). Compare mode frames the first URL as "your site" and the rest as competitors.

### Data model (Rust)
- New top-level struct `CompareReport`:
  - `schema_version: &'static str` (bump to "2" OR keep "1" with a separate compare schema — planner decides)
  - `compared_at: String` (RFC3339)
  - `sites: Vec<Report>` (full existing Report per URL)
  - `winners: Winners` (per-category + overall)
  - `check_diff: Vec<CheckDiff>` (one entry per unique check ID across all sites)
- `Winners` struct: `overall`, `technical`, `content`, `geo`, `performance` — each an `Option<String>` holding the winning URL (None if tied or category absent)
- `CheckDiff` struct: `check: String`, `results: Vec<SiteCheckOutcome>` where each outcome has `url: String`, `status: Option<Status>` (None if the site did not produce this check — e.g. perf checks without `--vitals`)
- Tie-breaking: if two sites are within 0.1 points of each other on a category, winner is `None` (tied). Rationale: avoids false precision from floating-point noise.

### Execution model
- **Analyze URLs sequentially in a loop** (not parallel). Rationale:
  - Shared headless browser instances — concurrent `new_page()` calls work but
    complicate error handling
  - Simpler progress logging (no interleaved output)
  - Politeness: sequential doesn't amplify load on any single site
- Reuse a single `reqwest::Client`, single optional JS `Browser`, single
  optional vitals `Browser` across all targets (same construction logic as
  existing single-URL path).
- On per-URL error: do not abort the whole compare. Log the error, mark the
  site as failed, continue with remaining URLs. The final `CompareReport`
  includes an error entry for failed sites (planner: define error shape).

### JSON output
- Primary output when not `--beauty`: pretty-printed JSON of `CompareReport`.
- Schema MUST be stable and documented (will be consumed by backend in Phase 9).

### Beauty mode
- Side-by-side table layout. Columns = sites (URLs abbreviated to domain).
- Sections:
  1. Header row: Overall score per site, color-coded
  2. Category row: Technical / Content / GEO / Performance scores per site
  3. Winners row: which site won each category (or "TIE")
  4. Per-check diff: compact table, one row per check ID, one column per site,
     cell shows status icon (✓ / ⚠ / ✗) with color
- Use `colored` crate (already a dependency). No new dependencies.
- Terminal width handling: if narrow terminal, truncate URLs to domain only;
  if still too narrow, log a warning and fall back to a vertical per-site report.

### Exit codes
- 0: analysis succeeded for all sites (or succeeded for first site and
  `--fail-under` threshold met)
- 1: first site's score below `--fail-under` threshold
- 2: first site failed to analyze (treat as CI failure signal)
- Other per-site failures do NOT affect exit code (competitors failing is
  informational, not a CI blocker)

### Claude's Discretion
- Exact module layout: new `compare.rs` module vs extending `lib.rs`
- Whether `CompareReport` lives in `lib.rs` or a new `compare` module
- Exact JSON field names (within schema guidelines)
- Test fixtures and mock strategy (existing mockito pattern)
- Progress logging format during compare loop
- Handling of duplicate URLs in args (dedupe silently? error? warn?)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Core library
- `src/lib.rs` — existing `analyze()` function signature, `Report` struct, `PageResult` struct. This is the reuse surface.
- `src/main.rs` — existing CLI entry, browser setup pattern, clap structure. New `compare` subcommand wires in here.
- `src/scoring.rs` — `CategoryScores`, `AnalysisResult`, `Status`, `calculate_score()`. CompareReport consumes these types.
- `src/beauty.rs` — existing single-report beauty printer. Compare beauty printer sits alongside (`print_beauty_compare_report`).

### Crawling
- `src/crawling.rs` — `aggregate_scores()` and URL normalization utilities. Compare may reuse `normalize_url` for deduplication.

### Existing patterns
- Clap derive usage — see `Cli` struct in `src/main.rs`
- Browser lifecycle (launch + handler spawn + cleanup) — see lines 70-115 in `src/main.rs`
- Tracing setup — stderr writer, env filter — see lines 43-50 in `src/main.rs`

### Project docs
- `cli/CLAUDE.md` — Rust conventions, dependency discipline, GSD workflow
- `cli/.planning/PROJECT.md` — product vision
- `cli/.planning/REQUIREMENTS.md` — requirement ID format (add new COMP-01 … if needed)

</canonical_refs>

<specifics>
## Specific Ideas

### CLI invocation examples (from user spec)
```bash
# Basic
geodaddy compare https://site1.com https://site2.com

# With three competitors
geodaddy compare https://mysite.com https://comp1.com https://comp2.com https://comp3.com

# With existing flags
geodaddy compare --max-pages 5 https://site1.com https://site2.com

# Beauty mode
geodaddy compare --beauty https://site1.com https://site2.com

# CI/CD usage — fail if target (first URL) below 80
geodaddy compare --fail-under 80 https://mysite.com https://competitor.com

# Full feature stack
geodaddy compare --enable-js --vitals --max-pages 10 https://a.com https://b.com
```

### JSON shape sketch (illustrative, planner refines)
```json
{
  "schema_version": "1",
  "compared_at": "2026-04-16T10:00:00Z",
  "sites": [
    { /* full Report for site 1 */ },
    { /* full Report for site 2 */ }
  ],
  "winners": {
    "overall": "https://site1.com",
    "technical": "https://site1.com",
    "content": "https://site2.com",
    "geo": null,
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
  ],
  "errors": [
    { "url": "https://unreachable.com", "message": "..." }
  ]
}
```

### Beauty mode sketch (illustrative)
```
geodaddy — Competitor Comparison Report
─────────────────────────────────────────────────────
                     site1.com   site2.com   site3.com
Overall Score        87.3        72.1        45.8
Technical            92.0        88.5        60.0
Content              85.0        70.0        50.0
GEO                  85.0        58.0        30.0
Performance          87.2        71.8        N/A

Winners
  Overall:     site1.com
  Technical:   site1.com
  Content:     site1.com
  GEO:         site1.com
  Performance: site1.com

Per-check diff
  tech-meta-title      ✓  ✗  ✓
  tech-https           ✓  ✓  ✗
  geo-llms-txt         ✓  ✗  ✗
  ...
```

</specifics>

<deferred>
## Deferred Ideas

Items explicitly out of scope for Phase 8; will be handled in later phases:

- **Backend `/compare` endpoint** — Phase 9 (add to backend's `.planning/`)
- **Web UI for compare** — Phase 10 (add to web's `.planning/`)
- **Parallel URL analysis** — Current decision is sequential. Parallelization can be revisited if benchmarks show it is a user pain point.
- **Compare history / trend over time** — requires persistence; belongs in a separate "monitoring" feature track.
- **Diff between two compare runs** — "how did competitive landscape change since last week?" — future phase.
- **Prompt-to-citation AI visibility tracking** — completely different feature track (paid tier).
- **AI crawler xray (GPTBot content diff)** — separate feature track.
- **Extension integration for compare** — not planned in current scope.

</deferred>

---

*Phase: 08-competitor-comparison*
*Context gathered: 2026-04-16 via user specification + conversation*
