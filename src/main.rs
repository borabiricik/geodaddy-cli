use anyhow::Result;
use chromiumoxide::{Browser, BrowserConfig};
use clap::{CommandFactory, Parser, Subcommand};
use futures::StreamExt;
use std::time::Duration;

use geodaddy::beauty::print_beauty_report;
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
    #[arg(long, value_name = "SCORE")]
    fail_under: Option<f64>,

    /// Enable crawling and stop after N pages. In `compare` mode, applied PER TARGET.
    #[arg(long, value_name = "N")]
    max_pages: Option<usize>,

    /// Enable JavaScript rendering for pages detected as JS-heavy.
    /// Note: --enable-js downloads Chromium (~150MB) on first use.
    #[arg(long)]
    enable_js: bool,

    /// Measure Core Web Vitals (LCP, FCP, CLS, TTFB, TBT) via headless browser for every page.
    /// Note: --vitals uses Chromium for measurement (~150MB download on first use).
    #[arg(long)]
    vitals: bool,

    /// Output a colored, human-readable report instead of JSON.
    #[arg(long)]
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
        (Some(Commands::Compare { urls: _ }), _) => {
            // Wave 0 stub: full compare flow lands in Wave 1 (08-02-PLAN)
            eprintln!("compare subcommand not yet implemented (Wave 0 stub)");
            std::process::exit(2);
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
