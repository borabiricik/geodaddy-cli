use anyhow::Result;
use chromiumoxide::{Browser, BrowserConfig};
use clap::{CommandFactory, Parser, Subcommand};
use futures::StreamExt;
use std::time::Duration;

use geodaddy::beauty::print_beauty_report;
use geodaddy::beauty::print_beauty_compare_report;
use geodaddy::compare;
use geodaddy::see;
use geodaddy::llms_txt;
use geodaddy::{AnalysisConfig, analyze};

#[derive(Parser)]
#[command(name = "geodaddy")]
#[command(about = "GEO analysis tool — surface actionable AI search optimization issues")]
#[command(version)]
struct Cli {
    /// URL to analyze (omit when using a subcommand like `compare`).
    url: Option<String>,

    /// Exit with code 1 if overall score is below this threshold (0-100).
    /// In `compare` mode, applies to the FIRST URL only.
    #[arg(long, value_name = "SCORE", global = true)]
    fail_under: Option<f64>,

    /// Enable crawling and stop after N pages. In `compare` mode, applied PER TARGET.
    #[arg(long, value_name = "N", global = true)]
    max_pages: Option<usize>,

    /// Enable JavaScript rendering for pages detected as JS-heavy.
    /// Note: --enable-js downloads Chromium (~150MB) on first use.
    #[arg(long, global = true)]
    enable_js: bool,

    /// Measure Core Web Vitals (LCP, FCP, CLS, TTFB, TBT) via headless browser for every page.
    /// Note: --vitals uses Chromium for measurement (~150MB download on first use).
    #[arg(long, global = true)]
    vitals: bool,

    /// Output a colored, human-readable report instead of JSON.
    #[arg(long, global = true)]
    beauty: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Compare 2+ URLs side-by-side with per-category diff and winner detection.
    /// Recommended maximum: ~10 URLs for readable --beauty output.
    Compare {
        /// URLs to compare (first URL treated as your site; rest as competitors).
        #[arg(num_args = 2..)]
        urls: Vec<String>,
    },

    /// Show what an AI crawler (GPTBot / ClaudeBot / PerplexityBot) sees when
    /// it visits the page. Defaults to JSON output; pass `--beauty` for the
    /// human-readable markdown view. Pass `--enable-js` to also compare
    /// against the JS-rendered view and surface content hidden from AI crawlers.
    See {
        /// URL to inspect.
        url: String,
    },

    /// Generate a spec-compliant llms.txt index from a site
    /// (per https://llmstxt.org). Crawls the site (sitemap-first, BFS
    /// fallback) and emits a markdown index of every page's title,
    /// URL, and description. Output goes to stdout; pass `-o <path>`
    /// to write to a file. Note: this command's output IS the
    /// llms.txt body — `--beauty` and JSON formatting do not apply.
    /// The crawl size is bounded by the global `--max-pages` flag
    /// (default 50 when omitted).
    LlmsTxt {
        /// URL to crawl.
        url: String,
        /// Write output to this file instead of stdout.
        #[arg(short = 'o', long = "output", value_name = "PATH")]
        output: Option<std::path::PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Init tracing — MUST use stderr writer, not stdout
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("RUST_LOG")
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("geodaddy=info")),
        )
        .init();

    let cli = Cli::parse();

    // Dispatch: subcommand > positional URL > help+exit(2)
    match (&cli.command, &cli.url) {
        (Some(Commands::Compare { urls }), _) => {
            run_compare_flow(&cli, urls).await?;
        }
        (Some(Commands::See { url }), _) => {
            run_see_flow(&cli, url).await?;
        }
        (Some(Commands::LlmsTxt { url, output }), _) => {
            run_llms_txt_flow(cli.max_pages, url, output.as_ref()).await?;
        }
        (None, Some(url)) => {
            run_analyze_flow(&cli, url).await?;
        }
        (None, None) => {
            Cli::command().print_help().ok();
            eprintln!();
            std::process::exit(2);
        }
    }

    Ok(())
}

