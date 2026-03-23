use crate::scoring::{AnalysisResult, Status};
use scraper::{Html, Selector};
use serde_json::Value;

/// CONT-01: Check that heading levels do not skip (e.g. H1 -> H3 without H2).
pub fn analyze_heading_structure(html: &Html) -> AnalysisResult {
    let heading_sel = Selector::parse("h1, h2, h3, h4, h5, h6").expect("valid selector");
    let levels: Vec<u8> = html
        .select(&heading_sel)
        .map(|el| {
            el.value()
                .name()
                .chars()
                .last()
                .unwrap()
                .to_digit(10)
                .unwrap() as u8
        })
        .collect();

    if levels.is_empty() {
        return AnalysisResult {
            check: "cont-heading-structure",
            status: Status::Pass,
            message: "No headings found on page (no hierarchy to validate)".to_string(),
            recommendation: "Consider adding a heading structure (H1 for title, H2 for sections) to improve content organization for AI crawlers".to_string(),
        };
    }

    let mut prev = 0u8;
    let mut skip_found = false;
    for &level in &levels {
        if prev > 0 && level > prev + 1 {
            skip_found = true;
            break;
        }
        prev = level;
    }

    if skip_found {
        AnalysisResult {
            check: "cont-heading-structure",
            status: Status::Fail,
            message: "Heading hierarchy has skipped levels (e.g., H1 followed by H3)".to_string(),
            recommendation: "Fix heading nesting: each heading level should increment by one (H1 \u{2192} H2 \u{2192} H3). Skipped levels confuse screen readers and AI content parsers that infer document structure from headings".to_string(),
        }
    } else {
        AnalysisResult {
            check: "cont-heading-structure",
            status: Status::Pass,
            message: "Heading hierarchy is correctly nested with no skipped levels".to_string(),
            recommendation: String::new(),
        }
    }
}

/// CONT-04: Check that all img elements have non-empty alt text.
pub fn analyze_alt_text(html: &Html) -> AnalysisResult {
    let img_sel = Selector::parse("img").expect("valid selector");
    let mut total = 0usize;
    let mut missing = 0usize;

    for img in html.select(&img_sel) {
        total += 1;
        match img.value().attr("alt") {
            None => missing += 1,
            Some(v) if v.trim().is_empty() => missing += 1,
            _ => {}
        }
    }

    if total == 0 {
        return AnalysisResult {
            check: "cont-alt-text",
            status: Status::Pass,
            message: "No images found on page".to_string(),
            recommendation: "When adding images, always include descriptive alt text for accessibility and AI indexing".to_string(),
        };
    }

    if missing == 0 {
        AnalysisResult {
            check: "cont-alt-text",
            status: Status::Pass,
            message: format!("All {} image(s) have alt text", total),
            recommendation: String::new(),
        }
    } else {
        AnalysisResult {
            check: "cont-alt-text",
            status: Status::Fail,
            message: format!("{} of {} image(s) are missing alt text", missing, total),
            recommendation: format!(
                "Add descriptive alt text to all {} image(s). Alt text is the primary signal AI search engines use to understand image content and improve page relevance",
                missing
            ),
        }
    }
}

