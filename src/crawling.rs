use scraper::{Html, Selector};
use serde::Deserialize;
use std::collections::{HashSet, VecDeque};
use url::Url;

use crate::scoring::CategoryScores;

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

fn default_priority() -> f64 {
    0.5
}

// ── Public functions ──────────────────────────────────────────────────────────

/// Fetches /sitemap.xml and returns URLs sorted by priority (highest first).
/// Returns None if the sitemap is unavailable or empty (triggers link-following fallback).
pub async fn fetch_sitemap_urls(
    client: &reqwest::Client,
    base_url: &Url,
) -> Option<Vec<String>> {
    let mut sitemap_url = base_url.clone();
    sitemap_url.set_path("/sitemap.xml");
    sitemap_url.set_query(None);
    sitemap_url.set_fragment(None);

    let body = client
        .get(sitemap_url.as_str())
        .send()
        .await
        .ok()?
        .text()
        .await
        .ok()?;

    let mut url_set: UrlSet = quick_xml::de::from_str(&body).ok()?;

    // Empty sitemap — trigger link-following fallback
    if url_set.urls.is_empty() {
        return None;
    }

    // Sort by priority descending (highest priority pages first)
    url_set
        .urls
        .sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal));

    // Filter to HTML-only locs. If a sitemapindex is served at /sitemap.xml,
    // quick_xml serde deserialises <sitemap><loc> children into UrlEntry structs
    // (field-name match, not element-name match), so all locs end in .xml.
    // Returning None here lets the caller fall back to BFS link-following.
    let html_locs: Vec<String> = url_set
        .urls
        .into_iter()
        .filter(|e| is_html_url(&e.loc))
        .map(|e| e.loc)
        .collect();

    if html_locs.is_empty() {
        return None;
    }

    Some(html_locs)
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
            let html_body = match client.get(&url_str).send().await {
                Ok(resp) if resp.status().is_success() => resp.text().await.unwrap_or_default(),
                _ => continue,
            };
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
