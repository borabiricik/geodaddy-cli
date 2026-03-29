use crate::analyzers::geo::extract_types;
use crate::scoring::{AnalysisResult, Status};
use regex::Regex;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashSet;

/// Analyze entity coverage in HTML content.
/// Returns exactly 4 results: geo-entity-schema, geo-entity-about,
/// geo-entity-proper-nouns, geo-entity-author.
pub fn analyze_entities(html: &Html) -> Vec<AnalysisResult> {
    let ld_sel =
        Selector::parse(r#"script[type="application/ld+json"]"#).expect("valid selector");
    let mut all_json: Vec<Value> = Vec::new();
    let mut found_types: HashSet<String> = HashSet::new();
    for block in html.select(&ld_sel) {
        let text: String = block.text().collect();
        if let Ok(val) = serde_json::from_str::<Value>(&text) {
            extract_types(&val, &mut found_types);
            all_json.push(val);
        }
    }

    let mut results = Vec::with_capacity(4);

    // geo-entity-schema: Person or Organization
    let has_person = found_types.contains("Person");
    let has_org = found_types.contains("Organization");
    if has_person || has_org {
        let mut types_found = Vec::new();
        if has_person {
            types_found.push("Person");
        }
        if has_org {
            types_found.push("Organization");
        }
        results.push(AnalysisResult {
            check: "geo-entity-schema",
            status: Status::Pass,
            message: format!("Entity schema found: {}.", types_found.join(", ")),
            recommendation: String::new(),
        });
    } else {
        results.push(AnalysisResult {
            check: "geo-entity-schema",
            status: Status::Warn,
            message: "No Person or Organization schema found.".to_string(),
            recommendation: "Add Person and/or Organization JSON-LD schema to establish entity identity. AI search engines use entity schema to build knowledge graph connections.".to_string(),
        });
    }

    // geo-entity-about: about or mentions properties in JSON-LD
    let has_about_mentions = all_json.iter().any(|v| has_about_or_mentions(v));
    if has_about_mentions {
        results.push(AnalysisResult {
            check: "geo-entity-about",
            status: Status::Pass,
            message: "Entity linking properties (about/mentions) found in JSON-LD.".to_string(),
            recommendation: String::new(),
        });
    } else {
        results.push(AnalysisResult {
            check: "geo-entity-about",
            status: Status::Warn,
            message: "No 'about' or 'mentions' properties found in JSON-LD schema.".to_string(),
            recommendation: "Add 'about' and 'mentions' properties to your JSON-LD to link content to specific entities. This helps AI systems understand what your content is about.".to_string(),
        });
    }

    // geo-entity-proper-nouns: mid-sentence capitalized words
    let body_sel = Selector::parse("body").expect("valid selector");
    let body_text: String = html
        .select(&body_sel)
        .flat_map(|el| el.text())
        .collect::<Vec<_>>()
        .join(" ");

    let proper_noun_re =
        Regex::new(r"(?m)\b[a-z]+\s+([A-Z][a-z]{2,})").expect("valid regex");
    let count = proper_noun_re.find_iter(&body_text).count();
    if count >= 3 {
        results.push(AnalysisResult {
            check: "geo-entity-proper-nouns",
            status: Status::Pass,
            message: format!("{} proper nouns/named entities detected in content.", count),
            recommendation: String::new(),
        });
    } else {
        results.push(AnalysisResult {
            check: "geo-entity-proper-nouns",
            status: Status::Warn,
            message: "Few or no named entities detected in content.".to_string(),
            recommendation: "Include specific names, organizations, and branded terms in your content. Named entities strengthen AI search citation by connecting content to knowledge graph entries.".to_string(),
        });
    }

    // geo-entity-author: meta tag, Person schema, or byline pattern
    let mut author_methods: Vec<&str> = Vec::new();

    // Method 1: meta author tag
    let meta_sel = Selector::parse(r#"meta[name="author"]"#).expect("valid selector");
    if let Some(meta) = html.select(&meta_sel).next() {
        if let Some(content) = meta.value().attr("content") {
            if !content.trim().is_empty() {
                author_methods.push("meta author tag");
            }
        }
    }

    // Method 2: Person type in JSON-LD
    if has_person {
        author_methods.push("Person schema");
    }

    // Method 3: byline pattern in body text
    let byline_re = Regex::new(r"(?i)\bby\s+[A-Z][a-z]+\s+[A-Z]").expect("valid regex");
    if byline_re.is_match(&body_text) {
        author_methods.push("byline pattern");
    }

    if !author_methods.is_empty() {
        results.push(AnalysisResult {
            check: "geo-entity-author",
            status: Status::Pass,
            message: format!("Author attribution found: {}.", author_methods.join(", ")),
            recommendation: String::new(),
        });
    } else {
        results.push(AnalysisResult {
            check: "geo-entity-author",
            status: Status::Warn,
            message: "No author attribution detected.".to_string(),
            recommendation: "Add author information via meta tag (<meta name=\"author\">), Person schema, or visible byline ('by [Name]'). Author attribution is a credibility signal for AI citation.".to_string(),
        });
    }

    results
}

/// Recursively check if a JSON-LD value contains "about" or "mentions" properties.
fn has_about_or_mentions(val: &Value) -> bool {
    if val.get("about").is_some() || val.get("mentions").is_some() {
        return true;
    }
    if let Some(Value::Array(graph)) = val.get("@graph") {
        for item in graph {
            if has_about_or_mentions(item) {
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

    // --- geo-entity-schema ---

    #[test]
    fn entity_schema_person_passes() {
        let html = r#"<html><head><script type="application/ld+json">{"@type":"Person","name":"John"}</script></head><body></body></html>"#;
        let doc = parse(html);
        let results = analyze_entities(&doc);
        let r = results.iter().find(|r| r.check == "geo-entity-schema").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn entity_schema_organization_passes() {
        let html = r#"<html><head><script type="application/ld+json">{"@type":"Organization","name":"ACME"}</script></head><body></body></html>"#;
        let doc = parse(html);
        let results = analyze_entities(&doc);
        let r = results.iter().find(|r| r.check == "geo-entity-schema").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn entity_schema_none_warns() {
        let html = r#"<html><head><script type="application/ld+json">{"@type":"Article"}</script></head><body></body></html>"#;
        let doc = parse(html);
        let results = analyze_entities(&doc);
        let r = results.iter().find(|r| r.check == "geo-entity-schema").unwrap();
        assert_eq!(r.status, Status::Warn);
    }

    // --- geo-entity-about ---

    #[test]
    fn entity_about_property_passes() {
        let html = r#"<html><head><script type="application/ld+json">{"@type":"Article","about":{"@type":"Thing","name":"Rust"}}</script></head><body></body></html>"#;
        let doc = parse(html);
        let results = analyze_entities(&doc);
        let r = results.iter().find(|r| r.check == "geo-entity-about").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn entity_mentions_property_passes() {
        let html = r#"<html><head><script type="application/ld+json">{"@type":"Article","mentions":{"@type":"Person","name":"John"}}</script></head><body></body></html>"#;
        let doc = parse(html);
        let results = analyze_entities(&doc);
        let r = results.iter().find(|r| r.check == "geo-entity-about").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn entity_about_none_warns() {
        let html = r#"<html><head><script type="application/ld+json">{"@type":"Article","name":"Test"}</script></head><body></body></html>"#;
        let doc = parse(html);
        let results = analyze_entities(&doc);
        let r = results.iter().find(|r| r.check == "geo-entity-about").unwrap();
        assert_eq!(r.status, Status::Warn);
    }

    // --- geo-entity-proper-nouns ---

    #[test]
    fn entity_proper_nouns_passes() {
        let doc = parse("<html><body><p>the company Google and Microsoft and Amazon are big players in the technology space</p></body></html>");
        let results = analyze_entities(&doc);
        let r = results.iter().find(|r| r.check == "geo-entity-proper-nouns").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn entity_proper_nouns_warns() {
        let doc = parse("<html><body><p>this is all lowercase text with no names at all</p></body></html>");
        let results = analyze_entities(&doc);
        let r = results.iter().find(|r| r.check == "geo-entity-proper-nouns").unwrap();
        assert_eq!(r.status, Status::Warn);
    }

    // --- geo-entity-author ---

    #[test]
    fn entity_author_meta_passes() {
        let doc = parse(r#"<html><head><meta name="author" content="John Doe"></head><body></body></html>"#);
        let results = analyze_entities(&doc);
        let r = results.iter().find(|r| r.check == "geo-entity-author").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn entity_author_person_schema_passes() {
        let html = r#"<html><head><script type="application/ld+json">{"@type":"Person","name":"Jane"}</script></head><body></body></html>"#;
        let doc = parse(html);
        let results = analyze_entities(&doc);
        let r = results.iter().find(|r| r.check == "geo-entity-author").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn entity_author_byline_passes() {
        let doc = parse("<html><body><p>by John Smith on March 1</p></body></html>");
        let results = analyze_entities(&doc);
        let r = results.iter().find(|r| r.check == "geo-entity-author").unwrap();
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn entity_author_none_warns() {
        let doc = parse("<html><body><p>no author information here</p></body></html>");
        let results = analyze_entities(&doc);
        let r = results.iter().find(|r| r.check == "geo-entity-author").unwrap();
        assert_eq!(r.status, Status::Warn);
    }

    // --- count ---

    #[test]
    fn entities_returns_exactly_4() {
        let doc = parse("<html><body><p>Hello</p></body></html>");
        let results = analyze_entities(&doc);
        assert_eq!(results.len(), 4);
    }
}
