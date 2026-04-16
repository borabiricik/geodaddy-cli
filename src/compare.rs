//! Competitor comparison module — accepts 2+ URLs, runs analyze() per URL,
//! and produces a CompareReport with per-category winners and per-check diff.
//!
//! Wave 0 scaffold: types compile, functions are `todo!()` stubs. Wave 1 fills
//! in the real logic against the red tests defined at the bottom of this file.

use chromiumoxide::Browser;
use chrono::Utc;
use serde::Serialize;

use crate::AnalysisConfig;
use crate::Report;
use crate::scoring::Status;

/// Absolute tolerance for tie detection between category scores.
/// 10× larger than f64::EPSILON, 10× smaller than smallest realistic
/// score delta (~2.5pt). See 08-RESEARCH.md Pitfall 3.
pub const TIE_EPSILON: f64 = 0.1;

/// JSON schema version. Shape discriminates from single-URL Report via
/// the presence of the `sites` key; no version bump needed.
pub const COMPARE_SCHEMA_VERSION: &str = "1";

/// Top-level result for a multi-URL compare run.
#[derive(Serialize, Debug)]
pub struct CompareReport {
    pub schema_version: &'static str,
    pub compared_at: String,
    pub sites: Vec<Report>,
    pub winners: Winners,
    pub check_diff: Vec<CheckDiff>,
    pub errors: Vec<CompareError>,
}

impl CompareReport {
    /// Empty-shell constructor, used by tests and stubs.
    pub fn empty() -> Self {
        Self {
            schema_version: COMPARE_SCHEMA_VERSION,
            compared_at: Utc::now().to_rfc3339(),
            sites: Vec::new(),
            winners: Winners::default(),
            check_diff: Vec::new(),
            errors: Vec::new(),
        }
    }
}

/// Per-category winner URLs. None = tied OR category absent from all sites.
#[derive(Serialize, Debug, Default, PartialEq)]
pub struct Winners {
    pub overall: Option<String>,
    pub technical: Option<String>,
    pub content: Option<String>,
    pub geo: Option<String>,
    pub performance: Option<String>,
}

/// One row of the per-check diff table — one check ID across all sites.
#[derive(Serialize, Debug, PartialEq)]
pub struct CheckDiff {
    pub check: String,
    pub results: Vec<SiteCheckOutcome>,
}

/// One site's outcome for a given check ID. `status: None` = check did not
/// run on this site (e.g. perf checks without --vitals).
#[derive(Serialize, Debug, PartialEq)]
pub struct SiteCheckOutcome {
    pub url: String,
    pub status: Option<Status>,
}

/// Per-URL failure record. Non-fatal for compare mode unless first URL.
#[derive(Serialize, Debug, PartialEq)]
pub struct CompareError {
    pub url: String,
    pub message: String,
}

// ── Public API (stubs; Wave 1 implements) ──────────────────────────────────

/// Run compare across a list of URLs, sharing HTTP client + optional browsers.
/// Returns a CompareReport containing successes (`sites`) and failures (`errors`).
/// Never returns Err — compare mode is best-effort.
pub async fn compare_sites(
    urls: &[String],
    config: &AnalysisConfig,
    client: &reqwest::Client,
    js_browser: Option<&Browser>,
    vitals_browser: Option<&Browser>,
) -> CompareReport {
    let _ = (urls, config, client, js_browser, vitals_browser);
    todo!("Wave 1: implement sequential analyze() loop with shared resources");
}

/// Compute per-category winners across successful site reports.
/// Uses TIE_EPSILON for tie detection; returns None for any tied or empty category.
pub fn compute_winners(sites: &[Report]) -> Winners {
    let _ = sites;
    todo!("Wave 1: implement winner-per-category with 0.1 epsilon tie detection");
}

/// Build alphabetically-sorted check diff by scanning every check ID across all
/// sites' pages. Aggregate a site's status for a check as: any Fail → Fail,
/// else any Warn → Warn, else Pass. Absent → None.
pub fn compute_check_diff(sites: &[Report]) -> Vec<CheckDiff> {
    let _ = sites;
    todo!("Wave 1: BTreeMap<&str, Vec<SiteCheckOutcome>> grouping");
}

/// Dedupe a URL list by canonical form (`crawling::normalize_url`), preserving
/// first occurrence. Emits a stderr warning via tracing for each duplicate dropped.
pub fn dedup_urls(urls: &[String]) -> Vec<String> {
    let _ = urls;
    todo!("Wave 1: HashSet<String> of normalized URLs, preserve first, warn on dup");
}

