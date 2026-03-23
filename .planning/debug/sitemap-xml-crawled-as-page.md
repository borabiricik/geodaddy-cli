---
status: awaiting_human_verify
trigger: "sitemap-xml-crawled-as-page"
created: 2026-03-23T00:00:00Z
updated: 2026-03-23T01:00:00Z
---

## Current Focus

hypothesis: Sitemap index files (containing <sitemap><loc>*.xml</loc></sitemap>) parse partially via UrlSet deserialization, causing either (A) xml child sitemap URLs to slip into the URL list, or (B) the empty UrlSet causes None return and BFS fallback discovers non-HTML URLs. Additionally, even well-formed sitemaps may list .xml, .pdf, or other non-HTML locs directly. The crawl loop in main.rs has no filter to exclude non-HTML URLs before running analyzers.
test: Read src/main.rs crawl loop — verify there is no is_html_url guard before the analyzer block
expecting: No such guard exists — confirmed by reading the code
next_action: Await human verification that sitemap.xml no longer appears in pages[] output

## Symptoms

expected: sitemap.xml should only be used as a URL discovery mechanism — its URL should NOT appear in the pages[] array of the JSON report, and no analyzers should run on it
actual: sitemap.xml (and possibly other non-HTML files like robots.txt) appears in the crawl queue and gets analyzed as a regular page
errors: no crash — just wrong behavior producing garbage analysis results for non-HTML URLs
reproduction: Run `cargo run -- https://example.com 2>/dev/null` — check if pages[] contains sitemap.xml or other .xml/.txt URLs
started: Introduced in Phase 04 plan 02 (04-02) — rewrite of src/main.rs with multi-page crawl loop

## Eliminated

- hypothesis: BFS link-following adds sitemap.xml via <a href> links
  evidence: extract_same_origin_links only follows <a[href]> elements, not Sitemap: directives in robots.txt
  timestamp: 2026-03-23T00:00:00Z

## Evidence

- timestamp: 2026-03-23T00:00:00Z
  checked: src/crawling.rs fetch_sitemap_urls
  found: Parses /sitemap.xml as UrlSet via quick_xml::de::from_str. If sitemap is a sitemapindex (uses <sitemap> not <url>), deserialization yields empty UrlSet.urls → returns None → BFS fallback runs. If sitemap is a urlset with .xml child sitemap locs listed, those .xml URLs pass through directly into the returned Vec<String>.
  implication: Both paths can introduce non-HTML URLs

- timestamp: 2026-03-23T00:00:00Z
  checked: src/main.rs crawl loop lines 142-241
  found: Zero filtering of URL type before running analyzers. Every URL in the `urls` Vec goes through all 15 analyzers with no is_html_url check.
  implication: This is the confirmed gap — non-HTML URLs enter the analysis pipeline

- timestamp: 2026-03-23T00:00:00Z
  checked: src/crawling.rs fetch_sitemap_urls return value
  found: Returns Some(url_set.urls.into_iter().map(|e| e.loc).collect()) — no extension filtering on loc values
  implication: A sitemap containing <loc>https://example.com/sitemap-posts.xml</loc> or <loc>https://example.com/feed.rss</loc> will include those non-HTML URLs in the crawl list

## Resolution

root_cause: The crawl loop in src/main.rs has no filter to exclude non-HTML URLs before running GEO analyzers. When sitemaps list child sitemap .xml files as <url><loc> entries (valid but rare), or when BFS follows links to non-HTML content, those URLs enter the analyzer pipeline and produce garbage results.

fix: Add a pub fn is_html_url(url: &str) -> bool helper in crawling.rs that returns false for URLs whose path ends with known non-HTML extensions (.xml, .rss, .atom, .json, .pdf, .txt, .png, .jpg, .jpeg, .gif, .svg, .webp, .css, .js, .ico, .woff, .woff2, .ttf, .eot, .zip, .gz). Apply this filter in the main.rs crawl loop immediately after deduplication — skip and continue if !is_html_url(&norm).

verification: cargo build succeeded (3.07s clean compile). All 73 tests pass including new test_is_html_url_allows_html_pages and test_is_html_url_filters_non_html.
files_changed: [src/crawling.rs, src/main.rs]
