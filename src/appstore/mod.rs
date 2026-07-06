//! Mobile app store analysis — scores how AI-recommendable an app's store
//! listing is, from an App Store or Play Store URL.
//!
//! Data sources are deliberately key-free, matching the project's local-first
//! design: the iTunes Lookup API (documented, no auth) for iOS and the
//! server-rendered Play Store web page (JSON-LD + microdata) for Android.

pub mod itunes;
pub mod play;
pub mod checks;
pub mod analysis;
pub mod compare;

pub use analysis::{analyze_app, AppReport, AppInfo, AppCategoryScores};
pub use compare::{compare_apps, AppCompareReport, AppWinners};

use anyhow::Result;
use serde::Serialize;
use url::Url;

/// Which store a URL points at. Serialized lowercase for the JSON contract.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Store {
    Ios,
    Android,
}

impl Store {
    pub fn label(&self) -> &'static str {
        match self {
            Store::Ios => "App Store",
            Store::Android => "Play Store",
        }
    }
}

/// A parsed store URL: which store, the app's identifier, and the storefront
/// country the listing should be fetched from.
#[derive(Debug, Clone, PartialEq)]
pub struct AppRef {
    pub store: Store,
    /// Numeric track ID for iOS, package name for Android.
    pub id: String,
    /// Lowercase ISO-3166 alpha-2 storefront (defaults to "us").
    pub country: String,
}

/// Unified listing metadata across both stores. Fields a store cannot
/// provide stay `None`; checks skip rather than fail on missing data so a
/// parser regression degrades the score gracefully instead of tanking it.
#[derive(Debug, Clone)]
pub struct AppMetadata {
    pub store: Store,
    pub app_id: String,
    pub country: String,
    /// True for desktop software (Mac App Store, `kind: mac-software`).
    /// Desktop apps never have a Play Store counterpart, so cross-store
    /// checks are skipped entirely instead of guessing.
    pub is_desktop: bool,
    pub name: String,
    pub description: String,
    pub release_notes: Option<String>,
    pub developer_name: Option<String>,
    pub developer_url: Option<String>,
    pub category: Option<String>,
    pub rating_avg: Option<f64>,
    pub rating_count: Option<u64>,
    pub price: Option<String>,
    pub screenshot_count: usize,
    /// ISO-8601 date (YYYY-MM-DD) of the last listing update, when known.
    pub last_updated: Option<String>,
    pub content_rating: Option<String>,
    /// Number of store-declared languages (iTunes only).
    pub language_count: Option<usize>,
    /// Play's install bucket, e.g. "10B+" (Android only).
    pub installs: Option<String>,
    pub icon_url: Option<String>,
    pub store_url: String,
}

/// Result of searching the *other* store for the same app.
///
/// `Found` requires BOTH the app name and the developer name to match —
/// a name-only hit is reported as `Unknown` (unverified), never linked.
#[derive(Debug, Clone)]
pub enum CrossStoreOutcome {
    Found {
        url: String,
        name: String,
        /// Strict post-normalization identity — drives the name-consistency
        /// check ("WhatsApp" vs "WhatsApp Messenger" is Found but not identical).
        name_identical: bool,
    },
    NotFound,
    /// Could not be verified: search failed, or a same-named listing exists
    /// but its developer differs. Scored as Warn, and no link is shown.
    Unknown(String),
}

/// Returns true when the URL points at a supported app store listing.
pub fn is_store_url(raw: &str) -> bool {
    parse_store_url(raw).is_ok()
}

