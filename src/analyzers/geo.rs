use crate::scoring::{AnalysisResult, Status};
use regex::Regex;
use robotstxt::DefaultMatcher;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashSet;

// Static check IDs for AI bot results
const GEO_AI_BOT_GPTBOT: &str = "geo-ai-bot-gptbot";
const GEO_AI_BOT_CLAUDEBOT: &str = "geo-ai-bot-claudebot";
const GEO_AI_BOT_PERPLEXITYBOT: &str = "geo-ai-bot-perplexitybot";
const GEO_AI_BOT_GOOGLEOTHER: &str = "geo-ai-bot-googleother";
const GEO_AI_BOT_BYTESPIDER: &str = "geo-ai-bot-bytespider";
const GEO_AI_BOT_CCBOT: &str = "geo-ai-bot-ccbot";
const GEO_AI_BOT_OAI_SEARCHBOT: &str = "geo-ai-bot-oai-searchbot";
const GEO_AI_BOT_CLAUDE_SEARCHBOT: &str = "geo-ai-bot-claude-searchbot";
const GEO_AI_BOT_META_WEBINDEXER: &str = "geo-ai-bot-meta-webindexer";

/// What a crawler's robots.txt token actually controls, per each vendor's docs.
#[derive(Clone, Copy, PartialEq, Debug)]
enum BotRole {
    /// Builds the index an AI search product cites from; blocking it removes the site from those answers.
    Search,
    /// Collects data for model training or research; blocking it does not affect AI search visibility.
    DataCollection,
}

/// (user_agent, check_id, what the token controls, role)
const AI_BOTS: &[(&str, &str, &str, BotRole)] = &[
    ("OAI-SearchBot", GEO_AI_BOT_OAI_SEARCHBOT, "ChatGPT search", BotRole::Search),
    ("Claude-SearchBot", GEO_AI_BOT_CLAUDE_SEARCHBOT, "Claude search", BotRole::Search),
    ("PerplexityBot", GEO_AI_BOT_PERPLEXITYBOT, "Perplexity search", BotRole::Search),
    ("Meta-WebIndexer", GEO_AI_BOT_META_WEBINDEXER, "Meta AI search", BotRole::Search),
    ("GPTBot", GEO_AI_BOT_GPTBOT, "OpenAI model training", BotRole::DataCollection),
    ("ClaudeBot", GEO_AI_BOT_CLAUDEBOT, "Anthropic model training", BotRole::DataCollection),
    ("GoogleOther", GEO_AI_BOT_GOOGLEOTHER, "Google internal research and development crawls", BotRole::DataCollection),
    ("Bytespider", GEO_AI_BOT_BYTESPIDER, "ByteDance AI data collection", BotRole::DataCollection),
    ("CCBot", GEO_AI_BOT_CCBOT, "the Common Crawl dataset used to train many AI models", BotRole::DataCollection),
];

const SCHEMA_TARGETS: &[&str] = &["Article", "ItemList", "FAQPage"];

