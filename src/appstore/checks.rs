//! App listing checks. Check-ID prefixes route into the app score
//! categories: `app-list-` (listing quality), `app-ans-` (AI answerability),
//! `app-rep-` (reputation signals), `app-web-` (developer web presence),
//! `app-x-` (cross-platform presence).
//!
//! Checks that depend on data a store doesn't expose (e.g. language count on
//! Play) are skipped rather than failed, so both stores score on an equal
//! footing.

use crate::scoring::{AnalysisResult, Status};
use crate::Report;

use super::{AppMetadata, CrossStoreOutcome, Store};

/// Marketing-fluff openers that tell an AI nothing about what the app does.
const FLUFF_PATTERNS: &[&str] = &[
    "#1", "no. 1", "number one", "award-winning", "award winning",
    "world's best", "worlds best", "best app", "millions of users",
    "million users", "million downloads", "join millions",
];

/// English function words excluded from keyword-repetition analysis —
/// they repeat naturally and say nothing about stuffing.
const STOPWORDS: &[&str] = &[
    "your", "with", "that", "this", "from", "have", "will", "they", "them",
    "when", "what", "where", "which", "about", "into", "over", "than", "then",
    "also", "more", "most", "some", "just", "like", "been", "being", "only",
    "other", "after", "before", "every", "each", "their", "there", "here",
    "yours", "ours", "these", "those", "while", "because", "through", "using",
];

/// Phrasings AI assistants map to user intents ("app for X" queries).
const USE_CASE_PATTERNS: &[&str] = &[
    "you can", "helps you", "help you", "use it to", "lets you", "allows you",
    "perfect for", "designed for", "ideal for", "whether you", "so you can",
    "keep track", "stay on top", "make it easy", "makes it easy",
];

/// Run every applicable check against the fetched listing metadata.
pub fn run_listing_checks(meta: &AppMetadata) -> Vec<AnalysisResult> {
    let mut results = vec![
        check_title(meta),
        check_description_length(meta),
        check_screenshots(meta),
        check_update_freshness(meta),
        check_release_notes(meta),
        check_category(meta),
        check_value_proposition(meta),
        check_use_cases(meta),
        check_keyword_stuffing(meta),
        check_feature_structure(meta),
        check_rating_average(meta),
        check_rating_volume(meta),
        check_content_rating(meta),
    ];
    if let Some(r) = check_localization(meta) {
        results.push(r);
    }
    if let Some(r) = check_installs(meta) {
        results.push(r);
    }
    results
}

// ── Listing quality ────────────────────────────────────────────────────────

fn check_title(meta: &AppMetadata) -> AnalysisResult {
    let len = meta.name.chars().count();
    if len == 0 {
        AnalysisResult {
            check: "app-list-title",
            status: Status::Fail,
            message: "App name could not be read from the listing.".to_string(),
            recommendation: "Ensure the store listing has a clear app name — it is the primary entity AI assistants anchor recommendations to.".to_string(),
        }
    } else if len <= 30 {
        AnalysisResult {
            check: "app-list-title",
            status: Status::Pass,
            message: format!("App name \"{}\" is clear and within store limits ({} chars).", meta.name, len),
            recommendation: String::new(),
        }
    } else {
        AnalysisResult {
            check: "app-list-title",
            status: Status::Warn,
            message: format!("App name is {} characters — beyond the 30-char store display limit, it may be truncated in search and AI citations.", len),
            recommendation: "Keep the core brand name short; move descriptive keywords into the subtitle/short description instead of the title.".to_string(),
        }
    }
}

fn check_description_length(meta: &AppMetadata) -> AnalysisResult {
    let len = meta.description.chars().count();
    if len >= 1000 {
        AnalysisResult {
            check: "app-list-description-length",
            status: Status::Pass,
            message: format!("Description is substantial ({} chars) — enough context for AI systems to understand and recommend the app.", len),
            recommendation: String::new(),
        }
    } else if len >= 300 {
        AnalysisResult {
            check: "app-list-description-length",
            status: Status::Warn,
            message: format!("Description is only {} characters — thin content gives AI assistants little to work with.", len),
            recommendation: "Expand the description toward 1,000-4,000 characters: what the app does, who it is for, and its key features in natural language.".to_string(),
        }
    } else {
        AnalysisResult {
            check: "app-list-description-length",
            status: Status::Fail,
            message: format!("Description is very short ({} chars). AI assistants cannot infer what the app does or who should use it.", len),
            recommendation: "Write a full description (1,000+ characters) covering the app's purpose, core features, and target users in complete sentences.".to_string(),
        }
    }
}

