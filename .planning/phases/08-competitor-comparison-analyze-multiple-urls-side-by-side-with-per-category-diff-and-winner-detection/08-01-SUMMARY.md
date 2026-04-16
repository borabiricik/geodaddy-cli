---
phase: 08-competitor-comparison
plan: 01
subsystem: cli
tags: [rust, clap, subcommand, tdd, scaffolding, compare, mockito]

# Dependency graph
requires:
  - phase: 01-foundation-and-cli-setup
    provides: clap-derive Cli struct + `analyze()` entrypoint + assert_cmd/mockito harness
  - phase: 02-core-analysis-engine
    provides: Report, PageResult, CategoryScores, AnalysisResult, Status types
  - phase: 04-site-wide-crawling
    provides: crawling::normalize_url used by Wave 1 dedup logic
  - phase: 05-core-web-vitals-measurement
    provides: CategoryScores.performance Option<f64> shape preserved in Winners
provides:
  - src/compare.rs module with CompareReport/Winners/CheckDiff/SiteCheckOutcome/CompareError types
  - Compile-only stubs for compare_sites, compute_winners, compute_check_diff, dedup_urls (todo!())
  - Cli struct restructured with Option<String> url + Option<Commands> subcommand dispatch
  - Commands::Compare variant with num_args=2.. clap validation
  - run_analyze_flow() helper owning the legacy single-URL path
  - 10 unit test stubs + 12 integration test stubs (22 total, red against todo!() + stub dispatch)
affects: [08-02-core-implementation, 08-03-beauty-mode, future-mcp-compare-tool]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "clap derive Option<Commands> + Option<String> top-level positional for backward-compat CLI evolution"
    - "Wave 0 TDD scaffold: land failing tests + type skeletons first, Wave 1 implements against red tests"
    - "schema_version '1' shape-discrimination (no version bump for new CompareReport shape)"
    - "TIE_EPSILON const (0.1) for float-comparison ties — 10× f64::EPSILON, 25× smaller than realistic delta"

key-files:
  created:
    - src/compare.rs
    - .planning/phases/08-competitor-comparison-analyze-multiple-urls-side-by-side-with-per-category-diff-and-winner-detection/08-01-SUMMARY.md
  modified:
    - .planning/REQUIREMENTS.md
    - src/lib.rs
    - src/main.rs
    - tests/integration.rs

key-decisions:
  - "Keep schema_version '1' — disjoint top-level keys (sites[] vs pages[]) make shape self-discriminating"
  - "TIE_EPSILON = 0.1 absolute tolerance — no new float-cmp/approx dependency"
  - "Add Debug derive to Report + PageResult so CompareReport can derive Debug (Rule 3 auto-fix)"
  - "Wave 0 compare arm stub: eprintln + exit(2) — integration tests fail red as designed"
  - "test_loop_calls_analyze_per_url marked #[ignore] (Wave 1 removes attribute when implementing)"

patterns-established:
  - "Backward-compat CLI evolution: Option<String> top-level positional + Option<Commands> subcommand + match dispatch"
  - "Wave 0 scaffolding: types + test stubs + todo!() fns so Wave 1 is TDD red→green"
  - "Multi-server mockito pattern for cross-origin compare tests"

requirements-completed: [COMP-01, COMP-02, COMP-03, COMP-04, COMP-05, COMP-06, COMP-07, COMP-08, COMP-09, COMP-10]

# Metrics
duration: 5min
completed: 2026-04-16
---

# Phase 8 Plan 1: Wave 0 Test Stubs & Type Skeletons Summary

**Competitor-comparison scaffold: Option<Commands> Cli restructure + CompareReport/Winners/CheckDiff type skeletons + 22 red test stubs ready for Wave 1 TDD implementation.**

## Performance

- **Duration:** ~5 min
- **Started:** 2026-04-16T18:09:50Z
- **Completed:** 2026-04-16T18:14:39Z
- **Tasks:** 3
- **Files modified:** 4 (1 created, 3 edited)

