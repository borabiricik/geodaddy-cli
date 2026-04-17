---
phase: 08-competitor-comparison
verified: 2026-04-17T12:05:00Z
status: passed
score: 14/14 must-haves verified
---

# Phase 8: Competitor Comparison Verification Report

**Phase Goal:** Competitor comparison: analyze multiple URLs side-by-side with per-category diff and winner detection
**Verified:** 2026-04-17T12:05:00Z
**Status:** passed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #  | Truth                                                                                                                        | Status     | Evidence                                                                                                                       |
| -- | ---------------------------------------------------------------------------------------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------ |
| 1  | `compare` CLI subcommand accepts 2+ URLs (clap `num_args = 2..`)                                                             | VERIFIED   | `src/main.rs:47-56` defines `Commands::Compare { urls: Vec<String> }` with `#[arg(num_args = 2..)]`; `test_compare_requires_two_urls` passes |
| 2  | Reuses existing `analyze()` per URL sequentially                                                                             | VERIFIED   | `src/compare.rs:101-112` — sequential `for` loop calling `crate::analyze(url, config, client, js_browser, vitals_browser).await` |
| 3  | Respects existing flags (`--enable-js`, `--vitals`, `--max-pages`, `--beauty`, `--fail-under`) via clap `global = true`      | VERIFIED   | `src/main.rs:22,26,31,36,40` — all 5 top-level flags carry `global = true`; `test_compare_max_pages_per_target`, `test_compare_fail_under_first_url`, `test_compare_beauty_prints_table` pass |
| 4  | Per-site scores (overall + per-category) present in output                                                                   | VERIFIED   | Runtime smoke: JSON contains `sites[*].score` + `sites[*].categories.{technical,content,geo,performance}`; `test_compare_winners_populated` passes |
| 5  | Per-check diff: which URLs pass/warn/fail each check                                                                         | VERIFIED   | `src/compare.rs:171-203` + `aggregate_site_check_status`; runtime smoke shows 39 check_diff entries; `test_compare_check_diff_populated` passes |
| 6  | Winner-per-category + overall (with 0.1 epsilon tie detection)                                                                | VERIFIED   | `src/compare.rs:130-166` — `compute_winners` with `TIE_EPSILON = 0.1`; `test_winner_highest_score`, `test_winner_tie_within_epsilon`, `test_winner_performance_absent` all pass |
| 7  | JSON output (stable schema: `schema_version`, `compared_at`, `sites`, `winners`, `check_diff`, `errors`)                      | VERIFIED   | Runtime smoke: all 6 keys present; `test_compare_json_schema_stable` passes; `schema_version == "1"` |
| 8  | Beauty mode: side-by-side colored table                                                                                      | VERIFIED   | `src/beauty.rs:144-229` — `print_beauty_compare_report` with COL_LABEL_WIDTH/COL_SITE_WIDTH padding + colored crate; `test_compare_beauty_variable_columns` (2/3/5/10 sites) + `test_compare_beauty_prints_table` pass |
| 9  | Sequential analysis (not parallel)                                                                                           | VERIFIED   | `src/compare.rs:101` — synchronous `for (i, url) in unique.iter().enumerate()` with `.await` per iteration |
| 10 | Shared `reqwest::Client` + optional browsers                                                                                 | VERIFIED   | `src/main.rs:199-250` — single client + `js_browser` + `vitals_browser` constructed once, passed to `compare_sites` by ref |
| 11 | Exit-code policy: 0 success, 1 if first URL below `--fail-under`, 2 if first URL fails                                       | VERIFIED   | `src/main.rs:283-304` — policy matches CONTEXT lines 93-99; smoke-tested: first URL unreachable → EXIT=2; `--fail-under 99` → EXIT=1 |
| 12 | Tie detection via 0.1 epsilon                                                                                                | VERIFIED   | `src/compare.rs:21` defines `TIE_EPSILON: f64 = 0.1`; `src/compare.rs:146-152` uses `(max - v).abs() < TIE_EPSILON` + `top_count > 1 → None`; `test_winner_tie_within_epsilon` passes |
| 13 | URL dedup via `crawling::normalize_url` with stderr warning                                                                  | VERIFIED   | `src/compare.rs:236-248` — `dedup_urls` uses `crate::crawling::normalize_url` + `tracing::warn!("Duplicate URL ignored: {}", url)`; smoke-tested (stderr warning visible); `test_compare_dedupes_duplicate_urls` passes |
| 14 | Per-URL errors non-fatal + backward-compat single-URL mode                                                                   | VERIFIED   | `src/compare.rs:105-111` — Err → `errors.push(CompareError{...})`, never returns Err; `test_compare_continues_on_per_url_error` passes; `test_json_output_has_score_categories_pages` (backward-compat sentinel) passes |

