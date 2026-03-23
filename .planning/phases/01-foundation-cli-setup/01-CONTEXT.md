# Phase 1: Foundation & CLI Setup - Context

**Gathered:** 2026-03-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver a working Rust CLI that fetches a single URL, checks robots.txt, and outputs a foundational JSON report. This is the scaffold that phases 2-4 build analysis features on top of. No analyzers, no scoring — just the crawler, CLI interface, and output shape.

Requirements: CRAWL-03, CRAWL-05, CLI-01, CLI-02, CLI-04

</domain>

<decisions>
## Implementation Decisions

### CLI Interface
- **D-01:** Flat command structure — `geodaddy <url>` with all flags directly on the root command. No subcommands. Flags at this phase: `--fail-under <score>`, `--help`.

### JSON Output Schema
- **D-02:** Page-centric structure from day one. Even single-URL output wraps pages in an array so the shape never changes when phase 4 adds multi-page crawling.

  Top-level structure:
  ```json
  {
    "schema_version": "1",
    "url": "<input url>",
    "crawled_at": "<iso8601>",
    "pages": [
      {
        "url": "<normalized url>",
        "robots_blocked": false,
        "results": []
      }
    ]
  }
  ```

  `results` is empty in phase 1 — analyzers populate it in phases 2-4. `schema_version` is a string to allow non-numeric versioning later.

### robots.txt Behavior
- **D-03:** Soft warn mode. Crawl always proceeds regardless of robots.txt directives. When the target URL is disallowed, set `robots_blocked: true` at the page level in JSON. No exit code impact from robots.txt alone. Missing robots.txt (common on localhost) = allow all.

### Project Structure
- **D-04:** Single Rust crate in `cli/` directory. Plain `cli/Cargo.toml` — no Cargo workspace at root. `web/` will be a separate non-Rust project and can sit alongside `cli/` when that time comes. No restructuring needed.

### Claude's Discretion
- Internal HTTP client configuration (timeouts, user-agent string, connection pooling settings) — use sensible defaults from CLAUDE.md tech stack.
- URL normalization implementation details — standard WHATWG normalization via the `url` crate.
- Error message formatting to stderr — keep it simple, human-readable.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Technology Stack
- `CLAUDE.md` — Full tech stack decisions: tokio, reqwest, clap, serde_json, url, robotstxt, anyhow, tracing. All library choices and versions are pre-decided here. Do not deviate.

### Requirements
- `.planning/REQUIREMENTS.md` — Phase 1 requirements: CRAWL-03, CRAWL-05, CLI-01, CLI-02, CLI-04

### Roadmap
- `.planning/ROADMAP.md` — Phase 1 success criteria (5 items), phase dependencies

No external specs — requirements fully captured in decisions above and referenced files.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — fresh project, no existing code.

### Established Patterns
- None yet — phase 1 establishes the patterns.

### Integration Points
- `cli/src/main.rs` — entry point (to be created)
- `cli/Cargo.toml` — crate manifest (to be created)

</code_context>

<specifics>
## Specific Ideas

- `web/` will use a non-Rust framework (stack TBD). No Cargo workspace needed — `cli/` and `web/` are independent projects in the same repo.
- `robots_blocked` field must appear at the page level (not top-level) to support phase 4's multi-page output where different pages may have different robots.txt outcomes.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 01-foundation-cli-setup*
*Context gathered: 2026-03-23*
