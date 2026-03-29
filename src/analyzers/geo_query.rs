use crate::scoring::{AnalysisResult, Status};
use regex::Regex;
use scraper::{Html, Selector};
use serde_json::Value;

/// Analyze conversational query optimization patterns in HTML content.
///
/// Returns exactly 4 results:
/// - geo-query-qa-patterns: Question-answer heading patterns
/// - geo-query-summary: TL;DR / key takeaway sections
/// - geo-query-snippet: Featured snippet formatting (40-60 word answer blocks)
/// - geo-query-faq: Dedicated FAQ section detection
pub fn analyze_query_optimization(html: &Html) -> Vec<AnalysisResult> {
    vec![
        check_qa_patterns(html),
        check_summary(html),
        check_snippet(html),
        check_faq(html),
    ]
}

fn check_qa_patterns(html: &Html) -> AnalysisResult {
    let heading_sel = Selector::parse("h2, h3").expect("valid selector");
    let question_re =
        Regex::new(r"(?i)^(what|how|why|when|where|who|which|can|does|is|are|should|will)\b.*\??$")
            .expect("valid regex");

    let mut count = 0u32;
    for el in html.select(&heading_sel) {
        let text: String = el.text().collect();
        let text = text.trim();
        if text.ends_with('?') || question_re.is_match(text) {
            count += 1;
        }
    }

    if count > 0 {
        AnalysisResult {
            check: "geo-query-qa-patterns",
            status: Status::Pass,
            message: format!("{} question-answer heading pattern(s) found.", count),
            recommendation: String::new(),
        }
    } else {
        AnalysisResult {
            check: "geo-query-qa-patterns",
            status: Status::Warn,
            message: "No question-answer heading patterns detected.".to_string(),
            recommendation: "Structure content with question-phrased headings (H2/H3) followed by direct answers. AI search engines extract Q&A patterns for featured responses.".to_string(),
        }
    }
}

fn check_summary(html: &Html) -> AnalysisResult {
    // Check elements with class/id containing summary-related keywords
    let class_selectors = [
        r#"[class*="tldr"]"#,
        r#"[id*="tldr"]"#,
        r#"[class*="summary"]"#,
        r#"[class*="takeaway"]"#,
        r#"[id*="takeaway"]"#,
        r#"[class*="key-takeaway"]"#,
        r#"[id*="key-takeaway"]"#,
        r#"[class*="overview"]"#,
        r#"[id*="overview"]"#,
    ];

    for sel_str in &class_selectors {
        if let Ok(sel) = Selector::parse(sel_str) {
            if html.select(&sel).next().is_some() {
                return AnalysisResult {
                    check: "geo-query-summary",
                    status: Status::Pass,
                    message: "Summary/TL;DR section detected.".to_string(),
                    recommendation: String::new(),
                };
            }
        }
    }

    // Check heading text for summary patterns
    let heading_sel = Selector::parse("h1, h2, h3, h4, h5, h6").expect("valid selector");
    let summary_re = Regex::new(
        r"(?i)^(tl;?\s*dr|key\s+takeaways?|summary|in\s+brief|at\s+a\s+glance|overview|the\s+bottom\s+line)$",
    )
    .expect("valid regex");

    for el in html.select(&heading_sel) {
        let text: String = el.text().collect();
        let text = text.trim();
        if summary_re.is_match(text) {
            return AnalysisResult {
                check: "geo-query-summary",
                status: Status::Pass,
                message: "Summary/TL;DR section detected.".to_string(),
                recommendation: String::new(),
            };
        }
    }

    AnalysisResult {
        check: "geo-query-summary",
        status: Status::Warn,
        message: "No TL;DR, key takeaways, or summary section found.".to_string(),
        recommendation: "Add a TL;DR or 'Key Takeaways' section near the top of your content. AI search engines prefer content with upfront summaries for quick-answer extraction.".to_string(),
    }
}

