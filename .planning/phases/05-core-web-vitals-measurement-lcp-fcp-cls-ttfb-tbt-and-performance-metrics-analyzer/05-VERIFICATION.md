---
phase: 05-core-web-vitals-measurement-lcp-fcp-cls-ttfb-tbt-and-performance-metrics-analyzer
verified: 2026-03-23T00:00:00Z
status: human_needed
score: 13/14 must-haves verified
re_verification: false
human_verification:
  - test: "Run: ./target/release/geodaddy https://example.com --vitals"
    expected: "JSON output contains non-null categories.performance with perf-lcp, perf-fcp, perf-cls, perf-ttfb, perf-tbt result entries in pages[0].results"
    why_human: "Requires live Chromium browser download and execution — cannot be verified without a running browser and a reachable URL"
  - test: "Run: ./target/release/geodaddy https://example.com --vitals --enable-js"
    expected: "Process completes without panic or crash (two browser instances co-exist)"
    why_human: "Tests two concurrent Browser instances — requires actual Chromium launch"
  - test: "Add PERF-01 through PERF-08 to .planning/REQUIREMENTS.md"
    expected: "Requirements PERF-01 through PERF-08 defined in REQUIREMENTS.md traceability table mapped to Phase 5"
    why_human: "REQUIREMENTS.md does not contain PERF-* entries at all — these requirement IDs were invented in ROADMAP.md and PLANs but never written into the requirements registry. A human must decide whether to back-fill them or keep Phase 5 as a roadmap-only phase."
---

# Phase 5: Core Web Vitals Measurement Verification Report

**Phase Goal:** Add `--vitals` flag that measures Core Web Vitals (LCP, FCP, CLS, TTFB, TBT) via chromiumoxide headless browser per crawled page, surfacing results as scored AnalysisResult entries in a new `performance` scoring category with a 4-way overall average

