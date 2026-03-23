---
phase: 04-site-wide-crawling-polish
verified: 2026-03-23T18:06:39Z
status: human_needed
score: 10/10 automated must-haves verified
human_verification:
  - test: "Run `cargo run -- https://example.com 2>/dev/null | python3 -m json.tool | head -20` and confirm JSON contains top-level `score`, `categories`, and `pages` fields"
    expected: "Valid JSON with score (number), categories (object with technical/content/geo), and pages (array)"
    why_human: "Requires live network call and running binary against a real URL"
  - test: "Run `cargo run -- https://example.com 2>&1 >/dev/null` and check stderr output"
    expected: "Lines in format `[1/N] https://example.com` (sitemap) or `Crawling page 1... https://example.com` (link-following)"
    why_human: "Requires live network call; progress format correct only observable at runtime"
  - test: "Run `cargo run -- https://example.com 2>/dev/null | python3 -c \"import sys,json; json.load(sys.stdin); print('valid JSON')\"`"
    expected: "Prints 'valid JSON' — progress lines must NOT appear in stdout"
    why_human: "stdout purity requires running the binary against a live URL"
  - test: "Run `cargo run -- https://example.com --max-pages 2 2>/dev/null | python3 -c \"import sys,json; d=json.load(sys.stdin); print(len(d['pages']), 'pages')\"`"
    expected: "2 pages (or fewer if site has fewer pages)"
    why_human: "--max-pages behavior only verifiable with a live crawl"
  - test: "Run `cargo run -- https://example.com --fail-under 0; echo \"exit: $?\"`"
    expected: "exit: 0"
    why_human: "Exit code behavior requires a complete run against a real URL"
---

# Phase 04: Site-Wide Crawling Polish — Verification Report

**Phase Goal:** Multi-page crawling with politeness controls and sitemap-first strategy
**Verified:** 2026-03-23T18:06:39Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #  | Truth                                                                 | Status      | Evidence                                                                                          |
|----|-----------------------------------------------------------------------|-------------|---------------------------------------------------------------------------------------------------|
| 1  | fetch_sitemap_urls() parses sitemap.xml and returns priority-sorted URLs | VERIFIED    | Function present at crawling.rs:31; sorts by priority desc at line 57; test_fetch_sitemap_urls_parses_xml passes |
| 2  | collect_links_bfs() BFS-crawls same-origin links up to depth 2       | VERIFIED    | Function at crawling.rs:96; max_depth parameter enforced at line 121; same-origin filter via extract_same_origin_links |
| 3  | normalize_url() strips fragments and trailing slashes (preserves root /) | VERIFIED    | Function at crawling.rs:165; test_url_normalization, test_url_normalization_fragment, test_url_normalization_root all pass |
| 4  | extract_crawl_delay() parses crawl-delay from robots.txt body        | VERIFIED    | Function at crawling.rs:177; test_extract_crawl_delay_present returns Some(3); test_extract_crawl_delay_absent returns None |
| 5  | needs_js_rendering() returns true for HTML with <3 headings and 0 p elements | VERIFIED | Function at crawling.rs:191; test_js_detection_thin_page and test_js_detection_rich_page pass |
| 6  | All 15 required unit tests compile and pass                          | VERIFIED    | All 15 plan-specified test names present; cargo test: 74 passed, 0 failed (includes 3 bonus tests) |
| 7  | Report has top-level score/categories (aggregate across all pages)   | VERIFIED    | Report struct at main.rs:58-65 has score: f64 and categories: CategoryScores; populated from aggregate_scores() at line 268 |
| 8  | --max-pages flag caps sitemap and BFS crawls                         | VERIFIED    | Cli struct at main.rs:49; truncate applied at line 111; passed to collect_links_bfs at line 118 |
| 9  | --enable-js flag triggers headless re-fetch with Chromium warning    | VERIFIED    | Cli struct at main.rs:54; Chromium download warning in doc comment at line 52; Browser::launch path at lines 130-142; needs_js_rendering gate at line 202 |
| 10 | Progress to stderr; stdout is pure JSON                              | VERIFIED    | All eprintln! at lines 117, 163, 165; only println! is serde_json output at line 281 |