fn check_snippet(html: &Html) -> AnalysisResult {
    let heading_sel = Selector::parse("h1, h2, h3, h4, h5, h6").expect("valid selector");

    for el in html.select(&heading_sel) {
        // Walk siblings after the heading looking for a <p>
        let mut sibling = el.next_sibling();
        while let Some(node) = sibling {
            if let Some(element) = node.value().as_element() {
                if element.name() == "p" {
                    let text: String = scraper::ElementRef::wrap(node)
                        .map(|er| er.text().collect::<String>())
                        .unwrap_or_default();
                    let word_count = text.split_whitespace().count();
                    if (40..=60).contains(&word_count) {
                        return AnalysisResult {
                            check: "geo-query-snippet",
                            status: Status::Pass,
                            message: "Featured snippet-formatted content found (concise answer blocks).".to_string(),
                            recommendation: String::new(),
                        };
                    }
                }
                // Stop at next heading or non-paragraph block element
                break;
            }
            sibling = node.next_sibling();
        }
    }

    AnalysisResult {
        check: "geo-query-snippet",
        status: Status::Warn,
        message: "No featured snippet-formatted content detected.".to_string(),
        recommendation: "Follow question headings with concise 40-60 word answer paragraphs. This format matches featured snippet extraction patterns used by AI search engines.".to_string(),
    }
}