**Verified:** 2026-03-23
**Status:** human_needed (13/14 automated checks pass; 1 behavioral item and 1 documentation gap need human action)
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|---------|
| 1 | CategoryScores has a `performance: Option<f64>` field that serializes as JSON null when None | VERIFIED | `src/scoring.rs` line 24: `pub performance: Option<f64>` — no `skip_serializing_if`, confirmed by `test_no_vitals_performance_null` integration test passing |
| 2 | `severity_points()` maps perf-lcp to 10 and perf-fcp/perf-cls/perf-ttfb/perf-tbt to 5 | VERIFIED | `src/scoring.rs` lines 41-42: `"perf-lcp" => 10` and `"perf-fcp" \| "perf-cls" \| "perf-ttfb" \| "perf-tbt" => 5` — confirmed by `test_perf_severity_points_lcp_is_10` and `test_perf_severity_points_others_are_5` |
| 3 | `calculate_score()` returns a 4-way average when perf checks are present and 3-way when absent | VERIFIED | `src/scoring.rs` lines 108-111: `match perf_score { Some(p) => (... / 4.0), None => (... / 3.0) }` — confirmed by `test_four_way_average_with_perf` and `test_three_way_average_without_perf` |
| 4 | `analyze_vitals(page)` returns exactly 5 AnalysisResult entries: perf-lcp, perf-fcp, perf-cls, perf-ttfb, perf-tbt | VERIFIED | `src/analyzers/performance.rs` lines 61-69: `vec![measure_lcp, measure_fcp, measure_cls, measure_ttfb, measure_tbt]` — 5 entries, correct check IDs |
| 5 | Each metric has pass/warn/fail thresholds matching Google's official CWV thresholds | VERIFIED | LCP: pass<=2.5s, warn<=4s, fail>4s; FCP: pass<=1.8s, warn<=3s; CLS: pass<=0.1, warn<=0.25; TTFB: pass<=800ms, warn<=1800ms; TBT: pass<=200ms, warn<=600ms — all boundary tests pass |
| 6 | LCP uses PerformanceObserver with `buffered: true` and 5-second setTimeout fallback | VERIFIED | `src/analyzers/performance.rs` line 14: `.observe({ type: 'largest-contentful-paint', buffered: true })` and line 15: `setTimeout(() => resolve(-1), 5000)` |
| 7 | CLS sums layout-shift entries where `hadRecentInput` is false | VERIFIED | `src/analyzers/performance.rs` lines 28-30: `if (!entry.hadRecentInput) { cls += entry.value; }` |
| 8 | TBT sums `Math.max(0, entry.duration - 50)` for longtask entries | VERIFIED | `src/analyzers/performance.rs` line 48: `tbt += Math.max(0, entry.duration - 50)` |
| 9 | `--vitals` flag is accepted by the CLI and launches a dedicated chromiumoxide browser instance | VERIFIED (partial — flag exists and wiring is correct; actual Chromium launch requires human test) | `src/main.rs` lines 57-60: `vitals: bool` field in Cli; lines 153-165: `vitals_browser` launch block; `test_vitals_flag_accepted` exists (marked `#[ignore]`) |
| 10 | When `--vitals` is active, `analyze_vitals` is called for every crawled page | VERIFIED | `src/main.rs` lines 263-275: `if cli.vitals { ... analyze_vitals(&vp).await ... results.extend(vitals_results) }` — complete wiring present |
| 11 | When `--vitals` is NOT passed, `CategoryScores.performance` is null in JSON output | VERIFIED | `test_no_vitals_performance_null` integration test PASSES — asserts both `categories.performance` and `pages[0].categories.performance` are JSON null |
| 12 | `--vitals` and `--enable-js` are independent and both can be combined without crash | VERIFIED (code path) / NEEDS HUMAN (runtime) | `src/main.rs`: two separate browser launch blocks — `browser` and `vitals_browser` — are independent; no shared state. Code path verified; runtime co-existence needs human test with actual Chromium |
| 13 | `aggregate_scores()` averages non-None performance values across pages; returns None when all None | VERIFIED | `src/crawling.rs` lines 219-227: `filter_map(|(_, c)| c.performance).collect()` — averages Some values, returns None for empty; `test_aggregate_score_performance_averages_some_values` and `test_aggregate_score_performance_none_when_all_none` both PASS |
| 14 | All cargo tests pass with no regressions | VERIFIED | `cargo test` output: 113 unit tests PASS, 8 integration tests PASS, 1 ignored (test_vitals_flag_accepted — by design) |