**Score:** 10/10 truths verified (automated)

### Required Artifacts

| Artifact       | Expected                                    | Status     | Details                                                                                        |
|----------------|---------------------------------------------|------------|-----------------------------------------------------------------------------------------------|
| `src/crawling.rs` | All 8 pub crawl-logic functions + 15 tests | VERIFIED   | 448 lines; all 8 functions present; 18 tests in cfg(test) block (15 required + 3 bonus)      |
| `src/main.rs`  | Full multi-page crawl orchestration         | VERIFIED   | 311 lines; mod crawling declared at line 3; crawl loop wired with all functions               |
| `Cargo.toml`   | chromiumoxide and futures dependencies      | VERIFIED   | chromiumoxide = { version = "0.9", features = ["fetcher", "zip8", "rustls"] }; futures = "0.3" |

### Key Link Verification

| From                                  | To                              | Via                             | Status    | Details                                                                  |
|---------------------------------------|---------------------------------|---------------------------------|-----------|-------------------------------------------------------------------------|
| main.rs crawl loop                    | crawling::fetch_sitemap_urls    | sitemap-first strategy          | WIRED     | Called at main.rs:107; result drives is_sitemap_driven flag             |
| main.rs crawl loop                    | crawling::collect_links_bfs     | fallback when sitemap None      | WIRED     | Called at main.rs:118 inside None arm of match                          |
| main.rs --enable-js path              | crawling::needs_js_rendering    | detection heuristic             | WIRED     | Called at main.rs:202 inside `if cli.enable_js` block                   |
| main.rs --fail-under                  | report.score (aggregate)        | aggregate_scores result         | WIRED     | agg_score compared at line 285; not pages[0].score                      |
| main.rs crawl loop                    | crawling::extract_crawl_delay   | robots.txt body parsed at start | WIRED     | Called at main.rs:102; result used in sleep() at line 250               |
| crawling.rs fetch_sitemap_urls        | quick-xml UrlSet deserialization | quick_xml::de::from_str         | WIRED     | from_str called at crawling.rs:49                                       |
| crawling.rs collect_links_bfs         | scraper a[href] CSS selector    | Selector::parse("a[href]")      | WIRED     | extract_same_origin_links uses "a[href]" selector at crawling.rs:82     |

### Data-Flow Trace (Level 4)

| Artifact    | Data Variable    | Source                           | Produces Real Data | Status   |
|-------------|------------------|----------------------------------|--------------------|----------|
| `main.rs`   | agg_score        | aggregate_scores(page_score_tuples) | Yes — computed from all pages' scores | FLOWING |
| `main.rs`   | pages            | crawl loop over urls Vec         | Yes — populated per fetched page | FLOWING |
| `main.rs`   | urls             | fetch_sitemap_urls / collect_links_bfs | Yes — real HTTP fetch (reqwest) | FLOWING |

### Behavioral Spot-Checks

| Behavior                            | Command                                                                  | Result          | Status  |
|-------------------------------------|--------------------------------------------------------------------------|-----------------|---------|
| Binary builds cleanly               | `cargo build`                                                             | 0 warnings, success | PASS |
| All 74 unit tests pass              | `cargo test`                                                              | 74 passed, 0 failed | PASS |
| --help shows Chromium warning       | `cargo run -- --help \| grep -i chromium`                                 | "downloads Chromium (~150MB) on first use" | PASS |
| --help shows --max-pages flag       | `cargo run -- --help \| grep max-pages`                                   | "--max-pages <N>" present | PASS |
| --help shows --enable-js flag       | `cargo run -- --help \| grep enable-js`                                   | "--enable-js" present | PASS |
| tokio::time::sleep used (not thread::sleep) | `grep "tokio::time::sleep" src/main.rs`                          | Found at line 14 and 250 | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description                                                              | Status          | Evidence                                                                                        |
|-------------|------------|--------------------------------------------------------------------------|-----------------|-------------------------------------------------------------------------------------------------|
| CRAWL-01    | 04-01       | CLI can crawl entire site starting from sitemap.xml                     | SATISFIED       | fetch_sitemap_urls in crawling.rs; wired in main.rs crawl loop at line 107                     |
| CRAWL-02    | 04-01       | CLI falls back to link-following if sitemap unavailable                 | SATISFIED       | collect_links_bfs in crawling.rs; wired at main.rs:118 in None fallback arm                   |
| CRAWL-04    | 04-02       | CLI has optional JavaScript rendering via headless browser flag         | SATISFIED       | --enable-js flag in Cli struct; chromiumoxide wired; needs_js_rendering gate present           |
| CLI-03      | 04-02       | CLI shows progress indicator during site crawl                          | SATISFIED       | eprintln! calls at main.rs:163,165 emit [N/TOTAL] or "Crawling page N..." format to stderr    |

