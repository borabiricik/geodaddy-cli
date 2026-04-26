pub mod scoring;
pub mod analyzers;
pub mod crawling;
pub mod beauty;
pub mod compare;
pub mod see;

use anyhow::Result;
use chromiumoxide::Browser;
use chrono::Utc;
use scraper::Html;
use serde::Serialize;
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::sleep;
use url::Url;

use crate::scoring::{AnalysisResult, CategoryScores, calculate_score};
use crate::analyzers::technical::{
    analyze_broken_links, analyze_meta_tags, analyze_headings_tech,
    analyze_mobile_viewport, analyze_https,
    analyze_redirect_chains, analyze_robots_txt, analyze_sitemap,
};
use crate::analyzers::content::{
    analyze_heading_structure, analyze_json_ld, analyze_semantic_html, analyze_alt_text,
};
use crate::analyzers::geo::{
    analyze_listicle, analyze_ai_bots, analyze_schema_stacking,
};
use crate::analyzers::geo_llms::analyze_llms_txt;
use crate::analyzers::geo_directives::{analyze_ai_meta_directives, analyze_ai_header_directives};
use crate::analyzers::geo_citations::{analyze_citations, analyze_faq_quality};
use crate::analyzers::geo_entities::analyze_entities;
use crate::analyzers::geo_query::analyze_query_optimization;
use crate::analyzers::geo_freshness::{analyze_freshness, analyze_howto_schema};
use crate::analyzers::agent::{
    AgentResources, fetch_agent_resources, analyze_link_headers, analyze_markdown_negotiation,
    analyze_content_signals, analyze_web_bot_auth, analyze_mcp_server_card, analyze_a2a_card,
    analyze_agent_skills, analyze_webmcp, analyze_api_catalog, analyze_oauth_discovery,
    analyze_oauth_protected_resource, analyze_x402, analyze_mpp, analyze_ucp, analyze_acp,
    analyze_ap2,
};
use crate::analyzers::performance::analyze_vitals;
use crate::crawling::{
    fetch_sitemap_urls, collect_links_bfs, normalize_url, is_html_url,
    extract_crawl_delay, needs_js_rendering, aggregate_scores,
    fetch_text_capped, truncate_utf8, MAX_BODY_BYTES,
};

/// Upper bound on how long any single headless navigation + DOM snapshot is
/// allowed to take before we give up and fall through to the reqwest HTML.
/// A hung page cannot be left to hold a blocking worker indefinitely.
const BROWSER_PAGE_TIMEOUT: Duration = Duration::from_secs(30);

/// Configuration for the analysis engine.
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub max_pages: Option<usize>,
    pub enable_js: bool,
    pub vitals: bool,
}

/// Top-level report containing aggregate scores and per-page results.
#[derive(Serialize, Debug)]
pub struct Report {
    pub schema_version: &'static str,
    pub url: String,
    pub crawled_at: String,
    pub score: f64,
    pub categories: CategoryScores,
    pub pages: Vec<PageResult>,
}

/// Per-page analysis result.
#[derive(Serialize, Debug)]
pub struct PageResult {
    pub url: String,
    pub robots_blocked: bool,
    pub score: f64,
    pub categories: CategoryScores,
    pub results: Vec<AnalysisResult>,
}

