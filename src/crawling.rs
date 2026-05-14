use futures::StreamExt;
use reqwest::header::HeaderMap;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::collections::{HashSet, VecDeque};
use std::time::Duration;
use url::Url;

use crate::scoring::CategoryScores;

/// Hard ceiling on a response body we are willing to load into memory for
/// analysis. A handful of real-world pages inline base64 images or ship
/// multi-MB JS payloads — without this cap, `resp.text()` plus the
/// downstream `scraper::Html` DOM (3–5× the raw HTML size) can push the
/// backend into GB-scale RAM spikes per request.
pub const MAX_BODY_BYTES: usize = 5 * 1024 * 1024;

/// Per-page deadline for any single HTTP body read. A slow server that
/// dribbles bytes cannot hold a blocking worker forever — once the timer
/// fires we return what we have (or nothing). Complements
/// `reqwest::Client::timeout` which only bounds the full request.
pub const BODY_READ_TIMEOUT: Duration = Duration::from_secs(30);

/// Returns the largest prefix of `s` that is valid UTF-8 and at most
/// `max_bytes` long. Used when truncating strings we already own
/// (e.g. `chromiumoxide::Page::content()` output).
pub fn truncate_utf8(s: &str, max_bytes: usize) -> &str {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

/// GET `url` and read its body, capped at `max_bytes`. Never panics and
/// never allocates beyond the cap, even for multi-GB responses. Returns
/// empty string + empty headers on network failure or non-success status
/// so callers can keep a single-path `(body, headers)` shape.
pub async fn fetch_text_capped(
    client: &reqwest::Client,
    url: &str,
    max_bytes: usize,
) -> (String, HeaderMap) {
    fetch_text_capped_with_builder(client.get(url), max_bytes).await
}

/// Same as `fetch_text_capped`, but accepts a pre-built `RequestBuilder`
/// so callers can attach headers (Accept, If-None-Match, …) before the
/// request is sent.
pub async fn fetch_text_capped_with_builder(
    req: reqwest::RequestBuilder,
    max_bytes: usize,
) -> (String, HeaderMap) {
    let resp = match req.send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("fetch failed: {}", e);
            return (String::new(), HeaderMap::new());
        }
    };
    let status = resp.status();
    let headers = resp.headers().clone();
    if !status.is_success() {
        return (String::new(), headers);
    }

    let mut stream = resp.bytes_stream();
    let mut buf: Vec<u8> = Vec::new();

    let read = async {
        while let Some(chunk) = stream.next().await {
            let chunk = match chunk {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("stream read error: {}", e);
                    break;
                }
            };
            let remaining = max_bytes.saturating_sub(buf.len());
            if remaining == 0 {
                tracing::warn!("response body exceeded {} bytes — truncating", max_bytes);
                break;
            }
            if chunk.len() > remaining {
                buf.extend_from_slice(&chunk[..remaining]);
                tracing::warn!("response body exceeded {} bytes — truncating", max_bytes);
                break;
            }
            buf.extend_from_slice(&chunk);
        }
    };

    if tokio::time::timeout(BODY_READ_TIMEOUT, read).await.is_err() {
        tracing::warn!("body read timed out — using partial content");
    }

    // from_utf8_lossy is safe for truncated bodies: we may have cut mid-codepoint
    // when hitting the cap, but the lossy decoder replaces invalid bytes instead
    // of panicking. The DOM parser tolerates the replacement characters.
    (String::from_utf8_lossy(&buf).into_owned(), headers)
}

// ── Sitemap structs ───────────────────────────────────────────────────────────

#[derive(Deserialize, Debug)]
struct UrlSet {
    #[serde(default, rename = "url")]
    urls: Vec<UrlEntry>,
}

#[derive(Deserialize, Debug)]
struct UrlEntry {
    loc: String,
    #[serde(default = "default_priority")]
    priority: f64,
}

/// `<sitemapindex>` envelope — a sitemap that points to other sitemaps.
/// Real-world hierarchies can be 1-3 levels deep (one index → product/category
/// shards → leaf urlsets), so recursion has a depth cap.
#[derive(Deserialize, Debug)]
struct SitemapIndex {
    #[serde(default, rename = "sitemap")]
    sitemaps: Vec<SitemapRef>,
}

#[derive(Deserialize, Debug)]
struct SitemapRef {
    loc: String,
}

fn default_priority() -> f64 {
    0.5
}