/// GEO-01: Detect listicle formatting patterns that AI search engines prefer.
pub fn analyze_listicle(html: &Html) -> AnalysisResult {
    let mut detected_patterns: Vec<&str> = Vec::new();

    // a. Heading patterns: Top N, Best N, N Best/Top
    let heading_sel = Selector::parse("h1, h2, h3").expect("valid selector");
    let top_n_re = Regex::new(r"(?i)\btop\s+\d+\b").expect("valid regex");
    let best_n_re = Regex::new(r"(?i)\bbest\s+\d+\b").expect("valid regex");
    let n_best_top_re = Regex::new(r"(?i)\b\d+\s+(?:best|top)\b").expect("valid regex");
    let numbered_heading_re = Regex::new(r"(?i)^\d+[\.\)]\s+").expect("valid regex");

    let mut has_top_n = false;
    let mut has_numbered_heading = false;

    for el in html.select(&heading_sel) {
        let text: String = el.text().collect();
        if top_n_re.is_match(&text) || best_n_re.is_match(&text) || n_best_top_re.is_match(&text)
        {
            has_top_n = true;
        }
        if numbered_heading_re.is_match(text.trim()) {
            has_numbered_heading = true;
        }
    }

    if has_top_n {
        detected_patterns.push("'Top N' heading pattern");
    }
    if has_numbered_heading {
        detected_patterns.push("numbered heading sequence");
    }

    // b. Ordered lists
    let ol_sel = Selector::parse("ol").expect("valid selector");
    if html.select(&ol_sel).next().is_some() {
        detected_patterns.push("ordered list (<ol>)");
    }

    // c. Comparison tables: table with <th> and >= 3 <tr> rows
    let table_sel = Selector::parse("table").expect("valid selector");
    let th_sel = Selector::parse("th").expect("valid selector");
    let tr_sel = Selector::parse("tr").expect("valid selector");

    for table in html.select(&table_sel) {
        let has_th = table.select(&th_sel).next().is_some();
        let row_count = table.select(&tr_sel).count();
        if has_th && row_count >= 3 {
            detected_patterns.push("comparison table");
            break;
        }
    }

    if detected_patterns.is_empty() {
        AnalysisResult {
            check: "geo-listicle",
            status: Status::Warn,
            message: "No listicle format detected on this page".to_string(),
            recommendation: "Consider restructuring content as a numbered list or 'Top N' format -- 74.2% of AI citations come from listicle-style content. Use ordered lists, 'Top N'/'Best N' headings, or comparison tables to improve AI search visibility".to_string(),
        }
    } else {
        AnalysisResult {
            check: "geo-listicle",
            status: Status::Pass,
            message: format!("Listicle format detected: {}", detected_patterns.join(", ")),
            recommendation: String::new(),
        }
    }
}

/// GEO-02: Check robots.txt for AI crawler access.
///
/// Search crawlers decide whether a page can be cited in AI search answers, so
/// blocking them fails. Data-collection crawlers (training, research) do not
/// affect AI search visibility, so blocking them is reported as a low-weight
/// warning that explains the trade-off.
pub fn analyze_ai_bots(robots_body: &str) -> Vec<AnalysisResult> {
    let mut matcher = DefaultMatcher::default();

    AI_BOTS
        .iter()
        .map(|&(user_agent, check_id, purpose, role)| {
            let allowed =
                matcher.one_agent_allowed_by_robots(robots_body, user_agent, "/");

            match (role, allowed) {
                (BotRole::Search, true) => AnalysisResult {
                    check: check_id,
                    status: Status::Pass,
                    message: format!(
                        "{} is allowed in robots.txt. Your content can appear in {} results.",
                        user_agent, purpose
                    ),
                    recommendation: String::new(),
                },
                (BotRole::Search, false) => AnalysisResult {
                    check: check_id,
                    status: Status::Fail,
                    message: format!(
                        "{} is blocked in robots.txt. This prevents your content from appearing in {} results.",
                        user_agent, purpose
                    ),
                    recommendation: format!(
                        "Remove or modify the Disallow rule for {} in robots.txt so {} can index and cite your content",
                        user_agent, purpose
                    ),
                },
                (BotRole::DataCollection, true) => AnalysisResult {
                    check: check_id,
                    status: Status::Pass,
                    message: format!(
                        "{} is allowed in robots.txt. It collects content for {}; it does not control AI search visibility.",
                        user_agent, purpose
                    ),
                    recommendation: String::new(),
                },
                (BotRole::DataCollection, false) => AnalysisResult {
                    check: check_id,
                    status: Status::Warn,
                    message: format!(
                        "{} is blocked in robots.txt. This opts your content out of {} but does not affect AI search visibility.",
                        user_agent, purpose
                    ),
                    recommendation: format!(
                        "Keep this rule if opting out of {} is intentional. Remove the Disallow rule for {} if you want your content included.",
                        purpose, user_agent
                    ),
                },
            }
        })
        .collect()
}

/// Extract @type values from a JSON-LD value, handling @graph arrays.
pub(crate) fn extract_types(val: &Value, types: &mut HashSet<String>) {
    // Direct @type
    match val.get("@type") {
        Some(Value::String(s)) => {
            types.insert(s.clone());
        }
        Some(Value::Array(arr)) => {
            for item in arr {
                if let Some(s) = item.as_str() {
                    types.insert(s.to_string());
                }
            }
        }
        _ => {}
    }

    // @graph array
    if let Some(Value::Array(graph)) = val.get("@graph") {
        for item in graph {
            extract_types(item, types);
        }
    }
}

