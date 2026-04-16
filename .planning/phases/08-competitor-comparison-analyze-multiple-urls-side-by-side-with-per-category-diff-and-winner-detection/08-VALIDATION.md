---
phase: 8
slug: competitor-comparison
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-16
---

# Phase 8 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Rust built-in test harness (`#[test]`) + `cargo test` |
| **Config file** | None — uses `Cargo.toml` `[dev-dependencies]` + `tests/integration.rs` |
| **Quick run command** | `cargo test --lib compare::` (unit tests for new module only) |
| **Full suite command** | `cargo test` (all lib unit tests + all integration tests) |
| **Estimated runtime** | Quick: ~5s · Full (non-chromium): ~30s |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib compare:: && cargo test --test integration -- --skip ignored`
- **After every plan wave:** Run `cargo test && cargo clippy --all-targets -- -D warnings`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** ~30 seconds (quick)

---

## Per-Task Verification Map

Derived from RESEARCH.md "Phase Requirements → Test Map" (lines 862-884). Planner fills in Task ID + Plan + Wave columns when breaking down phase.

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| TBD | TBD | 0 | COMP-01 | integration | `cargo test --test integration test_compare_requires_two_urls` | ❌ W0 | ⬜ pending |
| TBD | TBD | 0 | COMP-01 | integration | `cargo test --test integration test_json_output_has_score_categories_pages` | ✅ | ⬜ pending |
| TBD | TBD | 0 | COMP-02 | unit | `cargo test --lib compare::tests::test_loop_calls_analyze_per_url` | ❌ W0 | ⬜ pending |
| TBD | TBD | 0 | COMP-02 | integration | `cargo test --test integration test_compare_shares_http_client` | ❌ W0 | ⬜ pending |
| TBD | TBD | 0 | COMP-03 | integration | `cargo test --test integration test_compare_max_pages_per_target` | ❌ W0 | ⬜ pending |
| TBD | TBD | 0 | COMP-03 | integration | `cargo test --test integration test_compare_beauty_prints_table` | ❌ W0 | ⬜ pending |
| TBD | TBD | 0 | COMP-04 | integration | `cargo test --test integration test_compare_json_schema_stable` | ❌ W0 | ⬜ pending |
| TBD | TBD | 0 | COMP-05 | unit | `cargo test --lib compare::tests::test_winner_highest_score` | ❌ W0 | ⬜ pending |
| TBD | TBD | 0 | COMP-05 | unit | `cargo test --lib compare::tests::test_winner_tie_within_epsilon` | ❌ W0 | ⬜ pending |
| TBD | TBD | 0 | COMP-05 | unit | `cargo test --lib compare::tests::test_winner_performance_absent` | ❌ W0 | ⬜ pending |
| TBD | TBD | 0 | COMP-06 | unit | `cargo test --lib compare::tests::test_check_diff_unique_checks` | ❌ W0 | ⬜ pending |
| TBD | TBD | 0 | COMP-06 | unit | `cargo test --lib compare::tests::test_check_diff_missing_null` | ❌ W0 | ⬜ pending |
| TBD | TBD | 0 | COMP-06 | unit | `cargo test --lib compare::tests::test_aggregate_check_status` | ❌ W0 | ⬜ pending |
| TBD | TBD | 0 | COMP-07 | unit | `cargo test --lib beauty::tests::test_compare_beauty_variable_columns` | ❌ W0 | ⬜ pending |
| TBD | TBD | 0 | COMP-07 | unit | `cargo test --lib beauty::tests::test_narrow_terminal_fallback` | ❌ W0 | ⬜ pending |
| TBD | TBD | 0 | COMP-08 | integration | `cargo test --test integration test_compare_fail_under_first_url` | ❌ W0 | ⬜ pending |
| TBD | TBD | 0 | COMP-08 | integration | `cargo test --test integration test_compare_competitor_low_score_ignored` | ❌ W0 | ⬜ pending |
| TBD | TBD | 0 | COMP-09 | integration | `cargo test --test integration test_compare_continues_on_per_url_error` | ❌ W0 | ⬜ pending |
| TBD | TBD | 0 | COMP-09 | integration | `cargo test --test integration test_compare_first_url_failure_exit_2` | ❌ W0 | ⬜ pending |
| TBD | TBD | 0 | COMP-10 | integration | `cargo test --test integration test_compare_dedupes_duplicate_urls` | ❌ W0 | ⬜ pending |
| TBD | TBD | 0 | COMP-10 | unit | `cargo test --lib compare::tests::test_dedup_uses_normalize_url` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/compare.rs` — new module with `#[cfg(test)] mod tests` stubs for all 10 compare::tests::* entries above
- [ ] `tests/integration.rs` — add compare-specific test stubs (12 new `#[test]` fns from map above)
- [ ] Extend existing mockito fixture pattern to support multi-server (two `mockito::Server::new()` instances) for multi-origin compare tests
- [ ] Add Winners / CompareReport / CheckDiff / SiteCheckOutcome types to `src/compare.rs` so tests compile
- [ ] Wire new `compare` subcommand into `Cli` struct (clap derive) — stub body OK for Wave 0

*All 22 test commands in the map above reference tests that do NOT yet exist. Wave 0 MUST land stub files that compile and run (even if red) before any implementation task begins.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Visual inspection of beauty-mode side-by-side table rendering | COMP-07 | ANSI color + terminal width are inherently visual; automated test verifies structural output but not human readability | Run `cargo run --release -- compare --beauty https://example.com https://example.org` in a wide terminal (≥120 cols). Confirm: table has 3 columns (label + 2 sites), colors applied to score cells, winner row shows URLs not "TIE" for clear diffs, per-check diff rows use ✓/⚠/✗ icons. |
| Real-world multi-origin analysis with live sites | COMP-02 | Integration tests use mockito; end-to-end behavior against real HTTP servers not automated | Run `cargo run --release -- compare https://example.com https://example.org` once before merge. Confirm: both reports land in `sites[]`, no panics, winners populated. |
| Chromium-dependent flows (`--enable-js`, `--vitals`) across multiple URLs | COMP-02, COMP-03 | Chromium download in CI is gated (ignored in test suite); chromiumoxide session state across different origins not automated | Run `cargo run --release -- compare --vitals https://example.com https://example.org` locally with Chromium installed. Confirm: both sites get performance scores, no SingletonLock collisions, browser data dir cleaned up. |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references (compare::tests::*, new integration tests)
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
