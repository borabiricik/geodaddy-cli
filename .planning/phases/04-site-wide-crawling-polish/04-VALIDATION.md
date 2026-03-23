---
phase: 4
slug: site-wide-crawling-polish
status: draft
nyquist_compliant: false
wave_0_complete: false
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

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 4-01-01 | 01 | 1 | CRAWL-01 | unit | `cargo test --lib crawling::tests::test_sitemap_url_extraction` | ❌ W0 | ⬜ pending |
| 4-01-02 | 01 | 1 | CRAWL-01 | unit | `cargo test --lib crawling::tests::test_sitemap_priority_sort` | ❌ W0 | ⬜ pending |
| 4-01-03 | 01 | 1 | CRAWL-02 | unit | `cargo test --lib crawling::tests::test_link_extraction` | ❌ W0 | ⬜ pending |
| 4-01-04 | 01 | 1 | CRAWL-02 | unit | `cargo test --lib crawling::tests::test_link_depth_limit` | ❌ W0 | ⬜ pending |
| 4-01-05 | 01 | 1 | CRAWL-01 | unit | `cargo test --lib crawling::tests::test_url_deduplication` | ❌ W0 | ⬜ pending |
| 4-01-06 | 01 | 2 | CLI-03 | unit | `cargo test --lib crawling::tests::test_progress_format_known_total` | ❌ W0 | ⬜ pending |
| 4-01-07 | 01 | 2 | CLI-03 | unit | `cargo test --lib crawling::tests::test_progress_format_unknown_total` | ❌ W0 | ⬜ pending |
| 4-02-01 | 02 | 1 | CRAWL-04 | unit | `cargo test --lib crawling::tests::test_js_detection_thin_page` | ❌ W0 | ⬜ pending |
| 4-02-02 | 02 | 1 | CRAWL-04 | unit | `cargo test --lib crawling::tests::test_js_detection_rich_page` | ❌ W0 | ⬜ pending |
| 4-03-01 | 03 | 1 | CRAWL-01 | unit | `cargo test --lib` (Report struct compile check) | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/crawling.rs` — new module with stub functions and `#[cfg(test)]` block
- [ ] Unit test stubs for all 10 test cases in the Per-Task Verification Map
- [ ] `src/main.rs` — `mod crawling;` declaration added

*Wave 0 is the first task in Plan 01.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| chromiumoxide detects and re-fetches JS-rendered pages | CRAWL-04 | Requires Chromium binary download (~150MB) and a real JS-rendered test server | Run `cargo run -- https://spa-example.com --enable-js 2>/dev/null` and verify `pages[]` has real content |
| Progress indicator shows `[N/TOTAL]` format during live crawl | CLI-03 | Requires live multi-page site | Run `cargo run -- https://example.com 2>&1 >/dev/null` and verify stderr output format |
| robots.txt crawl-delay is respected | CRAWL-01 | Requires timed execution with a real delay | Run against site with `Crawl-delay: 2` and verify ≥2s between page fetches in logs |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 10s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
