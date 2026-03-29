use crate::scoring::{AnalysisResult, Status};
use reqwest::header::HeaderMap;
use scraper::{Html, Selector};

/// AI-specific directives to detect in meta robots and X-Robots-Tag headers.
const AI_DIRECTIVES: &[&str] = &["noai", "noimageai", "nosnippet"];

/// Check for AI-blocking directives in `<meta name="robots">` tags.
///
/// Detects noai, noimageai, and nosnippet directives (case-insensitive).
/// Standard SEO directives like noindex are ignored.
pub fn analyze_ai_meta_directives(html: &Html) -> AnalysisResult {
    let meta_sel = Selector::parse(r#"meta[name="robots"]"#).expect("valid selector");
    let mut found_directives: Vec<String> = Vec::new();

    for el in html.select(&meta_sel) {
        if let Some(content) = el.value().attr("content") {
            let lower = content.to_lowercase();
            for directive in AI_DIRECTIVES {
                if lower.contains(directive) && !found_directives.contains(&directive.to_string()) {
                    found_directives.push(directive.to_string());
                }
            }
        }
    }

    if found_directives.is_empty() {
        AnalysisResult {
            check: "geo-ai-meta-directives",
            status: Status::Pass,
            message: "No AI-blocking meta robot directives found. AI crawlers can process this page.".to_string(),
            recommendation: String::new(),
        }
    } else {
        AnalysisResult {
            check: "geo-ai-meta-directives",
            status: Status::Warn,
            message: format!(
                "AI-blocking meta directives detected: {}. These may prevent AI systems from using your content in search results.",
                found_directives.join(", ")
            ),
            recommendation: "Review your meta robots directives. If you want AI search visibility, remove noai/noimageai directives. If intentional, this is expected.".to_string(),
        }
    }
}

/// Check for AI-blocking directives in X-Robots-Tag HTTP headers.
///
/// Detects noai, noimageai, and nosnippet directives (case-insensitive).
pub fn analyze_ai_header_directives(headers: &HeaderMap) -> AnalysisResult {
    let mut found_directives: Vec<String> = Vec::new();

    if let Some(value) = headers.get("x-robots-tag") {
        if let Ok(val_str) = value.to_str() {
            let lower = val_str.to_lowercase();
            for directive in AI_DIRECTIVES {
                if lower.contains(directive) && !found_directives.contains(&directive.to_string()) {
                    found_directives.push(directive.to_string());
                }
            }
        }
    }

    if found_directives.is_empty() {
        AnalysisResult {
            check: "geo-ai-header-directives",
            status: Status::Pass,
            message: "No AI-blocking X-Robots-Tag headers found.".to_string(),
            recommendation: String::new(),
        }
    } else {
        AnalysisResult {
            check: "geo-ai-header-directives",
            status: Status::Warn,
            message: format!(
                "AI-blocking X-Robots-Tag directives detected: {}.",
                found_directives.join(", ")
            ),
            recommendation: "Review your X-Robots-Tag HTTP headers. noai/noimageai directives may prevent AI search engines from including your content in results.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderMap;
    use scraper::Html;

    fn parse(html: &str) -> Html {
        Html::parse_document(html)
    }

    // --- analyze_ai_meta_directives ---

    #[test]
    fn no_meta_robots_returns_pass() {
        let doc = parse("<html><head></head><body></body></html>");
        let r = analyze_ai_meta_directives(&doc);
        assert_eq!(r.check, "geo-ai-meta-directives");
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn noai_meta_returns_warn() {
        let doc = parse(r#"<html><head><meta name="robots" content="noai"></head></html>"#);
        let r = analyze_ai_meta_directives(&doc);
        assert_eq!(r.status, Status::Warn);
        assert!(r.message.contains("noai"));
    }

    #[test]
    fn multiple_ai_directives_detected() {
        let doc = parse(r#"<html><head><meta name="robots" content="NoAI, noimageai"></head></html>"#);
        let r = analyze_ai_meta_directives(&doc);
        assert_eq!(r.status, Status::Warn);
        assert!(r.message.contains("noai"));
        assert!(r.message.contains("noimageai"));
    }

    #[test]
    fn noindex_not_flagged() {
        let doc = parse(r#"<html><head><meta name="robots" content="noindex"></head></html>"#);
        let r = analyze_ai_meta_directives(&doc);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn case_insensitive_meta() {
        let doc = parse(r#"<html><head><meta name="robots" content="NOAI"></head></html>"#);
        let r = analyze_ai_meta_directives(&doc);
        assert_eq!(r.status, Status::Warn);
    }

    // --- analyze_ai_header_directives ---

    #[test]
    fn empty_headers_returns_pass() {
        let headers = HeaderMap::new();
        let r = analyze_ai_header_directives(&headers);
        assert_eq!(r.check, "geo-ai-header-directives");
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn noai_header_returns_warn() {
        let mut headers = HeaderMap::new();
        headers.insert("x-robots-tag", "noai".parse().unwrap());
        let r = analyze_ai_header_directives(&headers);
        assert_eq!(r.status, Status::Warn);
        assert!(r.message.contains("noai"));
    }

    #[test]
    fn nosnippet_header_returns_warn() {
        let mut headers = HeaderMap::new();
        headers.insert("x-robots-tag", "nosnippet".parse().unwrap());
        let r = analyze_ai_header_directives(&headers);
        assert_eq!(r.status, Status::Warn);
        assert!(r.message.contains("nosnippet"));
    }

    #[test]
    fn case_insensitive_header() {
        let mut headers = HeaderMap::new();
        headers.insert("x-robots-tag", "NOAI".parse().unwrap());
        let r = analyze_ai_header_directives(&headers);
        assert_eq!(r.status, Status::Warn);
    }

    #[test]
    fn no_ai_directives_in_header_passes() {
        let mut headers = HeaderMap::new();
        headers.insert("x-robots-tag", "noindex".parse().unwrap());
        let r = analyze_ai_header_directives(&headers);
        assert_eq!(r.status, Status::Pass);
    }
}