fn check_faq(html: &Html) -> AnalysisResult {
    // Check headings for FAQ patterns
    let heading_sel = Selector::parse("h1, h2, h3, h4, h5, h6").expect("valid selector");
    let faq_re =
        Regex::new(r"(?i)^(faq|faqs|frequently\s+asked\s+questions)$").expect("valid regex");

    for el in html.select(&heading_sel) {
        let text: String = el.text().collect();
        let text = text.trim();
        if faq_re.is_match(text) {
            return AnalysisResult {
                check: "geo-query-faq",
                status: Status::Pass,
                message: "FAQ section detected.".to_string(),
                recommendation: String::new(),
            };
        }
    }

    // Check for FAQPage JSON-LD schema
    let ld_sel =
        Selector::parse(r#"script[type="application/ld+json"]"#).expect("valid selector");

    for block in html.select(&ld_sel) {
        let text: String = block.text().collect();
        if let Ok(val) = serde_json::from_str::<Value>(&text) {
            if has_faq_type(&val) {
                return AnalysisResult {
                    check: "geo-query-faq",
                    status: Status::Pass,
                    message: "FAQ section detected.".to_string(),
                    recommendation: String::new(),
                };
            }
        }
    }

    AnalysisResult {
        check: "geo-query-faq",
        status: Status::Warn,
        message: "No FAQ section found.".to_string(),
        recommendation: "Add a dedicated FAQ section with a clear heading ('FAQ' or 'Frequently Asked Questions'). FAQ sections are high-value targets for AI search citation.".to_string(),
    }
}

fn has_faq_type(val: &Value) -> bool {
    match val.get("@type") {
        Some(Value::String(s)) if s == "FAQPage" => return true,
        Some(Value::Array(arr)) => {
            if arr.iter().any(|t| t.as_str() == Some("FAQPage")) {
                return true;
            }
        }
        _ => {}
    }
    if let Some(Value::Array(graph)) = val.get("@graph") {
        for item in graph {
            if has_faq_type(item) {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    fn parse(html: &str) -> Html {
        Html::parse_document(html)
    }

    // --- analyze_query_optimization returns exactly 4 results ---

    #[test]
    fn returns_exactly_4_results() {
        let doc = parse("<html><body><p>Simple page</p></body></html>");
        let results = analyze_query_optimization(&doc);
        assert_eq!(results.len(), 4);
    }

    // --- geo-query-qa-patterns ---

    #[test]
    fn qa_patterns_pass_with_question_heading() {
        let doc = parse("<html><body><h2>What is GEO?</h2><p>GEO stands for...</p></body></html>");
        let results = analyze_query_optimization(&doc);
        let r = results.iter().find(|r| r.check == "geo-query-qa-patterns").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn qa_patterns_pass_with_how_heading() {
        let doc = parse("<html><body><h3>How does it work?</h3></body></html>");
        let results = analyze_query_optimization(&doc);
        let r = results.iter().find(|r| r.check == "geo-query-qa-patterns").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn qa_patterns_warn_no_question_headings() {
        let doc = parse("<html><body><h2>Introduction</h2><h3>Details</h3></body></html>");
        let results = analyze_query_optimization(&doc);
        let r = results.iter().find(|r| r.check == "geo-query-qa-patterns").unwrap();
        assert_eq!(r.status, Status::Warn);
    }

    // --- geo-query-summary ---

    #[test]
    fn summary_pass_with_tldr_class() {
        let doc = parse(r#"<html><body><div class="tldr">Quick summary here</div></body></html>"#);
        let results = analyze_query_optimization(&doc);
        let r = results.iter().find(|r| r.check == "geo-query-summary").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn summary_pass_with_key_takeaways_id() {
        let doc = parse(r#"<html><body><div id="key-takeaway">Points here</div></body></html>"#);
        let results = analyze_query_optimization(&doc);
        let r = results.iter().find(|r| r.check == "geo-query-summary").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn summary_pass_with_tldr_heading() {
        let doc = parse("<html><body><h2>TL;DR</h2><p>Quick version.</p></body></html>");
        let results = analyze_query_optimization(&doc);
        let r = results.iter().find(|r| r.check == "geo-query-summary").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn summary_pass_with_key_takeaways_heading() {
        let doc = parse("<html><body><h2>Key Takeaways</h2><p>Points.</p></body></html>");
        let results = analyze_query_optimization(&doc);
        let r = results.iter().find(|r| r.check == "geo-query-summary").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn summary_warn_no_patterns() {
        let doc = parse("<html><body><h2>Intro</h2><p>Content here.</p></body></html>");
        let results = analyze_query_optimization(&doc);
        let r = results.iter().find(|r| r.check == "geo-query-summary").unwrap();
        assert_eq!(r.status, Status::Warn);
    }

    // --- geo-query-snippet ---

    #[test]
    fn snippet_pass_with_appropriate_paragraph() {
        // Create a paragraph with ~50 words following a heading
        let words = "word ".repeat(50);
        let html = format!(
            "<html><body><h2>What is SEO?</h2><p>{}</p></body></html>",
            words.trim()
        );
        let doc = parse(&html);
        let results = analyze_query_optimization(&doc);
        let r = results.iter().find(|r| r.check == "geo-query-snippet").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn snippet_warn_no_snippet_content() {
        let doc = parse("<html><body><h2>Title</h2><p>Short.</p></body></html>");
        let results = analyze_query_optimization(&doc);
        let r = results.iter().find(|r| r.check == "geo-query-snippet").unwrap();
        assert_eq!(r.status, Status::Warn);
    }

    // --- geo-query-faq ---

    #[test]
    fn faq_pass_with_faq_heading() {
        let doc = parse("<html><body><h2>FAQ</h2><p>Q: What? A: This.</p></body></html>");
        let results = analyze_query_optimization(&doc);
        let r = results.iter().find(|r| r.check == "geo-query-faq").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn faq_pass_with_long_heading() {
        let doc = parse("<html><body><h2>Frequently Asked Questions</h2></body></html>");
        let results = analyze_query_optimization(&doc);
        let r = results.iter().find(|r| r.check == "geo-query-faq").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn faq_pass_with_faqpage_schema() {
        let doc = parse(
            r#"<html><head><script type="application/ld+json">{"@type":"FAQPage"}</script></head><body></body></html>"#,
        );
        let results = analyze_query_optimization(&doc);
        let r = results.iter().find(|r| r.check == "geo-query-faq").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn faq_warn_no_faq() {
        let doc = parse("<html><body><h2>About</h2><p>Info.</p></body></html>");
        let results = analyze_query_optimization(&doc);
        let r = results.iter().find(|r| r.check == "geo-query-faq").unwrap();
        assert_eq!(r.status, Status::Warn);
    }
}