fn check_screenshots(meta: &AppMetadata) -> AnalysisResult {
    match meta.screenshot_count {
        0 => AnalysisResult {
            check: "app-list-screenshots",
            status: Status::Fail,
            message: "No screenshots detected on the listing.".to_string(),
            recommendation: "Add at least 5 screenshots. Visual assets signal listing quality and are referenced by review sites AI assistants cite.".to_string(),
        },
        1..=4 => AnalysisResult {
            check: "app-list-screenshots",
            status: Status::Warn,
            message: format!("Only {} screenshot(s) on the listing.", meta.screenshot_count),
            recommendation: "Add more screenshots (5-8) showing distinct features — richer listings rank better in store search, which feeds AI training and citation data.".to_string(),
        },
        n => AnalysisResult {
            check: "app-list-screenshots",
            status: Status::Pass,
            message: format!("{} screenshots present.", n),
            recommendation: String::new(),
        },
    }
}

fn check_update_freshness(meta: &AppMetadata) -> AnalysisResult {
    let parsed = meta
        .last_updated
        .as_deref()
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    let days = match parsed {
        Some(date) => (chrono::Utc::now().date_naive() - date).num_days(),
        None => {
            return AnalysisResult {
                check: "app-list-update-freshness",
                status: Status::Warn,
                message: "Could not determine when the app was last updated.".to_string(),
                recommendation: "Verify the listing shows a recent update date — stale apps are filtered out of both store rankings and AI recommendations.".to_string(),
            }
        }
    };

    if days <= 90 {
        AnalysisResult {
            check: "app-list-update-freshness",
            status: Status::Pass,
            message: format!("App updated {} days ago — actively maintained.", days),
            recommendation: String::new(),
        }
    } else if days <= 365 {
        AnalysisResult {
            check: "app-list-update-freshness",
            status: Status::Warn,
            message: format!("Last update was {} days ago.", days),
            recommendation: "Ship an update — recency is a strong trust signal. AI assistants deprioritize apps that look abandoned.".to_string(),
        }
    } else {
        AnalysisResult {
            check: "app-list-update-freshness",
            status: Status::Fail,
            message: format!("Last update was {} days ago — the app looks unmaintained.", days),
            recommendation: "Release an update. Apps untouched for over a year are routinely excluded from 'best app' roundups and AI answers.".to_string(),
        }
    }
}

fn check_release_notes(meta: &AppMetadata) -> AnalysisResult {
    match meta.release_notes.as_deref().map(str::trim) {
        Some(notes) if notes.chars().count() >= 50 => AnalysisResult {
            check: "app-list-release-notes",
            status: Status::Pass,
            message: "Release notes are present and descriptive.".to_string(),
            recommendation: String::new(),
        },
        Some(_) => AnalysisResult {
            check: "app-list-release-notes",
            status: Status::Warn,
            message: "Release notes exist but are minimal (e.g. \"bug fixes\").".to_string(),
            recommendation: "Describe what actually changed. Substantive release notes reinforce the active-maintenance signal.".to_string(),
        },
        None => AnalysisResult {
            check: "app-list-release-notes",
            status: Status::Fail,
            message: "No release notes found on the listing.".to_string(),
            recommendation: "Add release notes to every version — they are part of the freshness signal AI systems and review sites read.".to_string(),
        },
    }
}

fn check_category(meta: &AppMetadata) -> AnalysisResult {
    match meta.category.as_deref() {
        Some(cat) => AnalysisResult {
            check: "app-list-category",
            status: Status::Pass,
            message: format!("App is categorized as \"{}\".", cat),
            recommendation: String::new(),
        },
        None => AnalysisResult {
            check: "app-list-category",
            status: Status::Fail,
            message: "No category could be read from the listing.".to_string(),
            recommendation: "Set a precise store category — AI assistants answer 'best X app' queries by category framing.".to_string(),
        },
    }
}

fn check_localization(meta: &AppMetadata) -> Option<AnalysisResult> {
    // Only iTunes exposes the language list; skip on Play.
    let count = meta.language_count?;
    Some(if count >= 5 {
        AnalysisResult {
            check: "app-list-localization",
            status: Status::Pass,
            message: format!("Listing supports {} languages.", count),
            recommendation: String::new(),
        }
    } else if count >= 2 {
        AnalysisResult {
            check: "app-list-localization",
            status: Status::Warn,
            message: format!("Listing supports only {} languages.", count),
            recommendation: "Localize the listing for your key markets — AI assistants answer in the user's language and favor localized listings.".to_string(),
        }
    } else {
        AnalysisResult {
            check: "app-list-localization",
            status: Status::Warn,
            message: "Listing is available in a single language.".to_string(),
            recommendation: "Add localizations for your top non-English markets to widen AI visibility beyond one locale.".to_string(),
        }
    })
}

