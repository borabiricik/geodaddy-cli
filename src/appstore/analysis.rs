//! App report assembly: category scoring over `app-*` checks and the
//! `analyze_app` orchestrator that fans out to the store fetch, the
//! developer-site GEO analysis, and the cross-store search.

use anyhow::Result;
use chrono::Utc;
use serde::Serialize;

use crate::scoring::{AnalysisResult, Status};
use crate::{AnalysisConfig, Report};

use super::checks::{run_cross_platform_checks, run_listing_checks, run_web_presence_checks};
use super::{itunes, play, parse_store_url, AppMetadata, CrossStoreOutcome, Store};

pub const APP_SCHEMA_VERSION: &str = "1";

/// Category scores for an app report. The first four always score —
/// missing signals count against the app rather than vanishing, because
/// an absent website IS the finding. `cross_platform` is None (null in
/// JSON) when the concept doesn't apply — desktop software (Mac App Store)
/// has no Play Store counterpart to look for. Mirrors the web engine's
/// nullable `performance` category.
#[derive(Serialize, Debug, Clone)]
pub struct AppCategoryScores {
    pub listing: f64,
    pub answerability: f64,
    pub reputation: f64,
    pub web_presence: f64,
    pub cross_platform: Option<f64>,
}

/// Listing summary surfaced to consumers (web UI header card, beauty output).
#[derive(Serialize, Debug, Clone)]
pub struct AppInfo {
    pub store: Store,
    pub app_id: String,
    pub country: String,
    /// True for Mac App Store (desktop) listings — no cross-store checks.
    pub is_desktop: bool,
    pub name: String,
    pub developer: Option<String>,
    pub developer_url: Option<String>,
    pub category: Option<String>,
    pub rating_avg: Option<f64>,
    pub rating_count: Option<u64>,
    pub price: Option<String>,
    pub icon_url: Option<String>,
    pub store_url: String,
    pub last_updated: Option<String>,
    pub installs: Option<String>,
    pub screenshot_count: usize,
    /// Listing URL on the other store, when the cross-store search found one.
    pub other_store_url: Option<String>,
}

/// Top-level app analysis report. `report_type` discriminates the JSON shape
/// from the website `Report` for API consumers sharing an endpoint or cache.
#[derive(Serialize, Debug)]
pub struct AppReport {
    pub schema_version: &'static str,
    pub report_type: &'static str,
    pub url: String,
    pub analyzed_at: String,
    pub app: AppInfo,
    pub score: f64,
    pub categories: AppCategoryScores,
    pub results: Vec<AnalysisResult>,
    /// Single-page GEO analysis of the developer website, when one was
    /// declared and reachable. Full web report so UIs can drill in.
    pub developer_site: Option<Report>,
}

/// Severity points for app checks — same 10/5/2 scale as the web engine.
pub(crate) fn app_severity_points(check: &str) -> u32 {
    match check {
        // Critical: the signals AI recommendation visibility hinges on.
        "app-list-title"
        | "app-list-description-length"
        | "app-list-update-freshness"
        | "app-ans-value-proposition"
        | "app-rep-rating-average"
        | "app-rep-rating-volume"
        | "app-web-developer-site"
        | "app-web-geo-score"
        | "app-x-other-store" => 10,
        // Standard.
        "app-list-screenshots"
        | "app-list-category"
        | "app-ans-use-cases"
        | "app-ans-keyword-stuffing"
        | "app-rep-installs"
        | "app-web-llms-txt"
        | "app-web-structured-data"
        | "app-x-name-consistency" => 5,
        // Informational.
        "app-list-release-notes"
        | "app-list-localization"
        | "app-ans-feature-structure"
        | "app-rep-content-rating" => 2,
        _ => 5,
    }
}