**Score:** 14/14 truths verified

### Required Artifacts

| Artifact                                                                 | Expected                                                                      | Status     | Details                                                                                          |
| ------------------------------------------------------------------------ | ----------------------------------------------------------------------------- | ---------- | ------------------------------------------------------------------------------------------------ |
| `src/compare.rs`                                                         | Types + compare_sites + compute_winners + compute_check_diff + dedup_urls     | VERIFIED   | 463 lines; all 4 key fns present (lines 89, 130, 171, 236); zero `todo!()`; 10 unit tests green  |
| `src/main.rs` (`Commands::Compare` + `run_compare_flow` + global flags)  | Subcommand dispatch + run_compare_flow + exit policy                          | VERIFIED   | 308 lines; `Commands::Compare` matched at line 73; `run_compare_flow` at line 192; 5 `global = true` flags |
| `src/beauty.rs` (`print_beauty_compare_report` + helpers)                | Side-by-side table + narrow fallback + helpers                                 | VERIFIED   | 418 lines; `print_beauty_compare_report` at line 144; 5 helpers (detect_terminal_width, compare_column_header, print_compare_category_row, print_compare_winner_line, print_compare_check_diff_row) + vertical fallback; 3 unit tests green |
| `tests/integration.rs` (12 `test_compare_*` tests)                       | 12 compare integration tests covering COMP-01..COMP-10                         | VERIFIED   | 12 `test_compare_*` functions at lines 582, 603, 623, 646, 666, 689, 708, 746, 767, 781, 802, 823 |
| `.planning/REQUIREMENTS.md`                                              | COMP-01..COMP-10 entries + 10 traceability rows                                | VERIFIED   | Lines 68-77 (10 requirement entries, all `[x]`); lines 160-169 (10 traceability rows); Coverage bumped to 45 total |
| `Cargo.toml`                                                             | No new dependencies vs. CLAUDE.md stack                                        | VERIFIED   | 18 runtime deps match stack exactly (tokio, clap, reqwest, serde, serde_json, url, robotstxt, anyhow, tracing, tracing-subscriber, chrono, scraper, regex, jsonschema, quick-xml, chromiumoxide, futures, colored) + dev-deps (assert_cmd, mockito). Zero additions for Phase 8. |

### Key Link Verification

| From                                                    | To                                                   | Via                                                    | Status | Details                                                                                         |
| ------------------------------------------------------- | ---------------------------------------------------- | ------------------------------------------------------ | ------ | ----------------------------------------------------------------------------------------------- |
| `src/main.rs::run_compare_flow`                         | `src/compare.rs::compare_sites`                      | `compare::compare_sites(urls, &config, &client, ...)`  | WIRED  | `src/main.rs:255-262`; exact signature match                                                    |
| `src/main.rs::run_compare_flow`                         | CLI exit codes                                       | `std::process::exit(1|2)`                              | WIRED  | `src/main.rs:286` (exit 2 for first-URL failure), `src/main.rs:294,298` (exit 1 for --fail-under) |
| `src/compare.rs::compare_sites`                         | `geodaddy::analyze`                                  | `crate::analyze(url, config, client, js_browser, ...)` | WIRED  | `src/compare.rs:103`; exact signature                                                           |
| `src/compare.rs::dedup_urls`                            | `src/crawling.rs::normalize_url`                     | `crate::crawling::normalize_url(url)`                  | WIRED  | `src/compare.rs:240`                                                                            |
| `src/main.rs`                                           | `src/beauty.rs::print_beauty_compare_report`         | `use geodaddy::beauty::print_beauty_compare_report;` + call | WIRED  | Import at `src/main.rs:8`; call at `src/main.rs:266` inside `if cli.beauty` branch              |
| `src/beauty.rs::print_beauty_compare_report`            | `src/beauty.rs::print_beauty_report`                 | Narrow-fallback per-site loop                          | WIRED  | `src/beauty.rs:291`; vertical fallback reuses single-site renderer                              |
| `src/lib.rs`                                            | `src/compare.rs`                                     | `pub mod compare;`                                     | WIRED  | Module declaration exposes compare publicly                                                     |

### Data-Flow Trace (Level 4)