**Note on orphaned requirements:** REQUIREMENTS.md traceability table maps CRAWL-05 to Phase 1 (not Phase 4). No Phase 4 requirements are orphaned — all 4 claimed requirement IDs (CRAWL-01, CRAWL-02, CRAWL-04, CLI-03) have implementation evidence.

### Anti-Patterns Found

| File              | Line | Pattern                      | Severity | Impact |
|-------------------|------|------------------------------|----------|--------|
| `src/crawling.rs` | —    | No anti-patterns found       | —        | —      |
| `src/main.rs`     | —    | No anti-patterns found       | —        | —      |

No TODOs, FIXMEs, placeholder returns, hardcoded empty arrays, or stub implementations found. All state variables (urls, pages, agg_score) are populated from real computation paths.

### Human Verification Required

All automated checks pass. The following behaviors require a running binary against a live network target to confirm:

#### 1. JSON Report Shape

**Test:** `cargo run -- https://example.com 2>/dev/null | python3 -m json.tool | head -20`
**Expected:** JSON with top-level `score` (number), `categories` (object: technical/content/geo), `url` (the base URL string), and `pages` (array)
**Why human:** Requires live network call and a complete crawl run

#### 2. Progress Format to Stderr

**Test:** `cargo run -- https://example.com 2>&1 >/dev/null`
**Expected:** Lines in format `[1/N] https://example.com/...` (sitemap-driven) or `Crawling page 1... https://example.com/...` (BFS fallback)
**Why human:** Progress format correctness only verifiable with real HTTP responses

#### 3. Stdout Purity (No Progress Contamination)

**Test:** `cargo run -- https://example.com 2>/dev/null | python3 -c "import sys,json; json.load(sys.stdin); print('valid JSON')"`
**Expected:** Prints "valid JSON" — progress lines absent from stdout
**Why human:** Requires a complete crawl run to confirm no stderr leaks to stdout

#### 4. --max-pages Cap

**Test:** `cargo run -- https://example.com --max-pages 2 2>/dev/null | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d['pages']), 'pages')"`
**Expected:** "2 pages" (or 1 if https://example.com has only 1 page in sitemap)
**Why human:** Cap only observable during a live multi-page crawl

#### 5. --fail-under Exit Code

**Test:** `cargo run -- https://example.com --fail-under 0; echo "exit: $?"`
**Expected:** "exit: 0" (score will never be below 0)
**Why human:** Exit code behavior requires a complete run

### Gaps Summary

No automated gaps found. All 10 must-have truths verified, all 3 artifacts substantive and wired, all 5 key links confirmed, all 4 requirement IDs satisfied. Phase goal "multi-page crawling with politeness controls and sitemap-first strategy" is implemented in the codebase.

The 5 human-verification items are behavioral integration checks (live network, runtime output format, exit codes) that cannot be verified through static analysis. They do not represent missing implementation — the code is present and wired. They represent final end-to-end confirmation of the assembled system.

---

_Verified: 2026-03-23T18:06:39Z_
_Verifier: Claude (gsd-verifier)_
