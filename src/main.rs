mod scoring;
mod analyzers;
mod crawling;

use anyhow::Result;
use chromiumoxide::{Browser, BrowserConfig};
use clap::Parser;
use chrono::Utc;
use futures::StreamExt;
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
use crate::crawling::{
    fetch_sitemap_urls, collect_links_bfs, normalize_url, is_html_url,
    extract_crawl_delay, needs_js_rendering, aggregate_scores,
    format_progress_known, format_progress_unknown,
};

#[derive(Parser)]
#[command(name = "geodaddy")]
#[command(about = "GEO analysis tool — surface actionable AI search optimization issues")]
#[command(version)]
struct Cli {
    /// URL to analyze (supports http://localhost and http://127.0.0.1)
    url: String,

    /// Exit with code 1 if overall score is below this threshold (0-100).
    #[arg(long, value_name = "SCORE")]
    fail_under: Option<f64>,

    /// Stop crawling after N pages (applies to both sitemap and link-following crawls).
    #[arg(long, value_name = "N")]
    max_pages: Option<usize>,

    /// Enable JavaScript rendering for pages detected as JS-heavy.
    /// Note: --enable-js downloads Chromium (~150MB) on first use.
    #[arg(long)]
    enable_js: bool,
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    url: String,               // base URL / site root per D-02
    crawled_at: String,
    score: f64,                // aggregate average across all pages per D-01
    categories: CategoryScores, // aggregate per D-01
    pages: Vec<PageResult>,
}

