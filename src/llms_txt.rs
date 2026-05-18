//! `geodaddy llms-txt <url>` — crawl a site and emit a spec-compliant
//! llms.txt index (per https://llmstxt.org).
//!
//! Reuses the existing crawler (sitemap-first, BFS fallback) from
//! src/crawling.rs. Output is plain markdown — no JSON, no `--beauty`
//! variant — because llms.txt IS the output format.

use anyhow::{Context, Result};
use chromiumoxide::Browser;
use scraper::{Html, Selector};
use std::collections::BTreeMap;
use std::path::PathBuf;
use url::Url;

use crate::crawling::{
    collect_links_bfs_with_browser, fetch_html_maybe_rendered, fetch_sitemap_urls, is_html_url,
    normalize_url,
};

/// Default cap on pages crawled for llms.txt generation. Chosen so the
/// resulting file stays under ~50KB even on link-heavy sites — within
/// the size most LLM context windows can comfortably ingest.
pub const DEFAULT_MAX_PAGES: usize = 50;

/// One row in the final llms.txt — a crawled page reduced to its
/// LLM-relevant fields.
#[derive(Debug, Clone)]
struct PageEntry {
    url: String,
    title: String,
    description: Option<String>,
}

/// Whole-site metadata extracted from the root URL's HTML, used to
/// build the H1 + blockquote at the top of llms.txt.
#[derive(Debug, Default, Clone)]
struct SiteMeta {
    name: String,
    description: Option<String>,
}

/// CLI entry: crawl the URL, build the index, emit to stdout or
/// write to `output_path` if provided.
pub async fn run_llms_txt(
    url: &str,
    output_path: Option<&PathBuf>,
    max_pages: usize,
    client: &reqwest::Client,
    browser: Option<&Browser>,
) -> Result<()> {
    let body = produce_llms_txt(url, max_pages, client, browser).await?;
    match output_path {
        Some(p) => std::fs::write(p, &body)
            .with_context(|| format!("writing llms.txt to {}", p.display()))?,
        None => print!("{}", body),
    }
    Ok(())
}

/// Library-facing: crawl + format. Returns the llms.txt body as a
/// String (no I/O to stdout). Used by tests and by `run_llms_txt`.
///
/// When `browser` is `Some(...)` the crawler runs every HTML fetch
/// through Chromium so client-side-rendered SPAs (Next.js App Router,
/// Vite/React/Vue/Svelte SPAs) yield real navigation links instead of
/// the empty pre-hydration HTML. Sitemap XML still uses plain HTTP —
/// XML doesn't need a browser and forcing it through one only adds
/// latency.
pub async fn produce_llms_txt(
    url: &str,
    max_pages: usize,
    client: &reqwest::Client,
    browser: Option<&Browser>,
) -> Result<String> {
    let base = Url::parse(url)
        .map_err(|e| anyhow::anyhow!("Invalid URL '{}': {}", url, e))?;

    // 1. Site-wide meta from the root page's HTML — JS-render when
    //    the caller asked for it so SPAs surface a real <title>.
    let root_html_body =
        fetch_html_maybe_rendered(client, browser, base.as_str()).await;
    let root_doc = Html::parse_document(&root_html_body);
    let site_meta = extract_site_meta(&root_doc, &base);

    // 2. URL list — sitemap-first (plain HTTP is fine for XML),
    //    BFS fallback honours the browser flag.
    let urls: Vec<String> = match fetch_sitemap_urls(client, &base).await {
        Some(mut s) => {
            s.truncate(max_pages);
            s
        }
        None => {
            tracing::info!("No sitemap.xml found — falling back to link-following");
            collect_links_bfs_with_browser(client, browser, &base, 2, Some(max_pages)).await
        }
    };
    let urls: Vec<String> = urls.into_iter().filter(|u| is_html_url(u)).collect();

    // 3. Per-page extraction (reuse root_doc for the root URL — saves one fetch)
    let mut entries: Vec<PageEntry> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let root_norm = normalize_url(base.as_str()).unwrap_or_default();
    for u in &urls {
        let norm = normalize_url(u).unwrap_or_else(|| u.clone());
        if !seen.insert(norm.clone()) {
            continue;
        }
        if norm == root_norm {
            if let Some(e) = extract_page_entry(&root_doc, &norm) {
                entries.push(e);
            }
        } else {
            let b = fetch_html_maybe_rendered(client, browser, &norm).await;
            if b.is_empty() {
                continue;
            }
            let doc = Html::parse_document(&b);
            if let Some(e) = extract_page_entry(&doc, &norm) {
                entries.push(e);
            }
        }
    }

    // 4. Always include the root URL even if no sitemap/BFS pages found.
    let only_root_fallback = entries.is_empty();
    if only_root_fallback {
        if let Some(e) = extract_page_entry(&root_doc, base.as_str()) {
            entries.push(e);
        }
    }

    // 5. Surface a warning when discovery effectively failed — i.e. the
    //    crawler only managed to include the root page itself. Two cases:
    //    (a) entries was already empty before (4), so the fallback added
    //        one — typically means HTTP 403 / bot block on every fetch.
    //    (b) sitemap returned exactly one entry that points back at the
    //        root, leaving us with no additional pages to index.
    //    Either way, the output is too thin to be useful — appending an
    //    HTML comment lets users (and the UI) detect the situation
    //    without breaking the spec-compliant markdown body.
    let warn_low_discovery = entries.len() <= 1;

    let body = format_llms_txt(&site_meta, &entries);
    if warn_low_discovery {
        Ok(append_low_discovery_warning(body))
    } else {
        Ok(body)
    }
}

