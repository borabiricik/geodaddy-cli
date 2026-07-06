//! Play Store listing parser. Google publishes no metadata API, so this
//! reads the server-rendered details page — no headless browser needed.
//!
//! Extraction is layered by stability: schema.org JSON-LD first (machine-
//! intended, most stable), microdata (`itemprop`) second, raw-HTML label
//! patterns last. Each layer degrades to `None` on mismatch so a Google
//! layout change weakens the report instead of breaking it.

use anyhow::{Context, Result};
use regex::Regex;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashSet;
use std::sync::OnceLock;

use super::itunes::urlencoding_encode;
use super::{
    developers_match, names_identical, names_match, AppMetadata, AppRef, CrossStoreOutcome, Store,
};

/// Play serves a JS-shell to unknown agents; a browser UA gets the fully
/// server-rendered page. Applied per-request so library consumers (the
/// backend) keep their own client-level UA for every other fetch.
const BROWSER_UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) \
    AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0 Safari/537.36";

/// Play page bodies run ~1-2 MB; cap well above that but below the global
/// crawler cap to bound memory.
const PLAY_BODY_CAP: usize = 4 * 1024 * 1024;

fn updated_on_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"Updated on</div><div[^>]*>([^<]+)</div>").unwrap())
}

fn downloads_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"([\d.,]+[KMB]?\+?)</div><div[^>]*>Downloads").unwrap())
}

fn screenshot_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r#"(https://play-lh\.googleusercontent\.com/[^"'\s]+=w526-h296)"#).unwrap()
    })
}

fn details_link_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"/store/apps/details\?id=([A-Za-z0-9._]+)").unwrap())
}

/// Fetch and parse Android listing metadata from the Play details page.
pub async fn fetch_android_metadata(
    client: &reqwest::Client,
    app_ref: &AppRef,
) -> Result<AppMetadata> {
    // hl is pinned to en because the label-based extractors ("Updated on",
    // "Downloads") match the English page; gl keeps the user's storefront.
    let page_url = format!(
        "https://play.google.com/store/apps/details?id={}&hl=en&gl={}",
        app_ref.id,
        app_ref.country.to_ascii_uppercase()
    );

    let resp = client
        .get(&page_url)
        .header(reqwest::header::USER_AGENT, BROWSER_UA)
        .send()
        .await
        .with_context(|| format!("Play Store request failed for package {}", app_ref.id))?;

    let status = resp.status();
    if status.as_u16() == 404 {
        anyhow::bail!(
            "Package {} not found on the {} Play Store storefront",
            app_ref.id,
            app_ref.country
        );
    }
    if !status.is_success() {
        anyhow::bail!(
            "Play Store returned HTTP {} for package {} (Google may be rate-limiting — retry shortly)",
            status,
            app_ref.id
        );
    }

    let mut html = resp
        .text()
        .await
        .with_context(|| format!("Failed to read Play Store body for {}", app_ref.id))?;
    if html.len() > PLAY_BODY_CAP {
        html.truncate(crate::crawling::truncate_utf8(&html, PLAY_BODY_CAP).len());
    }

    let meta = parse_play_html(&html, app_ref);
    if meta.name.is_empty() {
        anyhow::bail!(
            "Could not parse the Play Store page for {} — Google may have changed the page layout or served a consent wall",
            app_ref.id
        );
    }
    Ok(meta)
}

/// Parse a Play details page. Pure function — testable against fixtures.
pub fn parse_play_html(html: &str, app_ref: &AppRef) -> AppMetadata {
    let doc = Html::parse_document(html);

    let ld = extract_json_ld(&doc);
    let ld_str = |path: &[&str]| -> Option<String> {
        let mut cur = ld.as_ref()?;
        for key in path {
            cur = cur.get(key)?;
        }
        cur.as_str()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };

    // Rating values arrive as JSON strings ("4.649…"), occasionally numbers.
    let ld_num = |path: &[&str]| -> Option<f64> {
        let mut cur = ld.as_ref()?;
        for key in path {
            cur = cur.get(key)?;
        }
        cur.as_f64()
            .or_else(|| cur.as_str().and_then(|s| s.parse::<f64>().ok()))
    };

    let (description, release_notes) = extract_descriptions(&doc);

    let last_updated = updated_on_re()
        .captures(html)
        .and_then(|c| parse_play_date(c.get(1).map_or("", |m| m.as_str())));

    let installs = downloads_re()
        .captures(html)
        .map(|c| c[1].to_string());

    let screenshot_count = screenshot_re()
        .captures_iter(html)
        .map(|c| c[1].to_string())
        .collect::<HashSet<_>>()
        .len();

    let price = ld
        .as_ref()
        .and_then(|v| v.get("offers"))
        .and_then(|o| o.as_array())
        .and_then(|arr| arr.first())
        .and_then(|offer| offer.get("price"))
        .and_then(|p| p.as_str().map(|s| s.to_string()).or_else(|| p.as_f64().map(|n| n.to_string())))
        .map(|p| if p == "0" { "Free".to_string() } else { p });

    AppMetadata {
        store: Store::Android,
        app_id: app_ref.id.clone(),
        country: app_ref.country.clone(),
        is_desktop: false,
        name: ld_str(&["name"]).unwrap_or_default(),
        description,
        release_notes,
        developer_name: ld_str(&["author", "name"]),
        developer_url: ld_str(&["author", "url"]),
        category: ld_str(&["applicationCategory"]).map(humanize_play_category),
        rating_avg: ld_num(&["aggregateRating", "ratingValue"]),
        rating_count: ld_num(&["aggregateRating", "ratingCount"]).map(|v| v as u64),
        price,
        screenshot_count,
        last_updated,
        content_rating: ld_str(&["contentRating"]),
        language_count: None,
        installs,
        icon_url: ld_str(&["image"]),
        store_url: format!(
            "https://play.google.com/store/apps/details?id={}",
            app_ref.id
        ),
    }
}