/// GEO-03: Detect triple schema stacking (Article + ItemList + FAQPage).
pub fn analyze_schema_stacking(html: &Html) -> AnalysisResult {
    let ld_sel =
        Selector::parse(r#"script[type="application/ld+json"]"#).expect("valid selector");

    let mut found_types: HashSet<String> = HashSet::new();

    for block in html.select(&ld_sel) {
        let text: String = block.text().collect();
        if let Ok(val) = serde_json::from_str::<Value>(&text) {
            extract_types(&val, &mut found_types);
        }
    }

    let found: Vec<&&str> = SCHEMA_TARGETS
        .iter()
        .filter(|t| found_types.contains(**t))
        .collect();
    let missing: Vec<&&str> = SCHEMA_TARGETS
        .iter()
        .filter(|t| !found_types.contains(**t))
        .collect();

    match found.len() {
        3 => AnalysisResult {
            check: "geo-schema-stacking",
            status: Status::Pass,
            message: "Triple schema stacking detected: Article + ItemList + FAQPage all present"
                .to_string(),
            recommendation: String::new(),
        },
        1 | 2 => {
            let found_list: Vec<&str> = found.iter().map(|t| **t).collect();
            let missing_list: Vec<&str> = missing.iter().map(|t| **t).collect();
            AnalysisResult {
                check: "geo-schema-stacking",
                status: Status::Warn,
                message: format!(
                    "Partial schema stacking: found {}, missing {}",
                    found_list.join(", "),
                    missing_list.join(", ")
                ),
                recommendation: format!(
                    "Add the missing schema type(s) ({}) to achieve triple schema stacking (Article + ItemList + FAQPage). Triple stacking improves visibility in AI-powered search results",
                    missing_list.join(", ")
                ),
            }
        }
        _ => AnalysisResult {
            check: "geo-schema-stacking",
            status: Status::Fail,
            message: "No GEO-relevant schema types found (Article, ItemList, FAQPage)".to_string(),
            recommendation: "Add JSON-LD schema markup with Article, ItemList, and FAQPage types. Triple schema stacking (all three on one page) is a GEO best practice for AI search visibility".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    fn parse(html: &str) -> Html {
        Html::parse_document(html)
    }

    // --- analyze_listicle ---

    #[test]
    fn top_n_heading_passes() {
        let doc = parse("<html><body><h2>Top 10 Tools for SEO</h2></body></html>");
        let r = analyze_listicle(&doc);
        assert_eq!(r.check, "geo-listicle");
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn case_insensitive_top_n() {
        let doc = parse("<html><body><h1>TOP 5 FRAMEWORKS</h1></body></html>");
        let r = analyze_listicle(&doc);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn best_n_heading_passes() {
        let doc = parse("<html><body><h2>Best 10 Practices</h2></body></html>");
        let r = analyze_listicle(&doc);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn n_best_heading_passes() {
        let doc = parse("<html><body><h3>10 Best Ways to Learn Rust</h3></body></html>");
        let r = analyze_listicle(&doc);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn ol_element_passes() {
        let doc = parse("<html><body><ol><li>First</li><li>Second</li></ol></body></html>");
        let r = analyze_listicle(&doc);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn numbered_heading_passes() {
        let doc = parse("<html><body><h2>1. First Item</h2><h2>2. Second Item</h2></body></html>");
        let r = analyze_listicle(&doc);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn comparison_table_passes() {
        let doc = parse(
            "<html><body><table><tr><th>Name</th><th>Score</th></tr>\
             <tr><td>A</td><td>1</td></tr>\
             <tr><td>B</td><td>2</td></tr>\
             <tr><td>C</td><td>3</td></tr></table></body></html>",
        );
        let r = analyze_listicle(&doc);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn no_listicle_warns() {
        let doc = parse("<html><body><p>Just a paragraph.</p></body></html>");
        let r = analyze_listicle(&doc);
        assert_eq!(r.status, Status::Warn);
        assert!(r.recommendation.contains("74.2%"), "recommendation should mention 74.2%");
    }

    // --- analyze_ai_bots ---

    #[test]
    fn all_allowed_returns_9_passes() {
        let results = analyze_ai_bots("");
        assert_eq!(results.len(), 9);
        for r in &results {
            assert_eq!(r.status, Status::Pass, "bot {} should pass", r.check);
        }
    }

    #[test]
    fn training_bot_blocked_returns_warn_not_fail() {
        let robots = "User-agent: GPTBot\nDisallow: /\n";
        let results = analyze_ai_bots(robots);
        let gpt = results.iter().find(|r| r.check == "geo-ai-bot-gptbot").unwrap();
        assert_eq!(gpt.status, Status::Warn);
        assert!(gpt.message.contains("GPTBot"));
        assert!(gpt.message.contains("does not affect AI search visibility"));
        let oai = results.iter().find(|r| r.check == "geo-ai-bot-oai-searchbot").unwrap();
        assert_eq!(oai.status, Status::Pass);
    }

    #[test]
    fn search_bot_blocked_returns_fail_even_if_training_bot_allowed() {
        let robots = "User-agent: OAI-SearchBot\nDisallow: /\n\nUser-agent: GPTBot\nAllow: /\n";
        let results = analyze_ai_bots(robots);
        let oai = results.iter().find(|r| r.check == "geo-ai-bot-oai-searchbot").unwrap();
        assert_eq!(oai.status, Status::Fail);
        assert!(oai.message.contains("ChatGPT search"));
        let gpt = results.iter().find(|r| r.check == "geo-ai-bot-gptbot").unwrap();
        assert_eq!(gpt.status, Status::Pass);
    }

    #[test]
    fn all_blocked_fails_search_bots_and_warns_data_collection_bots() {
        let robots = "User-agent: *\nDisallow: /\n";
        let results = analyze_ai_bots(robots);
        assert_eq!(results.len(), 9);
        let fails = results.iter().filter(|r| r.status == Status::Fail).count();
        let warns = results.iter().filter(|r| r.status == Status::Warn).count();
        assert_eq!(fails, 4, "search bots should fail");
        assert_eq!(warns, 5, "data-collection bots should warn");
    }

    // --- analyze_schema_stacking ---

    #[test]
    fn all_three_passes() {
        let html = r#"<html><head>
            <script type="application/ld+json">{"@context":"https://schema.org","@type":"Article"}</script>
            <script type="application/ld+json">{"@context":"https://schema.org","@type":"ItemList"}</script>
            <script type="application/ld+json">{"@context":"https://schema.org","@type":"FAQPage"}</script>
        </head></html>"#;
        let doc = parse(html);
        let r = analyze_schema_stacking(&doc);
        assert_eq!(r.check, "geo-schema-stacking");
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn two_of_three_warns() {
        let html = r#"<html><head>
            <script type="application/ld+json">{"@context":"https://schema.org","@type":"Article"}</script>
            <script type="application/ld+json">{"@context":"https://schema.org","@type":"FAQPage"}</script>
        </head></html>"#;
        let doc = parse(html);
        let r = analyze_schema_stacking(&doc);
        assert_eq!(r.status, Status::Warn);
        assert!(r.message.contains("ItemList"), "should mention missing ItemList");
    }

    #[test]
    fn none_fails() {
        let doc = parse("<html><body><p>No schema</p></body></html>");
        let r = analyze_schema_stacking(&doc);
        assert_eq!(r.status, Status::Fail);
    }

    #[test]
    fn array_type_handled() {
        let html = r#"<html><head>
            <script type="application/ld+json">{"@context":"https://schema.org","@type":["Article","ItemList","FAQPage"]}</script>
        </head></html>"#;
        let doc = parse(html);
        let r = analyze_schema_stacking(&doc);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn graph_array_handled() {
        let html = r#"<html><head>
            <script type="application/ld+json">{
                "@context":"https://schema.org",
                "@graph":[
                    {"@type":"Article"},
                    {"@type":"ItemList"},
                    {"@type":"FAQPage"}
                ]
            }</script>
        </head></html>"#;
        let doc = parse(html);
        let r = analyze_schema_stacking(&doc);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn multiple_blocks_combined() {
        let html = r#"<html><head>
            <script type="application/ld+json">{"@context":"https://schema.org","@type":"Article"}</script>
            <script type="application/ld+json">{"@context":"https://schema.org","@graph":[{"@type":"ItemList"},{"@type":"FAQPage"}]}</script>
        </head></html>"#;
        let doc = parse(html);
        let r = analyze_schema_stacking(&doc);
        assert_eq!(r.status, Status::Pass);
    }
}
