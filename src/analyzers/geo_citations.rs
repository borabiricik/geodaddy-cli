use crate::scoring::{AnalysisResult, Status};
use regex::Regex;
use scraper::{Html, Selector};
use serde_json::Value;

/// Analyze citation signals in HTML content.
/// Returns exactly 4 results: geo-citation-stats, geo-citation-sources,
/// geo-citation-quotes, geo-citation-references.
pub fn analyze_citations(html: &Html) -> Vec<AnalysisResult> {
    // Extract body text once
    let body_sel = Selector::parse("body").expect("valid selector");
    let body_text: String = html
        .select(&body_sel)
        .flat_map(|el| el.text())
        .collect::<Vec<_>>()
        .join(" ");

    let mut results = Vec::with_capacity(4);

    // geo-citation-stats: statistics / numerical data
    let stats_re =
        Regex::new(r"(?i)(\d+\.?\d*\s*%|\$\d[\d,]*\.?\d*|\d+\s+out\s+of\s+\d+|\d+x\s)")
            .expect("valid regex");
    if stats_re.is_match(&body_text) {
        results.push(AnalysisResult {
            check: "geo-citation-stats",
            status: Status::Pass,
            message: "Statistics/numerical data found in content.".to_string(),
            recommendation: String::new(),
        });
    } else {
        results.push(AnalysisResult {
            check: "geo-citation-stats",
            status: Status::Warn,
            message: "No statistics or numerical data found in content.".to_string(),
            recommendation: "Add specific statistics, percentages, or numerical data to strengthen content credibility. AI engines prefer content with verifiable data points.".to_string(),
        });
    }

    // geo-citation-sources: source attribution phrases
    let sources_re = Regex::new(
        r"(?i)(according\s+to|a\s+study\s+by|research\s+(from|by|shows)|data\s+from|report\s+by|survey\s+by|published\s+(in|by))",
    )
    .expect("valid regex");
    if sources_re.is_match(&body_text) {
        results.push(AnalysisResult {
            check: "geo-citation-sources",
            status: Status::Pass,
            message: "Source attribution patterns found in content.".to_string(),
            recommendation: String::new(),
        });
    } else {
        results.push(AnalysisResult {
            check: "geo-citation-sources",
            status: Status::Warn,
            message: "No source attributions found in content.".to_string(),
            recommendation: "Add source attributions (e.g., 'according to [Source]', 'a study by [Organization] found'). External authority citations boost AI visibility up to 40%.".to_string(),
        });
    }

    // geo-citation-quotes: blockquote elements or quotation patterns
    let blockquote_sel = Selector::parse("blockquote").expect("valid selector");
    let has_blockquote = html.select(&blockquote_sel).next().is_some();
    let quotes_re = Regex::new(r#"(?i)(as\s+\w+\s+said|"\s*[A-Z])"#).expect("valid regex");
    let has_quote_pattern = quotes_re.is_match(&body_text);
    if has_blockquote || has_quote_pattern {
        results.push(AnalysisResult {
            check: "geo-citation-quotes",
            status: Status::Pass,
            message: "Quotation/blockquote content found.".to_string(),
            recommendation: String::new(),
        });
    } else {
        results.push(AnalysisResult {
            check: "geo-citation-quotes",
            status: Status::Warn,
            message: "No blockquotes or quotation patterns found.".to_string(),
            recommendation: "Add expert quotes using <blockquote> elements or attribution patterns ('As [Expert] said...'). Quotes add credibility signals for AI citation.".to_string(),
        });
    }

    // geo-citation-references: heading with reference/sources text
    let heading_sel = Selector::parse("h2, h3, h4").expect("valid selector");
    let ref_re = Regex::new(
        r"(?i)^(references|sources|bibliography|citations|further\s+reading|works\s+cited)$",
    )
    .expect("valid regex");
    let has_ref_heading = html.select(&heading_sel).any(|el| {
        let text: String = el.text().collect::<String>();
        ref_re.is_match(text.trim())
    });
    if has_ref_heading {
        results.push(AnalysisResult {
            check: "geo-citation-references",
            status: Status::Pass,
            message: "Reference/bibliography section found.".to_string(),
            recommendation: String::new(),
        });
    } else {
        results.push(AnalysisResult {
            check: "geo-citation-references",
            status: Status::Warn,
            message: "No reference or bibliography section detected.".to_string(),
            recommendation: "Add a 'References' or 'Sources' section with links to cited material. AI systems use reference sections as credibility signals.".to_string(),
        });
    }

    results
}

/// Analyze FAQ answer quality against 40-60 word optimal range.
pub fn analyze_faq_quality(html: &Html) -> AnalysisResult {
    let ld_sel =
        Selector::parse(r#"script[type="application/ld+json"]"#).expect("valid selector");

    for block in html.select(&ld_sel) {
        let text: String = block.text().collect();
        if let Ok(val) = serde_json::from_str::<Value>(&text) {
            if is_faq_page(&val) {
                return score_faq_answers(&val);
            }
            // Check @graph
            if let Some(Value::Array(graph)) = val.get("@graph") {
                for item in graph {
                    if is_faq_page(item) {
                        return score_faq_answers(item);
                    }
                }
            }
        }
    }

    AnalysisResult {
        check: "geo-faq-quality",
        status: Status::Warn,
        message: "No FAQPage schema found for FAQ quality analysis.".to_string(),
        recommendation: "Add FAQPage JSON-LD schema with Question/Answer pairs. Optimal FAQ answer length is 40-60 words for AI search visibility.".to_string(),
    }
}

fn is_faq_page(val: &Value) -> bool {
    match val.get("@type") {
        Some(Value::String(s)) => s == "FAQPage",
        Some(Value::Array(arr)) => arr.iter().any(|v| v.as_str() == Some("FAQPage")),
        _ => false,
    }
}

fn score_faq_answers(val: &Value) -> AnalysisResult {
    let empty = vec![];
    let entities = match val.get("mainEntity") {
        Some(Value::Array(arr)) => arr,
        _ => &empty,
    };

    if entities.is_empty() {
        return AnalysisResult {
            check: "geo-faq-quality",
            status: Status::Warn,
            message: "FAQPage schema found but no mainEntity questions.".to_string(),
            recommendation: "Add Question/Answer pairs to the FAQPage mainEntity array. Optimal FAQ answer length is 40-60 words for AI search visibility.".to_string(),
        };
    }

    let mut good = 0usize;
    let total = entities.len();

    for entity in entities {
        if let Some(answer_text) = entity
            .get("acceptedAnswer")
            .and_then(|a| a.get("text"))
            .and_then(|t| t.as_str())
        {
            let word_count = answer_text.split_whitespace().count();
            if (40..=60).contains(&word_count) {
                good += 1;
            }
        }
    }

    if good == total {
        AnalysisResult {
            check: "geo-faq-quality",
            status: Status::Pass,
            message: format!(
                "{}/{} FAQ answers are in the optimal 40-60 word range.",
                good, total
            ),
            recommendation: String::new(),
        }
    } else {
        AnalysisResult {
            check: "geo-faq-quality",
            status: Status::Warn,
            message: format!(
                "{}/{} FAQ answers are in the optimal 40-60 word range.",
                good, total
            ),
            recommendation: "Aim for 40-60 words per FAQ answer -- this length is optimal for AI search engine citation. Shorter answers lack detail; longer answers reduce snippet compatibility.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    fn parse(html: &str) -> Html {
        Html::parse_document(html)
    }

    // --- analyze_citations: geo-citation-stats ---

    #[test]
    fn stats_percentage_passes() {
        let doc = parse("<html><body><p>74.2% of users prefer this</p></body></html>");
        let results = analyze_citations(&doc);
        let r = results.iter().find(|r| r.check == "geo-citation-stats").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn stats_dollar_amount_passes() {
        let doc = parse("<html><body><p>Revenue reached $1.2 million</p></body></html>");
        let results = analyze_citations(&doc);
        let r = results.iter().find(|r| r.check == "geo-citation-stats").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn stats_fraction_passes() {
        let doc = parse("<html><body><p>3 out of 4 experts agree</p></body></html>");
        let results = analyze_citations(&doc);
        let r = results.iter().find(|r| r.check == "geo-citation-stats").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn stats_no_numbers_warns() {
        let doc = parse("<html><body><p>This is just plain text with no data</p></body></html>");
        let results = analyze_citations(&doc);
        let r = results.iter().find(|r| r.check == "geo-citation-stats").unwrap();
        assert_eq!(r.status, Status::Warn);
    }

    // --- analyze_citations: geo-citation-sources ---

    #[test]
    fn sources_according_to_passes() {
        let doc = parse("<html><body><p>according to Harvard this is true</p></body></html>");
        let results = analyze_citations(&doc);
        let r = results.iter().find(|r| r.check == "geo-citation-sources").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn sources_study_by_passes() {
        let doc = parse("<html><body><p>a study by MIT found that results improved</p></body></html>");
        let results = analyze_citations(&doc);
        let r = results.iter().find(|r| r.check == "geo-citation-sources").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn sources_no_attribution_warns() {
        let doc = parse("<html><body><p>Things are getting better every day</p></body></html>");
        let results = analyze_citations(&doc);
        let r = results.iter().find(|r| r.check == "geo-citation-sources").unwrap();
        assert_eq!(r.status, Status::Warn);
    }

    // --- analyze_citations: geo-citation-quotes ---

    #[test]
    fn quotes_blockquote_passes() {
        let doc = parse("<html><body><blockquote>Great wisdom here</blockquote></body></html>");
        let results = analyze_citations(&doc);
        let r = results.iter().find(|r| r.check == "geo-citation-quotes").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn quotes_as_said_passes() {
        let doc = parse("<html><body><p>As Einstein said, energy equals mass</p></body></html>");
        let results = analyze_citations(&doc);
        let r = results.iter().find(|r| r.check == "geo-citation-quotes").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn quotes_none_warns() {
        let doc = parse("<html><body><p>Just regular text here</p></body></html>");
        let results = analyze_citations(&doc);
        let r = results.iter().find(|r| r.check == "geo-citation-quotes").unwrap();
        assert_eq!(r.status, Status::Warn);
    }

    // --- analyze_citations: geo-citation-references ---

    #[test]
    fn references_h2_passes() {
        let doc = parse("<html><body><h2>References</h2><ul><li>Source 1</li></ul></body></html>");
        let results = analyze_citations(&doc);
        let r = results.iter().find(|r| r.check == "geo-citation-references").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn references_h3_sources_passes() {
        let doc = parse("<html><body><h3>Sources</h3><ul><li>Source 1</li></ul></body></html>");
        let results = analyze_citations(&doc);
        let r = results.iter().find(|r| r.check == "geo-citation-references").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn references_none_warns() {
        let doc = parse("<html><body><p>No reference section here</p></body></html>");
        let results = analyze_citations(&doc);
        let r = results.iter().find(|r| r.check == "geo-citation-references").unwrap();
        assert_eq!(r.status, Status::Warn);
    }

    // --- analyze_citations: count ---

    #[test]
    fn citations_returns_exactly_4() {
        let doc = parse("<html><body><p>Hello world</p></body></html>");
        let results = analyze_citations(&doc);
        assert_eq!(results.len(), 4);
    }

    // --- analyze_faq_quality ---

    #[test]
    fn faq_quality_optimal_range_passes() {
        let words: Vec<&str> = vec!["word"; 50];
        let answer_text = words.join(" ");
        let html = format!(
            r#"<html><head><script type="application/ld+json">{{
                "@context": "https://schema.org",
                "@type": "FAQPage",
                "mainEntity": [{{
                    "@type": "Question",
                    "name": "What is this?",
                    "acceptedAnswer": {{
                        "@type": "Answer",
                        "text": "{}"
                    }}
                }}]
            }}</script></head><body></body></html>"#,
            answer_text
        );
        let doc = parse(&html);
        let r = analyze_faq_quality(&doc);
        assert_eq!(r.check, "geo-faq-quality");
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn faq_quality_outside_range_warns() {
        let html = r#"<html><head><script type="application/ld+json">{
            "@context": "https://schema.org",
            "@type": "FAQPage",
            "mainEntity": [{
                "@type": "Question",
                "name": "What is this?",
                "acceptedAnswer": {
                    "@type": "Answer",
                    "text": "This is a very short answer with few words here now."
                }
            }]
        }</script></head><body></body></html>"#;
        let doc = parse(html);
        let r = analyze_faq_quality(&doc);
        assert_eq!(r.status, Status::Warn);
    }

    #[test]
    fn faq_quality_no_schema_warns() {
        let doc = parse("<html><body><p>No FAQ here</p></body></html>");
        let r = analyze_faq_quality(&doc);
        assert_eq!(r.status, Status::Warn);
        assert!(r.message.contains("No FAQPage schema"));
    }
}
