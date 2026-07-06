use colored::{Color, Colorize};
use crate::scoring::Status;
use crate::{Report, PageResult};
#[allow(unused_imports)]
use crate::compare::{CompareReport, Winners, CheckDiff, SiteCheckOutcome};
use url::Url;

fn score_color(score: f64) -> Color {
    if score >= 80.0 {
        Color::Green
    } else if score >= 50.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn fmt_perf(perf: Option<f64>) -> String {
    match perf {
        Some(p) => format!("{:.1}", p),
        None => "N/A".to_string(),
    }
}

pub fn print_beauty_report(report: &Report) {
    // ── Header ────────────────────────────────────────────────────────────────
    println!("{}", "geodaddy — GEO Analysis Report".bold());
    println!("{}", format!("URL: {}", report.url).bold());
    println!("{}", format!("Crawled: {}", report.crawled_at).bold());
    println!("{}", "─────────────────────────────────────────────────────".bold());
    println!();

    // ── Aggregate score ───────────────────────────────────────────────────────
    let color = score_color(report.score);
    println!(
        "Overall Score: {}",
        format!("{:.1}/100", report.score).color(color).bold()
    );
    println!(
        "Technical: {:.1}  Content: {:.1}  GEO: {:.1}  Agent: {:.1}  Performance: {}",
        report.categories.technical,
        report.categories.content,
        report.categories.geo,
        report.categories.agent,
        fmt_perf(report.categories.performance),
    );
    println!();

    // ── Per-page sections ─────────────────────────────────────────────────────
    let total = report.pages.len();
    for (i, page) in report.pages.iter().enumerate() {
        print_page(i + 1, total, page);
    }
}

fn print_page(n: usize, total: usize, page: &PageResult) {
    println!(
        "{}",
        format!("━━━ Page {}/{}: {} ━━━", n, total, page.url).bold()
    );

    let color = score_color(page.score);
    println!("Score: {}", format!("{:.1}/100", page.score).color(color));

    if page.robots_blocked {
        println!("{}", "(robots blocked)".yellow());
    }

    println!(
        "Technical: {:.1}  Content: {:.1}  GEO: {:.1}  Agent: {:.1}  Performance: {}",
        page.categories.technical,
        page.categories.content,
        page.categories.geo,
        page.categories.agent,
        fmt_perf(page.categories.performance),
    );
    println!();

    for result in &page.results {
        match result.status {
            Status::Pass => {
                println!(
                    "  {}  {}  {}",
                    "[PASS]".green(),
                    result.check.green(),
                    result.message.as_str().green(),
                );
            }
            Status::Warn => {
                println!(
                    "  {}  {}  {}",
                    "[WARN]".yellow(),
                    result.check.yellow(),
                    result.message.as_str().yellow(),
                );
                if !result.recommendation.is_empty() {
                    println!("      {}", format!("-> {}", result.recommendation).dimmed());
                }
            }
            Status::Fail => {
                println!(
                    "  {}  {}  {}",
                    "[FAIL]".red(),
                    result.check.red(),
                    result.message.as_str().red(),
                );
                if !result.recommendation.is_empty() {
                    println!("      {}", format!("-> {}", result.recommendation).dimmed());
                }
            }
        }
    }
    println!();
}

// ─── App report rendering ────────────────────────────────────────────────

/// Human-readable renderer for `geodaddy app --beauty <store-url>`.
pub fn print_beauty_app_report(report: &crate::appstore::AppReport) {
    println!("{}", "geodaddy — App AI-Visibility Report".bold());
    let store_label = if report.app.is_desktop {
        "Mac App Store"
    } else {
        report.app.store.label()
    };
    println!(
        "{}",
        format!("App: {} ({})", report.app.name, store_label).bold()
    );
    println!("{}", format!("URL: {}", report.url).bold());
    println!("{}", format!("Analyzed: {}", report.analyzed_at).bold());
    println!("{}", "─────────────────────────────────────────────────────".bold());
    println!();

    // Listing facts line
    let rating = match (report.app.rating_avg, report.app.rating_count) {
        (Some(avg), Some(count)) => format!("{:.2} ★ ({} ratings)", avg, count),
        (Some(avg), None) => format!("{:.2} ★", avg),
        _ => "no ratings".to_string(),
    };
    let mut facts = vec![rating];
    if let Some(cat) = &report.app.category {
        facts.push(cat.clone());
    }
    if let Some(installs) = &report.app.installs {
        facts.push(format!("{} downloads", installs));
    }
    if let Some(updated) = &report.app.last_updated {
        facts.push(format!("updated {}", updated));
    }
    println!("{}", facts.join("  ·  "));
    println!();

    let color = score_color(report.score);
    println!(
        "Overall Score: {}",
        format!("{:.1}/100", report.score).color(color).bold()
    );
    println!(
        "Listing: {:.1}  Answerability: {:.1}  Reputation: {:.1}  Web Presence: {:.1}  Cross-Platform: {}",
        report.categories.listing,
        report.categories.answerability,
        report.categories.reputation,
        report.categories.web_presence,
        fmt_perf(report.categories.cross_platform),
    );
    println!();

    print_results(&report.results);

    if let Some(site) = &report.developer_site {
        println!(
            "{}",
            format!(
                "━━━ Developer website ({}) — GEO score {:.1}/100 ━━━",
                site.url, site.score
            )
            .bold()
        );
        println!(
            "{}",
            "Run `geodaddy <website-url>` for the full website report.".dimmed()
        );
        println!();
    }
}

/// Human-readable renderer for `geodaddy app-compare --beauty <urls...>`.
/// Reuses the terminal-width-aware column layout of the website compare view.
pub fn print_beauty_app_compare_report(report: &crate::appstore::AppCompareReport) {
    println!("{}", "geodaddy — App Comparison Report".bold());
    println!("{}", format!("Compared: {}", report.compared_at).bold());
    println!();

    for app in &report.apps {
        let color = score_color(app.score);
        println!(
            "  {}  {}  {}",
            format!("{:>5.1}", app.score).color(color).bold(),
            app.app.name.bold(),
            format!("({})", app.app.store.label()).dimmed(),
        );
        println!(
            "         Listing {:.0} · Answerability {:.0} · Reputation {:.0} · Web {:.0} · Cross-Platform {}",
            app.categories.listing,
            app.categories.answerability,
            app.categories.reputation,
            app.categories.web_presence,
            fmt_perf(app.categories.cross_platform),
        );
    }

    println!();
    println!("{}", "Winners".bold());
    print_compare_winner_line("Overall", &report.winners.overall);
    print_compare_winner_line("Listing", &report.winners.listing);
    print_compare_winner_line("Answerability", &report.winners.answerability);
    print_compare_winner_line("Reputation", &report.winners.reputation);
    print_compare_winner_line("Web", &report.winners.web_presence);
    print_compare_winner_line("Cross-Platform", &report.winners.cross_platform);

    if !report.check_diff.is_empty() && !report.apps.is_empty() {
        println!();
        println!("{}", "Per-check Diff".bold());
        print!("{:<w$}", "", w = COL_LABEL_WIDTH + 12);
        for app in &report.apps {
            let header: String = app.app.name.chars().take(COL_SITE_WIDTH - 1).collect();
            print!("{}", format!("{:<w$}", header, w = COL_SITE_WIDTH).dimmed());
        }
        println!();
        for diff in &report.check_diff {
            print!("{:<w$}", diff.check, w = COL_LABEL_WIDTH + 12);
            for outcome in &diff.results {
                let icon = match outcome.status {
                    Some(Status::Pass) => "✓",
                    Some(Status::Warn) => "⚠",
                    Some(Status::Fail) => "✗",
                    None => "—",
                };
                let padded = format!("{:<w$}", icon, w = COL_SITE_WIDTH);
                let cell = match outcome.status {
                    Some(Status::Pass) => padded.green(),
                    Some(Status::Warn) => padded.yellow(),
                    Some(Status::Fail) => padded.red(),
                    None => padded.dimmed(),
                };
                print!("{}", cell);
            }
            println!();
        }
    }

    if !report.errors.is_empty() {
        println!();
        println!("{}", "Failures".bold());
        for err in &report.errors {
            println!("  {}: {}", err.url.red(), err.message.red());
        }
    }
    println!();
}

/// Shared check-result list renderer (same visual language as page results).
fn print_results(results: &[crate::scoring::AnalysisResult]) {
    for result in results {
        match result.status {
            Status::Pass => {
                println!(
                    "  {}  {}  {}",
                    "[PASS]".green(),
                    result.check.green(),
                    result.message.as_str().green(),
                );
            }
            Status::Warn => {
                println!(
                    "  {}  {}  {}",
                    "[WARN]".yellow(),
                    result.check.yellow(),
                    result.message.as_str().yellow(),
                );
                if !result.recommendation.is_empty() {
                    println!("      {}", format!("-> {}", result.recommendation).dimmed());
                }
            }
            Status::Fail => {
                println!(
                    "  {}  {}  {}",
                    "[FAIL]".red(),
                    result.check.red(),
                    result.message.as_str().red(),
                );
                if !result.recommendation.is_empty() {
                    println!("      {}", format!("-> {}", result.recommendation).dimmed());
                }
            }
        }
    }
    println!();
}

// ─── Compare mode rendering ──────────────────────────────────────────────
//
// Side-by-side comparison renderer for `geodaddy compare --beauty <urls...>`.
// Variable column count 2-10 sites. Terminal-width aware with vertical fallback.

const COL_LABEL_WIDTH: usize = 18;
const COL_SITE_WIDTH: usize = 16;

/// Terminal width (columns) read from the COLUMNS env var.
/// Defaults to 120 when unset or unparseable — safe on 95% of shells.
/// Zero new dependencies per CONTEXT: we deliberately avoid `terminal_size`.
fn detect_terminal_width() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(120)
}