/// Upper bound on URLs we accumulate from a single sitemap-driven crawl.
/// Real-world sitemap indexes (e.g. pttavm.com) list 300+ child sitemaps
/// with thousands of URLs each. We need a generous internal cap so callers
/// can pick the top N for their own `max_pages` — but not so large that a
/// pathological site can stall the crawler indefinitely.
const SITEMAP_INTERNAL_CAP: usize = 5000;

/// Hard ceiling on recursive sitemap-index traversal. Most real-world
/// hierarchies are 1-2 levels (index → leaf sitemaps); 3 covers the
/// occasional nested-index edge case while keeping the worst-case
/// fan-out bounded.
const SITEMAP_MAX_DEPTH: u8 = 3;

// ── Public functions ──────────────────────────────────────────────────────────

/// Extracts `Sitemap:` directives from a robots.txt body.
/// Case-insensitive on the directive name; trims surrounding whitespace and
/// strips `#`-style comments. Returns the URL strings in the order they
/// appeared. Empty vec if no directives are present.
pub fn extract_sitemap_directives(robots_body: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in robots_body.lines() {
        // Strip inline comments, then trim
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if !key.trim().eq_ignore_ascii_case("sitemap") {
            continue;
        }
        let v = value.trim();
        if !v.is_empty() {
            out.push(v.to_string());
        }
    }
    out
}

/// Fetches /robots.txt and returns the URLs listed in `Sitemap:` directives.
/// Returns empty vec on network failure, missing robots, or no directives.
pub async fn fetch_robots_sitemap_urls(
    client: &reqwest::Client,
    base_url: &Url,
) -> Vec<String> {
    let mut robots_url = base_url.clone();
    robots_url.set_path("/robots.txt");
    robots_url.set_query(None);
    robots_url.set_fragment(None);
    // robots.txt should be tiny; capping at 256KB protects against a
    // misconfigured server streaming an arbitrarily large file.
    let (body, _) = fetch_text_capped(client, robots_url.as_str(), 256 * 1024).await;
    if body.is_empty() {
        return Vec::new();
    }
    extract_sitemap_directives(&body)
}

/// Discovers URLs from a sitemap, recursing into nested sitemap indexes
/// up to `SITEMAP_MAX_DEPTH` levels. Accumulates HTML-only URL entries
/// into `out` until it reaches `SITEMAP_INTERNAL_CAP`. The async recursion
/// is heap-boxed (Rust requires it for any directly-recursive async fn).
fn fetch_sitemap_recursive<'a>(
    client: &'a reqwest::Client,
    sitemap_url: &'a str,
    depth: u8,
    visited: &'a mut HashSet<String>,
    out: &'a mut Vec<UrlEntry>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + 'a + Send>> {
    Box::pin(async move {
        if depth == 0 || out.len() >= SITEMAP_INTERNAL_CAP {
            return;
        }
        if !visited.insert(sitemap_url.to_string()) {
            return;
        }

        let (body, _) =
            fetch_text_capped(client, sitemap_url, MAX_BODY_BYTES).await;
        if body.is_empty() {
            return;
        }

        // Try urlset first. quick_xml's serde driver matches by field name,
        // so a `<sitemapindex>` body parses to UrlSet { urls: [] } — i.e.
        // an empty result we then re-try as a SitemapIndex below.
        if let Ok(url_set) = quick_xml::de::from_str::<UrlSet>(&body) {
            if !url_set.urls.is_empty() {
                for e in url_set.urls {
                    if out.len() >= SITEMAP_INTERNAL_CAP {
                        return;
                    }
                    if is_html_url(&e.loc) {
                        out.push(e);
                    }
                }
                return;
            }
        }

        // Empty urlset → may be a sitemap index. Walk each child sitemap.
        if let Ok(idx) = quick_xml::de::from_str::<SitemapIndex>(&body) {
            for sm in idx.sitemaps {
                if out.len() >= SITEMAP_INTERNAL_CAP {
                    return;
                }
                fetch_sitemap_recursive(client, &sm.loc, depth - 1, visited, out).await;
            }
        }
    })
}