/// Parse an App Store / Play Store listing URL into an [`AppRef`].
///
/// Accepted forms:
/// - `https://apps.apple.com/{cc}/app/{slug}/id{digits}` (country optional)
/// - `https://itunes.apple.com/{cc}/app/{slug}/id{digits}` (legacy host)
/// - `https://play.google.com/store/apps/details?id={package}` (+ `gl`/`hl`)
pub fn parse_store_url(raw: &str) -> Result<AppRef> {
    let url = Url::parse(raw).map_err(|e| anyhow::anyhow!("Invalid URL '{}': {}", raw, e))?;
    let host = url
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("URL '{}' has no host", raw))?
        .to_ascii_lowercase();

    if host == "apps.apple.com" || host == "itunes.apple.com" {
        let segments: Vec<&str> = url
            .path_segments()
            .map(|s| s.filter(|p| !p.is_empty()).collect())
            .unwrap_or_default();

        let id = segments
            .iter()
            .rev()
            .find_map(|seg| {
                let digits = seg.strip_prefix("id")?;
                (!digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
                    .then(|| digits.to_string())
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "App Store URL '{}' is missing the app ID segment (…/id123456789)",
                    raw
                )
            })?;

        let country = segments
            .first()
            .filter(|seg| seg.len() == 2 && seg.chars().all(|c| c.is_ascii_alphabetic()))
            .map(|seg| seg.to_ascii_lowercase())
            .unwrap_or_else(|| "us".to_string());

        return Ok(AppRef {
            store: Store::Ios,
            id,
            country,
        });
    }

    if host == "play.google.com" {
        if !url.path().starts_with("/store/apps/details") {
            anyhow::bail!(
                "Play Store URL '{}' is not an app details page (/store/apps/details?id=…)",
                raw
            );
        }
        let id = url
            .query_pairs()
            .find(|(k, _)| k == "id")
            .map(|(_, v)| v.to_string())
            .filter(|v| !v.is_empty())
            .ok_or_else(|| {
                anyhow::anyhow!("Play Store URL '{}' is missing the ?id= package parameter", raw)
            })?;

        let country = url
            .query_pairs()
            .find(|(k, _)| k == "gl")
            .map(|(_, v)| v.to_ascii_lowercase())
            .filter(|v| v.len() == 2)
            .unwrap_or_else(|| "us".to_string());

        return Ok(AppRef {
            store: Store::Android,
            id,
            country,
        });
    }

    anyhow::bail!(
        "'{}' is not a supported app store URL. Expected apps.apple.com or play.google.com/store/apps/details.",
        raw
    )
}

/// Normalize an app name for cross-store comparison: lowercase, strip
/// everything but alphanumerics and spaces, collapse whitespace.
pub fn normalize_app_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut last_space = true;
    for c in name.chars() {
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
            last_space = false;
        } else if !last_space {
            out.push(' ');
            last_space = true;
        }
    }
    out.trim().to_string()
}

/// Developer-name equivalence for cross-store verification. Publisher
/// entities differ across stores ("WhatsApp Inc." vs "WhatsApp LLC"), so this
/// reuses the fuzzy name matcher. Missing names on either side mean the
/// match CANNOT be verified — callers must treat that as unconfirmed, never
/// as a pass, because a name-only match is exactly how wrong apps get linked.
pub fn developers_match(a: Option<&str>, b: Option<&str>) -> Option<bool> {
    match (a, b) {
        (Some(a), Some(b)) => Some(names_match(a, b)),
        _ => None,
    }
}

/// Strict name identity after normalization — used for the name-consistency
/// check once a listing is already confirmed as the same app.
pub fn names_identical(a: &str, b: &str) -> bool {
    let na = normalize_app_name(a);
    !na.is_empty() && na == normalize_app_name(b)
}

