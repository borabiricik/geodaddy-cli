use colored::{Color, Colorize};
use crate::scoring::Status;
use crate::{Report, PageResult};

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

pub(crate) fn print_beauty_report(report: &Report) {
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
        "Technical: {:.1}  Content: {:.1}  GEO: {:.1}  Performance: {}",
        report.categories.technical,
        report.categories.content,
        report.categories.geo,
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
        "Technical: {:.1}  Content: {:.1}  GEO: {:.1}  Performance: {}",
        page.categories.technical,
        page.categories.content,
        page.categories.geo,
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
