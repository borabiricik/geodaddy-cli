//! iTunes Lookup / Search API client — the documented, key-free Apple
//! endpoint (https://performance-partners.apple.com/search-api).
//!
//! Rate limit is ~20 requests/minute per IP, so callers embedding this in a
//! service must cache results by (id, country).

use anyhow::{Context, Result};
use serde_json::Value;

use super::{
    developers_match, names_identical, names_match, AppMetadata, AppRef, CrossStoreOutcome, Store,
};

/// Fetch iOS listing metadata via the iTunes Lookup API.
pub async fn fetch_ios_metadata(
    client: &reqwest::Client,
    app_ref: &AppRef,
) -> Result<AppMetadata> {
    let lookup_url = format!(
        "https://itunes.apple.com/lookup?id={}&country={}",
        app_ref.id, app_ref.country
    );

    let resp = client
        .get(&lookup_url)
        .send()
        .await
        .with_context(|| format!("iTunes Lookup request failed for app id {}", app_ref.id))?;

    let status = resp.status();
    if !status.is_success() {
        anyhow::bail!(
            "iTunes Lookup returned HTTP {} for app id {} (rate limit is ~20 req/min — retry shortly if 403/429)",
            status,
            app_ref.id
        );
    }

    let body: Value = resp
        .json()
        .await
        .context("iTunes Lookup returned non-JSON body")?;

    let result = body
        .get("results")
        .and_then(|r| r.as_array())
        .and_then(|arr| arr.first())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "App id {} not found on the {} App Store storefront",
                app_ref.id,
                app_ref.country
            )
        })?;

    Ok(parse_itunes_result(result, app_ref))
}

/// Map one iTunes lookup result object onto the unified metadata model.
/// Pure function so it can be tested against a captured API payload.
pub fn parse_itunes_result(result: &Value, app_ref: &AppRef) -> AppMetadata {
    let str_field = |key: &str| -> Option<String> {
        result
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };

    let screenshot_count = ["screenshotUrls", "ipadScreenshotUrls"]
        .iter()
        .filter_map(|k| result.get(*k).and_then(|v| v.as_array()).map(|a| a.len()))
        .max()
        .unwrap_or(0);

    // currentVersionReleaseDate is RFC3339 ("2026-06-12T17:52:59Z") — keep
    // just the date part for the unified model.
    let last_updated = str_field("currentVersionReleaseDate")
        .map(|d| d.chars().take(10).collect::<String>())
        .filter(|d| d.len() == 10);

    let is_desktop = result
        .get("kind")
        .and_then(|v| v.as_str())
        .is_some_and(|k| k == "mac-software");

    // Apple encodes "no data" as 0/0 rather than omitting the fields, and
    // Mac App Store ratings are not exposed through this API AT ALL — even
    // heavily-rated apps (Things 3) report 0/0 here. Never surface that as
    // a real 0.00 score:
    //   desktop 0/0  → both None (unknown — checks say "couldn't verify")
    //   mobile  0/0  → avg None, count 0 (genuinely unrated new app)
    let raw_avg = result.get("averageUserRating").and_then(|v| v.as_f64());
    let raw_count = result.get("userRatingCount").and_then(|v| v.as_u64());
    let (rating_avg, rating_count) = match (raw_avg, raw_count) {
        (Some(a), Some(0)) if a == 0.0 && is_desktop => (None, None),
        (Some(a), Some(0)) if a == 0.0 => (None, Some(0)),
        other => other,
    };

    AppMetadata {
        store: Store::Ios,
        app_id: app_ref.id.clone(),
        country: app_ref.country.clone(),
        // Mac App Store listings report kind "mac-software" — those never
        // have a Play Store counterpart, so cross-store checks are skipped.
        is_desktop,
        name: str_field("trackName").unwrap_or_default(),
        description: str_field("description").unwrap_or_default(),
        release_notes: str_field("releaseNotes"),
        developer_name: str_field("sellerName").or_else(|| str_field("artistName")),
        developer_url: str_field("sellerUrl"),
        category: str_field("primaryGenreName"),
        rating_avg,
        rating_count,
        price: str_field("formattedPrice"),
        screenshot_count,
        last_updated,
        content_rating: str_field("contentAdvisoryRating"),
        language_count: result
            .get("languageCodesISO2A")
            .and_then(|v| v.as_array())
            .map(|a| a.len()),
        installs: None,
        icon_url: str_field("artworkUrl512").or_else(|| str_field("artworkUrl100")),
        store_url: str_field("trackViewUrl").unwrap_or_else(|| {
            format!(
                "https://apps.apple.com/{}/app/id{}",
                app_ref.country, app_ref.id
            )
        }),
    }
}

