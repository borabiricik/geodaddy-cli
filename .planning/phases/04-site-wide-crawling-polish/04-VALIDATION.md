---
phase: 4
slug: site-wide-crawling-polish
status: approved
nyquist_compliant: true
wave_0_complete: true
created: 2026-03-23
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in Rust) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

Test names below match the `<behavior>` block in Plan 04-01 Task 1 exactly.

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 4-01-01 | 01 | 1 | CRAWL-01 | unit | `cargo test --lib crawling::tests::test_fetch_sitemap_urls_parses_xml` | ❌ W0 | ⬜ pending |
| 4-01-02 | 01 | 1 | CRAWL-01 | unit | `cargo test --lib crawling::tests::test_url_normalization` | ❌ W0 | ⬜ pending |
| 4-01-03 | 01 | 1 | CRAWL-01 | unit | `cargo test --lib crawling::tests::test_url_normalization_fragment` | ❌ W0 | ⬜ pending |
| 4-01-04 | 01 | 1 | CRAWL-01 | unit | `cargo test --lib crawling::tests::test_url_normalization_root` | ❌ W0 | ⬜ pending |
| 4-01-05 | 01 | 1 | CRAWL-02 | unit | `cargo test --lib crawling::tests::test_extract_same_origin_links` | ❌ W0 | ⬜ pending |
| 4-01-06 | 01 | 1 | CRAWL-02 | unit | `cargo test --lib crawling::tests::test_offsite_links_filtered` | ❌ W0 | ⬜ pending |
| 4-01-07 | 01 | 1 | CRAWL-02 | unit | `cargo test --lib crawling::tests::test_relative_link_resolution` | ❌ W0 | ⬜ pending |
| 4-01-08 | 01 | 1 | CRAWL-01 | unit | `cargo test --lib crawling::tests::test_extract_crawl_delay_present` | ❌ W0 | ⬜ pending |
| 4-01-09 | 01 | 1 | CRAWL-01 | unit | `cargo test --lib crawling::tests::test_extract_crawl_delay_absent` | ❌ W0 | ⬜ pending |
| 4-01-10 | 01 | 1 | CRAWL-04 | unit | `cargo test --lib crawling::tests::test_js_detection_thin_page` | ❌ W0 | ⬜ pending |
| 4-01-11 | 01 | 1 | CRAWL-04 | unit | `cargo test --lib crawling::tests::test_js_detection_rich_page` | ❌ W0 | ⬜ pending |
| 4-01-12 | 01 | 1 | CRAWL-01 | unit | `cargo test --lib crawling::tests::test_aggregate_score_average` | ❌ W0 | ⬜ pending |
| 4-01-13 | 01 | 1 | CRAWL-01 | unit | `cargo test --lib crawling::tests::test_aggregate_score_empty` | ❌ W0 | ⬜ pending |
| 4-01-14 | 01 | 1 | CLI-03 | unit | `cargo test --lib crawling::tests::test_progress_format_known` | ❌ W0 | ⬜ pending |
| 4-01-15 | 01 | 1 | CLI-03 | unit | `cargo test --lib crawling::tests::test_progress_format_unknown` | ❌ W0 | ⬜ pending |
| 4-02-01 | 02 | 2 | CRAWL-04 | build | `cargo build` (Report struct compile check) | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Wave 0 is satisfied by **Plan 04-01 Task 1** (tdd="true"). That task creates `src/crawling.rs` with all stub functions and the `#[cfg(test)]` block, and adds `mod crawling;` to `src/main.rs`. There is no separate stub-only step — the TDD task writes tests first (RED), then implements each function (GREEN), satisfying Wave 0 before Plan 04-02 executes.

- [x] `src/crawling.rs` — new module with functions and `#[cfg(test)]` block (Plan 04-01 Task 1)
- [x] Unit tests for all 15 behavior cases listed in Per-Task Verification Map (Plan 04-01 Task 1)
- [x] `src/main.rs` — `mod crawling;` declaration added (Plan 04-01 Task 1)

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| chromiumoxide detects and re-fetches JS-rendered pages | CRAWL-04 | Requires Chromium binary download (~150MB) and a real JS-rendered test server | Run `cargo run -- https://spa-example.com --enable-js 2>/dev/null` and verify `pages[]` has real content |
| Progress indicator shows `[N/TOTAL]` format during live crawl | CLI-03 | Requires live multi-page site | Run `cargo run -- https://example.com 2>&1 >/dev/null` and verify stderr output format |
| robots.txt crawl-delay is respected | CRAWL-01 | Requires timed execution with a real delay | Run against site with `Crawl-delay: 2` and verify ≥2s between page fetches in logs |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 10s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** approved