// ── AI answerability ───────────────────────────────────────────────────────

/// First paragraph of the description — what an AI reads first.
fn first_paragraph(description: &str) -> &str {
    let trimmed = description.trim();
    let para = trimmed.split("\n\n").next().unwrap_or(trimmed);
    // Fall back to a char-bounded window for single-block descriptions.
    match para.char_indices().nth(300) {
        Some((idx, _)) => &para[..idx],
        None => para,
    }
}

fn check_value_proposition(meta: &AppMetadata) -> AnalysisResult {
    let para = first_paragraph(&meta.description);
    let lower = para.to_lowercase();
    let len = para.chars().count();

    if len < 40 {
        return AnalysisResult {
            check: "app-ans-value-proposition",
            status: Status::Fail,
            message: "The description's opening is too short to state what the app does.".to_string(),
            recommendation: "Open with one plain sentence answering \"what does this app do and for whom?\" — it is the sentence AI assistants quote.".to_string(),
        };
    }

    let fluff = FLUFF_PATTERNS.iter().any(|p| lower.contains(p));
    if fluff {
        AnalysisResult {
            check: "app-ans-value-proposition",
            status: Status::Warn,
            message: "The description opens with marketing claims (awards/rankings/user counts) instead of stating what the app does.".to_string(),
            recommendation: "Lead with function, not accolades: \"X lets you do Y\" outperforms \"#1 award-winning app\" in AI answers, which strip unverifiable claims.".to_string(),
        }
    } else {
        AnalysisResult {
            check: "app-ans-value-proposition",
            status: Status::Pass,
            message: "Description opens with a substantive explanation of the app.".to_string(),
            recommendation: String::new(),
        }
    }
}

fn check_use_cases(meta: &AppMetadata) -> AnalysisResult {
    let lower = meta.description.to_lowercase();
    let hits = USE_CASE_PATTERNS
        .iter()
        .filter(|p| lower.contains(*p))
        .count();

    if hits >= 3 {
        AnalysisResult {
            check: "app-ans-use-cases",
            status: Status::Pass,
            message: format!("Description uses intent-oriented phrasing ({} use-case patterns found).", hits),
            recommendation: String::new(),
        }
    } else if hits >= 1 {
        AnalysisResult {
            check: "app-ans-use-cases",
            status: Status::Warn,
            message: "Description contains little use-case phrasing.".to_string(),
            recommendation: "Describe concrete scenarios (\"helps you…\", \"use it to…\", \"perfect for…\"). AI assistants match apps to user intents expressed in natural language.".to_string(),
        }
    } else {
        AnalysisResult {
            check: "app-ans-use-cases",
            status: Status::Fail,
            message: "Description never explains when or why someone would use the app.".to_string(),
            recommendation: "Rewrite features as user outcomes. \"Track expenses automatically\" answers the queries (\"app to track expenses\") that AI assistants actually receive.".to_string(),
        }
    }
}

fn check_keyword_stuffing(meta: &AppMetadata) -> AnalysisResult {
    // The app's own name repeating is branding, not stuffing.
    let brand_tokens: Vec<String> = super::normalize_app_name(&meta.name)
        .split(' ')
        .map(|t| t.to_string())
        .collect();
    let words: Vec<String> = meta
        .description
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| {
            w.chars().count() >= 4
                && !STOPWORDS.contains(w)
                && !brand_tokens.iter().any(|t| t == w)
        })
        .map(|w| w.to_string())
        .collect();

    if words.len() < 20 {
        // Too little text to judge — description-length checks already cover it.
        return AnalysisResult {
            check: "app-ans-keyword-stuffing",
            status: Status::Pass,
            message: "Description too short for repetition analysis; no stuffing detected.".to_string(),
            recommendation: String::new(),
        };
    }

    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for w in &words {
        *counts.entry(w.as_str()).or_insert(0) += 1;
    }
    let (top_word, top_count) = counts
        .iter()
        .max_by_key(|(_, c)| **c)
        .map(|(w, c)| (*w, *c))
        .unwrap_or(("", 0));
    let ratio = top_count as f64 / words.len() as f64;

    if ratio > 0.06 && top_count >= 8 {
        AnalysisResult {
            check: "app-ans-keyword-stuffing",
            status: Status::Fail,
            message: format!(
                "\"{}\" appears {} times ({:.0}% of meaningful words) — the description reads as keyword-stuffed.",
                top_word, top_count, ratio * 100.0
            ),
            recommendation: "Rewrite in natural language. LLMs summarize meaning, not keyword frequency — stuffing lowers the quality of AI-generated summaries of your app.".to_string(),
        }
    } else if ratio > 0.04 && top_count >= 6 {
        AnalysisResult {
            check: "app-ans-keyword-stuffing",
            status: Status::Warn,
            message: format!("\"{}\" repeats {} times — borderline keyword repetition.", top_word, top_count),
            recommendation: "Vary the phrasing; use synonyms and full sentences over repeated exact-match keywords.".to_string(),
        }
    } else {
        AnalysisResult {
            check: "app-ans-keyword-stuffing",
            status: Status::Pass,
            message: "Description reads as natural language without keyword stuffing.".to_string(),
            recommendation: String::new(),
        }
    }
}

