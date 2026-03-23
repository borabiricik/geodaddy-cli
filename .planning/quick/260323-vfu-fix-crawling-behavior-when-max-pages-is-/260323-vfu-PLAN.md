---
phase: quick-260323-vfu
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src/main.rs
autonomous: true
requirements: []
must_haves:
  truths:
    - "geodaddy https://example.com with no --max-pages flag analyzes exactly 1 page (the given URL)"
    - "geodaddy https://example.com --max-pages 5 crawls up to 5 pages via sitemap or BFS as before"
  artifacts:
    - path: "src/main.rs"
      provides: "URL list determination logic gated on --max-pages presence"
  key_links:
    - from: "src/main.rs"
      to: "crawling::fetch_sitemap_urls / collect_links_bfs"
      via: "cli.max_pages.is_some() guard"
      pattern: "cli\\.max_pages"
---

<objective>
Fix crawling behavior so that omitting --max-pages causes geodaddy to analyze only the single given URL. Currently the crawler unconditionally attempts sitemap discovery and BFS link-following regardless of --max-pages.

Purpose: Match the documented contract — crawling is opt-in via --max-pages, not default behavior.
Output: Modified src/main.rs where the URL list is `vec![cli.url.clone()]` when `cli.max_pages` is None.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@src/main.rs
@src/crawling.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: Gate URL discovery behind --max-pages presence</name>
  <files>src/main.rs</files>
  <action>
In `main()`, replace the URL list determination block (lines ~118-132) with a three-branch match:

```rust
let (urls, is_sitemap_driven): (Vec<String>, bool) = if cli.max_pages.is_none() {
    // No --max-pages: single-URL mode — analyze only the given URL, no crawling
    (vec![cli.url.clone()], false)
} else {
    match fetch_sitemap_urls(&client, &base_url).await {
        Some(mut sitemap_urls) => {
            if let Some(max) = cli.max_pages {
                sitemap_urls.truncate(max);
            }
            (sitemap_urls, true)
        }
        None => {
            eprintln!("No sitemap.xml found — falling back to link-following (depth 2)");
            (collect_links_bfs(&client, &base_url, 2, cli.max_pages).await, false)
        }
    }
};
```

Key constraints:
- The `cli.max_pages.is_none()` early-return MUST be the outermost branch so neither `fetch_sitemap_urls` nor `collect_links_bfs` is ever called in single-URL mode.
- Use `cli.url.clone()` (the raw string from the user) as the sole URL, consistent with the existing `normalize_url` deduplication loop that follows.
- `is_sitemap_driven` stays `false` in single-URL mode — progress format will use `format_progress_unknown` (shows "Crawling page 1... URL"), which is acceptable for a single page.
- Remove the now-redundant `fetch_sitemap_urls` call path that happened unconditionally before.
- Also update the `--max-pages` arg help text in the `Cli` struct to clarify that omitting the flag disables crawling: change `"Stop crawling after N pages (applies to both sitemap and link-following crawls)."` to `"Enable crawling and stop after N pages. Without this flag, only the given URL is analyzed."`.
  </action>
  <verify>
    <automated>cd /Users/borabiricik/Desktop/Repos/hobby/geodaddy/cli && cargo test 2>&1 | tail -20</automated>
  </verify>
  <done>
- `cargo test` passes with no regressions.
- When run without --max-pages, the URL list in main is exactly `[cli.url]` and neither sitemap nor BFS is invoked.
- When run with --max-pages N, behavior is identical to before.
  </done>
</task>

</tasks>

<verification>
cargo test — all existing crawling tests pass.
Manual smoke check: cargo build --release produces a binary that compiles cleanly.
</verification>

<success_criteria>
- `cargo test` green.
- Single-URL mode is the default (no --max-pages).
- Crawling only activates when --max-pages is explicitly provided.
- --max-pages help text accurately describes opt-in crawling behavior.
</success_criteria>

<output>
After completion, create `.planning/quick/260323-vfu-fix-crawling-behavior-when-max-pages-is-/260323-vfu-SUMMARY.md`
</output>