/// Score app checks into the five app categories. Mirrors the web engine's
/// model: Pass = full points, Warn = half, Fail = zero; category = earned/max.
pub fn calculate_app_score(results: &[AnalysisResult]) -> (f64, AppCategoryScores) {
    let mut earned = [0u32; 5];
    let mut max = [0u32; 5];

    for result in results {
        let idx = if result.check.starts_with("app-list-") {
            0
        } else if result.check.starts_with("app-ans-") {
            1
        } else if result.check.starts_with("app-rep-") {
            2
        } else if result.check.starts_with("app-web-") {
            3
        } else if result.check.starts_with("app-x-") {
            4
        } else {
            continue;
        };

        let pts = app_severity_points(result.check);
        max[idx] += pts;
        earned[idx] += match result.status {
            Status::Pass => pts,
            Status::Warn => pts / 2,
            Status::Fail => 0,
        };
    }

    let score_of = |i: usize| -> f64 {
        if max[i] == 0 {
            100.0
        } else {
            ((earned[i] as f64 / max[i] as f64) * 100.0).clamp(0.0, 100.0)
        }
    };

    // Cross-platform is null when no app-x checks ran (desktop software) —
    // the overall average then spans four categories, not five.
    let cross_platform: Option<f64> = (max[4] > 0).then(|| score_of(4));

    let categories = AppCategoryScores {
        listing: score_of(0),
        answerability: score_of(1),
        reputation: score_of(2),
        web_presence: score_of(3),
        cross_platform,
    };

    let base_sum = categories.listing
        + categories.answerability
        + categories.reputation
        + categories.web_presence;
    let overall = match categories.cross_platform {
        Some(cross) => ((base_sum + cross) / 5.0).clamp(0.0, 100.0),
        None => (base_sum / 4.0).clamp(0.0, 100.0),
    };

    (overall, categories)
}

