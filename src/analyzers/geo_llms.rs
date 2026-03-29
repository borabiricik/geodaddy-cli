use crate::scoring::{AnalysisResult, Status};

/// Analyze llms.txt presence and validate its structure.
///
/// Checks:
/// - Empty/whitespace body: Fail (no llms.txt found)
/// - Missing H1 heading (line starting with "# "): Warn
/// - Too short (<50 bytes): Warn
/// - Valid (has H1, >=50 bytes): Pass
pub fn analyze_llms_txt(body: &str) -> AnalysisResult {
    let trimmed = body.trim();

    if trimmed.is_empty() {
        return AnalysisResult {
            check: "geo-llms-txt",
            status: Status::Fail,
            message: "No /llms.txt file found. This file helps AI systems understand your site structure.".to_string(),
            recommendation: "Create a /llms.txt file at your site root following the llms.txt specification (https://llmstxt.org). Include an H1 with your site name, a summary blockquote, and links to key content sections.".to_string(),
        };
    }

    let has_h1 = trimmed.lines().any(|line| line.starts_with("# "));

    if !has_h1 {
        return AnalysisResult {
            check: "geo-llms-txt",
            status: Status::Warn,
            message: "/llms.txt exists but missing required H1 heading.".to_string(),
            recommendation: "Add an H1 heading (# Your Site Name) as the first line of llms.txt -- this is the only required element per the specification.".to_string(),
        };
    }

    if body.len() < 50 {
        return AnalysisResult {
            check: "geo-llms-txt",
            status: Status::Warn,
            message: "/llms.txt exists but appears too short to be useful.".to_string(),
            recommendation: "Expand your llms.txt with a summary blockquote and links to key content sections. AI systems use this to navigate your site efficiently.".to_string(),
        };
    }

    AnalysisResult {
        check: "geo-llms-txt",
        status: Status::Pass,
        message: format!("/llms.txt found ({} bytes) with valid structure.", body.len()),
        recommendation: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_body_returns_fail() {
        let r = analyze_llms_txt("");
        assert_eq!(r.check, "geo-llms-txt");
        assert_eq!(r.status, Status::Fail);
        assert!(r.message.contains("No /llms.txt"));
    }

    #[test]
    fn whitespace_body_returns_fail() {
        let r = analyze_llms_txt("   \n  \t  ");
        assert_eq!(r.status, Status::Fail);
    }

    #[test]
    fn too_short_returns_warn() {
        let r = analyze_llms_txt("just short");
        assert_eq!(r.check, "geo-llms-txt");
        assert_eq!(r.status, Status::Warn);
        assert!(r.message.contains("missing required H1") || r.message.contains("too short"));
    }

    #[test]
    fn missing_h1_returns_warn() {
        let body = "No heading here but long enough content to pass length check easily";
        let r = analyze_llms_txt(body);
        assert_eq!(r.status, Status::Warn);
        assert!(r.message.contains("missing required H1"));
    }

    #[test]
    fn valid_llms_txt_returns_pass() {
        let body = "# My Site\n\n> Summary of the site content and purpose here for AI systems";
        let r = analyze_llms_txt(body);
        assert_eq!(r.check, "geo-llms-txt");
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn pass_message_contains_byte_count() {
        let body = "# My Site\n\n> Summary of the site content and purpose here for AI systems";
        let r = analyze_llms_txt(body);
        assert!(r.message.contains(&format!("{} bytes", body.len())));
    }

    #[test]
    fn all_results_have_correct_check_id() {
        let cases = vec![
            "",
            "short",
            "No heading but enough content to be over fifty bytes easily here",
            "# Valid\n\n> Summary of the site content and purpose here for AI systems",
        ];
        for body in cases {
            let r = analyze_llms_txt(body);
            assert_eq!(r.check, "geo-llms-txt", "check ID mismatch for body: {}", body);
        }
    }

    #[test]
    fn h1_with_short_body_warns_about_h1_not_length() {
        // Has H1 but body < 50 bytes
        let body = "# Site\n> Short";
        let r = analyze_llms_txt(body);
        assert_eq!(r.status, Status::Warn);
        assert!(r.message.contains("too short"));
    }
}
