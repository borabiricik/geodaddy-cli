// Compile stub -- replaced in plan 05-02
use crate::scoring::AnalysisResult;

pub async fn analyze_vitals(_page: &chromiumoxide::Page) -> Vec<AnalysisResult> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoring::Status;

    // --- classify_lcp ---

    #[test]
    fn classify_lcp_pass_below_threshold() {
        let r = classify_lcp(2000.0);
        assert_eq!(r.check, "perf-lcp");
        assert_eq!(r.status, Status::Pass);
        assert!(r.message.contains("2.00s"), "message should contain '2.00s', got: {}", r.message);
    }

    #[test]
    fn classify_lcp_pass_at_boundary() {
        let r = classify_lcp(2500.0);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn classify_lcp_warn_above_pass_threshold() {
        let r = classify_lcp(3000.0);
        assert_eq!(r.status, Status::Warn);
    }

    #[test]
    fn classify_lcp_warn_at_boundary() {
        let r = classify_lcp(4000.0);
        assert_eq!(r.status, Status::Warn);
    }

    #[test]
    fn classify_lcp_fail_above_warn_threshold() {
        let r = classify_lcp(4001.0);
        assert_eq!(r.status, Status::Fail);
    }

    #[test]
    fn classify_lcp_fail_unmeasured() {
        let r = classify_lcp(-1.0);
        assert_eq!(r.status, Status::Fail);
        assert!(
            r.message.contains("could not be measured"),
            "message should contain 'could not be measured', got: {}",
            r.message
        );
    }

    // --- classify_fcp ---

    #[test]
    fn classify_fcp_pass_below_threshold() {
        let r = classify_fcp(1000.0);
        assert_eq!(r.check, "perf-fcp");
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn classify_fcp_pass_at_boundary() {
        let r = classify_fcp(1800.0);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn classify_fcp_warn_above_pass_threshold() {
        let r = classify_fcp(2500.0);
        assert_eq!(r.status, Status::Warn);
    }

    #[test]
    fn classify_fcp_warn_at_boundary() {
        let r = classify_fcp(3000.0);
        assert_eq!(r.status, Status::Warn);
    }

    #[test]
    fn classify_fcp_fail_above_warn_threshold() {
        let r = classify_fcp(3001.0);
        assert_eq!(r.status, Status::Fail);
    }

    #[test]
    fn classify_fcp_fail_unmeasured() {
        let r = classify_fcp(-1.0);
        assert_eq!(r.status, Status::Fail);
        assert!(
            r.message.contains("could not be measured"),
            "message should contain 'could not be measured', got: {}",
            r.message
        );
    }

    // --- classify_cls ---

    #[test]
    fn classify_cls_pass_at_zero() {
        let r = classify_cls(0.0);
        assert_eq!(r.check, "perf-cls");
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn classify_cls_pass_at_boundary() {
        let r = classify_cls(0.1);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn classify_cls_warn_above_pass_threshold() {
        let r = classify_cls(0.15);
        assert_eq!(r.status, Status::Warn);
    }

    #[test]
    fn classify_cls_warn_at_boundary() {
        let r = classify_cls(0.25);
        assert_eq!(r.status, Status::Warn);
    }

    #[test]
    fn classify_cls_fail_above_warn_threshold() {
        let r = classify_cls(0.26);
        assert_eq!(r.status, Status::Fail);
    }

    #[test]
    fn classify_cls_fail_unmeasured() {
        let r = classify_cls(-1.0);
        assert_eq!(r.status, Status::Fail);
        assert!(
            r.message.contains("could not be measured"),
            "message should contain 'could not be measured', got: {}",
            r.message
        );
    }

    // --- classify_ttfb ---

    #[test]
    fn classify_ttfb_pass_below_threshold() {
        let r = classify_ttfb(500.0);
        assert_eq!(r.check, "perf-ttfb");
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn classify_ttfb_pass_at_boundary() {
        let r = classify_ttfb(800.0);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn classify_ttfb_warn_above_pass_threshold() {
        let r = classify_ttfb(1000.0);
        assert_eq!(r.status, Status::Warn);
    }

    #[test]
    fn classify_ttfb_warn_at_boundary() {
        let r = classify_ttfb(1800.0);
        assert_eq!(r.status, Status::Warn);
    }

    #[test]
    fn classify_ttfb_fail_above_warn_threshold() {
        let r = classify_ttfb(1801.0);
        assert_eq!(r.status, Status::Fail);
    }

    #[test]
    fn classify_ttfb_fail_unmeasured() {
        let r = classify_ttfb(-1.0);
        assert_eq!(r.status, Status::Fail);
        assert!(
            r.message.contains("could not be measured"),
            "message should contain 'could not be measured', got: {}",
            r.message
        );
    }

    // --- classify_tbt ---

    #[test]
    fn classify_tbt_pass_at_zero() {
        let r = classify_tbt(0.0);
        assert_eq!(r.check, "perf-tbt");
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn classify_tbt_pass_at_boundary() {
        let r = classify_tbt(200.0);
        assert_eq!(r.status, Status::Pass);
    }

    #[test]
    fn classify_tbt_warn_above_pass_threshold() {
        let r = classify_tbt(400.0);
        assert_eq!(r.status, Status::Warn);
    }

    #[test]
    fn classify_tbt_warn_at_boundary() {
        let r = classify_tbt(600.0);
        assert_eq!(r.status, Status::Warn);
    }

    #[test]
    fn classify_tbt_fail_above_warn_threshold() {
        let r = classify_tbt(601.0);
        assert_eq!(r.status, Status::Fail);
    }

    #[test]
    fn classify_tbt_fail_unmeasured() {
        let r = classify_tbt(-1.0);
        assert_eq!(r.status, Status::Fail);
        assert!(
            r.message.contains("could not be measured"),
            "message should contain 'could not be measured', got: {}",
            r.message
        );
    }
}