**Score:** 13/14 truths fully verified (truth #9 and #12 have human-dependent runtime components; code is correct)

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/scoring.rs` | CategoryScores with performance field, severity_points perf entries, calculate_score 4-way logic | VERIFIED | `performance: Option<f64>` at line 24; `"perf-lcp" => 10` at line 41; `perf_max` accumulator at lines 54-55; 4-way match at lines 108-111 |
| `src/analyzers/mod.rs` | performance module declaration | VERIFIED | Line 4: `pub mod performance;` |
| `src/analyzers/performance.rs` | `analyze_vitals` + 5 `classify_*` functions + JS constants + unit tests | VERIFIED | 541 lines — full implementation with all 5 JS constants, `eval_f64` helper, `analyze_vitals` entry point, 5 `measure_*` functions, 5 `pub(crate) classify_*` functions, 30 unit tests |
| `src/main.rs` | `vitals: bool` Cli field, `vitals_browser Option<Browser>`, `analyze_vitals` call in crawl loop | VERIFIED | Lines 57-60: vitals field; lines 153-165: vitals_browser launch; lines 263-275: per-page analyze_vitals call |
| `src/crawling.rs` | `aggregate_scores` updated to average `performance Option<f64>` | VERIFIED | Lines 219-227: performance averaging logic; `test_aggregate_score_performance_averages_some_values` and `test_aggregate_score_performance_none_when_all_none` pass |
| `tests/integration.rs` | `test_no_vitals_performance_null`, `test_vitals_flag_accepted` | VERIFIED | Both tests present — `test_no_vitals_performance_null` passes; `test_vitals_flag_accepted` is `#[ignore]` by design (requires Chromium) |

---

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| `src/scoring.rs CategoryScores` | all call sites constructing CategoryScores | struct initialization with `performance` field | VERIFIED | All 3 call sites updated: scoring.rs line 113, crawling.rs lines 207-212 and 229-235, main.rs lines 298-306 — all include `performance` field |
| `src/scoring.rs calculate_score` | `perf_max > 0` guard | conditional overall average divisor | VERIFIED | `src/scoring.rs` line 108: `match perf_score { Some(p) => ... / 4.0, None => ... / 3.0 }` |
| `src/main.rs Cli.vitals` | `vitals_browser Option<Browser>` | `if cli.vitals { Browser::launch() }` | VERIFIED | `src/main.rs` line 153: `let vitals_browser: Option<Browser> = if cli.vitals {` |
| `src/main.rs crawl loop` | `analyze_vitals(vp)` | `if cli.vitals { vb.new_page(url).await -> analyze_vitals(&vp) }` | VERIFIED | `src/main.rs` lines 263-275: complete guard + page creation + call |
| `src/crawling.rs aggregate_scores` | `performance: Option<f64>` | average of Some() values; None when all None | VERIFIED | `src/crawling.rs` lines 219-227 |
| `src/analyzers/performance.rs analyze_vitals` | `chromiumoxide::Page` | `page.evaluate()` with injected JavaScript strings | VERIFIED | `src/analyzers/performance.rs` line 103: `page.evaluate(js).await` in `eval_f64` helper |
| `measure_lcp` | LCP_JS constant | PerformanceObserver `largest-contentful-paint` `buffered: true` | VERIFIED | Line 14 in LCP_JS: `largest-contentful-paint, buffered: true` |

---

### Data-Flow Trace (Level 4)

`analyze_vitals` renders dynamic data from live CDP evaluation — not applicable for static data-flow trace. The data source is a running browser page, not a database or static store. CDP evaluation failures return -1.0 (handled by `eval_f64`), which maps to `Status::Fail` in each `classify_*` function. The chain is: `CLI flag -> vitals_browser launch -> page.evaluate(JS) -> f64 value -> classify_* -> AnalysisResult -> calculate_score -> CategoryScores.performance`. All links verified at the code level.

---

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| `cargo test` exits 0 | `cargo test` | 113 unit + 8 integration = 121 PASS, 1 ignored | PASS |
| `--vitals` flag accepted by CLI | integration test `test_vitals_flag_accepted` | `#[ignore]` — requires Chromium | SKIP (needs human) |
| `performance: null` when no `--vitals` | integration test `test_no_vitals_performance_null` | PASS | PASS |
| `classify_lcp(2500.0)` returns Pass (boundary) | unit test `classify_lcp_pass_at_boundary` | PASS | PASS |
| `classify_tbt(0.0)` returns Pass (zero TBT) | unit test `classify_tbt_pass_at_zero` | PASS | PASS |
| `classify_cls(0.26)` returns Fail (above threshold) | unit test `classify_cls_fail_above_warn_threshold` | PASS | PASS |

---

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|---------|
| PERF-01 | 05-02, 05-03 | (Not defined in REQUIREMENTS.md) | ORPHANED | ID referenced in ROADMAP.md and PLANs but absent from .planning/REQUIREMENTS.md — content inferred as: LCP measurement via chromiumoxide CDP |
| PERF-02 | 05-02 | (Not defined in REQUIREMENTS.md) | ORPHANED | ID referenced in ROADMAP.md and PLANs but absent from .planning/REQUIREMENTS.md — content inferred as: FCP measurement |
| PERF-03 | 05-02, 05-03 | (Not defined in REQUIREMENTS.md) | ORPHANED | ID referenced in ROADMAP.md and PLANs but absent from .planning/REQUIREMENTS.md — content inferred as: CLS measurement |
| PERF-04 | 05-02 | (Not defined in REQUIREMENTS.md) | ORPHANED | ID referenced in ROADMAP.md and PLANs but absent from .planning/REQUIREMENTS.md — content inferred as: TTFB measurement |
| PERF-05 | 05-02 | (Not defined in REQUIREMENTS.md) | ORPHANED | ID referenced in ROADMAP.md and PLANs but absent from .planning/REQUIREMENTS.md — content inferred as: TBT measurement |
| PERF-06 | 05-02 | (Not defined in REQUIREMENTS.md) | ORPHANED | ID referenced in ROADMAP.md and PLANs but absent from .planning/REQUIREMENTS.md — content inferred as: Google CWV threshold classification |
| PERF-07 | 05-01 | (Not defined in REQUIREMENTS.md) | ORPHANED | ID referenced in ROADMAP.md and PLANs but absent from .planning/REQUIREMENTS.md — content inferred as: performance scoring category |
| PERF-08 | 05-01, 05-03 | (Not defined in REQUIREMENTS.md) | ORPHANED | ID referenced in ROADMAP.md and PLANs but absent from .planning/REQUIREMENTS.md — content inferred as: --vitals CLI flag |

**Note:** All 8 PERF requirement IDs are ORPHANED — they exist in ROADMAP.md and PLAN frontmatter but have no corresponding entries in `.planning/REQUIREMENTS.md`. The implementation itself is complete and correct; the traceability registry is out of sync. This is a documentation gap, not an implementation gap. The functionality that PERF-01 through PERF-08 presumably describe is fully implemented and tested.

---

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| None found | — | — | — | — |

Scanned `src/scoring.rs`, `src/analyzers/performance.rs`, `src/main.rs`, `src/crawling.rs`, `tests/integration.rs`. No TODO/FIXME/placeholder comments, no empty implementations in production paths, no hardcoded empty return values in final code. The Plan 01 stub (`vec![]` in performance.rs) was correctly replaced by Plan 02.

---

### Human Verification Required

#### 1. Live --vitals Measurement End-to-End

**Test:** Build a release binary (`cargo build --release`) and run `./target/release/geodaddy https://example.com --vitals`

**Expected:** JSON output contains non-null `categories.performance` (a number between 0-100) and `pages[0].results` contains 5 entries with check IDs `perf-lcp`, `perf-fcp`, `perf-cls`, `perf-ttfb`, `perf-tbt`, each with a `status` of `pass`, `warn`, or `fail` and a message containing the measured value.

**Why human:** Requires Chromium binary download (~150MB on first run) and a live reachable HTTPS URL. Cannot be verified in CI without a browser-capable environment.

#### 2. --vitals Combined with --enable-js

**Test:** Run `./target/release/geodaddy https://example.com --vitals --enable-js`

**Expected:** Process completes without panic or crash. Both browser instances launch independently. JSON output is valid.

**Why human:** Two concurrent `Browser` instances from chromiumoxide. Code review confirms no shared state between the two browser variables, but actual runtime behavior needs a live Chromium binary.

#### 3. REQUIREMENTS.md Back-fill

**Test:** Review whether PERF-01 through PERF-08 should be formally defined in `.planning/REQUIREMENTS.md`

**Expected:** Either (a) add 8 PERF-* requirement definitions and add them to the traceability table mapped to Phase 5, or (b) document a decision that Phase 5 is a roadmap-only phase with no formal requirement IDs.

**Why human:** Policy decision — the codebase is correct. The question is whether the requirements registry should be updated to match what was built.

---

### Gaps Summary

No implementation gaps. The phase goal is achieved:

- `--vitals` flag exists in the CLI and is wired end-to-end
- All 5 CWV metrics (LCP, FCP, CLS, TTFB, TBT) are implemented with correct Google thresholds
- Scoring infrastructure correctly extends to a 4-way average when performance data is present
- `performance: null` serialization is correct when `--vitals` is absent
- `aggregate_scores` handles optional performance averaging correctly
- 121 tests pass, 0 failures, 0 regressions

The only open item is a **documentation gap**: PERF-01 through PERF-08 are referenced in ROADMAP.md and all PLAN frontmatter but are not defined in `.planning/REQUIREMENTS.md`. This does not block goal achievement but should be resolved for traceability consistency.

---

_Verified: 2026-03-23_
_Verifier: Claude (gsd-verifier)_
