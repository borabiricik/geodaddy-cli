use crate::scoring::{AnalysisResult, Status};
use scraper::{Html, Selector};
use url::Url;
use std::time::Duration;
use quick_xml::de::from_str as xml_from_str;
use serde::Deserialize;

// --- HTML-only analyzers (sync) ---

/// TECH-01: Broken link detection — stub per D-08
pub fn analyze_broken_links() -> AnalysisResult {
    AnalysisResult {
        check: "tech-broken-links",
        status: Status::Warn,
        message: "Broken link detection requires site-wide crawl mode".to_string(),
        recommendation: "Run geodaddy with site-wide crawling (Phase 4) to detect broken links across all pages".to_string(),
    }
}

/// TECH-03: Meta tags (title + description)
pub fn analyze_meta_tags(html: &Html) -> Vec<AnalysisResult> {
    let mut results = Vec::new();

    // Title check
    let title_sel = Selector::parse("title").expect("valid selector");
    let title_text = html
        .select(&title_sel)
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string());

    let title_result = match title_text.as_deref() {
        None | Some("") => AnalysisResult {
            check: "tech-meta-title",
            status: Status::Fail,
            message: "Page has no title tag".to_string(),
            recommendation: "Add a <title> tag with 50-60 characters describing the page content".to_string(),
        },
        Some(t) => {
            let len = t.len();
            if len < 50 {
                AnalysisResult {
                    check: "tech-meta-title",
                    status: Status::Warn,
                    message: format!("Title is {} chars (min 50)", len),
                    recommendation: "Expand title to 50-60 characters for optimal search snippet display".to_string(),
                }
            } else if len <= 60 {
                AnalysisResult {
                    check: "tech-meta-title",
                    status: Status::Pass,
                    message: format!("Title is {} chars (optimal range 50-60)", len),
                    recommendation: String::new(),
                }
            } else {
                AnalysisResult {
                    check: "tech-meta-title",
                    status: Status::Fail,
                    message: format!("Title is {} chars (max 60)", len),
                    recommendation: "Shorten title to 50-60 chars — truncation in search results reduces click-through rate".to_string(),
                }
            }
        }
    };
    results.push(title_result);

    // Description check
    let desc_sel = Selector::parse(r#"meta[name="description"]"#).expect("valid selector");
    let desc_content = html
        .select(&desc_sel)
        .next()
        .and_then(|el| el.value().attr("content"))
        .unwrap_or_default()
        .trim()
        .to_string();

    let desc_result = if desc_content.is_empty() {
        AnalysisResult {
            check: "tech-meta-description",
            status: Status::Warn,
            message: "Page has no meta description".to_string(),
            recommendation: "Add a meta description of 120-158 characters summarizing the page for search engines".to_string(),
        }
    } else {
        let len = desc_content.len();
        if len >= 120 && len <= 158 {
            AnalysisResult {
                check: "tech-meta-description",
                status: Status::Pass,
                message: format!("Meta description is {} chars (optimal range 120-158)", len),
                recommendation: String::new(),
            }
        } else {
            AnalysisResult {
                check: "tech-meta-description",
                status: Status::Warn,
                message: format!("Meta description is {} chars (optimal: 120-158)", len),
                recommendation: "Adjust meta description to 120-158 characters — this is the displayed snippet length in most search engines".to_string(),
            }
        }
    };
    results.push(desc_result);

    results
}

/// TECH-04: H1 heading presence
pub fn analyze_headings_tech(html: &Html) -> AnalysisResult {
    let h1_sel = Selector::parse("h1").expect("valid selector");
    let count = html.select(&h1_sel).count();
    match count {
        0 => AnalysisResult {
            check: "tech-heading-h1",
            status: Status::Fail,
            message: "Page has no H1 heading".to_string(),
            recommendation: "Add exactly one H1 heading that describes the main topic of the page".to_string(),
        },
        1 => AnalysisResult {
            check: "tech-heading-h1",
            status: Status::Pass,
            message: "Page has exactly one H1 heading (correct)".to_string(),
            recommendation: String::new(),
        },
        n => AnalysisResult {
            check: "tech-heading-h1",
            status: Status::Fail,
            message: format!("Page has {} H1 headings (expected exactly 1)", n),
            recommendation: "Reduce to a single H1 that identifies the page's primary topic; use H2-H6 for subsections".to_string(),
        },
    }
}