#[derive(Serialize)]
struct PageResult {
    url: String,
    robots_blocked: bool,
    score: f64,
    categories: CategoryScores,
    results: Vec<AnalysisResult>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Init tracing — MUST use stderr writer, not stdout
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::filter::EnvFilter::from_default_env()
        )
        .init();

    let cli = Cli::parse();

    let base_url = Url::parse(&cli.url)
        .map_err(|e| anyhow::anyhow!("Invalid URL '{}': {}", cli.url, e))?;

    // Build reqwest HTTP client with sensible defaults
    let client = reqwest::Client::builder()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")
        ))
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .build()?;

    // robots.txt fetched ONCE at crawl start (anti-pattern: re-fetching per URL)
    let (_, robots_body) = check_robots(&client, &base_url).await;
    let crawl_delay = extract_crawl_delay(&robots_body).unwrap_or(1);

    // Determine URL list — sitemap-first strategy (CRAWL-01 / CRAWL-02)
    // Track whether total is known upfront (sitemap) or unknown (link-following)
    let (urls, is_sitemap_driven): (Vec<String>, bool) =
        match fetch_sitemap_urls(&client, &base_url).await {
            Some(mut sitemap_urls) => {
                // Apply --max-pages cap to sitemap list (D-04)
                if let Some(max) = cli.max_pages {
                    sitemap_urls.truncate(max);
                }
                (sitemap_urls, true)
            }
            None => {
                // Sitemap unavailable — fall back to BFS depth 2 (D-05)
                tracing::info!("No sitemap.xml found — falling back to link-following (depth 2)");
                (collect_links_bfs(&client, &base_url, 2, cli.max_pages).await, false)
            }
        };

    let total = urls.len();

    // Optionally launch headless browser (D-10 — only when --enable-js)
    let browser: Option<Browser> = if cli.enable_js {
        let config = BrowserConfig::builder()
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build BrowserConfig: {}", e))?;
        let (b, mut handler) = Browser::launch(config).await?;
        // CRITICAL: handler MUST be spawned or browser hangs (pitfall 1)
        tokio::spawn(async move {
            while let Some(_event) = handler.next().await {}
        });
        Some(b)
    } else {
        None
    };

    let mut pages: Vec<PageResult> = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();

    for (idx, url_str) in urls.iter().enumerate() {
        // Deduplicate (D-06)
        let norm = normalize_url(url_str).unwrap_or_else(|| url_str.clone());
        if visited.contains(&norm) {
            continue;
        }
        visited.insert(norm.clone());

        // Skip non-HTML resources (sitemaps, feeds, media, etc.) — they produce
        // garbage GEO analysis results and should never appear in pages[].
        if !is_html_url(&norm) {
            tracing::debug!("Skipping non-HTML URL: {}", norm);
            continue;
        }

        // Progress to stderr (D-07/D-08, CLI-03)
        if is_sitemap_driven {
            eprintln!("{}", format_progress_known(idx + 1, total, url_str));
        } else {
            eprintln!("{}", format_progress_unknown(idx + 1, url_str));
        }

        // Check robots.txt per-URL using cached body
        let page_url = match Url::parse(&norm) {
            Ok(u) => u,
            Err(_) => {
                tracing::warn!("Skipping invalid URL: {}", norm);
                continue;
            }
        };
        let mut robots_matcher = robotstxt::DefaultMatcher::default();
        let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
        let robots_blocked = !robots_matcher.one_agent_allowed_by_robots(
            &robots_body,
            user_agent,
            page_url.as_str(),
        );

        // Fetch HTML — reqwest first, then headless if enabled and thin (D-09/D-10)
        let html_body = match client.get(page_url.as_str()).send().await {
            Ok(resp) if resp.status().is_success() => resp.text().await.unwrap_or_default(),
            Ok(resp) => {
                tracing::warn!("HTTP {} fetching {}", resp.status(), page_url);
                String::new()
            }
            Err(e) => {
                tracing::warn!("Failed to fetch {}: {}", page_url, e);
                String::new()
            }
        };

        let mut html_doc = Html::parse_document(&html_body);

        // JS detection and re-fetch (D-09) — only when --enable-js is active
        if cli.enable_js {
            if let Some(ref b) = browser {
                if needs_js_rendering(&html_doc) {
                    tracing::info!(
                        "JS-rendered page detected: {} — re-fetching via headless browser",
                        page_url
                    );
                    match b.new_page(page_url.as_str()).await {
                        Ok(page) => {
                            if let Ok(content) = page.content().await {
                                html_doc = Html::parse_document(&content);
                            }
                        }
                        Err(e) => tracing::warn!("Headless fetch failed for {}: {}", page_url, e),
                    }
                }
            }
        }

        // Run all analyzers (same sequence as single-page flow — unchanged)
        let mut results: Vec<AnalysisResult> = Vec::new();

        results.push(analyze_broken_links());
        results.push(analyze_redirect_chains(&client, &page_url).await);
        results.extend(analyze_meta_tags(&html_doc));
        results.push(analyze_headings_tech(&html_doc));
        results.push(analyze_mobile_viewport(&html_doc));
        results.push(analyze_robots_txt(&client, &page_url).await);
        results.push(analyze_sitemap(&client, &page_url).await);
        results.push(analyze_https(&html_doc, &page_url));
        results.push(analyze_heading_structure(&html_doc));
        results.push(analyze_json_ld(&html_doc));
        results.push(analyze_semantic_html(&html_doc));
        results.push(analyze_alt_text(&html_doc));
        results.push(analyze_listicle(&html_doc));
        results.extend(analyze_ai_bots(&robots_body));
        results.push(analyze_schema_stacking(&html_doc));

        let (page_score, page_categories) = calculate_score(&results);

        pages.push(PageResult {
            url: norm.clone(),
            robots_blocked,
            score: page_score,
            categories: page_categories,
            results,
        });

        // Polite crawl delay between pages — tokio::time::sleep, NOT std::thread::sleep
        if idx + 1 < urls.len() {
            sleep(Duration::from_secs(crawl_delay)).await;
        }
    }

    // Aggregate score (D-01) — average across all pages
    let page_score_tuples: Vec<(f64, CategoryScores)> = pages
        .iter()
        .map(|p| {
            (
                p.score,
                CategoryScores {
                    technical: p.categories.technical,
                    content: p.categories.content,
                    geo: p.categories.geo,
                },
            )
        })
        .collect();
    let (agg_score, agg_categories) = aggregate_scores(&page_score_tuples);

    // Build report — url is base URL per D-02
    let report = Report {
        schema_version: "1",
        url: cli.url.clone(),
        crawled_at: Utc::now().to_rfc3339(),
        score: agg_score,
        categories: agg_categories,
        pages,
    };

    // JSON to stdout — MUST happen before process::exit (CLI-01)
    println!("{}", serde_json::to_string_pretty(&report)?);

    // --fail-under compares against aggregate score, not per-page (pitfall 6)
    if let Some(threshold) = cli.fail_under {
        if agg_score < threshold {
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn check_robots(client: &reqwest::Client, page_url: &Url) -> (bool, String) {
    // Build robots.txt URL from origin — MUST use set_path(), not string concat
    let mut robots_url = page_url.clone();
    robots_url.set_path("/robots.txt");
    robots_url.set_query(None);
    robots_url.set_fragment(None);

    let body = match client.get(robots_url.as_str()).send().await {
        Ok(resp) if resp.status().is_success() => resp.text().await.unwrap_or_default(),
        // 404, network error, localhost with no robots.txt = allow all (D-03)
        _ => String::new(),
    };

    let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    let mut matcher = robotstxt::DefaultMatcher::default();
    let blocked = !matcher.one_agent_allowed_by_robots(&body, user_agent, page_url.as_str());
    (blocked, body)
}
