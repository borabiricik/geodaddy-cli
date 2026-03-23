---
phase: quick-260323-vfu
plan: "01"
subsystem: crawling / cli
tags: [crawling, cli, single-url-mode, max-pages, opt-in]
dependency_graph:
  requires: []
  provides: [single-url-default-mode, opt-in-crawling-via-max-pages]
  affects: [src/main.rs, tests/integration.rs]
tech_stack:
  added: []
  patterns: [opt-in crawling via CLI flag guard]
key_files:
  modified:
    - src/main.rs
    - tests/integration.rs
decisions:
  - "Crawling is opt-in: without --max-pages the URL list is always [cli.url] and neither fetch_sitemap_urls nor collect_links_bfs are called"
  - "Integration tests for sitemap-driven and BFS-fallback progress formats require --max-pages to exercise the crawling code path"
metrics:
  duration_minutes: 5
  completed_date: "2026-03-23"
  tasks_completed: 1
  files_modified: 2
---

# Quick Task 260323-vfu: Fix Crawling Behavior When --max-pages Is Absent

**One-liner:** Made crawling opt-in by gating sitemap/BFS discovery behind a `cli.max_pages.is_none()` early-return so the default `geodaddy <url>` invocation analyzes exactly one page.

## Summary

The crawler was unconditionally calling `fetch_sitemap_urls` and falling back to `collect_links_bfs` on every run regardless of whether `--max-pages` was passed. This violated the documented contract that crawling is opt-in. With this fix:

- `geodaddy https://example.com` → URL list is `[cli.url]`, no network calls for sitemap or BFS.
- `geodaddy https://example.com --max-pages 5` → sitemap-first strategy, BFS fallback, capped at 5 pages — unchanged behavior.

The `--max-pages` help text was also updated to describe the opt-in semantics.

## Tasks Completed

| # | Name | Commit | Files |
|---|------|--------|-------|
| 1 | Gate URL discovery behind --max-pages presence | af98719 | src/main.rs, tests/integration.rs |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Integration tests for crawling progress omitted --max-pages**

- **Found during:** Task 1 verification (`cargo test`)
- **Issue:** `test_progress_to_stderr_sitemap_format` and `test_progress_to_stderr_bfs_fallback` both invoked the CLI without `--max-pages`, so with the new guard they now hit single-URL mode and their sitemap/BFS assertions fail.
- **Fix:** Added `--max-pages 10` to both test invocations so they exercise the crawling code path they were designed to verify.
- **Files modified:** tests/integration.rs
- **Commit:** af98719

## Known Stubs

None — no placeholder data or wired-but-empty paths introduced.

## Self-Check: PASSED

- `src/main.rs` exists and contains `cli.max_pages.is_none()` guard: FOUND
- `tests/integration.rs` exists with `--max-pages` args in both fixed tests: FOUND
- Commit af98719 exists: FOUND
- `cargo test`: 8 integration + 113 unit tests pass, 0 failures