/// Appends a sentinel HTML comment to the body. Markdown viewers ignore
/// it; humans editing the file see the explanation; the UI / clients can
/// pattern-match the sentinel to render an inline banner.
fn append_low_discovery_warning(mut body: String) -> String {
    if !body.ends_with('\n') {
        body.push('\n');
    }
    body.push_str(LOW_DISCOVERY_WARNING);
    body.push('\n');
    body
}

/// Sentinel string the frontend can grep for. Keep stable across releases.
pub const LOW_DISCOVERY_WARNING: &str =
    "<!-- geodaddy:warning low-discovery — the crawler reached only the homepage. \
The site likely blocks automated requests (bot detection / IP filtering), \
or its sitemap is unreachable. Manual editing recommended. -->";

fn extract_site_meta(doc: &Html, base: &Url) -> SiteMeta {
    let og_site = select_attr(doc, r#"meta[property="og:site_name"]"#, "content");
    let title = select_text(doc, "title");
    let name = og_site
        .or(title)
        .unwrap_or_else(|| base.host_str().unwrap_or("Site").to_string());
    let description = select_attr(doc, r#"meta[name="description"]"#, "content");
    SiteMeta { name, description }
}

fn extract_page_entry(doc: &Html, url: &str) -> Option<PageEntry> {
    // Title precedence: <title> → first <h1> → URL path tail.
    let title = select_text(doc, "title")
        .or_else(|| select_text(doc, "h1"))
        .unwrap_or_else(|| {
            Url::parse(url)
                .ok()
                .and_then(|u| {
                    u.path_segments()
                        .and_then(|s| s.last().map(String::from))
                })
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| url.to_string())
        });
    let description = select_attr(doc, r#"meta[name="description"]"#, "content");
    Some(PageEntry {
        url: url.to_string(),
        title,
        description,
    })
}

fn select_text(doc: &Html, sel: &str) -> Option<String> {
    let s = Selector::parse(sel).ok()?;
    doc.select(&s)
        .next()
        .map(|el| {
            el.text()
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|t| !t.is_empty())
}

fn select_attr(doc: &Html, sel: &str, attr: &str) -> Option<String> {
    let s = Selector::parse(sel).ok()?;
    doc.select(&s)
        .next()
        .and_then(|el| el.value().attr(attr).map(|v| v.trim().to_string()))
        .filter(|s| !s.is_empty())
}

fn format_llms_txt(site: &SiteMeta, entries: &[PageEntry]) -> String {
    let mut out = String::new();
    out.push_str(&format!("# {}\n\n", site.name));
    if let Some(d) = &site.description {
        out.push_str(&format!("> {}\n\n", d));
    }

    // Group entries by first path segment when 2+ entries share a prefix.
    let groups = group_by_prefix(entries);
    for (section, items) in &groups {
        out.push_str(&format!("## {}\n\n", section));
        for e in items {
            if let Some(d) = &e.description {
                out.push_str(&format!("- [{}]({}): {}\n", e.title, e.url, d));
            } else {
                out.push_str(&format!("- [{}]({})\n", e.title, e.url));
            }
        }
        out.push('\n');
    }
    out
}

/// Returns groups in stable insertion order: named prefix groups
/// (alphabetical) first, then a `Pages` group containing leftovers.
fn group_by_prefix(entries: &[PageEntry]) -> Vec<(String, Vec<PageEntry>)> {
    let mut by_prefix: BTreeMap<String, Vec<PageEntry>> = BTreeMap::new();
    let mut leftovers: Vec<PageEntry> = Vec::new();
    // Pass 1: collect candidate prefixes (count occurrences of each first segment).
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for e in entries {
        if let Some(seg) = first_segment(&e.url) {
            *counts.entry(seg).or_insert(0) += 1;
        }
    }
    // Pass 2: assign each entry to its named group iff that prefix has >=2 entries.
    for e in entries {
        match first_segment(&e.url) {
            Some(seg) if counts.get(&seg).copied().unwrap_or(0) >= 2 => {
                by_prefix
                    .entry(title_case_segment(&seg))
                    .or_default()
                    .push(e.clone());
            }
            _ => leftovers.push(e.clone()),
        }
    }
    let mut out: Vec<(String, Vec<PageEntry>)> = by_prefix.into_iter().collect();
    if !leftovers.is_empty() || out.is_empty() {
        out.push(("Pages".to_string(), leftovers));
    }
    out
}

fn first_segment(url: &str) -> Option<String> {
    let u = Url::parse(url).ok()?;
    let mut segs = u.path_segments()?;
    let first = segs.next()?;
    if first.is_empty() {
        return None;
    }
    Some(first.to_lowercase())
}

fn title_case_segment(s: &str) -> String {
    s.split('-')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(url: &str, title: &str, description: Option<&str>) -> PageEntry {
        PageEntry {
            url: url.to_string(),
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_format_renders_h1_from_site_name() {
        let site = SiteMeta {
            name: "Acme Docs".to_string(),
            description: None,
        };
        let entries = vec![entry("http://example.com/", "Home", None)];
        let out = format_llms_txt(&site, &entries);
        assert!(
            out.starts_with("# Acme Docs\n"),
            "expected H1 with site name at start, got: {:?}",
            &out[..out.len().min(40)]
        );
    }

    #[test]
    fn test_format_includes_blockquote_when_description_present() {
        let site = SiteMeta {
            name: "Acme".to_string(),
            description: Some("The Acme documentation site.".to_string()),
        };
        let entries = vec![entry("http://example.com/", "Home", None)];
        let out = format_llms_txt(&site, &entries);
        // First non-empty line is "# Acme"; second non-empty line should be the blockquote.
        let non_empty: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();
        assert_eq!(non_empty[0], "# Acme");
        assert_eq!(non_empty[1], "> The Acme documentation site.");
    }

    #[test]
    fn test_format_omits_blockquote_when_no_description() {
        let site = SiteMeta {
            name: "Acme".to_string(),
            description: None,
        };
        let entries = vec![entry("http://example.com/", "Home", None)];
        let out = format_llms_txt(&site, &entries);
        assert!(
            !out.contains("\n> "),
            "expected no blockquote line, got:\n{}",
            out
        );
    }

    #[test]
    fn test_format_default_section_is_pages() {
        // All URLs share the root path (no clear sub-prefix) → single ## Pages section.
        let site = SiteMeta {
            name: "Acme".to_string(),
            description: None,
        };
        let entries = vec![
            entry("http://example.com/", "Home", None),
            entry("http://example.com/about", "About", None),
        ];
        let out = format_llms_txt(&site, &entries);
        // Note: /about has first-segment "about" (count=1) → falls back to Pages.
        // / has no first-segment → falls back to Pages.
        let h2_lines: Vec<&str> = out.lines().filter(|l| l.starts_with("## ")).collect();
        assert_eq!(
            h2_lines,
            vec!["## Pages"],
            "expected exactly one ## Pages section, got: {:?}",
            h2_lines
        );
    }

    #[test]
    fn test_format_groups_by_path_prefix() {
        let site = SiteMeta {
            name: "Acme".to_string(),
            description: None,
        };
        let entries = vec![
            entry("http://example.com/docs/intro", "Intro", None),
            entry("http://example.com/docs/api", "API", None),
            entry("http://example.com/blog/post-1", "Post 1", None),
            entry("http://example.com/blog/post-2", "Post 2", None),
        ];
        let out = format_llms_txt(&site, &entries);
        let h2_lines: Vec<&str> = out.lines().filter(|l| l.starts_with("## ")).collect();
        // Both prefixes have >= 2 entries → both get their own section.
        // BTreeMap ordering: Blog before Docs (alphabetical).
        assert!(
            h2_lines.contains(&"## Blog"),
            "expected ## Blog section, got: {:?}",
            h2_lines
        );
        assert!(
            h2_lines.contains(&"## Docs"),
            "expected ## Docs section, got: {:?}",
            h2_lines
        );

        // Docs section contains only docs links.
        let docs_section_idx = out.find("## Docs\n").expect("Docs section");
        let after_docs = &out[docs_section_idx..];
        let next_h2 = after_docs[8..].find("## ").map(|i| i + 8 + docs_section_idx);
        let docs_block = match next_h2 {
            Some(end) => &out[docs_section_idx..end],
            None => &out[docs_section_idx..],
        };
        assert!(docs_block.contains("/docs/intro"));
        assert!(docs_block.contains("/docs/api"));
        assert!(!docs_block.contains("/blog/"));
    }

    #[test]
    fn test_format_link_lines() {
        let site = SiteMeta {
            name: "Acme".to_string(),
            description: None,
        };
        let entries = vec![entry(
            "http://example.com/about",
            "About",
            Some("The about page"),
        )];
        let out = format_llms_txt(&site, &entries);
        assert!(
            out.contains("- [About](http://example.com/about): The about page"),
            "expected link line with description, got:\n{}",
            out
        );
    }

    #[test]
    fn test_format_link_lines_no_description() {
        let site = SiteMeta {
            name: "Acme".to_string(),
            description: None,
        };
        let entries = vec![entry("http://example.com/about", "About", None)];
        let out = format_llms_txt(&site, &entries);
        assert!(
            out.contains("- [About](http://example.com/about)\n"),
            "expected link line without description, got:\n{}",
            out
        );
        assert!(
            !out.contains("- [About](http://example.com/about):"),
            "expected NO trailing colon when no description, got:\n{}",
            out
        );
    }

    #[test]
    fn test_section_title_from_segment() {
        assert_eq!(title_case_segment("docs"), "Docs");
        assert_eq!(title_case_segment("api-reference"), "Api Reference");
    }

    #[test]
    fn test_append_low_discovery_warning_idempotent_terminator() {
        let body = "# Site\n\n## Pages\n\n- [Home](https://example.com/)\n".to_string();
        let out = append_low_discovery_warning(body);
        assert!(
            out.contains("geodaddy:warning low-discovery"),
            "warning sentinel missing from output:\n{}",
            out
        );
        // Ends with a newline so concatenating files stays clean.
        assert!(out.ends_with('\n'));
    }

    #[test]
    fn test_warning_sentinel_is_html_comment() {
        // The sentinel must be a valid HTML comment so it renders as
        // nothing in markdown viewers but is preserved on disk.
        assert!(LOW_DISCOVERY_WARNING.starts_with("<!--"));
        assert!(LOW_DISCOVERY_WARNING.ends_with("-->"));
    }
}