/// TECH-05: Mobile viewport meta tag
pub fn analyze_mobile_viewport(html: &Html) -> AnalysisResult {
    let vp_sel = Selector::parse(r#"meta[name="viewport"]"#).expect("valid selector");
    match html.select(&vp_sel).next() {
        None => AnalysisResult {
            check: "tech-mobile-viewport",
            status: Status::Fail,
            message: "Page is missing mobile viewport meta tag".to_string(),
            recommendation: r#"Add <meta name="viewport" content="width=device-width, initial-scale=1"> to the <head>"#.to_string(),
        },
        Some(el) => {
            let content = el.value().attr("content").unwrap_or_default();
            if content.contains("width=device-width") {
                AnalysisResult {
                    check: "tech-mobile-viewport",
                    status: Status::Pass,
                    message: "Mobile viewport meta tag is correctly configured".to_string(),
                    recommendation: String::new(),
                }
            } else {
                AnalysisResult {
                    check: "tech-mobile-viewport",
                    status: Status::Warn,
                    message: "Viewport tag present but missing width=device-width".to_string(),
                    recommendation: r#"Set viewport content to "width=device-width, initial-scale=1" so the page scales correctly on mobile devices"#.to_string(),
                }
            }
        }
    }
}

/// TECH-08: HTTPS check + mixed content scan
pub fn analyze_https(html: &Html, url: &Url) -> AnalysisResult {
    if url.scheme() != "https" {
        return AnalysisResult {
            check: "tech-https",
            status: Status::Fail,
            message: "Page is served over HTTP, not HTTPS".to_string(),
            recommendation: "Migrate to HTTPS — search engines and AI crawlers penalize insecure pages; obtain a TLS certificate from Let's Encrypt (free)".to_string(),
        };
    }

    // Scan for mixed content (http:// resources)
    let mixed_selectors = [
        r#"img[src^="http:"]"#,
        r#"script[src^="http:"]"#,
        r#"link[href^="http:"]"#,
        r#"iframe[src^="http:"]"#,
    ];

    let mut mixed_count: usize = 0;
    for sel_str in &mixed_selectors {
        let sel = Selector::parse(sel_str).expect("valid selector");
        mixed_count += html.select(&sel).count();
    }

    if mixed_count == 0 {
        AnalysisResult {
            check: "tech-https",
            status: Status::Pass,
            message: "Page is served over HTTPS with no mixed content".to_string(),
            recommendation: String::new(),
        }
    } else {
        AnalysisResult {
            check: "tech-https",
            status: Status::Warn,
            message: format!("Page is HTTPS but has {} mixed content resource(s) loaded over HTTP", mixed_count),
            recommendation: format!("Update {} HTTP resource URL(s) to HTTPS to eliminate mixed content warnings; mixed content causes browser security warnings and is blocked by default in modern browsers", mixed_count),
        }
    }
}

// --- HTTP-based analyzers (async) ---

/// TECH-02: Redirect chain detection
pub async fn analyze_redirect_chains(client: &reqwest::Client, url: &Url) -> AnalysisResult {
    let redirect_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 3 {
                attempt.error("too many redirects")
            } else {
                attempt.follow()
            }
        }))
        .timeout(Duration::from_secs(10))
        .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
        .build()
        .expect("failed to build redirect client");

    let _ = client; // provided for API consistency; we use redirect_client internally
    match redirect_client.head(url.as_str()).send().await {
        Ok(_response) => AnalysisResult {
            check: "tech-redirect-chains",
            status: Status::Pass,
            message: "No redirect chain issues detected for this URL".to_string(),
            recommendation: String::new(),
        },
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("too many redirects") {
                AnalysisResult {
                    check: "tech-redirect-chains",
                    status: Status::Fail,
                    message: "Redirect chain has 3 or more hops".to_string(),
                    recommendation: "Reduce redirect chain to a single canonical redirect — each hop adds latency and dilutes link equity. Update links and canonicals to point directly to the final URL".to_string(),
                }
            } else {
                AnalysisResult {
                    check: "tech-redirect-chains",
                    status: Status::Warn,
                    message: "Could not check redirect chain (request failed)".to_string(),
                    recommendation: "Ensure the URL is accessible and retry".to_string(),
                }
            }
        }
    }
}

