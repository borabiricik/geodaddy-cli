---
phase: 5
slug: core-web-vitals-measurement-lcp-fcp-cls-ttfb-tbt-and-performance-metrics-analyzer
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-23
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml (existing) |
| **Quick run command** | `cargo test 2>&1` |
| **Full suite command** | `cargo test -- --include-ignored 2>&1` |
| **Estimated runtime** | ~10 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test 2>&1`
- **After every plan wave:** Run `cargo test -- --include-ignored 2>&1`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 5-01-01 | 01 | 1 | D-04/D-05/D-06 | unit | `cargo test test_performance_category` | ✅ existing | ⬜ pending |
| 5-01-02 | 01 | 1 | D-07..D-11 | unit | `cargo test test_perf_severity_points` | ✅ existing | ⬜ pending |
| 5-01-03 | 01 | 1 | D-12..D-16 | unit | `cargo test test_perf_thresholds` | ❌ W0 | ⬜ pending |
| 5-02-01 | 02 | 2 | D-01/D-02 | unit | `cargo test test_vitals_flag` | ❌ W0 | ⬜ pending |
| 5-02-02 | 02 | 2 | D-03 | integration | `cargo test -- --ignored test_per_page_vitals` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/performance_thresholds.rs` — unit tests for LCP/FCP/CLS/TTFB/TBT threshold logic (D-12..D-16)
- [ ] `tests/vitals_flag.rs` — unit tests for `--vitals` flag parsing and CategoryScores serialization (D-01, D-04, D-05)

*Existing `src/scoring.rs` test module covers the scoring calculation changes.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| `--vitals` measures real LCP/FCP/CLS/TTFB/TBT via chromiumoxide | D-01..D-16 | Requires live browser + real page | Run `cargo build --release && ./target/release/geodaddy https://example.com --vitals` and verify JSON contains non-null `performance` category with all 5 perf-* results |
| `performance: null` when `--vitals` not passed | D-05 | JSON serialization behaviour | Run without `--vitals`, confirm `"performance":null` in `categories` |
| Independent `--vitals` and `--enable-js` | D-02 | Flag interaction | Run with both flags; confirm 2 separate chromiumoxide passes or combined pass; no crash |
| Per-page measurement in multi-page crawl | D-03 | Requires crawlable site | Run `--vitals --max-pages 3` against local dev server; verify 3 entries each with `perf-*` results |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
