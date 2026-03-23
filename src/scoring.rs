use serde::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pass,
    Fail,
    Warn,
}

#[derive(Serialize, Debug, Clone)]
pub struct AnalysisResult {
    pub check: &'static str,
    pub status: Status,
    pub message: String,
    pub recommendation: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct CategoryScores {
    pub technical: f64,
    pub content: f64,
    pub geo: f64,
}

fn severity_points(check: &str) -> u32 {
    match check {
        "tech-meta-title" | "tech-heading-h1" | "tech-mobile-viewport" | "tech-https"
        | "cont-json-ld" => 10,
        "tech-broken-links"
        | "tech-redirect-chains"
        | "tech-meta-description"
        | "tech-heading-hierarchy"
        | "tech-robots-txt"
        | "cont-heading-structure"
        | "cont-alt-text" => 5,
        "tech-sitemap-xml" | "cont-semantic-html" => 2,
        "geo-listicle" | "geo-schema-stacking" => 5,
        _ if check.starts_with("geo-ai-bot-") => 10,
        _ => 5,
    }
}

pub fn calculate_score(results: &[AnalysisResult]) -> (f64, CategoryScores) {
    let mut tech_earned: u32 = 0;
    let mut tech_max: u32 = 0;
    let mut cont_earned: u32 = 0;
    let mut cont_max: u32 = 0;
    let mut geo_earned: u32 = 0;
    let mut geo_max: u32 = 0;

    for result in results {
        let pts = severity_points(result.check);
        let earned = match result.status {
            Status::Pass => pts,
            Status::Fail => 0,
            Status::Warn => pts / 2,
        };

        if result.check.starts_with("tech-") {
            tech_earned += earned;
            tech_max += pts;
        } else if result.check.starts_with("cont-") {
            cont_earned += earned;
            cont_max += pts;
        } else if result.check.starts_with("geo-") {
            geo_earned += earned;
            geo_max += pts;
        }
    }

    let tech_score = if tech_max == 0 {
        100.0_f64
    } else {
        (tech_earned as f64 / tech_max as f64) * 100.0
    };

    let cont_score = if cont_max == 0 {
        100.0_f64
    } else {
        (cont_earned as f64 / cont_max as f64) * 100.0
    };

    let geo_score = if geo_max == 0 {
        100.0_f64
    } else {
        (geo_earned as f64 / geo_max as f64) * 100.0
    };

    let tech_score = tech_score.clamp(0.0, 100.0);
    let cont_score = cont_score.clamp(0.0, 100.0);
    let geo_score = geo_score.clamp(0.0, 100.0);

    let overall = ((tech_score + cont_score + geo_score) / 3.0).clamp(0.0, 100.0);

    (overall, CategoryScores { technical: tech_score, content: cont_score, geo: geo_score })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(check: &'static str, status: Status) -> AnalysisResult {
        AnalysisResult {
            check,
            status,
            message: String::new(),
            recommendation: String::new(),
        }
    }

    #[test]
    fn test_empty_results_returns_100() {
        let (overall, cats) = calculate_score(&[]);
        assert_eq!(overall, 100.0);
        assert_eq!(cats.technical, 100.0);
        assert_eq!(cats.content, 100.0);
        assert_eq!(cats.geo, 100.0);
    }

    #[test]
    fn test_critical_fail_deducts_10_points() {
        // tech-meta-title is critical (10 pts), Fail => 0 earned out of 10
        let results = vec![make_result("tech-meta-title", Status::Fail)];
        let (_, cats) = calculate_score(&results);
        assert_eq!(cats.technical, 0.0);
        assert_eq!(cats.content, 100.0);
    }

    #[test]
    fn test_warn_deducts_half_points() {
        // tech-meta-title is critical (10 pts), Warn => 5 earned out of 10 = 50%
        let results = vec![make_result("tech-meta-title", Status::Warn)];
        let (_, cats) = calculate_score(&results);
        assert_eq!(cats.technical, 50.0);
        assert_eq!(cats.content, 100.0);
    }

    #[test]
    fn test_category_separation() {
        // tech check affects technical, cont check affects content
        let results = vec![
            make_result("tech-meta-title", Status::Fail),
            make_result("cont-json-ld", Status::Pass),
        ];
        let (_, cats) = calculate_score(&results);
        assert_eq!(cats.technical, 0.0);
        assert_eq!(cats.content, 100.0);
        assert_eq!(cats.geo, 100.0);
    }

    #[test]
    fn test_overall_is_average() {
        // technical = 0% (fail critical), content = 100% (pass critical), geo = 100% (no geo checks)
        // overall = (0 + 100 + 100) / 3 = 66.67
        let results = vec![
            make_result("tech-meta-title", Status::Fail),
            make_result("cont-json-ld", Status::Pass),
        ];
        let (overall, _) = calculate_score(&results);
        assert!((overall - 200.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_geo_category_routing() {
        // geo-listicle Fail should affect only geo score, not tech/content
        let results = vec![make_result("geo-listicle", Status::Fail)];
        let (_, cats) = calculate_score(&results);
        assert_eq!(cats.technical, 100.0);
        assert_eq!(cats.content, 100.0);
        assert_eq!(cats.geo, 0.0);
    }

    #[test]
    fn test_geo_ai_bot_critical_severity() {
        // geo-ai-bot-gptbot is critical (10 pts), Fail => 0 earned out of 10
        let results = vec![
            make_result("geo-ai-bot-gptbot", Status::Pass),
            make_result("geo-ai-bot-claudebot", Status::Fail),
        ];
        let (_, cats) = calculate_score(&results);
        // 10 earned + 0 earned = 10 out of 20 = 50%
        assert_eq!(cats.geo, 50.0);
    }

    #[test]
    fn test_geo_mixed_with_tech_cont() {
        // tech: fail critical (0%), cont: pass critical (100%), geo: fail listicle (0%)
        // overall = (0 + 100 + 0) / 3 = 33.33
        let results = vec![
            make_result("tech-meta-title", Status::Fail),
            make_result("cont-json-ld", Status::Pass),
            make_result("geo-listicle", Status::Fail),
        ];
        let (overall, cats) = calculate_score(&results);
        assert_eq!(cats.technical, 0.0);
        assert_eq!(cats.content, 100.0);
        assert_eq!(cats.geo, 0.0);
        assert!((overall - 100.0 / 3.0).abs() < 0.01);
    }
}