| Artifact                                   | Data Variable                 | Source                                                        | Produces Real Data | Status   |
| ------------------------------------------ | ----------------------------- | ------------------------------------------------------------- | ------------------ | -------- |
| `src/compare.rs::compare_sites`            | `sites: Vec<Report>`          | `crate::analyze()` per URL (existing engine, full pipeline)   | Yes                | FLOWING  |
| `src/compare.rs::compute_winners`          | `scored: Vec<(&str, f64)>`    | Extractor closure over `site.score` / `site.categories.*`     | Yes                | FLOWING  |
| `src/compare.rs::compute_check_diff`       | `all_checks: HashSet<&str>`   | Triple-nested scan: `sites[*].pages[*].results[*].check`      | Yes                | FLOWING  |
| `src/beauty.rs::print_beauty_compare_report` | `report: &CompareReport`    | Caller passes populated report from `run_compare_flow`         | Yes                | FLOWING  |
| Runtime smoke: `geodaddy compare https://example.com https://example.org` | JSON output | Full analyze pipeline | Yes | FLOWING (2 sites, 39 check_diff rows, winners populated, errors=[]) |

### Behavioral Spot-Checks

| Behavior                                                                      | Command                                                                                                                                         | Result                                                                                               | Status |
| ----------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- | ------ |
| Build succeeds (release)                                                      | `cargo build --release`                                                                                                                          | 0 errors, 0 warnings                                                                                 | PASS   |
| All unit tests pass                                                           | `cargo test --lib`                                                                                                                               | 202 passed; 0 failed; 0 ignored                                                                      | PASS   |
| All integration tests pass (skip ignored)                                     | `cargo test --test integration -- --skip ignored`                                                                                                | 19 passed; 0 failed; 1 ignored (chromium-gated); 1 filtered                                           | PASS   |
| Full test suite                                                               | `cargo test`                                                                                                                                    | 20 integration passed + 202 lib passed + 0 doc-tests, 1 ignored                                       | PASS   |
| Valid JSON output with all 6 schema keys                                       | `./target/release/geodaddy compare https://example.com https://example.org`                                                                      | schema_version=1, compared_at (string), sites[2], winners (5 keys), check_diff (39 rows), errors[0] | PASS   |
| First URL failure → exit 2                                                    | `geodaddy compare http://127.0.0.1:1 https://example.com; echo $?`                                                                              | EXIT=2, errors[] contains unreachable URL, sites[1]                                                  | PASS   |
| `--fail-under` triggers exit 1 when first URL below threshold                 | `geodaddy compare --fail-under 99 https://example.com https://example.org; echo $?`                                                              | EXIT=1                                                                                               | PASS   |
| Duplicate URL emits stderr warning                                            | `geodaddy compare https://example.com https://example.com https://example.org 2>&1 1>/dev/null \| grep -i duplicate`                            | Stderr: `WARN geodaddy::compare: Duplicate URL ignored: https://example.com`                         | PASS   |
| Narrow terminal fallback emits stderr warning                                 | `COLUMNS=40 geodaddy compare --beauty https://example.com https://example.org 2>&1 1>/dev/null`                                                  | Stderr: `Terminal too narrow for side-by-side table (50 cols needed, 40 available). Falling back...` | PASS   |

### Requirements Coverage

| Requirement | Source Plan         | Description                                                                                                          | Status    | Evidence                                                                                                                   |
| ----------- | ------------------- | -------------------------------------------------------------------------------------------------------------------- | --------- | -------------------------------------------------------------------------------------------------------------------------- |
| COMP-01     | 08-01, 08-02        | CLI has `compare` subcommand accepting ≥ 2 URLs                                                                      | SATISFIED | `src/main.rs:47-56` Commands::Compare with num_args=2..; `test_compare_requires_two_urls` passes                            |
| COMP-02     | 08-01, 08-02        | `compare` reuses `analyze()` per URL sequentially, sharing client + browsers                                          | SATISFIED | `src/compare.rs:101-113` sequential loop; `src/main.rs:199-262` shared resources constructed once                           |
| COMP-03     | 08-01, 08-02, 08-03 | Existing flags work under `compare` with correct semantics                                                            | SATISFIED | `src/main.rs:22-40` all 5 flags have `global = true`; `test_compare_max_pages_per_target`, `test_compare_fail_under_first_url`, `test_compare_beauty_prints_table` pass |
| COMP-04     | 08-01, 08-02        | JSON output: stable schema with 6 keys                                                                                | SATISFIED | `src/compare.rs:28-36` CompareReport derives Serialize; `test_compare_json_schema_stable` passes + runtime smoke confirms all 6 keys |
| COMP-05     | 08-01, 08-02        | Per-category winner detection with 0.1 tie epsilon                                                                    | SATISFIED | `src/compare.rs:21, 130-166` TIE_EPSILON + compute_winners; 4 winner tests pass                                             |
| COMP-06     | 08-01, 08-02        | Per-check diff table                                                                                                  | SATISFIED | `src/compare.rs:171-232`; `test_check_diff_unique_checks`, `test_check_diff_missing_null`, `test_aggregate_check_status`, `test_compare_check_diff_populated` pass |
| COMP-07     | 08-01, 08-03        | Beauty mode: side-by-side colored table, 2-10 sites, no new deps                                                      | SATISFIED | `src/beauty.rs:144-229` print_beauty_compare_report; Cargo.toml unchanged; `test_compare_beauty_variable_columns` (2/3/5/10), `test_narrow_terminal_fallback`, `test_compare_beauty_prints_table` pass |
| COMP-08     | 08-01, 08-02        | `--fail-under` applies to first URL only                                                                              | SATISFIED | `src/main.rs:289-303` uses `first_url`; `test_compare_fail_under_first_url` + `test_compare_competitor_low_score_ignored` pass |
| COMP-09     | 08-01, 08-02        | Per-URL failures non-fatal; errors surface in errors[]; exit 2 only on first URL                                      | SATISFIED | `src/compare.rs:105-111`; `src/main.rs:283-287`; `test_compare_continues_on_per_url_error` + `test_compare_first_url_failure_exit_2` pass |
| COMP-10     | 08-01, 08-02        | Duplicate URL handling via normalize_url + stderr warning                                                             | SATISFIED | `src/compare.rs:236-248`; `test_dedup_uses_normalize_url` + `test_compare_dedupes_duplicate_urls` pass + runtime smoke confirms stderr warning |