/// Run a full GEO analysis on the given URL.
///
/// The caller is responsible for creating the HTTP client and optionally
/// launching browser instances. Browsers can be reused across calls —
/// only new pages/tabs are created per analysis, not new processes.
pub async fn analyze(
    url: &str,
    config: &AnalysisConfig,
    client: &reqwest::Client,
    js_browser: Option<&Browser>,
    vitals_browser: Option<&Browser>,
) -> Result<Report> {
    let base_url = Url::parse(url)
        .map_err(|e| anyhow::anyhow!("Invalid URL '{}': {}", url, e))?;

    // Connectivity check — fail fast if the target is unreachable
    match client.head(base_url.as_str()).send().await {
        Ok(_) => {}
        Err(e) => {
            if e.is_connect() || e.is_timeout() {
                anyhow::bail!(
                    "Cannot connect to '{}': {}. Check the URL and ensure the server is running.",
                    url, e
                );
            }
            tracing::warn!("Connectivity pre-check warning for {}: {}", url, e);
        }
    }

    // robots.txt fetched ONCE at crawl start
    let (_, robots_body) = check_robots(client, &base_url).await;
    let crawl_delay = extract_crawl_delay(&robots_body).unwrap_or(1);

    // llms.txt fetched ONCE at crawl start (site-wide resource, like robots.txt)
    let llms_txt_body = fetch_llms_txt(client, &base_url).await;

    // Agent-readiness site-wide resources (MCP/A2A/OAuth/API catalogue/…) fetched
    // ONCE. Individual per-page checks reuse this struct.
    let agent_resources: AgentResources = fetch_agent_resources(client, &base_url).await;

    // Determine URL list — crawling is opt-in via max_pages
    let (urls, is_sitemap_driven): (Vec<String>, bool) = if config.max_pages.is_none() {
        (vec![url.to_string()], false)
    } else {
        match fetch_sitemap_urls(client, &base_url).await {
            Some(mut sitemap_urls) => {
                if let Some(max) = config.max_pages {
                    sitemap_urls.truncate(max);
                }
                (sitemap_urls, true)
            }
            None => {
                tracing::info!("No sitemap.xml found — falling back to link-following (depth 2)");
                (collect_links_bfs(client, &base_url, 2, config.max_pages).await, false)
            }
        }
    };

    // Filter to HTML-only before the loop
    let urls: Vec<String> = urls.into_iter().filter(|u| is_html_url(u)).collect();
    let total = urls.len();

    let mut pages: Vec<PageResult> = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut page_num: usize = 0;

    for url_str in &urls {
        // Deduplicate
        let norm = normalize_url(url_str).unwrap_or_else(|| url_str.clone());
        if visited.contains(&norm) {
            continue;
        }
        visited.insert(norm.clone());
        page_num += 1;

        // Progress logging
        if is_sitemap_driven {
            tracing::info!("[{}/{}] {}", page_num, total, url_str);
        } else {
            tracing::info!("Crawling page {}... {}", page_num, url_str);
        }

        let page_url = match Url::parse(&norm) {
            Ok(u) => u,
            Err(_) => {
                tracing::warn!("Skipping invalid URL: {}", norm);
                continue;
            }
        };

        // Check robots.txt per-URL using cached body
        let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
        let mut robots_matcher = robotstxt::DefaultMatcher::default();
        let robots_blocked = !robots_matcher.one_agent_allowed_by_robots(
            &robots_body,
            user_agent,
            page_url.as_str(),
        );

        // Fetch HTML — reqwest first, then headless if enabled and thin.
        // Body is streamed and capped at MAX_BODY_BYTES so one oversized page
        // cannot explode memory for the whole process (e.g. 100MB inline base64).
        let (html_body, response_headers) =
            fetch_text_capped(client, page_url.as_str(), MAX_BODY_BYTES).await;

        let mut html_doc = Html::parse_document(&html_body);
        // The raw HTML string is now owned by the DOM tree — release the
        // original copy before any further allocations.
        drop(html_body);

        // JS detection and re-fetch — only when enable_js is active
        if config.enable_js {
            if let Some(b) = js_browser {
                if needs_js_rendering(&html_doc) {
                    tracing::info!(
                        "JS-rendered page detected: {} — re-fetching via headless browser",
                        page_url
                    );
                    let js_fetch = async {
                        let page = b.new_page(page_url.as_str()).await?;
                        let content_result = page.content().await;
                        // Close the browser tab to prevent memory leaks —
                        // chromiumoxide::Page has no Drop impl, so tabs
                        // persist in the browser process unless explicitly closed.
                        if let Err(e) = page.close().await {
                            tracing::warn!("Failed to close JS page for {}: {}", page_url, e);
                        }
                        content_result.map_err(|e| anyhow::anyhow!(e))
                    };
                    match tokio::time::timeout(BROWSER_PAGE_TIMEOUT, js_fetch).await {
                        Ok(Ok(content)) => {
                            let safe = truncate_utf8(&content, MAX_BODY_BYTES);
                            html_doc = Html::parse_document(safe);
                        }
                        Ok(Err(e)) => {
                            tracing::warn!("Headless fetch failed for {}: {}", page_url, e);
                        }
                        Err(_) => {
                            tracing::warn!(
                                "Headless fetch timed out for {} — falling back to HTTP HTML",
                                page_url
                            );
                        }
                    }
                }
            }
        }

        // Run all analyzers
        let mut results: Vec<AnalysisResult> = Vec::new();

        results.push(analyze_broken_links());
        results.push(analyze_redirect_chains(client, &page_url).await);
        results.extend(analyze_meta_tags(&html_doc));
        results.push(analyze_headings_tech(&html_doc));
        results.push(analyze_mobile_viewport(&html_doc));
        results.push(analyze_robots_txt(client, &page_url).await);
        results.push(analyze_sitemap(client, &page_url).await);
        results.push(analyze_https(&html_doc, &page_url));
        results.push(analyze_heading_structure(&html_doc));
        results.push(analyze_json_ld(&html_doc));
        results.push(analyze_semantic_html(&html_doc));
        results.push(analyze_alt_text(&html_doc));
        results.push(analyze_listicle(&html_doc));
        results.extend(analyze_ai_bots(&robots_body));
        results.push(analyze_schema_stacking(&html_doc));

        // Phase 7: GEO metrics expansion
        results.push(analyze_llms_txt(&llms_txt_body));
        results.push(analyze_ai_meta_directives(&html_doc));
        results.push(analyze_ai_header_directives(&response_headers));
        results.extend(analyze_citations(&html_doc));
        results.push(analyze_faq_quality(&html_doc));
        results.extend(analyze_entities(&html_doc));
        results.extend(analyze_query_optimization(&html_doc));
        results.push(analyze_freshness(&html_doc, &response_headers));
        results.push(analyze_howto_schema(&html_doc));

        // Agent-readiness checks (Cloudflare "Is it Agent Ready?" coverage)
        results.push(analyze_link_headers(&response_headers));
        results.push(analyze_markdown_negotiation(&agent_resources));
        results.push(analyze_content_signals(&robots_body));
        results.push(analyze_web_bot_auth(&agent_resources));
        results.push(analyze_mcp_server_card(&agent_resources));
        results.push(analyze_a2a_card(&agent_resources));
        results.push(analyze_agent_skills(&agent_resources));
        results.push(analyze_webmcp(&html_doc));
        results.push(analyze_api_catalog(&agent_resources));
        results.push(analyze_oauth_discovery(&agent_resources));
        results.push(analyze_oauth_protected_resource(&agent_resources));
        results.push(analyze_x402(client, &base_url, &html_doc).await);
        results.push(analyze_mpp(&agent_resources));
        results.push(analyze_ucp(&agent_resources));
        results.push(analyze_acp(&agent_resources));
        results.push(analyze_ap2(&agent_resources));

        // Core Web Vitals measurement — only when vitals is active
        if config.vitals {
            if let Some(vb) = vitals_browser {
                let vitals_fetch = async {
                    let vp = vb.new_page(page_url.as_str()).await?;
                    let vitals_results = analyze_vitals(&vp).await;
                    // Close the browser tab to prevent memory leaks —
                    // chromiumoxide::Page has no Drop impl, so tabs
                    // persist in the browser process unless explicitly closed.
                    if let Err(e) = vp.close().await {
                        tracing::warn!("Failed to close vitals page for {}: {}", page_url, e);
                    }
                    Ok::<_, chromiumoxide::error::CdpError>(vitals_results)
                };
                match tokio::time::timeout(BROWSER_PAGE_TIMEOUT, vitals_fetch).await {
                    Ok(Ok(vitals_results)) => results.extend(vitals_results),
                    Ok(Err(e)) => {
                        tracing::warn!("Vitals measurement failed for {}: {}", page_url, e);
                    }
                    Err(_) => {
                        tracing::warn!(
                            "Vitals measurement timed out for {} — skipping vitals for this page",
                            page_url
                        );
                    }
                }
            }
        }

        let (page_score, page_categories) = calculate_score(&results);

        pages.push(PageResult {
            url: norm.clone(),
            robots_blocked,
            score: page_score,
            categories: page_categories,
            results,
        });

        // Polite crawl delay between pages
        if page_num < total {
            sleep(Duration::from_secs(crawl_delay)).await;
        }
    }

    // Aggregate score across all pages
    let page_score_tuples: Vec<(f64, CategoryScores)> = pages
        .iter()
        .map(|p| {
            (
                p.score,
                CategoryScores {
                    technical: p.categories.technical,
                    content: p.categories.content,
                    geo: p.categories.geo,
                    agent: p.categories.agent,
                    performance: p.categories.performance,
                },
            )
        })
        .collect();
    let (agg_score, agg_categories) = aggregate_scores(&page_score_tuples);

    Ok(Report {
        schema_version: "1",
        url: url.to_string(),
        crawled_at: Utc::now().to_rfc3339(),
        score: agg_score,
        categories: agg_categories,
        pages,
    })
}

/// Fetch /llms.txt for the given base URL. Returns body or empty string on failure.
async fn fetch_llms_txt(client: &reqwest::Client, base_url: &Url) -> String {
    let mut llms_url = base_url.clone();
    llms_url.set_path("/llms.txt");
    llms_url.set_query(None);
    llms_url.set_fragment(None);

    let (body, _) = fetch_text_capped(client, llms_url.as_str(), MAX_BODY_BYTES).await;
    body
}

/// Fetch and check robots.txt for the given URL.
pub async fn check_robots(client: &reqwest::Client, page_url: &Url) -> (bool, String) {
    let mut robots_url = page_url.clone();
    robots_url.set_path("/robots.txt");
    robots_url.set_query(None);
    robots_url.set_fragment(None);

    let (body, _) = fetch_text_capped(client, robots_url.as_str(), MAX_BODY_BYTES).await;

    let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    let mut matcher = robotstxt::DefaultMatcher::default();
    let blocked = !matcher.one_agent_allowed_by_robots(&body, user_agent, page_url.as_str());
    (blocked, body)
}
