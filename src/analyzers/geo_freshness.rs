use crate::scoring::{AnalysisResult, Status};
use reqwest::header::HeaderMap;
use scraper::{Html, Selector};
use serde_json::Value;

/// Check content freshness signals: dateModified in JSON-LD, Last-Modified HTTP header,
/// and meta tag equivalents (article:modified_time, http-equiv last-modified).
///
/// Returns Fail if no freshness signals found (critical severity, 10pts).
pub fn analyze_freshness(html: &Html, headers: &HeaderMap) -> AnalysisResult {
    let mut signals: Vec<&str> = Vec::new();

    // 1. Check JSON-LD blocks for dateModified
    let ld_sel =
        Selector::parse(r#"script[type="application/ld+json"]"#).expect("valid selector");
    for block in html.select(&ld_sel) {
        let text: String = block.text().collect();
        if let Ok(val) = serde_json::from_str::<Value>(&text) {
            if has_date_modified(&val) {
                signals.push("dateModified in JSON-LD");
                break;
            }
        }
    }

    // 2. Check HTTP headers for Last-Modified
    if headers.get("last-modified").is_some() {
        signals.push("Last-Modified HTTP header");
    }

    // 3. Check meta tags
    let meta_selectors = [
        r#"meta[property="article:modified_time"]"#,
        r#"meta[http-equiv="last-modified"]"#,
    ];
    for sel_str in &meta_selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            if html.select(&sel).next().is_some() {
                signals.push("last-modified meta tag");
                break;
            }
        }
    }

    if signals.is_empty() {
        AnalysisResult {
            check: "geo-freshness",
            status: Status::Fail,
            message: "No content freshness signals found.".to_string(),
            recommendation: "Add dateModified to your JSON-LD schema and configure Last-Modified HTTP headers. Content without freshness signals loses citation priority in AI search results.".to_string(),
        }
    } else {
        AnalysisResult {
            check: "geo-freshness",
            status: Status::Pass,
            message: format!("Freshness signals found: {}.", signals.join(", ")),
            recommendation: String::new(),
        }
    }
}

/// Check for dateModified property in a JSON-LD value, including @graph arrays.
fn has_date_modified(val: &Value) -> bool {
    if val.get("dateModified").is_some() {
        return true;
    }
    if let Some(Value::Array(graph)) = val.get("@graph") {
        for item in graph {
            if has_date_modified(item) {
                return true;
            }
        }
    }
    false
}