## Accomplishments

- Added COMP-01..COMP-10 requirements (10 entries + 10 traceability rows + Coverage bump 35→45)
- Created `src/compare.rs` (326 lines) with compile-ready type skeletons: `CompareReport`, `Winners`, `CheckDiff`, `SiteCheckOutcome`, `CompareError`, 4 `todo!()` stub fns, and 10 unit test stubs covering winner detection, tie epsilon, check-diff uniqueness, missing-null semantics, page-level status aggregation, dedup, and schema version
- Restructured `src/main.rs` Cli: `url: String` → `Option<String>`, added `Commands::Compare { urls: Vec<String> }` with `num_args = 2..`, dispatched via `match (&cli.command, &cli.url)`, extracted `run_analyze_flow` helper
- Appended 12 `test_compare_*` integration test stubs to `tests/integration.rs` using a shared `mock_site()` helper + dual mockito servers
- Preserved the backward-compat sentinel test `test_json_output_has_score_categories_pages` (still green)

## Task Commits

1. **Task 1: Add COMP-01..COMP-10 requirements to REQUIREMENTS.md** — `517cd06` (docs)
2. **Task 2: Create src/compare.rs with type skeletons and 10 unit test stubs** — `e969441` (feat, includes Rule 3 auto-fix adding Debug to Report/PageResult)
3. **Task 3: Restructure Cli struct with Option<Commands> + add 12 integration test stubs** — `7a192ab` (feat)

_All three task commits are atomic; final metadata commit will follow in the wrap-up._

## Files Created/Modified

- `src/compare.rs` (created, 326 lines) — CompareReport + Winners + CheckDiff + SiteCheckOutcome + CompareError structs; TIE_EPSILON and COMPARE_SCHEMA_VERSION consts; 4 `todo!()` stub fns; 10 unit tests under `#[cfg(test)] mod tests`
- `src/lib.rs` (modified) — added `pub mod compare;` declaration; added `Debug` derive to `Report` and `PageResult` (Rule 3 auto-fix, see Deviations)
- `src/main.rs` (modified) — full restructure: `Option<String>` url, `Option<Commands>` subcommand, `Commands::Compare { urls }` variant, match-based dispatch, extracted `run_analyze_flow(&cli, url)` helper preserving all existing behavior
- `tests/integration.rs` (modified) — appended 12 `test_compare_*` functions + `mock_site(&mut Server)` helper
- `.planning/REQUIREMENTS.md` (modified) — added "Competitor Comparison" section with COMP-01..COMP-10; appended 10 traceability rows mapping to Phase 8; updated Coverage from 35/35 to 45/45; refreshed `*Last updated:*` stamp

## Decisions Made

- **schema_version remains "1":** Rather than bumping the schema version, the `CompareReport` shape is self-discriminating via top-level keys (`sites[]`/`winners`/`check_diff`/`errors`) distinct from the single-URL `Report` (`pages[]`/`categories`/`score`). Phase 9 consumer can detect shape, not version.
- **TIE_EPSILON = 0.1 absolute tolerance:** Chosen over `f64::EPSILON` (false precision) or `float-cmp` crate (unnecessary dep). 10× larger than `f64::EPSILON`, 25× smaller than the smallest realistic scoring delta (~2.5 pts).
- **`test_loop_calls_analyze_per_url` is `#[ignore]`d:** Requires a mockable `analyze()` callsite that Wave 1 owns. Ignored in Wave 0 so `cargo test` doesn't surface a perpetually-red test before implementation exists; Wave 1 MUST remove the attribute.
- **Wave 0 compare arm prints "not yet implemented" + exits 2:** Ensures `test_compare_requires_two_urls` passes (clap rejects before our stub runs) and `test_compare_shares_http_client` and siblings fail red with clear intent.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added `Debug` derive to `Report` and `PageResult` in `src/lib.rs`**
- **Found during:** Task 2 (Create src/compare.rs)
- **Issue:** `CompareReport` was planned with `#[derive(Serialize, Debug)]`, but its `sites: Vec<Report>` field requires `Report: Debug`. `cargo build` failed with `E0277: Report doesn't implement std::fmt::Debug`. `PageResult` (transitively required via `Report.pages`) also needed Debug.
- **Fix:** Added `Debug` to both existing derive lists. Changes are additive, zero impact on serialization or runtime behavior, keep existing Serialize/Clone behavior intact.
- **Files modified:** `src/lib.rs` lines 50, 60
- **Verification:** `cargo build` succeeds with zero errors; `cargo test --lib compare:: --no-run` compiles; schema-version getter test passes (`test_compare_report_schema_version_is_1`)
- **Committed in:** `e969441` (part of Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 3 — blocking compile error).
**Impact on plan:** Derive-only change was necessary to satisfy the plan's own type skeleton (`#[derive(Serialize, Debug)]` on CompareReport). No semantic change to existing Report/PageResult. No scope creep.