/// Fuzzy name equivalence: exact after normalization, containment either way
/// (handles "WhatsApp" vs "WhatsApp Messenger"), or a shared meaningful
/// first token (≥4 chars) covering brand-led names like "Spotify: Music".
pub fn names_match(a: &str, b: &str) -> bool {
    let na = normalize_app_name(a);
    let nb = normalize_app_name(b);
    if na.is_empty() || nb.is_empty() {
        return false;
    }
    if na == nb || na.contains(&nb) || nb.contains(&na) {
        return true;
    }
    match (na.split(' ').next(), nb.split(' ').next()) {
        (Some(fa), Some(fb)) => fa == fb && fa.len() >= 4,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ios_url_with_country_and_slug() {
        let r = parse_store_url("https://apps.apple.com/tr/app/whatsapp-messenger/id310633997")
            .unwrap();
        assert_eq!(r.store, Store::Ios);
        assert_eq!(r.id, "310633997");
        assert_eq!(r.country, "tr");
    }

    #[test]
    fn parses_ios_url_without_country() {
        let r = parse_store_url("https://apps.apple.com/app/id310633997").unwrap();
        assert_eq!(r.store, Store::Ios);
        assert_eq!(r.id, "310633997");
        assert_eq!(r.country, "us");
    }

    #[test]
    fn parses_legacy_itunes_host() {
        let r = parse_store_url("https://itunes.apple.com/us/app/example/id123").unwrap();
        assert_eq!(r.store, Store::Ios);
        assert_eq!(r.id, "123");
    }

    #[test]
    fn parses_ios_url_with_query_params() {
        let r = parse_store_url("https://apps.apple.com/us/app/slug/id42?mt=8&uo=4").unwrap();
        assert_eq!(r.id, "42");
    }

    #[test]
    fn rejects_ios_url_without_id() {
        assert!(parse_store_url("https://apps.apple.com/us/app/whatsapp-messenger").is_err());
    }

    #[test]
    fn parses_android_url() {
        let r = parse_store_url("https://play.google.com/store/apps/details?id=com.whatsapp")
            .unwrap();
        assert_eq!(r.store, Store::Android);
        assert_eq!(r.id, "com.whatsapp");
        assert_eq!(r.country, "us");
    }

    #[test]
    fn parses_android_url_with_gl_country() {
        let r = parse_store_url(
            "https://play.google.com/store/apps/details?id=com.whatsapp&hl=tr&gl=TR",
        )
        .unwrap();
        assert_eq!(r.country, "tr");
    }

    #[test]
    fn rejects_android_url_without_package() {
        assert!(parse_store_url("https://play.google.com/store/apps/details").is_err());
    }

    #[test]
    fn rejects_play_non_details_path() {
        assert!(parse_store_url("https://play.google.com/store/search?q=x&c=apps").is_err());
    }

    #[test]
    fn rejects_regular_website() {
        assert!(parse_store_url("https://example.com").is_err());
        assert!(!is_store_url("https://example.com/store/apps/details?id=x"));
    }

    #[test]
    fn normalize_strips_punctuation_and_case() {
        assert_eq!(normalize_app_name("Spotify: Music & Podcasts"), "spotify music podcasts");
        assert_eq!(normalize_app_name("  WhatsApp   Messenger "), "whatsapp messenger");
    }

    #[test]
    fn developer_matching_requires_both_sides() {
        assert_eq!(developers_match(Some("WhatsApp Inc."), Some("WhatsApp LLC")), Some(true));
        assert_eq!(developers_match(Some("Robert Jänisch"), Some("Some Other Dev")), Some(false));
        assert_eq!(developers_match(None, Some("WhatsApp LLC")), None);
        assert_eq!(developers_match(Some("WhatsApp Inc."), None), None);
    }

    #[test]
    fn names_identical_is_strict() {
        assert!(names_identical("WhatsApp Messenger", "WhatsApp  Messenger!"));
        assert!(!names_identical("WhatsApp", "WhatsApp Messenger"));
        assert!(!names_identical("", ""));
    }

    #[test]
    fn names_match_containment_and_first_token() {
        assert!(names_match("WhatsApp Messenger", "WhatsApp Messenger"));
        assert!(names_match("WhatsApp", "WhatsApp Messenger"));
        assert!(names_match("Spotify: Music and Podcasts", "Spotify - Web Player"));
        assert!(!names_match("Signal", "Telegram"));
        assert!(!names_match("", "Telegram"));
    }
}