fn check_feature_structure(meta: &AppMetadata) -> AnalysisResult {
    let desc = &meta.description;
    let bullet_lines = desc
        .lines()
        .map(str::trim)
        .filter(|l| {
            l.starts_with('•') || l.starts_with('-') || l.starts_with('*') || l.starts_with('✓')
                || l.starts_with('★')
        })
        .count();
    // Play descriptions are often collapsed to one line by the HTML-to-text
    // pass; count inline bullet glyphs as structure too.
    let inline_bullets = desc.matches('•').count();

    if bullet_lines >= 3 || inline_bullets >= 3 {
        AnalysisResult {
            check: "app-ans-feature-structure",
            status: Status::Pass,
            message: "Description structures features as a scannable list.".to_string(),
            recommendation: String::new(),
        }
    } else {
        AnalysisResult {
            check: "app-ans-feature-structure",
            status: Status::Warn,
            message: "Description is a wall of text without a feature list.".to_string(),
            recommendation: "Add a bulleted feature list — extractable structure is what listicle authors and AI summaries lift directly.".to_string(),
        }
    }
}

// ── Reputation signals ─────────────────────────────────────────────────────

fn check_rating_average(meta: &AppMetadata) -> AnalysisResult {
    // avg None + count None = the store doesn't expose ratings for this
    // listing (Mac App Store via the lookup API) — unverifiable, not zero.
    if meta.rating_avg.is_none() && meta.rating_count.is_none() {
        return AnalysisResult {
            check: "app-rep-rating-average",
            status: Status::Warn,
            message: "Ratings are not exposed for this listing via the store's public API.".to_string(),
            recommendation: "Check the rating directly in the store. This is a data-availability gap (common for Mac App Store apps), not a finding against the app.".to_string(),
        };
    }
    // avg None + count 0 = genuinely unrated new app.
    if meta.rating_avg.is_none() {
        return AnalysisResult {
            check: "app-rep-rating-average",
            status: Status::Fail,
            message: "The app has no ratings yet.".to_string(),
            recommendation: "Prompt engaged users for ratings via the official in-app review APIs.".to_string(),
        };
    }
    match meta.rating_avg {
        Some(avg) if avg >= 4.4 => AnalysisResult {
            check: "app-rep-rating-average",
            status: Status::Pass,
            message: format!("Strong average rating of {:.2}.", avg),
            recommendation: String::new(),
        },
        Some(avg) if avg >= 3.8 => AnalysisResult {
            check: "app-rep-rating-average",
            status: Status::Warn,
            message: format!("Average rating of {:.2} is decent but below the ~4.4 bar of typical 'best app' recommendations.", avg),
            recommendation: "Address the themes in critical reviews and prompt satisfied users to rate — AI recommendations lean heavily on rating quality.".to_string(),
        },
        Some(avg) => AnalysisResult {
            check: "app-rep-rating-average",
            status: Status::Fail,
            message: format!("Average rating of {:.2} actively deters recommendation.", avg),
            recommendation: "Fix the top complaint drivers before investing in visibility — a low rating caps every other signal.".to_string(),
        },
        // Unreachable — the None cases return above; kept for exhaustiveness.
        None => AnalysisResult {
            check: "app-rep-rating-average",
            status: Status::Fail,
            message: "The app has no visible rating yet.".to_string(),
            recommendation: "Prompt engaged users for ratings via the official in-app review APIs.".to_string(),
        },
    }
}

