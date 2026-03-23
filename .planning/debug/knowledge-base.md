# GSD Debug Knowledge Base

Resolved debug sessions. Used by `gsd-debugger` to surface known-pattern hypotheses at the start of new investigations.

---

## cursor-com-pages-empty-and-sitemap-ux — sitemapindex causes pages:[] + no BFS-fallback notice on stderr
- **Date:** 2026-03-23
- **Error patterns:** pages empty, pages:[], sitemap, sitemapindex, sitemap-fallback, BFS fallback, link-following, is_html_url, xml urls filtered
- **Root cause:** cursor.com serves a sitemapindex at /sitemap.xml. quick_xml serde field-name-matches <sitemap><loc> children into UrlEntry structs, so fetch_sitemap_urls returned Some([child-sitemap.xml URLs]). Those all end in .xml, so the is_html_url pre-filter in main.rs removed every entry, leaving pages:[]. BFS fallback never triggered (Some was returned, not None). Separately, the BFS fallback notice used tracing::info! which only emits with RUST_LOG=info set.
- **Fix:** (1) Filter to HTML-only locs inside fetch_sitemap_urls; return None when filtered list is empty so BFS fallback triggers. (2) Change tracing::info! to eprintln! for the fallback notice so it always appears on stderr.
- **Files changed:** src/crawling.rs, src/main.rs
---