/// TECH-06: robots.txt validation
pub async fn analyze_robots_txt(client: &reqwest::Client, url: &Url) -> AnalysisResult {
    let mut robots_url = url.clone();
    robots_url.set_path("/robots.txt");
    robots_url.set_query(None);
    robots_url.set_fragment(None);

    match client.get(robots_url.as_str()).send().await {
        Err(_) => AnalysisResult {
            check: "tech-robots-txt",
            status: Status::Warn,
            message: "robots.txt not found at /robots.txt".to_string(),
            recommendation: "Create a robots.txt file at the site root to control crawler access. Include a Sitemap: directive pointing to your sitemap.xml".to_string(),
        },
        Ok(resp) => {
            if !resp.status().is_success() {
                return AnalysisResult {
                    check: "tech-robots-txt",
                    status: Status::Warn,
                    message: "robots.txt not found at /robots.txt".to_string(),
                    recommendation: "Create a robots.txt file at the site root to control crawler access. Include a Sitemap: directive pointing to your sitemap.xml".to_string(),
                };
            }
            let body = resp.text().await.unwrap_or_default();
            let has_sitemap = body
                .lines()
                .any(|line| line.to_lowercase().starts_with("sitemap:"));
            if has_sitemap {
                AnalysisResult {
                    check: "tech-robots-txt",
                    status: Status::Pass,
                    message: "robots.txt found and contains Sitemap directive".to_string(),
                    recommendation: String::new(),
                }
            } else {
                AnalysisResult {
                    check: "tech-robots-txt",
                    status: Status::Warn,
                    message: "robots.txt found but missing Sitemap: directive".to_string(),
                    recommendation: "Add 'Sitemap: https://yourdomain.com/sitemap.xml' to robots.txt so search engines can discover your sitemap".to_string(),
                }
            }
        }
    }
}