/// Run a full app analysis on a store URL.
///
/// Sequence: fetch listing metadata, then run the developer-site GEO
/// analysis and the cross-store search concurrently, then score.
/// No headless browser is ever needed — both stores serve rendered HTML/JSON.
pub async fn analyze_app(url: &str, client: &reqwest::Client) -> Result<AppReport> {
    let app_ref = parse_store_url(url)?;

    tracing::info!("Fetching {} listing for {}", app_ref.store.label(), app_ref.id);
    let meta: AppMetadata = match app_ref.store {
        Store::Ios => itunes::fetch_ios_metadata(client, &app_ref).await?,
        Store::Android => play::fetch_android_metadata(client, &app_ref).await?,
    };

    // Developer-site GEO analysis and cross-store search are independent —
    // run them concurrently. Both are best-effort: failures become findings,
    // not errors.
    let web_config = AnalysisConfig {
        max_pages: None,
        enable_js: false,
        vitals: false,
    };
    let site_fut = async {
        match &meta.developer_url {
            Some(dev_url) => {
                tracing::info!("Analyzing developer website {}", dev_url);
                match crate::analyze(dev_url, &web_config, client, None, None).await {
                    Ok(report) => Some(report),
                    Err(e) => {
                        tracing::warn!("Developer site analysis failed for {}: {}", dev_url, e);
                        None
                    }
                }
            }
            None => None,
        }
    };
    // Desktop software (Mac App Store) has no other-store counterpart —
    // skip the search entirely instead of guessing; the cross_platform
    // category is null for these listings.
    let cross_fut = async {
        if meta.is_desktop {
            tracing::info!("Desktop app — skipping cross-store search");
            return None;
        }
        tracing::info!("Searching the other store for \"{}\"", meta.name);
        let developer = meta.developer_name.as_deref();
        Some(match app_ref.store {
            Store::Ios => {
                play::search_play_by_name(client, &meta.name, developer, &app_ref.country).await
            }
            Store::Android => {
                itunes::search_ios_by_name(client, &meta.name, developer, &app_ref.country).await
            }
        })
    };
    let (developer_site, cross_outcome): (Option<Report>, Option<CrossStoreOutcome>) =
        tokio::join!(site_fut, cross_fut);

    let mut results = run_listing_checks(&meta);
    results.extend(run_web_presence_checks(&meta, developer_site.as_ref()));
    if let Some(outcome) = &cross_outcome {
        results.extend(run_cross_platform_checks(&meta, outcome));
    }

    let (score, categories) = calculate_app_score(&results);

    let other_store_url = match &cross_outcome {
        Some(CrossStoreOutcome::Found { url, .. }) => Some(url.clone()),
        _ => None,
    };

    Ok(AppReport {
        schema_version: APP_SCHEMA_VERSION,
        report_type: "app",
        url: url.to_string(),
        analyzed_at: Utc::now().to_rfc3339(),
        app: AppInfo {
            store: meta.store,
            app_id: meta.app_id.clone(),
            country: meta.country.clone(),
            is_desktop: meta.is_desktop,
            name: meta.name.clone(),
            developer: meta.developer_name.clone(),
            developer_url: meta.developer_url.clone(),
            category: meta.category.clone(),
            rating_avg: meta.rating_avg,
            rating_count: meta.rating_count,
            price: meta.price.clone(),
            icon_url: meta.icon_url.clone(),
            store_url: meta.store_url.clone(),
            last_updated: meta.last_updated.clone(),
            installs: meta.installs.clone(),
            screenshot_count: meta.screenshot_count,
            other_store_url,
        },
        score,
        categories,
        results,
        developer_site,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn res(check: &'static str, status: Status) -> AnalysisResult {
        AnalysisResult {
            check,
            status,
            message: String::new(),
            recommendation: String::new(),
        }
    }

    #[test]
    fn empty_results_score_100_with_null_cross() {
        let (overall, cats) = calculate_app_score(&[]);
        assert_eq!(overall, 100.0);
        assert_eq!(cats.listing, 100.0);
        assert_eq!(cats.cross_platform, None, "no app-x checks → null category");
    }

    #[test]
    fn categories_route_by_prefix() {
        let results = vec![
            res("app-list-title", Status::Fail),
            res("app-ans-use-cases", Status::Pass),
            res("app-rep-rating-average", Status::Pass),
            res("app-web-developer-site", Status::Pass),
            res("app-x-other-store", Status::Pass),
        ];
        let (_, cats) = calculate_app_score(&results);
        assert_eq!(cats.listing, 0.0);
        assert_eq!(cats.answerability, 100.0);
        assert_eq!(cats.reputation, 100.0);
        assert_eq!(cats.web_presence, 100.0);
        assert_eq!(cats.cross_platform, Some(100.0));
    }

    #[test]
    fn desktop_apps_average_four_categories() {
        // No app-x checks (desktop skip): listing 0 + three 100s → 75, not 80.
        let results = vec![
            res("app-list-title", Status::Fail),
            res("app-ans-use-cases", Status::Pass),
        ];
        let (overall, cats) = calculate_app_score(&results);
        assert_eq!(cats.cross_platform, None);
        assert!((overall - 75.0).abs() < 0.01);
    }

    #[test]
    fn warn_earns_half_points() {
        let results = vec![res("app-list-title", Status::Warn)];
        let (_, cats) = calculate_app_score(&results);
        assert_eq!(cats.listing, 50.0);
    }

    #[test]
    fn overall_is_five_way_average_when_cross_present() {
        let results = vec![
            res("app-list-title", Status::Fail),    // listing 0
            res("app-ans-use-cases", Status::Pass), // answerability 100
            res("app-x-other-store", Status::Pass), // cross 100
            // reputation, web empty → 100 each
        ];
        let (overall, _) = calculate_app_score(&results);
        assert!((overall - 80.0).abs() < 0.01);
    }

    #[test]
    fn severity_mapping() {
        assert_eq!(app_severity_points("app-rep-rating-average"), 10);
        assert_eq!(app_severity_points("app-x-other-store"), 10);
        assert_eq!(app_severity_points("app-web-llms-txt"), 5);
        assert_eq!(app_severity_points("app-rep-content-rating"), 2);
        assert_eq!(app_severity_points("app-unknown-future-check"), 5);
    }

    #[test]
    fn mixed_severities_weight_correctly() {
        // listing: title (10) Pass + release-notes (2) Fail → 10/12
        let results = vec![
            res("app-list-title", Status::Pass),
            res("app-list-release-notes", Status::Fail),
        ];
        let (_, cats) = calculate_app_score(&results);
        assert!((cats.listing - (10.0 / 12.0 * 100.0)).abs() < 0.01);
    }
}
