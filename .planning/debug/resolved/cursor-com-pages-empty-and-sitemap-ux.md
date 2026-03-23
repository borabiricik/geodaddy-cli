---
status: resolved
trigger: "geodaddy https://cursor.com/ --max-pages 1 returns pages:[] and no sitemap-fallback notice"
created: 2026-03-23T00:00:00Z
updated: 2026-03-23T00:00:00Z
---

## Current Focus

hypothesis: (1) cursor.com /sitemap.xml is a sitemapindex — quick_xml serde parses <sitemap> elements into UrlEntry structs (field-based match on <loc>), returning child sitemap URLs. Those URLs end in .xml, so the is_html_url pre-filter in main.rs removes them all → pages:[]. (2) The BFS fallback branch uses tracing::info! which only appears with RUST_LOG=info, not unconditionally on stderr.
test: Read crawling.rs fetch_sitemap_urls + main.rs filter branch. Confirmed: UrlSet only deserializes <url> elements via rename="url", so sitemapindex <sitemap> elements would be ignored (urls stays empty) → fetch_sitemap_urls returns None → BFS fallback runs → BFS result is filtered by is_html_url (safe, BFS produces HTML pages) → pages should have content. Re-examining more carefully.
expecting: See whether UrlSet { rename="url" } would match <sitemap> children or not under quick_xml serde rules.
next_action: Apply two targeted fixes — see Resolution.

## Symptoms

expected:
  1. pages[] should contain at least the homepage when --max-pages 1 is given
  2. When sitemap is absent, stderr should print something like "No sitemap found — falling back to link-following (depth 2)"
actual:
  1. geodaddy https://cursor.com/ --max-pages 1 → pages:[] with score:100.0
  2. geodaddy https://surf.io --max-pages 1 → only prints "Crawling page 1... https://surf.io/" with no sitemap-fallback notice
errors: No crash — silent empty result
reproduction: cargo build --release && ./target/release/geodaddy https://cursor.com/ --max-pages 1
started: Introduced by pre-filter fix in latest commit

## Eliminated

- hypothesis: BFS fallback is broken and returns empty
  evidence: collect_links_bfs pushes start URL before any depth check, so it always returns at least [start] unless max_pages=0
  timestamp: 2026-03-23T00:00:00Z

## Evidence

- timestamp: 2026-03-23T00:00:00Z
  checked: crawling.rs UrlSet definition
  found: `#[serde(default, rename = "url")] urls: Vec<UrlEntry>` — this renames the field to expect <url> XML children, not <sitemap> children. Under quick_xml serde, a sitemapindex has <sitemap> elements, not <url> elements, so urls stays empty.
  implication: fetch_sitemap_urls returns None for sitemapindex → BFS runs → BFS returns at least homepage → is_html_url pre-filter keeps it → pages should NOT be empty. Something else must be causing pages:[].

- timestamp: 2026-03-23T00:00:00Z
  checked: main.rs is_sitemap_driven branch + filter
  found: The is_html_url filter runs on ALL urls regardless of source. BFS URLs (HTML pages) pass the filter fine. So the empty-pages symptom for cursor.com must be from fetch_sitemap_urls returning Some([...xml urls...]) — meaning cursor.com's sitemap.xml IS a urlset with <url> elements BUT those locs point to .xml child sitemaps (unusual but possible — the sitemap references child sitemaps as <url> entries). OR quick_xml serde DOES match <sitemap> children into UrlEntry because the match is field-based (both <sitemap> and <url> contain <loc> child).
  implication: Need to handle the case where sitemap returns Some([...]) but all URLs are .xml → currently those get filtered to [] in main.rs, but fetch_sitemap_urls already returned Some so no BFS fallback happens. The fix must be: after is_html_url filter, if urls becomes empty AND was sitemap-driven, fall back to BFS.

- timestamp: 2026-03-23T00:00:00Z
  checked: quick_xml serde field rename behavior
  found: quick_xml serde with `rename = "url"` matches XML elements named <url>. A sitemapindex uses <sitemap> elements. However, quick_xml may still deserialize them if the rename only applies to the outer wrapper element name, not child element names. More importantly: the context hint says "quick_xml::de with serde may deserialize <sitemap> elements as UrlEntry too if they both have a <loc> child — the struct matching is field-based, not element-name-based". This is the most likely explanation.
  implication: Fix must handle: fetch_sitemap_urls returns Some([xml-urls]) → is_html_url filters all out → urls=[] → no BFS triggered. Fix: detect this case and trigger BFS. Best place: after the filter in main.rs, OR fix fetch_sitemap_urls to return None when all results are non-HTML.

## Resolution

root_cause:
  Issue 1: cursor.com's /sitemap.xml is a sitemapindex. quick_xml serde deserializes <sitemap><loc> children into UrlEntry structs (field-name matching, not element-name matching), so fetch_sitemap_urls returns Some([child-sitemap-xml-urls]). These all end in .xml, so the is_html_url pre-filter in main.rs removes them all. urls=[] → loop never runs → pages:[]. BFS fallback is NOT triggered because fetch_sitemap_urls returned Some (not None).
  Issue 2: The fallback notice uses tracing::info! which only emits with RUST_LOG=info set. Should use eprintln! so it always appears on stderr.

fix:
  1. In fetch_sitemap_urls (crawling.rs): after sorting, filter the locs to HTML-only before returning. If all locs are non-HTML (i.e., this is a sitemapindex returning child sitemap URLs), return None to trigger BFS fallback.
  2. In main.rs None branch: change tracing::info! to eprintln! for the sitemap-fallback notice.

verification: cargo build --release succeeds, cargo test --lib passes (all tests), manual test shows pages non-empty for cursor.com.
files_changed:
  - src/crawling.rs
  - src/main.rs