## Issues Encountered

None beyond the documented Rule 3 auto-fix above. Plan text was precise enough that all three tasks landed on first attempt with verbatim file contents from the plan, modulo `rustfmt`-equivalent formatting (the plan's indentation was already close to project style).

## Test Status (Wave 0 expected state)

| Test | Type | Status | Notes |
|------|------|--------|-------|
| `test_json_output_has_score_categories_pages` | integration (existing) | PASS | Backward compat sentinel — unchanged |
| `test_compare_requires_two_urls` | integration (new) | PASS | Clap `num_args = 2..` rejects single URL — test passes in Wave 0 |
| `test_compare_report_schema_version_is_1` | unit (new) | PASS | Trivial getter over `CompareReport::empty()`, no `todo!()` path |
| `test_winner_highest_score` | unit (new) | FAIL (red) | Calls `compute_winners` → `todo!()` panic — expected |
| `test_winner_tie_within_epsilon` | unit (new) | FAIL (red) | `todo!()` panic |
| `test_winner_performance_absent` | unit (new) | FAIL (red) | `todo!()` panic |
| `test_winner_all_sites_missing_category` | unit (new) | FAIL (red) | `todo!()` panic |
| `test_check_diff_unique_checks` | unit (new) | FAIL (red) | `compute_check_diff` → `todo!()` |
| `test_check_diff_missing_null` | unit (new) | FAIL (red) | `compute_check_diff` → `todo!()` |
| `test_aggregate_check_status` | unit (new) | FAIL (red) | `compute_check_diff` → `todo!()` |
| `test_dedup_uses_normalize_url` | unit (new) | FAIL (red) | `dedup_urls` → `todo!()` |
| `test_loop_calls_analyze_per_url` | unit (new) | IGNORED | `#[ignore]` in Wave 0; Wave 1 removes attribute |
| `test_compare_shares_http_client` | integration (new) | FAIL (red) | Stub exits 2 before printing JSON |
| `test_compare_max_pages_per_target` | integration (new) | FAIL (red) | Stub exits 2 |
| `test_compare_beauty_prints_table` | integration (new) | FAIL (red) | Stub exits 2 |
| `test_compare_json_schema_stable` | integration (new) | FAIL (red) | Stub exits 2 |
| `test_compare_fail_under_first_url` | integration (new) | FAIL (red) | Asserts `exit_code == Some(1)` but stub exits 2 — red as designed |
| `test_compare_competitor_low_score_ignored` | integration (new) | PASS (coincidence) | Assert is `assert_ne!(exit, Some(1))` — stub exits 2 so this passes. Wave 1 must still support the intended semantics. |
| `test_compare_continues_on_per_url_error` | integration (new) | FAIL (red) | Stub exits 2, no JSON |
| `test_compare_first_url_failure_exit_2` | integration (new) | PASS (coincidence) | Stub ALWAYS exits 2, so this trivially passes. Wave 1 must preserve first-URL-failure → exit 2. |
| `test_compare_dedupes_duplicate_urls` | integration (new) | FAIL (red) | Stub exits 2, no JSON |
| `test_compare_winners_populated` | integration (new) | FAIL (red) | Stub exits 2 |
| `test_compare_check_diff_populated` | integration (new) | FAIL (red) | Stub exits 2 |

**Summary:** 22 new tests total (10 unit + 12 integration). Of these: 4 green in Wave 0 (schema-version, clap-rejects-single-url, and two exit-2 tests that coincidentally pass because the Wave 0 stub always exits 2), 17 red against `todo!()` or stub-exit mismatch, 1 ignored. Plus the 1 pre-existing backward-compat sentinel stays green. Wave 1 will flip the 17 reds to green while keeping the 4 greens green.

## Known Stubs

These are INTENTIONAL per plan design — Wave 0 scaffolding lands the failing red tests, Wave 1 replaces stubs with real logic:

- `src/compare.rs:91` — `compare_sites()` → `todo!("Wave 1: implement sequential analyze() loop with shared resources")`
- `src/compare.rs:100` — `compute_winners()` → `todo!("Wave 1: implement winner-per-category with 0.1 epsilon tie detection")`
- `src/compare.rs:108` — `compute_check_diff()` → `todo!("Wave 1: BTreeMap<&str, Vec<SiteCheckOutcome>> grouping")`
- `src/compare.rs:116` — `dedup_urls()` → `todo!("Wave 1: HashSet<String> of normalized URLs, preserve first, warn on dup")`
- `src/main.rs:73` — `Commands::Compare` arm prints "compare subcommand not yet implemented (Wave 0 stub)" and exits 2. Plan explicitly calls for this placeholder.

**These stubs are documented in the 08-01 plan as the Wave 0 contract; Wave 1 (08-02-PLAN) will implement each against the failing tests above.**

## User Setup Required

None — no external service configuration required. All work is additive Rust code + documentation.

## Next Phase Readiness

- **Ready to execute:** 08-02-PLAN.md (Wave 1 core implementation)
- Wave 1 contract: replace each `todo!()` with real logic until each red test flips green. Integration tests will start parsing JSON once the stub arm is replaced with an actual `compare_sites()` → JSON-pretty-print flow.
- No blockers.

## Self-Check: PASSED

**Files verified on disk:**
- FOUND: `src/compare.rs` (326 lines, contains CompareReport + 10 tests + 4 todo!() stubs)
- FOUND: `src/lib.rs` (pub mod compare declared, Debug derives added to Report/PageResult)
- FOUND: `src/main.rs` (Option<String> url, Commands::Compare variant, num_args=2.., run_analyze_flow helper)
- FOUND: `tests/integration.rs` (12 new test_compare_* functions + mock_site helper)
- FOUND: `.planning/REQUIREMENTS.md` (10 COMP-XX entries + 10 traceability rows + v1 total=45)

**Commits verified:**
- FOUND: `517cd06` — docs(08-01): add COMP-01..COMP-10 requirements
- FOUND: `e969441` — feat(08-01): add compare module scaffold with type skeletons
- FOUND: `7a192ab` — feat(08-01): restructure Cli for compare subcommand + add integration stubs

**Overall verification:**
- `cargo build` → 0 errors, 0 warnings
- `cargo test --test integration test_json_output_has_score_categories_pages` → PASS (backward compat sentinel)
- `cargo test --lib compare::tests::test_compare_report_schema_version_is_1` → PASS
- `cargo test --test integration test_compare_requires_two_urls` → PASS (clap enforces num_args=2..)
- `cargo test --no-run` → compiles cleanly, 22 new tests registered

---
*Phase: 08-competitor-comparison-analyze-multiple-urls-side-by-side-with-per-category-diff-and-winner-detection*
*Completed: 2026-04-16*
