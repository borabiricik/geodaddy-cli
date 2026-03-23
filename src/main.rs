mod scoring;
mod analyzers;

use anyhow::Result;
use clap::Parser;
use chrono::Utc;
use serde::Serialize;
use url::Url;
use std::time::Duration;
use scraper::Html;

use crate::scoring::{AnalysisResult, CategoryScores, calculate_score};
use crate::analyzers::technical::{
    analyze_broken_links, analyze_meta_tags, analyze_headings_tech,
    analyze_mobile_viewport, analyze_https,
    analyze_redirect_chains, analyze_robots_txt, analyze_sitemap,
};
use crate::analyzers::content::{
    analyze_heading_structure, analyze_json_ld, analyze_semantic_html, analyze_alt_text,
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
}

#[derive(Serialize)]
struct Report {
    schema_version: &'static str,
    url: String,
    crawled_at: String,
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
    // Step 1: Init tracing — MUST use stderr writer, not stdout
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::filter::EnvFilter::from_default_env()
        )
        .init();

    // Step 2: Parse CLI args (clap handles --help and --version automatically)
    let cli = Cli::parse();

    // Step 3: Normalize URL via WHATWG url crate (CRAWL-03 — handles localhost)
    let normalized_url = Url::parse(&cli.url)
        .map_err(|e| anyhow::anyhow!("Invalid URL '{}': {}", cli.url, e))?;

    // Step 4: Build reqwest HTTP client with sensible defaults
    let client = reqwest::Client::builder()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")
        ))
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .build()?;

    // Step 5: robots.txt soft-warn check (D-03 — CRAWL-05)
    let robots_blocked = check_robots(&client, &normalized_url).await;

    // Step 5b: Fetch page HTML for analysis
    let html_body = match client.get(normalized_url.as_str()).send().await {
        Ok(resp) if resp.status().is_success() => resp.text().await.unwrap_or_default(),
        Ok(resp) => {
            tracing::warn!("HTTP {} fetching {}", resp.status(), normalized_url);
            String::new()
        }
        Err(e) => {
            tracing::warn!("Failed to fetch {}: {}", normalized_url, e);
            String::new()
        }
    };
    let html_doc = Html::parse_document(&html_body);

    // Step 5c: Run all analyzers and collect results
    let mut results: Vec<AnalysisResult> = Vec::new();

    // TECH-01: broken links stub (no HTML needed)
    results.push(analyze_broken_links());

    // TECH-02: redirect chains (async, needs client + url)
    results.push(analyze_redirect_chains(&client, &normalized_url).await);

    // TECH-03: meta tags (HTML) — returns 2 results (title + description)
    results.extend(analyze_meta_tags(&html_doc));

    // TECH-04: heading H1 check (HTML)
    results.push(analyze_headings_tech(&html_doc));

    // TECH-05: mobile viewport (HTML)
    results.push(analyze_mobile_viewport(&html_doc));

    // TECH-06: robots.txt (async, needs client + url)
    results.push(analyze_robots_txt(&client, &normalized_url).await);

    // TECH-07: sitemap.xml (async, needs client + url)
    results.push(analyze_sitemap(&client, &normalized_url).await);

    // TECH-08: HTTPS + mixed content (HTML + url)
    results.push(analyze_https(&html_doc, &normalized_url));

    // CONT-01: heading hierarchy (HTML)
    results.push(analyze_heading_structure(&html_doc));

    // CONT-02: JSON-LD schema (HTML)
    results.push(analyze_json_ld(&html_doc));

    // CONT-03: semantic HTML (HTML)
    results.push(analyze_semantic_html(&html_doc));

    // CONT-04: alt text (HTML)
    results.push(analyze_alt_text(&html_doc));

    // Step 5d: Calculate scores
    let (overall_score, category_scores) = calculate_score(&results);

    // Step 6: Build report (D-02 schema)
    let report = Report {
        schema_version: "1",
        url: cli.url.clone(),
        crawled_at: Utc::now().to_rfc3339(),
        pages: vec![PageResult {
            url: normalized_url.to_string(),
            robots_blocked,
            score: overall_score,
            categories: category_scores,
            results,
        }],
    };

    // Step 7: Output JSON to stdout — MUST be printed before process::exit
    println!("{}", serde_json::to_string_pretty(&report)?);

    // Step 8: Apply --fail-under exit code (CLI-02)
    if let Some(threshold) = cli.fail_under {
        if overall_score < threshold {
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn check_robots(client: &reqwest::Client, page_url: &Url) -> bool {
    // Build robots.txt URL from origin — MUST use set_path(), not string concat
    // (avoids pitfall of appending to non-root paths like /blog/post/1)
    let mut robots_url = page_url.clone();
    robots_url.set_path("/robots.txt");
    robots_url.set_query(None);
    robots_url.set_fragment(None);

    let body = match client.get(robots_url.as_str()).send().await {
        Ok(resp) if resp.status().is_success() => {
            resp.text().await.unwrap_or_default()
        }
        // 404, network error, localhost with no robots.txt = allow all (D-03)
        _ => String::new(),
    };

    let user_agent = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
    let mut matcher = robotstxt::DefaultMatcher::default();
    // Returns true if BLOCKED (inverted from allowed)
    !matcher.one_agent_allowed_by_robots(&body, user_agent, page_url.as_str())
}