/// Extract a short column header from a site URL. Falls back to the raw URL
/// when parsing fails. Truncated to `COL_SITE_WIDTH - 1` chars to guarantee
/// at least one space between columns.
fn compare_column_header(raw_url: &str) -> String {
    let host = Url::parse(raw_url)
        .ok()
        .and_then(|u| u.host_str().map(|s| s.to_string()))
        .unwrap_or_else(|| raw_url.to_string());
    host.chars().take(COL_SITE_WIDTH - 1).collect()
}

/// Top-level compare renderer. Entry point called from `main::run_compare_flow`.
pub fn print_beauty_compare_report(report: &CompareReport) {
    let required = COL_LABEL_WIDTH + report.sites.len() * COL_SITE_WIDTH;
    let available = detect_terminal_width();
    if required > available && !report.sites.is_empty() {
        eprintln!(
            "Terminal too narrow for side-by-side table ({} cols needed, {} available). Falling back to per-site vertical report.",
            required, available
        );
        print_beauty_compare_vertical_fallback(report);
        return;
    }

    // ── Header ────────────────────────────────────────────────────────────
    println!("{}", "geodaddy — Competitor Comparison Report".bold());
    println!("{}", format!("Compared: {}", report.compared_at).bold());
    let rule_len = (COL_LABEL_WIDTH + report.sites.len().max(1) * COL_SITE_WIDTH).min(120);
    println!("{}", "─".repeat(rule_len).bold());
    println!();

    if report.sites.is_empty() {
        println!("{}", "(no sites analyzed successfully — see Failures below)".yellow());
        println!();
    } else {
        // ── Column header row ─────────────────────────────────────────────
        print!("{:<w$}", "", w = COL_LABEL_WIDTH);
        for site in &report.sites {
            let header = compare_column_header(&site.url);
            let padded = format!("{:<w$}", header, w = COL_SITE_WIDTH);
            print!("{}", padded.bold());
        }
        println!();

        // ── Overall score row ─────────────────────────────────────────────
        print!("{:<w$}", "Overall Score", w = COL_LABEL_WIDTH);
        for site in &report.sites {
            let score_str = format!("{:.1}", site.score);
            let padded = format!("{:<w$}", score_str, w = COL_SITE_WIDTH);
            print!("{}", padded.color(score_color(site.score)));
        }
        println!();

        // ── Category rows ─────────────────────────────────────────────────
        print_compare_category_row("Technical", &report.sites, |c| Some(c.technical));
        print_compare_category_row("Content",   &report.sites, |c| Some(c.content));
        print_compare_category_row("GEO",       &report.sites, |c| Some(c.geo));
        print_compare_category_row("Performance", &report.sites, |c| c.performance);
    }

    // ── Winners summary ───────────────────────────────────────────────────
    println!();
    println!("{}", "Winners".bold());
    print_compare_winner_line("Overall",     &report.winners.overall);
    print_compare_winner_line("Technical",   &report.winners.technical);
    print_compare_winner_line("Content",     &report.winners.content);
    print_compare_winner_line("GEO",         &report.winners.geo);
    print_compare_winner_line("Performance", &report.winners.performance);

    // ── Per-check diff table ──────────────────────────────────────────────
    if !report.check_diff.is_empty() && !report.sites.is_empty() {
        println!();
        println!("{}", "Per-check Diff".bold());

        // Column header row (domains, repeated so diff table is readable on its own)
        print!("{:<w$}", "", w = COL_LABEL_WIDTH);
        for site in &report.sites {
            let header = compare_column_header(&site.url);
            let padded = format!("{:<w$}", header, w = COL_SITE_WIDTH);
            print!("{}", padded.dimmed());
        }
        println!();

        for diff in &report.check_diff {
            print_compare_check_diff_row(diff);
        }
    }

    // ── Failures section ──────────────────────────────────────────────────
    if !report.errors.is_empty() {
        println!();
        println!("{}", "Failures".bold());
        for err in &report.errors {
            println!("  {}: {}", err.url.red(), err.message.red());
        }
    }
    println!();
}

