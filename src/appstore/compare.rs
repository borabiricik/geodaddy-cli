//! App-vs-app comparison — mirrors the website compare module: sequential
//! best-effort analysis, per-category winners with the shared tie epsilon,
//! and an alphabetical per-check diff.

use chrono::Utc;
use serde::Serialize;
use std::collections::{BTreeMap, HashSet};

use crate::compare::{CheckDiff, CompareError, SiteCheckOutcome, TIE_EPSILON};
#[cfg(test)]
use crate::scoring::Status;

use super::analysis::{analyze_app, AppReport};

pub const APP_COMPARE_SCHEMA_VERSION: &str = "1";

/// Top-level result for a multi-app compare run.
#[derive(Serialize, Debug)]
pub struct AppCompareReport {
    pub schema_version: &'static str,
    pub report_type: &'static str,
    pub compared_at: String,
    pub apps: Vec<AppReport>,
    pub winners: AppWinners,
    pub check_diff: Vec<CheckDiff>,
    pub errors: Vec<CompareError>,
}

/// Per-category winner URLs. None = tied.
#[derive(Serialize, Debug, Default, PartialEq)]
pub struct AppWinners {
    pub overall: Option<String>,
    pub listing: Option<String>,
    pub answerability: Option<String>,
    pub reputation: Option<String>,
    pub web_presence: Option<String>,
    pub cross_platform: Option<String>,
}

/// Run compare across store URLs. Never returns Err — failures per app are
/// collected into `errors`, matching web compare semantics.
pub async fn compare_apps(urls: &[String], client: &reqwest::Client) -> AppCompareReport {
    let unique = dedup_app_urls(urls);
    let total = unique.len();
    let mut apps: Vec<AppReport> = Vec::new();
    let mut errors: Vec<CompareError> = Vec::new();

    for (i, url) in unique.iter().enumerate() {
        tracing::info!("Comparing app {}/{}: {}", i + 1, total, url);
        match analyze_app(url, client).await {
            Ok(report) => apps.push(report),
            Err(e) => {
                tracing::warn!("App analysis failed for {}: {}", url, e);
                errors.push(CompareError {
                    url: url.clone(),
                    message: e.to_string(),
                });
            }
        }
    }

    let winners = compute_app_winners(&apps);
    let check_diff = compute_app_check_diff(&apps);

    AppCompareReport {
        schema_version: APP_COMPARE_SCHEMA_VERSION,
        report_type: "app_compare",
        compared_at: Utc::now().to_rfc3339(),
        apps,
        winners,
        check_diff,
        errors,
    }
}

/// Dedup by parsed (store, id) so the same app pasted with different query
/// params or storefronts in the path counts once. Unparseable URLs pass
/// through so analyze_app can surface the real error.
pub fn dedup_app_urls(urls: &[String]) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut result = Vec::with_capacity(urls.len());
    for url in urls {
        let key = match super::parse_store_url(url) {
            Ok(r) => format!("{:?}:{}", r.store, r.id),
            Err(_) => url.clone(),
        };
        if seen.insert(key) {
            result.push(url.clone());
        } else {
            tracing::warn!("Duplicate app URL ignored: {}", url);
        }
    }
    result
}

pub fn compute_app_winners(apps: &[AppReport]) -> AppWinners {
    fn winner<F>(apps: &[AppReport], extract: F) -> Option<String>
    where
        F: Fn(&AppReport) -> f64,
    {
        if apps.is_empty() {
            return None;
        }
        let max = apps
            .iter()
            .map(|a| extract(a))
            .fold(f64::NEG_INFINITY, f64::max);
        let top: Vec<&AppReport> = apps
            .iter()
            .filter(|a| (max - extract(a)).abs() < TIE_EPSILON)
            .collect();
        if top.len() == 1 {
            Some(top[0].url.clone())
        } else {
            None
        }
    }

    // Cross-platform is nullable (desktop apps) — the winner is computed
    // only across apps that actually have the category, like the web
    // engine's performance winner.
    fn optional_winner<F>(apps: &[AppReport], extract: F) -> Option<String>
    where
        F: Fn(&AppReport) -> Option<f64>,
    {
        let scored: Vec<(&AppReport, f64)> = apps
            .iter()
            .filter_map(|a| extract(a).map(|v| (a, v)))
            .collect();
        if scored.is_empty() {
            return None;
        }
        let max = scored
            .iter()
            .map(|(_, v)| *v)
            .fold(f64::NEG_INFINITY, f64::max);
        let top: Vec<&&AppReport> = scored
            .iter()
            .filter(|(_, v)| (max - v).abs() < TIE_EPSILON)
            .map(|(a, _)| a)
            .collect();
        if top.len() == 1 {
            Some(top[0].url.clone())
        } else {
            None
        }
    }

    AppWinners {
        overall: winner(apps, |a| a.score),
        listing: winner(apps, |a| a.categories.listing),
        answerability: winner(apps, |a| a.categories.answerability),
        reputation: winner(apps, |a| a.categories.reputation),
        web_presence: winner(apps, |a| a.categories.web_presence),
        cross_platform: optional_winner(apps, |a| a.categories.cross_platform),
    }
}