/// CONT-02: Check that JSON-LD blocks are present, valid JSON, have @type, and use schema.org @context.
pub fn analyze_json_ld(html: &Html) -> AnalysisResult {
    let ld_sel =
        Selector::parse(r#"script[type="application/ld+json"]"#).expect("valid selector");
    let blocks: Vec<_> = html.select(&ld_sel).collect();

    if blocks.is_empty() {
        return AnalysisResult {
            check: "cont-json-ld",
            status: Status::Warn,
            message: "No JSON-LD schema markup found on page".to_string(),
            recommendation: "Add JSON-LD schema markup (e.g., Article, Product, FAQPage) to help AI search engines understand your content type and structure. Schema markup is a key signal for AI-powered search results".to_string(),
        };
    }

    for block in blocks {
        let text: String = block.text().collect();
        match serde_json::from_str::<Value>(&text) {
            Err(_) => {
                return AnalysisResult {
                    check: "cont-json-ld",
                    status: Status::Fail,
                    message: "JSON-LD block contains malformed JSON".to_string(),
                    recommendation: "Fix the JSON syntax in your schema markup. Use https://validator.schema.org/ to validate before deploying".to_string(),
                };
            }
            Ok(val) => {
                if val.get("@type").is_none() {
                    return AnalysisResult {
                        check: "cont-json-ld",
                        status: Status::Fail,
                        message: "JSON-LD block is missing required @type property".to_string(),
                        recommendation: "Add an @type property to your JSON-LD (e.g., \"@type\": \"Article\"). @type tells search engines what kind of content this is".to_string(),
                    };
                }
                let context_ok = val
                    .get("@context")
                    .and_then(|v| v.as_str())
                    .map(|s| s.contains("schema.org"))
                    .unwrap_or(false);
                if !context_ok {
                    return AnalysisResult {
                        check: "cont-json-ld",
                        status: Status::Warn,
                        message: "JSON-LD block has @type but @context does not reference schema.org".to_string(),
                        recommendation: "Set @context to \"https://schema.org\" to use the Schema.org vocabulary \u{2014} this is required for search engine recognition".to_string(),
                    };
                }
                // This block passes; continue to check remaining blocks
            }
        }
    }

    AnalysisResult {
        check: "cont-json-ld",
        status: Status::Pass,
        message: "JSON-LD schema markup is valid with @type and schema.org context".to_string(),
        recommendation: String::new(),
    }
}

/// CONT-03: Check that the page uses semantic HTML landmark elements.
pub fn analyze_semantic_html(html: &Html) -> AnalysisResult {
    let semantic_sel =
        Selector::parse("article, main, nav, section, aside, header, footer").expect("valid selector");
    let count = html.select(&semantic_sel).count();

    if count == 0 {
        AnalysisResult {
            check: "cont-semantic-html",
            status: Status::Warn,
            message: "Page uses no semantic HTML elements (article, main, nav, section, aside, header, footer)".to_string(),
            recommendation: "Replace generic <div> containers with semantic HTML elements: <main> for primary content, <nav> for navigation, <article> for standalone content, <section> for thematic groupings. AI crawlers use these elements to understand page structure and extract content boundaries".to_string(),
        }
    } else {
        AnalysisResult {
            check: "cont-semantic-html",
            status: Status::Pass,
            message: format!("Page uses {} semantic HTML element(s) (good content structure)", count),
            recommendation: String::new(),
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

    // --- analyze_heading_structure ---

    #[test]
    fn test_heading_structure_no_headings_passes() {
        let doc = parse("<html><body><p>No headings</p></body></html>");
        let result = analyze_heading_structure(&doc);
        assert_eq!(result.status, Status::Pass);
        assert_eq!(result.check, "cont-heading-structure");
    }

    #[test]
    fn test_heading_structure_h1_h2_h3_passes() {
        let doc = parse("<html><body><h1>A</h1><h2>B</h2><h3>C</h3></body></html>");
        let result = analyze_heading_structure(&doc);
        assert_eq!(result.status, Status::Pass);
    }

    #[test]
    fn test_heading_structure_h1_h3_skip_fails() {
        let doc = parse("<html><body><h1>A</h1><h3>C</h3></body></html>");
        let result = analyze_heading_structure(&doc);
        assert_eq!(result.status, Status::Fail);
        assert_eq!(result.check, "cont-heading-structure");
    }

    #[test]
    fn test_heading_structure_h1_h2_h4_skip_fails() {
        let doc = parse("<html><body><h1>A</h1><h2>B</h2><h4>D</h4></body></html>");
        let result = analyze_heading_structure(&doc);
        assert_eq!(result.status, Status::Fail);
    }

    #[test]
    fn test_heading_structure_multiple_h2_after_h1_passes() {
        let doc = parse("<html><body><h1>A</h1><h2>B</h2><h2>C</h2></body></html>");
        let result = analyze_heading_structure(&doc);
        assert_eq!(result.status, Status::Pass);
    }

    // --- analyze_alt_text ---

    #[test]
    fn test_alt_text_no_images_passes() {
        let doc = parse("<html><body><p>Text</p></body></html>");
        let result = analyze_alt_text(&doc);
        assert_eq!(result.status, Status::Pass);
        assert_eq!(result.check, "cont-alt-text");
    }

    #[test]
    fn test_alt_text_all_images_have_alt_passes() {
        let doc = parse(r#"<html><body><img src="a.jpg" alt="desc"/><img src="b.jpg" alt="desc2"/></body></html>"#);
        let result = analyze_alt_text(&doc);
        assert_eq!(result.status, Status::Pass);
    }

    #[test]
    fn test_alt_text_one_missing_fails_with_count() {
        let doc = parse(r#"<html><body><img src="a.jpg" alt=""/><img src="b.jpg" alt="ok"/></body></html>"#);
        let result = analyze_alt_text(&doc);
        assert_eq!(result.status, Status::Fail);
        assert!(result.message.contains("1 of 2"));
    }

    // --- analyze_json_ld ---

    #[test]
    fn test_json_ld_no_blocks_warns() {
        let doc = parse("<html><body></body></html>");
        let result = analyze_json_ld(&doc);
        assert_eq!(result.status, Status::Warn);
        assert_eq!(result.check, "cont-json-ld");
    }

    #[test]
    fn test_json_ld_valid_passes() {
        let html = r#"<html><head><script type="application/ld+json">{"@context":"https://schema.org","@type":"Article","name":"Test"}</script></head></html>"#;
        let doc = parse(html);
        let result = analyze_json_ld(&doc);
        assert_eq!(result.status, Status::Pass);
    }

    #[test]
    fn test_json_ld_malformed_fails() {
        let html = r#"<html><head><script type="application/ld+json">not valid json{</script></head></html>"#;
        let doc = parse(html);
        let result = analyze_json_ld(&doc);
        assert_eq!(result.status, Status::Fail);
        assert!(result.message.contains("malformed JSON"));
    }

    #[test]
    fn test_json_ld_missing_type_fails() {
        let html = r#"<html><head><script type="application/ld+json">{"@context":"https://schema.org","name":"No type"}</script></head></html>"#;
        let doc = parse(html);
        let result = analyze_json_ld(&doc);
        assert_eq!(result.status, Status::Fail);
        assert!(result.message.contains("@type"));
    }

    // --- analyze_semantic_html ---

    #[test]
    fn test_semantic_html_none_warns() {
        let doc = parse("<html><body><div>content</div></body></html>");
        let result = analyze_semantic_html(&doc);
        assert_eq!(result.status, Status::Warn);
        assert_eq!(result.check, "cont-semantic-html");
    }

    #[test]
    fn test_semantic_html_main_passes() {
        let doc = parse("<html><body><main>content</main></body></html>");
        let result = analyze_semantic_html(&doc);
        assert_eq!(result.status, Status::Pass);
    }
}