fn check_rating_volume(meta: &AppMetadata) -> AnalysisResult {
    match meta.rating_count {
        Some(n) if n >= 1000 => AnalysisResult {
            check: "app-rep-rating-volume",
            status: Status::Pass,
            message: format!("{} ratings — enough social proof for AI systems to treat the app as established.", format_count(n)),
            recommendation: String::new(),
        },
        Some(n) if n >= 100 => AnalysisResult {
            check: "app-rep-rating-volume",
            status: Status::Warn,
            message: format!("Only {} ratings — the app reads as niche or new.", n),
            recommendation: "Grow the rating base past ~1,000 with in-app review prompts at moments of success.".to_string(),
        },
        Some(n) => AnalysisResult {
            check: "app-rep-rating-volume",
            status: Status::Fail,
            message: format!("Only {} rating(s) — too little social proof for AI assistants to surface the app.", n),
            recommendation: "Ratings volume is the entry ticket: without a base of reviews, apps rarely appear in either store rankings or AI answers.".to_string(),
        },
        // None = the store didn't expose a count (Mac App Store, or a Play
        // parse gap) — unverifiable, so Warn rather than a false Fail.
        None => AnalysisResult {
            check: "app-rep-rating-volume",
            status: Status::Warn,
            message: "The rating count is not exposed for this listing via the store's public API.".to_string(),
            recommendation: "Verify the rating volume directly in the store. This is a data-availability gap, not a finding against the app.".to_string(),
        },
    }
}

fn check_installs(meta: &AppMetadata) -> Option<AnalysisResult> {
    // Play-only signal — iTunes never exposes install counts.
    if meta.store != Store::Android {
        return None;
    }
    let installs = meta.installs.as_deref()?;
    let magnitude = parse_install_magnitude(installs);
    Some(match magnitude {
        Some(n) if n >= 1_000_000 => AnalysisResult {
            check: "app-rep-installs",
            status: Status::Pass,
            message: format!("{} downloads — strong distribution signal.", installs),
            recommendation: String::new(),
        },
        Some(n) if n >= 10_000 => AnalysisResult {
            check: "app-rep-installs",
            status: Status::Warn,
            message: format!("{} downloads — moderate traction.", installs),
            recommendation: "Install volume compounds visibility; store featuring and external coverage both key off it.".to_string(),
        },
        Some(_) => AnalysisResult {
            check: "app-rep-installs",
            status: Status::Fail,
            message: format!("{} downloads — too little traction to register in recommendation data.", installs),
            recommendation: "Focus on distribution basics (ASO, referrals, press) — AI visibility follows real-world adoption.".to_string(),
        },
        None => AnalysisResult {
            check: "app-rep-installs",
            status: Status::Warn,
            message: format!("Could not interpret the download figure \"{}\".", installs),
            recommendation: String::new(),
        },
    })
}

fn check_content_rating(meta: &AppMetadata) -> AnalysisResult {
    match meta.content_rating.as_deref() {
        Some(rating) => AnalysisResult {
            check: "app-rep-content-rating",
            status: Status::Pass,
            message: format!("Content rating is declared (\"{}\").", rating),
            recommendation: String::new(),
        },
        None => AnalysisResult {
            check: "app-rep-content-rating",
            status: Status::Warn,
            message: "No content rating visible on the listing.".to_string(),
            recommendation: "Complete the store's content-rating questionnaire — missing ratings suppress the listing in family-safe filtered surfaces.".to_string(),
        },
    }
}

// ── Developer web presence ─────────────────────────────────────────────────