/// Returns URLs from the site's sitemap(s), sorted by priority (highest first).
///
/// Discovery order:
/// 1. Every `Sitemap:` directive listed in `/robots.txt`
/// 2. The conventional `/sitemap.xml` location
///
/// Each candidate is walked recursively so sitemap indexes (the common
/// pattern on large e-commerce / CMS sites) yield the URLs from their
/// child sitemaps. Returns None when no sitemap-discovered URL survives —
/// callers fall back to link-following BFS in that case.
pub async fn fetch_sitemap_urls(
    client: &reqwest::Client,
    base_url: &Url,
) -> Option<Vec<String>> {
    let mut candidates: Vec<String> = fetch_robots_sitemap_urls(client, base_url).await;

    // Always try /sitemap.xml in addition — many sites omit the directive
    // even though the file exists at the standard location.
    let mut default_url = base_url.clone();
    default_url.set_path("/sitemap.xml");
    default_url.set_query(None);
    default_url.set_fragment(None);
    let default_str = default_url.to_string();
    if !candidates.iter().any(|c| c == &default_str) {
        candidates.push(default_str);
    }

    let mut visited: HashSet<String> = HashSet::new();
    let mut entries: Vec<UrlEntry> = Vec::new();

    for sm_url in &candidates {
        fetch_sitemap_recursive(client, sm_url, SITEMAP_MAX_DEPTH, &mut visited, &mut entries)
            .await;
        // Stop after the first candidate that yields URLs — avoids walking
        // a duplicate sitemap when robots.txt already pointed us at the
        // canonical one.
        if !entries.is_empty() {
            break;
        }
    }

    if entries.is_empty() {
        return None;
    }

    entries.sort_by(|a, b| {
        b.priority
            .partial_cmp(&a.priority)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Some(entries.into_iter().map(|e| e.loc).collect())
}

/// Extracts same-origin links from an HTML document, resolved against `base`.
/// Fragments are stripped; off-site links are filtered out.
pub fn extract_same_origin_links(html: &Html, base: &Url) -> Vec<String> {
    let sel = Selector::parse("a[href]").expect("valid selector");
    html.select(&sel)
        .filter_map(|el| el.value().attr("href"))
        .filter_map(|href| base.join(href).ok())
        .filter(|u| u.origin() == base.origin())
        .map(|mut u| {
            u.set_fragment(None);
            u.to_string()
        })
        .collect()
}

/// BFS link-following crawler up to `max_depth`. Returns discovered URLs in
/// visit order. Respects `max_pages` cap if provided.
pub async fn collect_links_bfs(
    client: &reqwest::Client,
    start: &Url,
    max_depth: u8,
    max_pages: Option<usize>,
) -> Vec<String> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<(String, u8)> = VecDeque::new();
    let mut result: Vec<String> = Vec::new();

    let start_norm = normalize_url(start.as_str()).unwrap_or_else(|| start.to_string());
    queue.push_back((start_norm, 0));

    while let Some((url_str, depth)) = queue.pop_front() {
        if visited.contains(&url_str) {
            continue;
        }
        if let Some(max) = max_pages {
            if result.len() >= max {
                break;
            }
        }
        visited.insert(url_str.clone());
        result.push(url_str.clone());

        if depth < max_depth {
            let (html_body, _) = fetch_text_capped(client, &url_str, MAX_BODY_BYTES).await;
            if html_body.is_empty() {
                continue;
            }
            let html = Html::parse_document(&html_body);
            let base = Url::parse(&url_str).unwrap_or_else(|_| start.clone());
            for link in extract_same_origin_links(&html, &base) {
                if let Some(norm) = normalize_url(&link) {
                    if !visited.contains(&norm) {
                        queue.push_back((norm, depth + 1));
                    }
                }
            }
        }
    }

    result
}

/// Returns true when the URL looks like an HTML page that should be analyzed.
/// Filters out sitemaps, feeds, media files, stylesheets, scripts, and other
/// non-HTML resources that would produce garbage GEO analysis results.
pub fn is_html_url(url_str: &str) -> bool {
    let path = match Url::parse(url_str) {
        Ok(u) => u.path().to_lowercase(),
        // Unparseable URL — skip it
        Err(_) => return false,
    };
    // Extensions that are never HTML content pages
    const NON_HTML_EXTS: &[&str] = &[
        ".xml", ".rss", ".atom",
        ".json", ".jsonld",
        ".pdf", ".txt",
        ".png", ".jpg", ".jpeg", ".gif", ".svg", ".webp", ".ico",
        ".css", ".js", ".map",
        ".woff", ".woff2", ".ttf", ".eot",
        ".zip", ".gz", ".tar",
        ".mp4", ".mp3", ".webm", ".ogg",
    ];
    !NON_HTML_EXTS.iter().any(|ext| path.ends_with(ext))
}

/// Normalizes a URL: strips fragments and trailing slashes from non-root paths.
pub fn normalize_url(url_str: &str) -> Option<String> {
    let mut u = Url::parse(url_str).ok()?;
    u.set_fragment(None);
    let path = u.path().to_string();
    if path.len() > 1 && path.ends_with('/') {
        u.set_path(path.trim_end_matches('/'));
    }
    Some(u.to_string())
}

/// Extracts the `Crawl-delay` value from a robots.txt body string.
/// Returns None if the directive is absent or unparseable.
pub fn extract_crawl_delay(robots_body: &str) -> Option<u64> {
    for line in robots_body.lines() {
        let lower = line.to_lowercase();
        if lower.starts_with("crawl-delay:") {
            if let Some(val) = lower.split(':').nth(1) {
                return val.trim().parse::<u64>().ok();
            }
        }
    }
    None
}

/// Returns true when the HTML document appears JS-rendered (thin content):
/// fewer than 3 headings AND zero `<p>` elements.
pub fn needs_js_rendering(html: &Html) -> bool {
    let heading_sel = Selector::parse("h1,h2,h3,h4,h5,h6").expect("valid");
    let p_sel = Selector::parse("p").expect("valid");
    let heading_count = html.select(&heading_sel).count();
    let p_count = html.select(&p_sel).count();
    heading_count < 3 && p_count == 0
}

/// Computes aggregate score (overall + categories) across a set of pages.
/// Each entry is (overall_score, CategoryScores) for one page.
/// Returns (100.0, all-100) for an empty slice.
pub fn aggregate_scores(page_scores: &[(f64, CategoryScores)]) -> (f64, CategoryScores) {
    if page_scores.is_empty() {
        return (
            100.0,
            CategoryScores {
                technical: 100.0,
                content: 100.0,
                geo: 100.0,
                agent: 100.0,
                performance: None,
            },
        );
    }
    let n = page_scores.len() as f64;
    let score = page_scores.iter().map(|(s, _)| s).sum::<f64>() / n;
    let tech = page_scores.iter().map(|(_, c)| c.technical).sum::<f64>() / n;
    let cont = page_scores.iter().map(|(_, c)| c.content).sum::<f64>() / n;
    let geo = page_scores.iter().map(|(_, c)| c.geo).sum::<f64>() / n;
    let agent = page_scores.iter().map(|(_, c)| c.agent).sum::<f64>() / n;
    let perf_scores: Vec<f64> = page_scores
        .iter()
        .filter_map(|(_, c)| c.performance)
        .collect();
    let performance = if perf_scores.is_empty() {
        None
    } else {
        Some(perf_scores.iter().sum::<f64>() / perf_scores.len() as f64)
    };
    (
        score,
        CategoryScores {
            technical: tech,
            content: cont,
            geo,
            agent,
            performance,
        },
    )
}

/// Formats progress for sitemap-driven crawls (total known).
/// Example: "[3/42] https://example.com/about"
pub fn format_progress_known(current: usize, total: usize, url: &str) -> String {
    format!("[{}/{}] {}", current, total, url)
}

/// Formats progress for link-following crawls (total unknown).
/// Example: "Crawling page 5... https://example.com/page"
pub fn format_progress_unknown(current: usize, url: &str) -> String {
    format!("Crawling page {}... {}", current, url)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// When all locs in the parsed XML are non-HTML (e.g. a sitemapindex whose
    /// <sitemap> children were matched by quick_xml serde), the HTML filter
    /// must produce an empty vec — confirming BFS fallback would be triggered.
    #[test]
    fn test_sitemapindex_locs_are_filtered_out() {
        let xml = r#"<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
            <sitemap><loc>https://example.com/sitemap-pages.xml</loc></sitemap>
            <sitemap><loc>https://example.com/sitemap-blog.xml</loc></sitemap>
        </sitemapindex>"#;

        // quick_xml serde ignores <sitemap> elements because the field is
        // renamed to "url" — so urls is empty and html_locs is also empty.
        // Either way, the HTML-only filter must produce an empty result.
        let url_set: UrlSet = quick_xml::de::from_str(xml).unwrap_or(UrlSet { urls: vec![] });
        let html_locs: Vec<String> = url_set
            .urls
            .into_iter()
            .filter(|e| is_html_url(&e.loc))
            .map(|e| e.loc)
            .collect();
        assert!(
            html_locs.is_empty(),
            "sitemapindex locs should be filtered to empty, got: {:?}",
            html_locs
        );
    }

    /// Parses a sitemap XML string and verifies priority-descending sort.
    #[test]
    fn test_fetch_sitemap_urls_parses_xml() {
        let xml = r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
            <url><loc>https://example.com/a</loc><priority>0.8</priority></url>
            <url><loc>https://example.com/b</loc><priority>1.0</priority></url>
        </urlset>"#;

        let mut url_set: UrlSet = quick_xml::de::from_str(xml).expect("parse ok");
        url_set
            .urls
            .sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal));

        let locs: Vec<&str> = url_set.urls.iter().map(|e| e.loc.as_str()).collect();
        assert_eq!(locs, vec!["https://example.com/b", "https://example.com/a"]);
    }

    #[test]
    fn test_url_normalization() {
        assert_eq!(
            normalize_url("https://example.com/page/"),
            Some("https://example.com/page".to_string())
        );
    }

    #[test]
    fn test_url_normalization_fragment() {
        assert_eq!(
            normalize_url("https://example.com/page#section"),
            Some("https://example.com/page".to_string())
        );
    }

    #[test]
    fn test_url_normalization_root() {
        // Root path "/" must NOT have trailing slash stripped
        let result = normalize_url("https://example.com/");
        assert!(result.is_some());
        let url = Url::parse(result.unwrap().as_str()).unwrap();
        assert_eq!(url.path(), "/");
    }

    #[test]
    fn test_extract_sitemap_directives_basic() {
        let body = "User-agent: *\nSitemap: https://example.com/sitemap_index.xml\nDisallow: /admin\n";
        let urls = extract_sitemap_directives(body);
        assert_eq!(urls, vec!["https://example.com/sitemap_index.xml"]);
    }

    #[test]
    fn test_extract_sitemap_directives_case_insensitive() {
        let body = "sitemap: https://example.com/a.xml\nSITEMAP: https://example.com/b.xml\nSiteMap: https://example.com/c.xml\n";
        let urls = extract_sitemap_directives(body);
        assert_eq!(
            urls,
            vec![
                "https://example.com/a.xml",
                "https://example.com/b.xml",
                "https://example.com/c.xml",
            ]
        );
    }

    #[test]
    fn test_extract_sitemap_directives_multiple_and_comments() {
        let body = "# comments are ignored\nUser-agent: *\nSitemap: https://example.com/a.xml # inline comment\nSitemap: https://example.com/b.xml\n\nSitemap:    \n";
        let urls = extract_sitemap_directives(body);
        // 3rd line is empty after trimming the value — drop it.
        assert_eq!(
            urls,
            vec!["https://example.com/a.xml", "https://example.com/b.xml"]
        );
    }

    #[test]
    fn test_extract_sitemap_directives_empty_robots() {
        assert!(extract_sitemap_directives("").is_empty());
        assert!(extract_sitemap_directives("User-agent: *\nDisallow: /").is_empty());
    }

    #[test]
    fn test_sitemapindex_parses_with_new_struct() {
        let xml = r#"<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
            <sitemap><loc>https://example.com/sitemap-pages.xml</loc></sitemap>
            <sitemap><loc>https://example.com/sitemap-blog.xml</loc></sitemap>
        </sitemapindex>"#;
        let idx: SitemapIndex = quick_xml::de::from_str(xml).expect("parse");
        let locs: Vec<&str> = idx.sitemaps.iter().map(|s| s.loc.as_str()).collect();
        assert_eq!(
            locs,
            vec![
                "https://example.com/sitemap-pages.xml",
                "https://example.com/sitemap-blog.xml",
            ]
        );
    }

    #[test]
    fn test_extract_crawl_delay_present() {
        assert_eq!(
            extract_crawl_delay("User-agent: *\nCrawl-delay: 3\n"),
            Some(3)
        );
    }

    #[test]
    fn test_extract_crawl_delay_absent() {
        assert_eq!(
            extract_crawl_delay("User-agent: *\nDisallow: /admin\n"),
            None
        );
    }

    #[test]
    fn test_js_detection_thin_page() {
        let html = Html::parse_document("<html><body><div>nothing</div></body></html>");
        assert!(needs_js_rendering(&html));
    }

    #[test]
    fn test_js_detection_rich_page() {
        let html = Html::parse_document(
            "<html><body><h1>A</h1><h2>B</h2><h3>C</h3><p>text</p></body></html>",
        );
        assert!(!needs_js_rendering(&html));
    }

    #[test]
    fn test_is_html_url_allows_html_pages() {
        assert!(is_html_url("https://example.com/"));
        assert!(is_html_url("https://example.com/about"));
        assert!(is_html_url("https://example.com/blog/post-title"));
        assert!(is_html_url("https://example.com/page?q=1"));
    }

    #[test]
    fn test_is_html_url_filters_non_html() {
        assert!(!is_html_url("https://example.com/sitemap.xml"));
        assert!(!is_html_url("https://example.com/sitemap-posts.xml"));
        assert!(!is_html_url("https://example.com/feed.rss"));
        assert!(!is_html_url("https://example.com/robots.txt"));
        assert!(!is_html_url("https://example.com/style.css"));
        assert!(!is_html_url("https://example.com/app.js"));
        assert!(!is_html_url("https://example.com/logo.png"));
        assert!(!is_html_url("https://example.com/doc.pdf"));
        assert!(!is_html_url("https://example.com/data.json"));
    }

    #[test]
    fn test_extract_same_origin_links() {
        let html = Html::parse_document(
            r#"<html><body>
                <a href="/about">About</a>
                <a href="https://other.com/page">Off-site</a>
            </body></html>"#,
        );
        let base = Url::parse("https://example.com/").unwrap();
        let links = extract_same_origin_links(&html, &base);
        assert!(links.iter().any(|l| l.contains("example.com/about")));
        assert!(!links.iter().any(|l| l.contains("other.com")));
    }

    #[test]
    fn test_offsite_links_filtered() {
        let html = Html::parse_document(
            r#"<html><body><a href="https://twitter.com/foo">Twitter</a></body></html>"#,
        );
        let base = Url::parse("https://example.com/").unwrap();
        let links = extract_same_origin_links(&html, &base);
        assert!(!links.iter().any(|l| l.contains("twitter.com")));
    }

    #[test]
    fn test_relative_link_resolution() {
        let html =
            Html::parse_document(r#"<html><body><a href="../contact">Contact</a></body></html>"#);
        let base = Url::parse("https://example.com/blog/").unwrap();
        let links = extract_same_origin_links(&html, &base);
        // ../contact from /blog/ resolves to /contact
        assert!(links.iter().any(|l| l.contains("example.com/contact")));
    }

    #[test]
    fn test_aggregate_score_average() {
        let page_scores = vec![
            (
                80.0,
                CategoryScores {
                    technical: 80.0,
                    content: 80.0,
                    geo: 80.0,
                    agent: 80.0,
                    performance: None,
                },
            ),
            (
                60.0,
                CategoryScores {
                    technical: 60.0,
                    content: 60.0,
                    geo: 60.0,
                    agent: 60.0,
                    performance: None,
                },
            ),
        ];
        let (overall, _) = aggregate_scores(&page_scores);
        assert!((overall - 70.0).abs() < 0.001);
    }

    #[test]
    fn test_aggregate_score_empty() {
        let (overall, cats) = aggregate_scores(&[]);
        assert_eq!(overall, 100.0);
        assert_eq!(cats.technical, 100.0);
        assert_eq!(cats.content, 100.0);
        assert_eq!(cats.geo, 100.0);
        assert_eq!(cats.agent, 100.0);
        assert_eq!(cats.performance, None);
    }

    #[test]
    fn test_aggregate_score_performance_averages_some_values() {
        let page_scores = vec![
            (80.0, CategoryScores { technical: 80.0, content: 80.0, geo: 80.0, agent: 80.0, performance: Some(80.0) }),
            (60.0, CategoryScores { technical: 60.0, content: 60.0, geo: 60.0, agent: 60.0, performance: Some(60.0) }),
        ];
        let (_, cats) = aggregate_scores(&page_scores);
        assert_eq!(cats.performance, Some(70.0));
    }

    #[test]
    fn test_aggregate_score_performance_none_when_all_none() {
        let page_scores = vec![
            (80.0, CategoryScores { technical: 80.0, content: 80.0, geo: 80.0, agent: 80.0, performance: None }),
            (60.0, CategoryScores { technical: 60.0, content: 60.0, geo: 60.0, agent: 60.0, performance: None }),
        ];
        let (_, cats) = aggregate_scores(&page_scores);
        assert_eq!(cats.performance, None);
    }

    #[test]
    fn test_progress_format_known() {
        assert_eq!(
            format_progress_known(3, 42, "https://example.com/about"),
            "[3/42] https://example.com/about"
        );
    }

    #[test]
    fn test_progress_format_unknown() {
        assert_eq!(
            format_progress_unknown(5, "https://example.com/page"),
            "Crawling page 5... https://example.com/page"
        );
    }
}
