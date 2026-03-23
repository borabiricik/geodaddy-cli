---
status: awaiting_human_verify
trigger: "progress-counter-skips-non-html"
created: 2026-03-23T00:00:00Z
updated: 2026-03-23T00:00:01Z
---

## Current Focus

hypothesis: enumerate() idx advances past skipped non-HTML URLs, causing progress counter to start at a higher number than 1
test: read src/main.rs crawl loop — confirmed
expecting: fix by pre-filtering URLs to HTML-only before the loop, recomputing total, and using a separate page_num counter
next_action: apply fix to src/main.rs

## Symptoms

expected: Progress should show [1/5], [2/5], [3/5]... counting only HTML pages that are actually analyzed
actual: Counter starts at [3/5] because idx from enumerate() already advanced past 2 skipped non-HTML URLs before the first real HTML page
errors: No crash — just misleading progress output to stderr
reproduction: ./target/release/geodaddy https://cursor.com --max-pages 5 — progress starts at [3/5] instead of [1/5]
started: Introduced together with the is_html_url() filter fix in commit ed5c8a1

## Eliminated

(none yet)

## Evidence

- timestamp: 2026-03-23T00:00:00Z
  checked: src/main.rs lines 142-162
  found: |
    Loop uses `for (idx, url_str) in urls.iter().enumerate()` then calls `continue`
    for non-HTML URLs. The `idx` value from enumerate() has already been incremented
    before the is_html_url() check, so format_progress_known(idx + 1, total, ...)
    reports wrong numbers. Also `total = urls.len()` at line 122 counts all URLs
    including non-HTML ones.
  implication: Both the numerator (idx+1) and denominator (total) are wrong when
    non-HTML URLs exist in the list.

## Resolution

root_cause: |
  In the sitemap-driven crawl loop (src/main.rs line 142), the loop index from
  enumerate() advances for every URL including those skipped by is_html_url().
  Progress is reported using this raw index. Additionally, `total` is set from
  urls.len() before filtering, so both numerator and denominator reflect raw
  sitemap URLs rather than actual HTML pages being analyzed.

fix: |
  1. Pre-filter urls to html_urls using is_html_url() before the loop.
  2. Recompute total = html_urls.len() from the filtered list.
  3. Use a separate page_num counter (incremented only when a URL is actually processed).
  4. Remove the per-URL is_html_url() check inside the loop (no longer needed).
  5. Fix the crawl-delay guard at loop end to use page_num instead of idx.

verification: "cargo build clean, all 73 tests pass"
files_changed: ["src/main.rs"]