/// Checks derived from the developer-website GEO analysis. `site_report` is
/// the single-page web analysis of the listing's developer URL, when one ran.
pub fn run_web_presence_checks(
    meta: &AppMetadata,
    site_report: Option<&Report>,
) -> Vec<AnalysisResult> {
    let mut results = Vec::new();

    match (&meta.developer_url, site_report) {
        (None, _) => {
            results.push(AnalysisResult {
                check: "app-web-developer-site",
                status: Status::Fail,
                message: "The listing declares no developer website.".to_string(),
                recommendation: "Add your website to the store listing. AI assistants cross-reference apps with their web presence — no website means no citable footprint.".to_string(),
            });
        }
        (Some(url), None) => {
            results.push(AnalysisResult {
                check: "app-web-developer-site",
                status: Status::Fail,
                message: format!("Developer website {} could not be analyzed (unreachable or failed).", url),
                recommendation: "Make sure the website on your listing loads reliably — a dead link is a negative trust signal for both stores and AI systems.".to_string(),
            });
        }
        (Some(url), Some(report)) => {
            results.push(AnalysisResult {
                check: "app-web-developer-site",
                status: Status::Pass,
                message: format!("Developer website {} is live and was analyzed.", url),
                recommendation: String::new(),
            });

            let geo_score = report.score;
            results.push(if geo_score >= 70.0 {
                AnalysisResult {
                    check: "app-web-geo-score",
                    status: Status::Pass,
                    message: format!("Developer site scores {:.0}/100 on GEO — a solid citable web footprint.", geo_score),
                    recommendation: String::new(),
                }
            } else if geo_score >= 40.0 {
                AnalysisResult {
                    check: "app-web-geo-score",
                    status: Status::Warn,
                    message: format!("Developer site scores {:.0}/100 on GEO.", geo_score),
                    recommendation: "Improve the site's GEO fundamentals (see the embedded website report) — AI answers about apps cite the developer's web content, not the store page.".to_string(),
                }
            } else {
                AnalysisResult {
                    check: "app-web-geo-score",
                    status: Status::Fail,
                    message: format!("Developer site scores only {:.0}/100 on GEO.", geo_score),
                    recommendation: "The website is the app's citable surface for AI engines. Run a full geodaddy analysis on it and fix the failing checks.".to_string(),
                }
            });

            // Surface two high-leverage site signals directly in the app report.
            results.push(derive_site_check(
                report,
                "geo-llms-txt",
                "app-web-llms-txt",
                "Developer site provides an llms.txt index.",
                "Developer site has no (or an invalid) llms.txt.",
                "Add an llms.txt to the developer site so AI crawlers can navigate straight to app documentation and feature pages.",
            ));
            results.push(derive_site_check(
                report,
                "cont-json-ld",
                "app-web-structured-data",
                "Developer site ships structured data (JSON-LD).",
                "Developer site lacks structured data.",
                "Add JSON-LD to the developer site — ideally a SoftwareApplication schema describing the app, its platforms, and store links.",
            ));
        }
    }

    results
}

/// Map one check's status from the embedded site report onto an app-web
/// check ID. Absent from the site report → Warn (cannot verify).
fn derive_site_check(
    report: &Report,
    source_check: &str,
    target_check: &'static str,
    pass_msg: &str,
    fail_msg: &str,
    recommendation: &str,
) -> AnalysisResult {
    let status = report
        .pages
        .first()
        .and_then(|p| p.results.iter().find(|r| r.check == source_check))
        .map(|r| r.status.clone());

    match status {
        Some(Status::Pass) => AnalysisResult {
            check: target_check,
            status: Status::Pass,
            message: pass_msg.to_string(),
            recommendation: String::new(),
        },
        Some(Status::Warn) => AnalysisResult {
            check: target_check,
            status: Status::Warn,
            message: fail_msg.to_string(),
            recommendation: recommendation.to_string(),
        },
        Some(Status::Fail) => AnalysisResult {
            check: target_check,
            status: Status::Fail,
            message: fail_msg.to_string(),
            recommendation: recommendation.to_string(),
        },
        None => AnalysisResult {
            check: target_check,
            status: Status::Warn,
            message: format!("Could not evaluate {} on the developer site.", source_check),
            recommendation: recommendation.to_string(),
        },
    }
}

// ── Cross-platform presence ────────────────────────────────────────────────