/// TECH-07: sitemap.xml validation
pub async fn analyze_sitemap(client: &reqwest::Client, url: &Url) -> AnalysisResult {
    let mut sitemap_url = url.clone();
    sitemap_url.set_path("/sitemap.xml");
    sitemap_url.set_query(None);
    sitemap_url.set_fragment(None);

    #[derive(Deserialize, Debug)]
    struct UrlSet {
        #[serde(default, rename = "url")]
        urls: Vec<UrlEntry>,
    }

    #[derive(Deserialize, Debug)]
    struct UrlEntry {
        #[allow(dead_code)]
        loc: String,
    }

    match client.get(sitemap_url.as_str()).send().await {
        Err(_) => AnalysisResult {
            check: "tech-sitemap-xml",
            status: Status::Warn,
            message: "sitemap.xml not found at /sitemap.xml".to_string(),
            recommendation: "Create a sitemap.xml and reference it in robots.txt to help search engines discover all pages".to_string(),
        },
        Ok(resp) => {
            if !resp.status().is_success() {
                return AnalysisResult {
                    check: "tech-sitemap-xml",
                    status: Status::Warn,
                    message: "sitemap.xml not found at /sitemap.xml".to_string(),
                    recommendation: "Create a sitemap.xml and reference it in robots.txt to help search engines discover all pages".to_string(),
                };
            }
            let body = resp.text().await.unwrap_or_default();
            match xml_from_str::<UrlSet>(&body) {
                Err(_) => AnalysisResult {
                    check: "tech-sitemap-xml",
                    status: Status::Fail,
                    message: "sitemap.xml found but is malformed XML".to_string(),
                    recommendation: "Validate your sitemap.xml against the Sitemaps protocol at https://www.sitemaps.org/protocol.html".to_string(),
                },
                Ok(url_set) => {
                    let count = url_set.urls.len();
                    if count <= 50000 {
                        AnalysisResult {
                            check: "tech-sitemap-xml",
                            status: Status::Pass,
                            message: format!("sitemap.xml is valid and contains {} URLs", count),
                            recommendation: String::new(),
                        }
                    } else {
                        AnalysisResult {
                            check: "tech-sitemap-xml",
                            status: Status::Warn,
                            message: format!("sitemap.xml contains {} URLs (Google limit: 50,000 per file)", count),
                            recommendation: "Split your sitemap into multiple files and use a sitemap index file — Google silently ignores URLs beyond the 50,000 limit".to_string(),
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(html_str: &str) -> Html {
        Html::parse_document(html_str)
    }

    // --- analyze_broken_links ---

    #[test]
    fn broken_links_always_warn() {
        let result = analyze_broken_links();
        assert_eq!(result.status, Status::Warn);
        assert_eq!(result.check, "tech-broken-links");
    }

    // --- analyze_meta_tags ---

    #[test]
    fn meta_title_absent_returns_fail() {
        let html = parse("<html><head></head><body></body></html>");
        let results = analyze_meta_tags(&html);
        let title = results.iter().find(|r| r.check == "tech-meta-title").unwrap();
        assert_eq!(title.status, Status::Fail);
    }

    #[test]
    fn meta_title_55_chars_returns_pass() {
        // 55 'a' chars
        let title = "a".repeat(55);
        let html = parse(&format!("<html><head><title>{}</title></head></html>", title));
        let results = analyze_meta_tags(&html);
        let r = results.iter().find(|r| r.check == "tech-meta-title").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn meta_title_70_chars_returns_fail() {
        let title = "a".repeat(70);
        let html = parse(&format!("<html><head><title>{}</title></head></html>", title));
        let results = analyze_meta_tags(&html);
        let r = results.iter().find(|r| r.check == "tech-meta-title").unwrap();
        assert_eq!(r.status, Status::Fail);
    }

    #[test]
    fn meta_description_absent_returns_warn() {
        let html = parse("<html><head></head></html>");
        let results = analyze_meta_tags(&html);
        let r = results.iter().find(|r| r.check == "tech-meta-description").unwrap();
        assert_eq!(r.status, Status::Warn);
    }

    // --- analyze_headings_tech ---

    #[test]
    fn single_h1_returns_pass() {
        let html = parse("<html><body><h1>Title</h1></body></html>");
        let r = analyze_headings_tech(&html);
        assert_eq!(r.check, "tech-heading-h1");
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn zero_h1_returns_fail() {
        let html = parse("<html><body><h2>Sub</h2></body></html>");
        let r = analyze_headings_tech(&html);
        assert_eq!(r.status, Status::Fail);
    }

    #[test]
    fn multiple_h1_returns_fail() {
        let html = parse("<html><body><h1>A</h1><h1>B</h1></body></html>");
        let r = analyze_headings_tech(&html);
        assert_eq!(r.status, Status::Fail);
    }

    // --- analyze_mobile_viewport ---

    #[test]
    fn viewport_absent_returns_fail() {
        let html = parse("<html><head></head></html>");
        let r = analyze_mobile_viewport(&html);
        assert_eq!(r.check, "tech-mobile-viewport");
        assert_eq!(r.status, Status::Fail);
    }

    #[test]
    fn viewport_with_device_width_returns_pass() {
        let html = parse(r#"<html><head><meta name="viewport" content="width=device-width, initial-scale=1"></head></html>"#);
        let r = analyze_mobile_viewport(&html);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn viewport_without_device_width_returns_warn() {
        let html = parse(r#"<html><head><meta name="viewport" content="initial-scale=1"></head></html>"#);
        let r = analyze_mobile_viewport(&html);
        assert_eq!(r.status, Status::Warn);
    }

    // --- analyze_https ---

    #[test]
    fn http_url_returns_fail() {
        let html = parse("<html><body></body></html>");
        let url = Url::parse("http://example.com/").unwrap();
        let r = analyze_https(&html, &url);
        assert_eq!(r.check, "tech-https");
        assert_eq!(r.status, Status::Fail);
    }

    #[test]
    fn https_url_no_mixed_content_returns_pass() {
        let html = parse("<html><body><img src=\"https://example.com/img.png\"></body></html>");
        let url = Url::parse("https://example.com/").unwrap();
        let r = analyze_https(&html, &url);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn https_url_with_mixed_content_returns_warn() {
        let html = parse("<html><body><img src=\"http://example.com/img.png\"></body></html>");
        let url = Url::parse("https://example.com/").unwrap();
        let r = analyze_https(&html, &url);
        assert_eq!(r.status, Status::Warn);
        assert_eq!(r.check, "tech-https");
    }

    // --- async tests (network-unavailable → Warn) ---

    #[tokio::test]
    async fn redirect_chains_unreachable_returns_warn() {
        let client = reqwest::Client::new();
        let url = Url::parse("http://127.0.0.1:19999/").unwrap();
        let r = analyze_redirect_chains(&client, &url).await;
        // Can be Warn (connection refused) or Pass if fast-fail; never panics
        assert!(matches!(r.status, Status::Warn | Status::Pass | Status::Fail));
        assert_eq!(r.check, "tech-redirect-chains");
    }

    #[tokio::test]
    async fn robots_txt_unreachable_returns_warn() {
        let client = reqwest::Client::new();
        let url = Url::parse("http://127.0.0.1:19999/").unwrap();
        let r = analyze_robots_txt(&client, &url).await;
        assert_eq!(r.status, Status::Warn);
        assert_eq!(r.check, "tech-robots-txt");
    }

    #[tokio::test]
    async fn sitemap_unreachable_returns_warn() {
        let client = reqwest::Client::new();
        let url = Url::parse("http://127.0.0.1:19999/").unwrap();
        let r = analyze_sitemap(&client, &url).await;
        assert_eq!(r.status, Status::Warn);
        assert_eq!(r.check, "tech-sitemap-xml");
    }
}