/// Single-URL analyze flow — extracted from main() to keep the subcommand
/// dispatch readable. Wave 1's `compare` flow will parallel this with a loop
/// wrapper around `compare_sites()`.
async fn run_analyze_flow(cli: &Cli, url: &str) -> Result<()> {
    // Build reqwest HTTP client with sensible defaults
    let client = reqwest::Client::builder()
        .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .build()?;

    let config = AnalysisConfig {
        max_pages: cli.max_pages,
        enable_js: cli.enable_js,
        vitals: cli.vitals,
    };

    // Optionally launch headless browser (only when --enable-js)
    let js_data_dir =
        std::env::temp_dir().join(format!("geodaddy-js-{}", std::process::id()));
    let browser: Option<Browser> = if cli.enable_js {
        let mut builder = BrowserConfig::builder();
        if let Ok(path) = std::env::var("CHROME_PATH") {
            builder = builder.chrome_executable(path);
        }
        // Unique user-data-dir per invocation to avoid SingletonLock conflicts
        builder = builder.no_sandbox().user_data_dir(js_data_dir.clone());
        let config = builder
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

    // Optionally launch dedicated vitals browser (independent from --enable-js)
    let vitals_data_dir =
        std::env::temp_dir().join(format!("geodaddy-vitals-{}", std::process::id()));
    let vitals_browser: Option<Browser> = if cli.vitals {
        let mut builder = BrowserConfig::builder();
        if let Ok(path) = std::env::var("CHROME_PATH") {
            builder = builder.chrome_executable(path);
        }
        builder = builder.no_sandbox().user_data_dir(vitals_data_dir.clone());
        let config = builder
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build vitals BrowserConfig: {}", e))?;
        let (b, mut handler) = Browser::launch(config).await?;
        tokio::spawn(async move {
            while let Some(_event) = handler.next().await {}
        });
        Some(b)
    } else {
        None
    };

    let report = analyze(
        url,
        &config,
        &client,
        browser.as_ref(),
        vitals_browser.as_ref(),
    )
    .await?;

    // Output — beauty mode or JSON
    if cli.beauty {
        print_beauty_report(&report);
    } else {
        println!("{}", serde_json::to_string_pretty(&report)?);
    }

    // Clean up per-process browser data dirs to avoid disk buildup
    if cli.enable_js {
        let _ = std::fs::remove_dir_all(&js_data_dir);
    }
    if cli.vitals {
        let _ = std::fs::remove_dir_all(&vitals_data_dir);
    }

    // --fail-under compares against aggregate score
    if let Some(threshold) = cli.fail_under {
        if report.score < threshold {
            std::process::exit(1);
        }
    }

    Ok(())
}

/// `geodaddy see <url>` — render the page as an AI crawler sees it.
///
/// Without `--enable-js`: fetches via reqwest and outputs markdown + stats.
/// With `--enable-js`: also fetches via headless Chromium and shows the diff
/// (content hidden from JS-free AI crawlers).
///
/// Output format matches the project convention: JSON by default, markdown
/// when `--beauty` is passed.
async fn run_see_flow(cli: &Cli, url: &str) -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .build()?;

    // Browser is only launched when --enable-js is passed — otherwise we stay
    // in "pure HTTP, zero chromium download" mode matching the command's intent.
    let js_data_dir =
        std::env::temp_dir().join(format!("geodaddy-see-{}", std::process::id()));
    let browser: Option<Browser> = if cli.enable_js {
        let mut builder = BrowserConfig::builder();
        if let Ok(path) = std::env::var("CHROME_PATH") {
            builder = builder.chrome_executable(path);
        }
        builder = builder.no_sandbox().user_data_dir(js_data_dir.clone());
        let config = builder
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build BrowserConfig: {}", e))?;
        let (b, mut handler) = Browser::launch(config).await?;
        tokio::spawn(async move {
            while let Some(_event) = handler.next().await {}
        });
        Some(b)
    } else {
        None
    };

    // Convention match: JSON by default, markdown when --beauty is set.
    see::run_see(url, cli.beauty, &client, browser.as_ref()).await?;

    if cli.enable_js {
        let _ = std::fs::remove_dir_all(&js_data_dir);
    }

    Ok(())
}

/// `geodaddy llms-txt <url>` — generate a spec-compliant llms.txt
/// index for the site. Reuses the existing reqwest client config.
/// Resolution: global --max-pages > DEFAULT_MAX_PAGES.
///
/// Note: a local `--max-pages` was originally specified in the plan but
/// removed during execution because clap forbids defining the same long
/// option name twice (global on `Cli` and local on the `LlmsTxt` variant).
/// The global flag (with `global = true`) already appears in
/// `geodaddy llms-txt --help`, so user-facing behavior is unchanged.
async fn run_llms_txt_flow(
    global_max: Option<usize>,
    url: &str,
    output: Option<&std::path::PathBuf>,
) -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .build()?;
    let max_pages = global_max.unwrap_or(llms_txt::DEFAULT_MAX_PAGES);
    llms_txt::run_llms_txt(url, output, max_pages, &client).await
}