// ── Tests (red in Wave 0, green in Wave 1) ─────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PageResult;
    use crate::scoring::{AnalysisResult, CategoryScores, Status};

    fn make_report(
        url: &str,
        tech: f64,
        content: f64,
        geo: f64,
        perf: Option<f64>,
        results: Vec<AnalysisResult>,
    ) -> Report {
        Report {
            schema_version: "1",
            url: url.to_string(),
            crawled_at: "2026-04-16T00:00:00Z".to_string(),
            score: (tech + content + geo) / 3.0,
            categories: CategoryScores {
                technical: tech,
                content,
                geo,
                performance: perf,
            },
            pages: vec![PageResult {
                url: url.to_string(),
                robots_blocked: false,
                score: (tech + content + geo) / 3.0,
                categories: CategoryScores {
                    technical: tech,
                    content,
                    geo,
                    performance: perf,
                },
                results,
            }],
        }
    }

    fn res(check: &'static str, status: Status) -> AnalysisResult {
        AnalysisResult {
            check,
            status,
            message: String::new(),
            recommendation: String::new(),
        }
    }

    #[test]
    fn test_winner_highest_score() {
        let a = make_report("https://a.com", 87.3, 85.0, 85.0, None, vec![]);
        let b = make_report("https://b.com", 72.1, 70.0, 58.0, None, vec![]);
        let w = compute_winners(&[a, b]);
        assert_eq!(w.technical.as_deref(), Some("https://a.com"));
    }

    #[test]
    fn test_winner_tie_within_epsilon() {
        let a = make_report("https://a.com", 80.05, 50.0, 50.0, None, vec![]);
        let b = make_report("https://b.com", 80.10, 50.0, 50.0, None, vec![]);
        let w = compute_winners(&[a, b]);
        assert_eq!(w.technical, None, "0.05 diff must be within 0.1 epsilon → tie");
    }

    #[test]
    fn test_winner_performance_absent() {
        let a = make_report("https://a.com", 80.0, 80.0, 80.0, None, vec![]);
        let b = make_report("https://b.com", 90.0, 90.0, 90.0, None, vec![]);
        let w = compute_winners(&[a, b]);
        assert_eq!(w.performance, None, "all-None perf → no winner");
    }

    #[test]
    fn test_winner_all_sites_missing_category() {
        let a = make_report("https://a.com", 80.0, 80.0, 80.0, None, vec![]);
        let w = compute_winners(&[a]);
        assert_eq!(w.performance, None);
    }

    #[test]
    fn test_check_diff_unique_checks() {
        let a = make_report(
            "https://a.com",
            100.0,
            100.0,
            100.0,
            None,
            vec![
                res("tech-meta-title", Status::Pass),
                res("tech-https", Status::Pass),
            ],
        );
        let b = make_report(
            "https://b.com",
            100.0,
            100.0,
            100.0,
            None,
            vec![
                res("tech-meta-title", Status::Fail),
                res("geo-llms-txt", Status::Fail),
            ],
        );
        let diff = compute_check_diff(&[a, b]);
        let checks: Vec<&str> = diff.iter().map(|d| d.check.as_str()).collect();
        assert_eq!(
            checks,
            vec!["geo-llms-txt", "tech-https", "tech-meta-title"],
            "alphabetical + unique"
        );
    }

    #[test]
    fn test_check_diff_missing_null() {
        let a = make_report(
            "https://a.com",
            100.0,
            100.0,
            100.0,
            None,
            vec![res("tech-meta-title", Status::Pass)],
        );
        let b = make_report(
            "https://b.com",
            100.0,
            100.0,
            100.0,
            None,
            vec![res("geo-llms-txt", Status::Pass)],
        );
        let diff = compute_check_diff(&[a, b]);
        let meta = diff
            .iter()
            .find(|d| d.check == "tech-meta-title")
            .expect("meta diff present");
        let b_outcome = meta
            .results
            .iter()
            .find(|o| o.url == "https://b.com")
            .expect("b entry present");
        assert_eq!(b_outcome.status, None, "b did not run tech-meta-title → None");
    }

    #[test]
    fn test_aggregate_check_status() {
        // Site with multiple pages, same check appearing with different statuses.
        let mut rep = make_report(
            "https://a.com",
            100.0,
            100.0,
            100.0,
            None,
            vec![res("tech-meta-title", Status::Pass)],
        );
        rep.pages.push(PageResult {
            url: "https://a.com/page2".to_string(),
            robots_blocked: false,
            score: 100.0,
            categories: CategoryScores {
                technical: 100.0,
                content: 100.0,
                geo: 100.0,
                performance: None,
            },
            results: vec![
                res("tech-meta-title", Status::Warn),
                res("tech-meta-title", Status::Fail),
            ],
        });
        let diff = compute_check_diff(&[rep]);
        let meta = diff.iter().find(|d| d.check == "tech-meta-title").unwrap();
        assert_eq!(meta.results[0].status, Some(Status::Fail), "any Fail → Fail");
    }

    #[test]
    fn test_dedup_uses_normalize_url() {
        let urls = vec![
            "https://site.com/".to_string(),
            "https://site.com".to_string(),
            "https://other.com/".to_string(),
        ];
        let out = dedup_urls(&urls);
        assert_eq!(
            out.len(),
            2,
            "site.com/ and site.com are duplicates after normalization"
        );
        assert_eq!(out[0], "https://site.com/");
        assert_eq!(out[1], "https://other.com/");
    }

    #[test]
    fn test_compare_report_schema_version_is_1() {
        let r = CompareReport::empty();
        assert_eq!(r.schema_version, "1");
    }

    #[test]
    #[ignore] // Wave 1 integration-style — requires a real or mocked analyze() call path.
    fn test_loop_calls_analyze_per_url() {
        // Placeholder — real assertion wired in Wave 1 once compare_sites is implemented.
        // For Wave 0, this test is `#[ignore]` so `cargo test` does not mark it red;
        // Wave 1 MUST remove the `#[ignore]` attribute.
    }
}