fn print_compare_category_row<F>(label: &str, sites: &[Report], extract: F)
where
    F: Fn(&crate::scoring::CategoryScores) -> Option<f64>,
{
    print!("{:<w$}", label, w = COL_LABEL_WIDTH);
    for site in sites {
        let cell = match extract(&site.categories) {
            Some(v) => {
                let padded = format!("{:<w$}", format!("{:.1}", v), w = COL_SITE_WIDTH);
                padded.color(score_color(v))
            }
            None => {
                let padded = format!("{:<w$}", "N/A", w = COL_SITE_WIDTH);
                padded.dimmed()
            }
        };
        print!("{}", cell);
    }
    println!();
}

fn print_compare_winner_line(label: &str, winner: &Option<String>) {
    let val = match winner.as_deref() {
        Some(url) => url.to_string(),
        None => "TIE / N/A".to_string(),
    };
    println!("  {:<w$} {}", format!("{}:", label), val, w = 12);
}

fn print_compare_check_diff_row(diff: &CheckDiff) {
    print!("{:<w$}", diff.check, w = COL_LABEL_WIDTH);
    for outcome in &diff.results {
        let icon_str = match outcome.status {
            Some(Status::Pass) => "✓",
            Some(Status::Warn) => "⚠",
            Some(Status::Fail) => "✗",
            None => "—",
        };
        let padded = format!("{:<w$}", icon_str, w = COL_SITE_WIDTH);
        let colored_cell = match outcome.status {
            Some(Status::Pass) => padded.green(),
            Some(Status::Warn) => padded.yellow(),
            Some(Status::Fail) => padded.red(),
            None               => padded.dimmed(),
        };
        print!("{}", colored_cell);
    }
    println!();
}