pub fn run_cross_platform_checks(
    meta: &AppMetadata,
    outcome: &CrossStoreOutcome,
) -> Vec<AnalysisResult> {
    let other_store = match meta.store {
        Store::Ios => "Play Store",
        Store::Android => "App Store",
    };

    match outcome {
        CrossStoreOutcome::Found { name, name_identical, .. } => {
            let mut results = vec![AnalysisResult {
                check: "app-x-other-store",
                status: Status::Pass,
                message: format!(
                    "App also found on the {} (\"{}\" — developer verified).",
                    other_store, name
                ),
                recommendation: String::new(),
            }];
            results.push(if *name_identical {
                AnalysisResult {
                    check: "app-x-name-consistency",
                    status: Status::Pass,
                    message: "App name is consistent across both stores.".to_string(),
                    recommendation: String::new(),
                }
            } else {
                AnalysisResult {
                    check: "app-x-name-consistency",
                    status: Status::Warn,
                    message: format!(
                        "The {} listing is named \"{}\" — it differs from \"{}\".",
                        other_store, name, meta.name
                    ),
                    recommendation: "Use an identical app name on both stores. Inconsistent naming splits the app's identity in AI training data and weakens entity recognition.".to_string(),
                }
            });
            results
        }
        CrossStoreOutcome::NotFound => vec![AnalysisResult {
            check: "app-x-other-store",
            status: Status::Fail,
            message: format!("No matching listing found on the {}.", other_store),
            recommendation: format!(
                "Publish on the {} if feasible, or ensure your website links both platforms. Single-platform apps get recommended less because assistants can't serve both iOS and Android askers.",
                other_store
            ),
        }],
        CrossStoreOutcome::Unknown(reason) => vec![AnalysisResult {
            check: "app-x-other-store",
            status: Status::Warn,
            message: format!("Could not verify {} presence ({}).", other_store, reason),
            recommendation: "Cross-store presence could not be checked this run — treat this as unverified rather than missing.".to_string(),
        }],
    }
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn format_count(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

/// Turn Play's install bucket ("10B+", "500K+", "1,000+") into a number.
fn parse_install_magnitude(raw: &str) -> Option<u64> {
    let cleaned = raw.trim().trim_end_matches('+').replace(',', "");
    let (num_part, mult) = match cleaned.chars().last()? {
        'B' => (&cleaned[..cleaned.len() - 1], 1_000_000_000u64),
        'M' => (&cleaned[..cleaned.len() - 1], 1_000_000u64),
        'K' => (&cleaned[..cleaned.len() - 1], 1_000u64),
        _ => (cleaned.as_str(), 1u64),
    };
    let value: f64 = num_part.parse().ok()?;
    Some((value * mult as f64) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_meta() -> AppMetadata {
        AppMetadata {
            store: Store::Ios,
            app_id: "1".to_string(),
            country: "us".to_string(),
            is_desktop: false,
            name: "Test App".to_string(),
            description: String::new(),
            release_notes: None,
            developer_name: None,
            developer_url: None,
            category: None,
            rating_avg: None,
            rating_count: None,
            price: None,
            screenshot_count: 0,
            last_updated: None,
            content_rating: None,
            language_count: None,
            installs: None,
            icon_url: None,
            store_url: "https://apps.apple.com/us/app/id1".to_string(),
        }
    }

    #[test]
    fn title_length_thresholds() {
        let mut m = base_meta();
        assert_eq!(check_title(&m).status, Status::Pass);
        m.name = "A".repeat(31);
        assert_eq!(check_title(&m).status, Status::Warn);
        m.name = String::new();
        assert_eq!(check_title(&m).status, Status::Fail);
    }

    #[test]
    fn description_length_thresholds() {
        let mut m = base_meta();
        m.description = "x".repeat(1200);
        assert_eq!(check_description_length(&m).status, Status::Pass);
        m.description = "x".repeat(500);
        assert_eq!(check_description_length(&m).status, Status::Warn);
        m.description = "short".to_string();
        assert_eq!(check_description_length(&m).status, Status::Fail);
    }

    #[test]
    fn freshness_thresholds() {
        let mut m = base_meta();
        let today = chrono::Utc::now().date_naive();
        m.last_updated = Some((today - chrono::Duration::days(10)).format("%Y-%m-%d").to_string());
        assert_eq!(check_update_freshness(&m).status, Status::Pass);
        m.last_updated = Some((today - chrono::Duration::days(200)).format("%Y-%m-%d").to_string());
        assert_eq!(check_update_freshness(&m).status, Status::Warn);
        m.last_updated = Some((today - chrono::Duration::days(500)).format("%Y-%m-%d").to_string());
        assert_eq!(check_update_freshness(&m).status, Status::Fail);
        m.last_updated = None;
        assert_eq!(check_update_freshness(&m).status, Status::Warn);
    }

    #[test]
    fn value_proposition_detects_fluff() {
        let mut m = base_meta();
        m.description = "The #1 award-winning app loved by millions of users around the entire world today".to_string();
        assert_eq!(check_value_proposition(&m).status, Status::Warn);
        m.description = "Track your daily expenses automatically by connecting your bank accounts and get weekly spending insights.".to_string();
        assert_eq!(check_value_proposition(&m).status, Status::Pass);
        m.description = "Great app!".to_string();
        assert_eq!(check_value_proposition(&m).status, Status::Fail);
    }

    #[test]
    fn use_case_pattern_counting() {
        let mut m = base_meta();
        m.description = "It helps you plan. You can share lists. Perfect for families.".to_string();
        assert_eq!(check_use_cases(&m).status, Status::Pass);
        m.description = "It helps you plan things.".to_string();
        assert_eq!(check_use_cases(&m).status, Status::Warn);
        m.description = "Feature A. Feature B. Feature C.".to_string();
        assert_eq!(check_use_cases(&m).status, Status::Fail);
    }

    #[test]
    fn keyword_stuffing_detection() {
        let mut m = base_meta();
        let natural: String = "This journal application records daily thoughts with photos \
            location tags and gentle reminders while keeping everything encrypted locally \
            so nothing leaves the device without explicit permission from the owner"
            .split_whitespace().collect::<Vec<_>>().join(" ");
        m.description = natural;
        assert_eq!(check_keyword_stuffing(&m).status, Status::Pass);

        let stuffed = "budget planner budget tracker budget app budget tool budget manager \
            budget helper budget assistant budget wizard budget expert budget master \
            budget guide budget keeper daily budget smart budget easy budget";
        m.description = stuffed.to_string();
        let r = check_keyword_stuffing(&m);
        assert!(matches!(r.status, Status::Fail | Status::Warn), "stuffed description must not pass");
    }

    #[test]
    fn rating_thresholds() {
        let mut m = base_meta();
        m.rating_avg = Some(4.7);
        assert_eq!(check_rating_average(&m).status, Status::Pass);
        m.rating_avg = Some(4.0);
        assert_eq!(check_rating_average(&m).status, Status::Warn);
        m.rating_avg = Some(3.2);
        assert_eq!(check_rating_average(&m).status, Status::Fail);
        // avg None + count None → store doesn't expose ratings → Warn
        m.rating_avg = None;
        m.rating_count = None;
        assert_eq!(check_rating_average(&m).status, Status::Warn);
        assert_eq!(check_rating_volume(&m).status, Status::Warn);
        // avg None + count 0 → genuinely unrated → Fail
        m.rating_count = Some(0);
        assert_eq!(check_rating_average(&m).status, Status::Fail);
        assert_eq!(check_rating_volume(&m).status, Status::Fail);

        m.rating_count = Some(50_000);
        assert_eq!(check_rating_volume(&m).status, Status::Pass);
        m.rating_count = Some(300);
        assert_eq!(check_rating_volume(&m).status, Status::Warn);
        m.rating_count = Some(12);
        assert_eq!(check_rating_volume(&m).status, Status::Fail);
    }

    #[test]
    fn installs_only_emitted_for_android() {
        let mut m = base_meta();
        m.installs = Some("10B+".to_string());
        assert!(check_installs(&m).is_none(), "iOS never emits the installs check");
        m.store = Store::Android;
        let r = check_installs(&m).unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn install_magnitude_parsing() {
        assert_eq!(parse_install_magnitude("10B+"), Some(10_000_000_000));
        assert_eq!(parse_install_magnitude("500K+"), Some(500_000));
        assert_eq!(parse_install_magnitude("1,000+"), Some(1_000));
        assert_eq!(parse_install_magnitude("garbage"), None);
    }

    #[test]
    fn localization_only_when_known() {
        let mut m = base_meta();
        assert!(check_localization(&m).is_none());
        m.language_count = Some(12);
        assert_eq!(check_localization(&m).unwrap().status, Status::Pass);
        m.language_count = Some(1);
        assert_eq!(check_localization(&m).unwrap().status, Status::Warn);
    }

    #[test]
    fn web_presence_without_url_fails() {
        let m = base_meta();
        let results = run_web_presence_checks(&m, None);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].check, "app-web-developer-site");
        assert_eq!(results[0].status, Status::Fail);
    }

    #[test]
    fn web_presence_with_unreachable_site_fails() {
        let mut m = base_meta();
        m.developer_url = Some("https://example.com".to_string());
        let results = run_web_presence_checks(&m, None);
        assert_eq!(results[0].status, Status::Fail);
        assert!(results[0].message.contains("could not be analyzed"));
    }

    #[test]
    fn cross_platform_outcomes() {
        let m = base_meta();
        let found = CrossStoreOutcome::Found {
            url: "https://play.google.com/store/apps/details?id=x".to_string(),
            name: "Test App".to_string(),
            name_identical: true,
        };
        let results = run_cross_platform_checks(&m, &found);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.status == Status::Pass));

        let results = run_cross_platform_checks(&m, &CrossStoreOutcome::NotFound);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, Status::Fail);

        let results =
            run_cross_platform_checks(&m, &CrossStoreOutcome::Unknown("timeout".to_string()));
        assert_eq!(results[0].status, Status::Warn);
    }
}