All 10 COMP-XX requirements marked `[x]` (Complete) in REQUIREMENTS.md.
Traceability table still shows "Planned" status (stale row labels) — this is a minor documentation nit, not a correctness gap. The checkbox state `[x]` and all 10 traceability rows for Phase 8 exist at REQUIREMENTS.md lines 160-169.

### Anti-Patterns Found

| File             | Line | Pattern                                                           | Severity | Impact |
| ---------------- | ---- | ----------------------------------------------------------------- | -------- | ------ |
| — no matches —   | —    | Zero TODO/FIXME/XXX/HACK/PLACEHOLDER/todo!()/unimplemented! found | —        | —      |

Grep across `src/compare.rs`, `src/main.rs`, `src/beauty.rs` for `TODO|FIXME|XXX|HACK|PLACEHOLDER|todo!\(|unimplemented!|not yet implemented|Wave 1 placeholder|Wave 2 replaces` returned zero matches.

### Human Verification Required

None. All automated checks pass, including runtime smoke tests against real endpoints (example.com, example.org) and localhost unreachable URLs. The one ignored integration test (`test_vitals_flag_accepted`) is intentionally chromium-gated and unrelated to Phase 8 scope.

Optional manual UX check (not blocking):
- Visual inspection of `--beauty` output in a wide (≥120 col) terminal with TTY color support. The rendered table uses `colored` crate escapes; `cargo run -- compare --beauty <url1> <url2>` in an interactive terminal produces the side-by-side colored layout described in CONTEXT lines 190-213.

### Gaps Summary

No gaps. Phase 8 goal is fully achieved.

**Summary of verified contract:**
- Subcommand and flags: `geodaddy compare <url1> <url2>...` with `--enable-js`, `--vitals`, `--max-pages`, `--beauty`, `--fail-under` all honored under the subcommand.
- Sequential engine reuse: single shared `reqwest::Client` + optional `chromiumoxide::Browser` instances passed by reference through `compare_sites` → `analyze` per URL.
- Data model: `CompareReport { schema_version, compared_at, sites[], winners{overall,technical,content,geo,performance}, check_diff[], errors[] }` fully populated.
- Winner logic: TIE_EPSILON=0.1 absolute tolerance; `performance` category skips None-only sites; tie/empty → `None`.
- Check diff: alphabetical by check ID (BTreeMap); per-site aggregation rule any-Fail→Fail else any-Warn→Warn else Pass; absent→None.
- Exit-code policy: 0 on success, 1 if first-URL score below `--fail-under`, 2 if first URL in errors[] (competitor failures informational).
- Dedup: `crawling::normalize_url` canonicalization preserves first occurrence + `tracing::warn` to stderr.
- Beauty mode: side-by-side table when `required_width ≤ COLUMNS`, narrow-terminal fallback to vertical per-site reports otherwise.
- Zero new dependencies: Cargo.toml unchanged from the pre-Phase-8 stack.

**Test matrix:** 20 integration + 202 lib tests all green (plus 1 ignored chromium-gated test unrelated to Phase 8). Backward-compat sentinel `test_json_output_has_score_categories_pages` still passes, confirming the single-URL path `geodaddy <URL>` is unaffected.

---

_Verified: 2026-04-17T12:05:00Z_
_Verifier: Claude (gsd-verifier)_
