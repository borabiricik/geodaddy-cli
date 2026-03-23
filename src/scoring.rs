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

#[derive(Serialize, Debug)]
pub struct CategoryScores {
    pub technical: f64,
    pub content: f64,
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
        _ => 5,
    }
}

pub fn calculate_score(results: &[AnalysisResult]) -> (f64, CategoryScores) {
    let mut tech_earned: u32 = 0;
    let mut tech_max: u32 = 0;
    let mut cont_earned: u32 = 0;
    let mut cont_max: u32 = 0;

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

    let tech_score = tech_score.clamp(0.0, 100.0);
    let cont_score = cont_score.clamp(0.0, 100.0);

    let overall = ((tech_score + cont_score) / 2.0).clamp(0.0, 100.0);

    (overall, CategoryScores { technical: tech_score, content: cont_score })
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
    }

    #[test]
    fn test_overall_is_average() {
        // technical = 0% (fail critical), content = 100% (pass critical)
        // overall = (0 + 100) / 2 = 50
        let results = vec![
            make_result("tech-meta-title", Status::Fail),
            make_result("cont-json-ld", Status::Pass),
        ];
        let (overall, _) = calculate_score(&results);
        assert_eq!(overall, 50.0);
    }
}