/// Search the App Store for an app by name (used for the cross-platform
/// presence check when the analyzed app lives on the Play Store).
///
/// A result only counts as Found when BOTH the app name and the developer
/// name match the source listing — name-only hits are how unrelated apps
/// that share a title get linked, so those come back as Unknown instead.
pub async fn search_ios_by_name(
    client: &reqwest::Client,
    name: &str,
    developer: Option<&str>,
    country: &str,
) -> CrossStoreOutcome {
    let search_url = format!(
        "https://itunes.apple.com/search?term={}&entity=software&limit=5&country={}",
        urlencoding_encode(name),
        country
    );

    let body: Value = match client.get(&search_url).send().await {
        Ok(resp) if resp.status().is_success() => match resp.json().await {
            Ok(v) => v,
            Err(e) => return CrossStoreOutcome::Unknown(format!("iTunes Search bad JSON: {}", e)),
        },
        Ok(resp) => {
            return CrossStoreOutcome::Unknown(format!("iTunes Search HTTP {}", resp.status()))
        }
        Err(e) => return CrossStoreOutcome::Unknown(format!("iTunes Search failed: {}", e)),
    };

    let results = match body.get("results").and_then(|r| r.as_array()) {
        Some(arr) if !arr.is_empty() => arr,
        _ => return CrossStoreOutcome::NotFound,
    };

    let mut name_only_hit = false;
    for r in results {
        let found_name = match r.get("trackName").and_then(|v| v.as_str()) {
            Some(n) if names_match(n, name) => n,
            _ => continue,
        };
        let found_dev = r
            .get("sellerName")
            .and_then(|v| v.as_str())
            .or_else(|| r.get("artistName").and_then(|v| v.as_str()));

        match developers_match(found_dev, developer) {
            Some(true) => {
                let url = r
                    .get("trackViewUrl")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .or_else(|| {
                        r.get("trackId")
                            .and_then(|v| v.as_u64())
                            .map(|id| format!("https://apps.apple.com/{}/app/id{}", country, id))
                    });
                if let Some(url) = url {
                    return CrossStoreOutcome::Found {
                        name_identical: names_identical(found_name, name),
                        url,
                        name: found_name.to_string(),
                    };
                }
            }
            // Same-named app by a different (or unverifiable) developer —
            // remember it, keep scanning for a confirmed match.
            Some(false) | None => name_only_hit = true,
        }
    }

    if name_only_hit {
        CrossStoreOutcome::Unknown(format!(
            "a listing named like \"{}\" exists but its developer could not be confirmed",
            name
        ))
    } else {
        CrossStoreOutcome::NotFound
    }
}