/// Alphabetical per-check diff across apps. App reports are flat (single
/// listing, no pages), so a check's status maps 1:1.
pub fn compute_app_check_diff(apps: &[AppReport]) -> Vec<CheckDiff> {
    let mut all_checks: HashSet<&'static str> = HashSet::new();
    for app in apps {
        for r in &app.results {
            all_checks.insert(r.check);
        }
    }

    let mut by_check: BTreeMap<&'static str, Vec<SiteCheckOutcome>> = BTreeMap::new();
    for check in all_checks {
        let outcomes = apps
            .iter()
            .map(|app| SiteCheckOutcome {
                url: app.url.clone(),
                status: app
                    .results
                    .iter()
                    .find(|r| r.check == check)
                    .map(|r| r.status.clone()),
            })
            .collect();
        by_check.insert(check, outcomes);
    }

    by_check
        .into_iter()
        .map(|(check, results)| CheckDiff {
            check: check.to_string(),
            results,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::appstore::analysis::{AppCategoryScores, AppInfo};
    use crate::appstore::Store;
    use crate::scoring::AnalysisResult;

    fn make_app(url: &str, score: f64, results: Vec<AnalysisResult>) -> AppReport {
        AppReport {
            schema_version: "1",
            report_type: "app",
            url: url.to_string(),
            analyzed_at: "2026-07-03T00:00:00Z".to_string(),
            app: AppInfo {
                store: Store::Ios,
                app_id: "1".to_string(),
                country: "us".to_string(),
                is_desktop: false,
                name: "X".to_string(),
                developer: None,
                developer_url: None,
                category: None,
                rating_avg: None,
                rating_count: None,
                price: None,
                icon_url: None,
                store_url: url.to_string(),
                last_updated: None,
                installs: None,
                screenshot_count: 0,
                other_store_url: None,
            },
            score,
            categories: AppCategoryScores {
                listing: score,
                answerability: score,
                reputation: score,
                web_presence: score,
                cross_platform: Some(score),
            },
            results,
            developer_site: None,
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
    fn winners_highest_score() {
        let a = make_app("https://apps.apple.com/us/app/id1", 90.0, vec![]);
        let b = make_app("https://apps.apple.com/us/app/id2", 70.0, vec![]);
        let w = compute_app_winners(&[a, b]);
        assert_eq!(w.overall.as_deref(), Some("https://apps.apple.com/us/app/id1"));
        assert_eq!(w.reputation.as_deref(), Some("https://apps.apple.com/us/app/id1"));
    }

    #[test]
    fn winners_tie_is_none() {
        let a = make_app("https://apps.apple.com/us/app/id1", 80.0, vec![]);
        let b = make_app("https://apps.apple.com/us/app/id2", 80.05, vec![]);
        let w = compute_app_winners(&[a, b]);
        assert_eq!(w.overall, None);
    }

    #[test]
    fn check_diff_marks_absent_checks_none() {
        let a = make_app(
            "https://apps.apple.com/us/app/id1",
            80.0,
            vec![res("app-rep-installs", Status::Pass)],
        );
        let b = make_app(
            "https://apps.apple.com/us/app/id2",
            80.0,
            vec![res("app-list-title", Status::Pass)],
        );
        let diff = compute_app_check_diff(&[a, b]);
        let installs = diff.iter().find(|d| d.check == "app-rep-installs").unwrap();
        assert_eq!(installs.results[1].status, None);
        // Alphabetical ordering
        assert_eq!(diff[0].check, "app-list-title");
    }

    #[test]
    fn dedup_by_store_and_id() {
        let urls = vec![
            "https://apps.apple.com/us/app/name/id42".to_string(),
            "https://apps.apple.com/tr/app/other-slug/id42?x=1".to_string(),
            "https://play.google.com/store/apps/details?id=com.a".to_string(),
        ];
        let out = dedup_app_urls(&urls);
        assert_eq!(out.len(), 2, "same track id across storefronts dedupes");
    }
}