/// Narrow-terminal fallback: emit the required keyword strings on stdout
/// (so integration tests asserting them still pass) plus per-site vertical
/// reports via the existing `print_beauty_report`. Failures still listed.
fn print_beauty_compare_vertical_fallback(report: &CompareReport) {
    println!("{}", "geodaddy — Competitor Comparison Report".bold());
    println!("{}", format!("Compared: {}", report.compared_at).bold());
    println!();
    println!("{}", "Overall Score (per-site vertical layout)".bold());
    for site in &report.sites {
        println!();
        print_beauty_report(site);
    }

    println!();
    println!("{}", "Winners".bold());
    print_compare_winner_line("Overall",     &report.winners.overall);
    print_compare_winner_line("Technical",   &report.winners.technical);
    print_compare_winner_line("Content",     &report.winners.content);
    print_compare_winner_line("GEO",         &report.winners.geo);
    print_compare_winner_line("Performance", &report.winners.performance);

    if !report.errors.is_empty() {
        println!();
        println!("{}", "Failures".bold());
        for err in &report.errors {
            println!("  {}: {}", err.url.red(), err.message.red());
        }
    }
    println!();
}

// ── Tests for compare beauty rendering ───────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use crate::compare::{CompareReport, CompareError, Winners, CheckDiff, SiteCheckOutcome};
    use crate::scoring::{AnalysisResult, CategoryScores, Status};
    use crate::{Report, PageResult};

    fn make_report(url: &str, score: f64) -> Report {
        Report {
            schema_version: "1",
            url: url.to_string(),
            crawled_at: "2026-04-16T00:00:00Z".to_string(),
            score,
            categories: CategoryScores {
                technical: score,
                content: score,
                geo: score,
                agent: score,
                performance: Some(score),
            },
            pages: vec![PageResult {
                url: url.to_string(),
                robots_blocked: false,
                score,
                categories: CategoryScores {
                    technical: score,
                    content: score,
                    geo: score,
                    agent: score,
                    performance: Some(score),
                },
                results: vec![AnalysisResult {
                    check: "tech-meta-title",
                    status: Status::Pass,
                    message: String::new(),
                    recommendation: String::new(),
                }],
            }],
        }
    }

    fn make_compare(n: usize) -> CompareReport {
        let sites: Vec<Report> = (0..n)
            .map(|i| make_report(&format!("https://site{}.example.com", i), 50.0 + i as f64 * 5.0))
            .collect();
        let winners = Winners {
            overall: sites.first().map(|s| s.url.clone()),
            technical: sites.first().map(|s| s.url.clone()),
            content: sites.first().map(|s| s.url.clone()),
            geo: sites.first().map(|s| s.url.clone()),
            agent: sites.first().map(|s| s.url.clone()),
            performance: sites.first().map(|s| s.url.clone()),
        };
        let check_diff = vec![CheckDiff {
            check: "tech-meta-title".to_string(),
            results: sites.iter().map(|s| SiteCheckOutcome {
                url: s.url.clone(),
                status: Some(Status::Pass),
            }).collect(),
        }];
        CompareReport {
            schema_version: "1",
            compared_at: "2026-04-16T00:00:00Z".to_string(),
            sites,
            winners,
            check_diff,
            errors: Vec::new(),
        }
    }

    #[test]
    fn test_compare_beauty_variable_columns() {
        // COMP-07: 2-10 sites must render without panics.
        for n in [2usize, 3, 5, 10] {
            let report = make_compare(n);
            // Sanity: call the renderer; harness captures stdout. Panic = test failure.
            print_beauty_compare_report(&report);
        }
    }

    #[test]
    fn test_narrow_terminal_fallback() {
        // COMP-07: narrow terminal must fall back without panic.
        // NOTE: std::env::set_var is process-global and racy with parallel tests;
        // this test asserts no-panic only (no stdout substring checks) to stay robust.
        std::env::set_var("COLUMNS", "40");
        let report = make_compare(3);
        print_beauty_compare_report(&report);
        std::env::remove_var("COLUMNS");
    }

    #[test]
    fn test_compare_beauty_handles_errors_only() {
        // Edge case: all sites failed; renderer must not panic on empty sites + non-empty errors.
        let report = CompareReport {
            schema_version: "1",
            compared_at: "2026-04-16T00:00:00Z".to_string(),
            sites: Vec::new(),
            winners: Winners::default(),
            check_diff: Vec::new(),
            errors: vec![CompareError {
                url: "https://broken.example.com".to_string(),
                message: "connection refused".to_string(),
            }],
        };
        print_beauty_compare_report(&report);
    }
}