/// Minimal percent-encoding for query values — avoids pulling in a new crate
/// for one call site. Encodes everything except unreserved characters.
pub(crate) fn urlencoding_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for byte in s.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            _ => out.push_str(&format!("%{:02X}", byte)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample_ref() -> AppRef {
        AppRef {
            store: Store::Ios,
            id: "310633997".to_string(),
            country: "us".to_string(),
        }
    }

    #[test]
    fn parses_full_lookup_result() {
        let result = json!({
            "trackName": "WhatsApp Messenger",
            "description": "Simple. Reliable. Private messaging and calling.",
            "releaseNotes": "Bug fixes and improvements",
            "sellerName": "WhatsApp Inc.",
            "sellerUrl": "https://www.whatsapp.com",
            "primaryGenreName": "Social Networking",
            "averageUserRating": 4.7412,
            "userRatingCount": 12345678u64,
            "formattedPrice": "Free",
            "screenshotUrls": ["a", "b", "c"],
            "ipadScreenshotUrls": ["a"],
            "currentVersionReleaseDate": "2026-06-12T17:52:59Z",
            "contentAdvisoryRating": "12+",
            "languageCodesISO2A": ["EN", "TR", "DE"],
            "artworkUrl512": "https://example.com/icon.png",
            "trackViewUrl": "https://apps.apple.com/us/app/whatsapp-messenger/id310633997"
        });
        let m = parse_itunes_result(&result, &sample_ref());
        assert_eq!(m.name, "WhatsApp Messenger");
        assert!(!m.is_desktop, "no kind field → not desktop software");
        assert_eq!(m.developer_name.as_deref(), Some("WhatsApp Inc."));
        assert_eq!(m.developer_url.as_deref(), Some("https://www.whatsapp.com"));
        assert_eq!(m.category.as_deref(), Some("Social Networking"));
        assert_eq!(m.rating_count, Some(12345678));
        assert_eq!(m.screenshot_count, 3);
        assert_eq!(m.last_updated.as_deref(), Some("2026-06-12"));
        assert_eq!(m.language_count, Some(3));
        assert_eq!(m.price.as_deref(), Some("Free"));
        assert_eq!(m.installs, None);
    }

    #[test]
    fn parses_sparse_lookup_result_without_panicking() {
        let result = json!({ "trackName": "Tiny App" });
        let m = parse_itunes_result(&result, &sample_ref());
        assert_eq!(m.name, "Tiny App");
        assert_eq!(m.description, "");
        assert_eq!(m.developer_url, None);
        assert_eq!(m.rating_avg, None);
        assert_eq!(m.screenshot_count, 0);
        assert!(m.store_url.contains("id310633997"));
    }

    #[test]
    fn mac_software_kind_marks_desktop() {
        let result = json!({ "trackName": "Klack", "kind": "mac-software" });
        let m = parse_itunes_result(&result, &sample_ref());
        assert!(m.is_desktop);
    }

    #[test]
    fn desktop_zero_ratings_become_fully_unknown() {
        // Apple reports 0/0 for ALL Mac App Store apps via this API,
        // regardless of their real ratings — must never read as a 0.00 score.
        let result = json!({
            "trackName": "Klack", "kind": "mac-software",
            "averageUserRating": 0, "userRatingCount": 0
        });
        let m = parse_itunes_result(&result, &sample_ref());
        assert_eq!(m.rating_avg, None);
        assert_eq!(m.rating_count, None);
    }

    #[test]
    fn mobile_zero_ratings_mean_genuinely_unrated() {
        let result = json!({
            "trackName": "Brand New App",
            "averageUserRating": 0, "userRatingCount": 0
        });
        let m = parse_itunes_result(&result, &sample_ref());
        assert_eq!(m.rating_avg, None, "0 is not an average");
        assert_eq!(m.rating_count, Some(0), "count 0 is real for mobile");
    }

    #[test]
    fn real_ratings_pass_through_untouched() {
        let result = json!({
            "trackName": "X", "averageUserRating": 4.7, "userRatingCount": 123u64
        });
        let m = parse_itunes_result(&result, &sample_ref());
        assert_eq!(m.rating_avg, Some(4.7));
        assert_eq!(m.rating_count, Some(123));
    }

    #[test]
    fn empty_strings_become_none() {
        let result = json!({ "trackName": "X", "sellerUrl": "  " });
        let m = parse_itunes_result(&result, &sample_ref());
        assert_eq!(m.developer_url, None);
    }

    #[test]
    fn encodes_query_values() {
        assert_eq!(urlencoding_encode("a b&c"), "a%20b%26c");
        assert_eq!(urlencoding_encode("Safe-1_.~"), "Safe-1_.~");
    }
}