/// Multi-URL compare flow — sequentially analyzes each URL sharing a single
/// reqwest::Client + optional headless browsers, renders the result as JSON
/// (or a side-by-side colored table when --beauty is passed via
/// beauty::print_beauty_compare_report), and enforces the
/// first-URL-centric exit-code policy from 08-CONTEXT.md:
///   exit 2 if the first URL failed to analyze
///   exit 1 if the first URL's score is below --fail-under
///   exit 0 otherwise (competitor failures are informational)
async fn run_compare_flow(cli: &Cli, urls: &[String]) -> Result<()> {
    if urls.len() < 2 {
        // Defensive — clap should already enforce num_args = 2.., but double-check.
        anyhow::bail!("compare requires at least 2 URLs");
    }

    // Build shared reqwest client (same config as single-URL path).
    let client = reqwest::Client::builder()
        .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .build()?;

    let config = AnalysisConfig {
        max_pages: cli.max_pages,
        enable_js: cli.enable_js,
        vitals: cli.vitals,
    };

    // Optional browsers — shared across all targets. Same launch pattern as run_analyze_flow.
    let js_data_dir =
        std::env::temp_dir().join(format!("geodaddy-js-{}", std::process::id()));
    let js_browser: Option<Browser> = if cli.enable_js {
        let mut builder = BrowserConfig::builder();
        if let Ok(path) = std::env::var("CHROME_PATH") {
            builder = builder.chrome_executable(path);
        }
        builder = builder.no_sandbox().user_data_dir(js_data_dir.clone());
        let config = builder
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build BrowserConfig: {}", e))?;
        let (b, mut handler) = Browser::launch(config).await?;
        tokio::spawn(async move {
            while let Some(_event) = handler.next().await {}
        });
        Some(b)
    } else {
        None
    };

    let vitals_data_dir =
        std::env::temp_dir().join(format!("geodaddy-vitals-{}", std::process::id()));
    let vitals_browser: Option<Browser> = if cli.vitals {
        let mut builder = BrowserConfig::builder();
        if let Ok(path) = std::env::var("CHROME_PATH") {
            builder = builder.chrome_executable(path);
        }
        builder = builder.no_sandbox().user_data_dir(vitals_data_dir.clone());
        let config = builder
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build vitals BrowserConfig: {}", e))?;
        let (b, mut handler) = Browser::launch(config).await?;
        tokio::spawn(async move {
            while let Some(_event) = handler.next().await {}
        });
        Some(b)
    } else {
        None
    };

    // Remember first URL (post-clap, pre-dedup) for exit-code policy.
    let first_url = urls[0].clone();

    let report = compare::compare_sites(
        urls,
        &config,
        &client,
        js_browser.as_ref(),
        vitals_browser.as_ref(),
    )
    .await;

    // Render output.
    if cli.beauty {
        print_beauty_compare_report(&report);
    } else {
        println!("{}", serde_json::to_string_pretty(&report)?);
    }

    // Cleanup temp browser dirs.
    if cli.enable_js {
        let _ = std::fs::remove_dir_all(&js_data_dir);
    }
    if cli.vitals {
        let _ = std::fs::remove_dir_all(&vitals_data_dir);
    }

    // Exit-code policy (CONTEXT lines 93-99):
    //   exit 2 if first URL failed (surfaced in errors[])
    //   exit 1 if first URL's score < --fail-under
    //   exit 0 otherwise (competitor failures are informational)
    let first_failed = report.errors.iter().any(|e| e.url == first_url);
    if first_failed {
        eprintln!("First URL failed to analyze; cannot evaluate --fail-under.");
        std::process::exit(2);
    }

    if let Some(threshold) = cli.fail_under {
        // Dedup preserves first occurrence, so sites[0].url should match first_url
        // in the common case. Fall back to sites[0] if not found.
        let first_site = report.sites.iter().find(|s| s.url == first_url);
        match first_site {
            Some(s) if s.score < threshold => std::process::exit(1),
            None => {
                if let Some(s0) = report.sites.first() {
                    if s0.score < threshold {
                        std::process::exit(1);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(())
}