/// Pull the SoftwareApplication JSON-LD block out of the page.
fn extract_json_ld(doc: &Html) -> Option<Value> {
    let selector = Selector::parse(r#"script[type="application/ld+json"]"#).ok()?;
    for script in doc.select(&selector) {
        let text: String = script.text().collect();
        if let Ok(v) = serde_json::from_str::<Value>(&text) {
            let is_app = v
                .get("@type")
                .and_then(|t| t.as_str())
                .is_some_and(|t| t == "SoftwareApplication");
            if is_app {
                return Some(v);
            }
        }
    }
    None
}

/// The full description lives in `div[data-g-id="description"]`; the
/// "What's new" section is the `div[itemprop="description"]` block (the
/// `meta[itemprop="description"]` tag only carries the short tagline).
/// When the data-g-id anchor disappears, fall back to the longest
/// itemprop block so a Google layout change degrades instead of breaking.
fn extract_descriptions(doc: &Html) -> (String, Option<String>) {
    let text_of = |el: scraper::ElementRef| -> String {
        el.text()
            .collect::<Vec<_>>()
            .join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };

    let mut itemprop_texts: Vec<String> = Selector::parse(r#"div[itemprop="description"]"#)
        .ok()
        .map(|sel| {
            doc.select(&sel)
                .map(text_of)
                .filter(|t| !t.is_empty())
                .collect()
        })
        .unwrap_or_default();
    itemprop_texts.sort_by_key(|t| std::cmp::Reverse(t.len()));

    let description = Selector::parse(r#"div[data-g-id="description"]"#)
        .ok()
        .and_then(|sel| doc.select(&sel).map(text_of).max_by_key(String::len))
        .filter(|t| !t.is_empty())
        .unwrap_or_else(|| itemprop_texts.first().cloned().unwrap_or_default());

    let release_notes = itemprop_texts
        .into_iter()
        .find(|t| t.len() >= 20 && *t != description);

    (description, release_notes)
}

/// Parse Play's "Jul 1, 2026" style date into ISO YYYY-MM-DD.
fn parse_play_date(raw: &str) -> Option<String> {
    chrono::NaiveDate::parse_from_str(raw.trim(), "%b %d, %Y")
        .ok()
        .map(|d| d.format("%Y-%m-%d").to_string())
}

/// Play categories arrive as enum-style tokens ("MAPS_AND_NAVIGATION");
/// humanize for display parity with iTunes genre names.
fn humanize_play_category(raw: String) -> String {
    raw.split('_')
        .map(|w| {
            let lower = w.to_ascii_lowercase();
            if lower == "and" {
                lower
            } else {
                let mut c = lower.chars();
                match c.next() {
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    None => String::new(),
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// How many distinct search-result packages get fetched and verified before
/// giving up. Each candidate costs one details-page fetch; the right app is
/// almost always in the first slots when it exists at all.
const MAX_SEARCH_CANDIDATES: usize = 3;

/// Search the Play Store for an app by name (cross-platform check for iOS
/// apps). Search-result titles are not reliably extractable from the search
/// page markup, so the top candidates' details pages are fetched and their
/// JSON-LD name AND developer compared against the source listing — a
/// name-only match is reported as Unknown (unverified), never linked.
pub async fn search_play_by_name(
    client: &reqwest::Client,
    name: &str,
    developer: Option<&str>,
    country: &str,
) -> CrossStoreOutcome {
    let search_url = format!(
        "https://play.google.com/store/search?q={}&c=apps&hl=en&gl={}",
        urlencoding_encode(name),
        country.to_ascii_uppercase()
    );

    let html = match client
        .get(&search_url)
        .header(reqwest::header::USER_AGENT, BROWSER_UA)
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => match resp.text().await {
            Ok(t) => t,
            Err(e) => return CrossStoreOutcome::Unknown(format!("Play search body error: {}", e)),
        },
        Ok(resp) => {
            return CrossStoreOutcome::Unknown(format!("Play search HTTP {}", resp.status()))
        }
        Err(e) => return CrossStoreOutcome::Unknown(format!("Play search failed: {}", e)),
    };

    let mut candidates: Vec<String> = Vec::new();
    for cap in details_link_re().captures_iter(&html) {
        let package = cap[1].to_string();
        if !candidates.contains(&package) {
            candidates.push(package);
        }
        if candidates.len() >= MAX_SEARCH_CANDIDATES {
            break;
        }
    }
    if candidates.is_empty() {
        return CrossStoreOutcome::NotFound;
    }

    let mut name_only_hit = false;
    for package in candidates {
        let candidate_ref = AppRef {
            store: Store::Android,
            id: package.clone(),
            country: country.to_string(),
        };
        let meta = match fetch_android_metadata(client, &candidate_ref).await {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!("Play candidate {} could not be fetched: {}", package, e);
                continue;
            }
        };
        if !names_match(&meta.name, name) {
            continue;
        }
        match developers_match(meta.developer_name.as_deref(), developer) {
            Some(true) => {
                return CrossStoreOutcome::Found {
                    name_identical: names_identical(&meta.name, name),
                    url: meta.store_url,
                    name: meta.name,
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ref() -> AppRef {
        AppRef {
            store: Store::Android,
            id: "com.whatsapp".to_string(),
            country: "us".to_string(),
        }
    }

    fn fixture_html() -> String {
        r##"<html><head>
        <script type="application/ld+json">{
          "@context": "https://schema.org",
          "@type": "SoftwareApplication",
          "name": "WhatsApp Messenger",
          "description": "Simple. Reliable. Private.",
          "applicationCategory": "COMMUNICATION",
          "contentRating": "Everyone",
          "image": "https://play-lh.googleusercontent.com/icon",
          "author": {"@type": "Person", "name": "WhatsApp LLC", "url": "http://www.whatsapp.com/"},
          "aggregateRating": {"@type": "AggregateRating", "ratingValue": "4.6492", "ratingCount": "238072648"},
          "offers": [{"@type": "Offer", "price": "0", "priceCurrency": "USD"}]
        }</script>
        </head><body>
        <meta itemprop="description" content="Simple. Reliable. Private.">
        <div class="bARER" data-g-id="description">WhatsApp from Meta is a FREE messaging and video calling app. It is used by over 2B people in more than 180 countries. Private messaging that works across mobile and desktop with end to end encryption for your personal conversations every single day.</div>
        <div itemprop="description">Bug fixes and improvements for group calls and channels.</div>
        <div>Updated on</div><div class="x">Jul 1, 2026</div>
        <div>10B+</div><div class="y">Downloads</div>
        <img src="https://play-lh.googleusercontent.com/shot1=w526-h296">
        <img src="https://play-lh.googleusercontent.com/shot2=w526-h296">
        <img src="https://play-lh.googleusercontent.com/shot2=w526-h296">
        </body></html>"##
            .to_string()
    }

    #[test]
    fn parses_fixture_page() {
        let m = parse_play_html(&fixture_html(), &sample_ref());
        assert_eq!(m.name, "WhatsApp Messenger");
        assert_eq!(m.developer_name.as_deref(), Some("WhatsApp LLC"));
        assert_eq!(m.developer_url.as_deref(), Some("http://www.whatsapp.com/"));
        assert_eq!(m.category.as_deref(), Some("Communication"));
        assert!(m.rating_avg.unwrap() > 4.6 && m.rating_avg.unwrap() < 4.7);
        assert_eq!(m.rating_count, Some(238072648));
        assert_eq!(m.price.as_deref(), Some("Free"));
        assert_eq!(m.last_updated.as_deref(), Some("2026-07-01"));
        assert_eq!(m.installs.as_deref(), Some("10B+"));
        assert_eq!(m.screenshot_count, 2, "duplicate screenshot URLs are deduped");
        assert!(m.description.starts_with("WhatsApp from Meta"));
        assert_eq!(
            m.release_notes.as_deref(),
            Some("Bug fixes and improvements for group calls and channels.")
        );
        assert_eq!(m.content_rating.as_deref(), Some("Everyone"));
    }

    #[test]
    fn empty_page_degrades_to_empty_metadata() {
        let m = parse_play_html("<html><body>nothing here</body></html>", &sample_ref());
        assert_eq!(m.name, "");
        assert_eq!(m.rating_avg, None);
        assert_eq!(m.last_updated, None);
        assert_eq!(m.installs, None);
        assert_eq!(m.screenshot_count, 0);
    }

    #[test]
    fn parses_play_dates() {
        assert_eq!(parse_play_date("Jul 1, 2026").as_deref(), Some("2026-07-01"));
        assert_eq!(parse_play_date("Dec 25, 2025").as_deref(), Some("2025-12-25"));
        assert_eq!(parse_play_date("not a date"), None);
    }

    #[test]
    fn humanizes_categories() {
        assert_eq!(humanize_play_category("MAPS_AND_NAVIGATION".into()), "Maps and Navigation");
        assert_eq!(humanize_play_category("COMMUNICATION".into()), "Communication");
    }

    #[test]
    fn extracts_first_details_link() {
        let html = r#"<a href="/store/apps/details?id=com.whatsapp"></a>
                      <a href="/store/apps/details?id=org.telegram.messenger"></a>"#;
        let first = details_link_re()
            .captures_iter(html)
            .map(|c| c[1].to_string())
            .next();
        assert_eq!(first.as_deref(), Some("com.whatsapp"));
    }
}
