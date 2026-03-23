---
phase: 5
slug: core-web-vitals-measurement-lcp-fcp-cls-ttfb-tbt-and-performance-metrics-analyzer
status: draft
nyquist_compliant: true
wave_0_complete: true
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
| 5-01-03 | 01 | 1 | D-12..D-16 | unit | `cargo test classify` | ✅ inline (#[cfg(test)] in performance.rs, Plan 02) | ⬜ pending |
| 5-02-01 | 02 | 2 | D-01/D-02 | unit | `cargo test test_vitals_flag` | ✅ inline (#[cfg(test)] in performance.rs, Plan 02) | ⬜ pending |
| 5-02-02 | 02 | 2 | D-03 | integration | `cargo test -- --ignored test_per_page_vitals` | ✅ inline (tests/integration.rs, Plan 03) | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Notes

Wave 0 external test files are NOT required for this phase. All threshold classification unit tests live inline as `#[cfg(test)] mod tests` blocks inside `src/analyzers/performance.rs` (created in Plan 02). Plan 02's task action explicitly names all classify_* tests starting with `classify_` so they are addressable via `cargo test classify`.

- D-12..D-16 threshold tests: inline in `src/analyzers/performance.rs` — covered by `cargo test classify`
- D-01/D-04/D-05 flag/scoring tests: inline in `src/scoring.rs` (Plan 01) and `tests/integration.rs` (Plan 03)

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

- [x] All tasks have `<automated>` verify commands
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 coverage satisfied by inline #[cfg(test)] blocks in Plan 02 (no external test files needed)
- [x] No watch-mode flags
- [x] Feedback latency < 30s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