/// Validate HowTo JSON-LD schema type presence and structure.
///
/// Returns Pass if HowTo with steps found, Warn if HowTo without steps or no HowTo at all.
pub fn analyze_howto_schema(html: &Html) -> AnalysisResult {
    let ld_sel =
        Selector::parse(r#"script[type="application/ld+json"]"#).expect("valid selector");

    for block in html.select(&ld_sel) {
        let text: String = block.text().collect();
        if let Ok(val) = serde_json::from_str::<Value>(&text) {
            if let Some(howto) = find_howto_object(&val) {
                if howto.get("step").is_some() {
                    return AnalysisResult {
                        check: "geo-howto-schema",
                        status: Status::Pass,
                        message: "HowTo schema found with step definitions.".to_string(),
                        recommendation: String::new(),
                    };
                } else {
                    return AnalysisResult {
                        check: "geo-howto-schema",
                        status: Status::Warn,
                        message: "HowTo schema found but missing 'step' property.".to_string(),
                        recommendation: "Add 'step' property to your HowTo schema with individual HowToStep entries for each instruction step.".to_string(),
                    };
                }
            }
        }
    }

    AnalysisResult {
        check: "geo-howto-schema",
        status: Status::Warn,
        message: "No HowTo schema found. Consider adding if this page contains step-by-step instructions.".to_string(),
        recommendation: "Add HowTo JSON-LD schema for instructional content. HowTo schema enables rich results and improves AI search visibility for how-to queries.".to_string(),
    }
}

/// Find a HowTo object in a JSON-LD value, handling @graph arrays and array @type.
fn find_howto_object(val: &Value) -> Option<&Value> {
    if val.get("@type").and_then(|t| t.as_str()) == Some("HowTo") {
        return Some(val);
    }
    if let Some(Value::Array(arr)) = val.get("@type") {
        if arr.iter().any(|t| t.as_str() == Some("HowTo")) {
            return Some(val);
        }
    }
    if let Some(Value::Array(graph)) = val.get("@graph") {
        for item in graph {
            if let Some(found) = find_howto_object(item) {
                return Some(found);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderMap;
    use scraper::Html;

    fn parse(html: &str) -> Html {
        Html::parse_document(html)
    }

    // --- analyze_freshness ---

    #[test]
    fn freshness_pass_with_date_modified_jsonld() {
        let doc = parse(
            r#"<html><head><script type="application/ld+json">{"@type":"Article","dateModified":"2026-01-01"}</script></head></html>"#,
        );
        let headers = HeaderMap::new();
        let r = analyze_freshness(&doc, &headers);
        assert_eq!(r.check, "geo-freshness");
        assert_eq!(r.status, Status::Pass);
        assert!(r.message.contains("dateModified in JSON-LD"));
    }

    #[test]
    fn freshness_pass_with_last_modified_header() {
        let doc = parse("<html><body></body></html>");
        let mut headers = HeaderMap::new();
        headers.insert("last-modified", "Sat, 01 Jan 2026 00:00:00 GMT".parse().unwrap());
        let r = analyze_freshness(&doc, &headers);
        assert_eq!(r.status, Status::Pass);
        assert!(r.message.contains("Last-Modified HTTP header"));
    }

    #[test]
    fn freshness_pass_with_article_modified_time_meta() {
        let doc = parse(
            r#"<html><head><meta property="article:modified_time" content="2026-01-01"></head></html>"#,
        );
        let headers = HeaderMap::new();
        let r = analyze_freshness(&doc, &headers);
        assert_eq!(r.status, Status::Pass);
        assert!(r.message.contains("last-modified meta tag"));
    }

    #[test]
    fn freshness_pass_with_http_equiv_last_modified() {
        let doc = parse(
            r#"<html><head><meta http-equiv="last-modified" content="2026-01-01"></head></html>"#,
        );
        let headers = HeaderMap::new();
        let r = analyze_freshness(&doc, &headers);
        assert_eq!(r.status, Status::Pass);
        assert!(r.message.contains("last-modified meta tag"));
    }

    #[test]
    fn freshness_fail_no_signals() {
        let doc = parse("<html><body><p>No freshness signals.</p></body></html>");
        let headers = HeaderMap::new();
        let r = analyze_freshness(&doc, &headers);
        assert_eq!(r.status, Status::Fail);
        assert!(r.message.contains("No content freshness signals"));
    }

    #[test]
    fn freshness_pass_message_lists_signals() {
        let doc = parse(
            r#"<html><head>
            <script type="application/ld+json">{"@type":"Article","dateModified":"2026-01-01"}</script>
            <meta property="article:modified_time" content="2026-01-01">
            </head></html>"#,
        );
        let mut headers = HeaderMap::new();
        headers.insert("last-modified", "Sat, 01 Jan 2026 00:00:00 GMT".parse().unwrap());
        let r = analyze_freshness(&doc, &headers);
        assert_eq!(r.status, Status::Pass);
        assert!(r.message.contains("dateModified in JSON-LD"));
        assert!(r.message.contains("Last-Modified HTTP header"));
        assert!(r.message.contains("last-modified meta tag"));
    }

    // --- analyze_howto_schema ---

    #[test]
    fn howto_pass_with_steps() {
        let doc = parse(
            r#"<html><head><script type="application/ld+json">{"@type":"HowTo","name":"Fix a tire","step":[{"@type":"HowToStep","text":"Remove the tire"}]}</script></head></html>"#,
        );
        let r = analyze_howto_schema(&doc);
        assert_eq!(r.check, "geo-howto-schema");
        assert_eq!(r.status, Status::Pass);
        assert!(r.message.contains("step definitions"));
    }

    #[test]
    fn howto_warn_without_steps() {
        let doc = parse(
            r#"<html><head><script type="application/ld+json">{"@type":"HowTo","name":"Fix a tire"}</script></head></html>"#,
        );
        let r = analyze_howto_schema(&doc);
        assert_eq!(r.status, Status::Warn);
        assert!(r.message.contains("missing 'step' property"));
    }

    #[test]
    fn howto_warn_no_schema() {
        let doc = parse("<html><body><p>No schema.</p></body></html>");
        let r = analyze_howto_schema(&doc);
        assert_eq!(r.status, Status::Warn);
        assert!(r.message.contains("No HowTo schema found"));
    }

    #[test]
    fn howto_in_graph_array() {
        let doc = parse(
            r#"<html><head><script type="application/ld+json">{
                "@context":"https://schema.org",
                "@graph":[
                    {"@type":"Article","name":"Guide"},
                    {"@type":"HowTo","name":"Steps","step":[{"@type":"HowToStep","text":"Step 1"}]}
                ]
            }</script></head></html>"#,
        );
        let r = analyze_howto_schema(&doc);
        assert_eq!(r.status, Status::Pass);
    }
}
