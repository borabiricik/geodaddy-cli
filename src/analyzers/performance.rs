use chromiumoxide::Page;
use crate::scoring::{AnalysisResult, Status};

// --- JavaScript constants for PerformanceObserver measurements ---

const LCP_JS: &str = r#"() => {
    return new Promise((resolve) => {
        new PerformanceObserver((l) => {
            const entries = l.getEntries();
            if (entries.length > 0) {
                const last = entries[entries.length - 1];
                resolve(last.renderTime || last.loadTime);
            }
        }).observe({ type: 'largest-contentful-paint', buffered: true });
        setTimeout(() => resolve(-1), 5000);
    });
}"#;

const FCP_JS: &str = r#"() => {
    const entries = performance.getEntriesByName('first-contentful-paint');
    return entries.length > 0 ? entries[0].startTime : -1;
}"#;

const CLS_JS: &str = r#"() => {
    return new Promise((resolve) => {
        let cls = 0;
        new PerformanceObserver((list) => {
            for (const entry of list.getEntries()) {
                if (!entry.hadRecentInput) { cls += entry.value; }
            }
            resolve(cls);
        }).observe({ type: 'layout-shift', buffered: true });
        setTimeout(() => resolve(cls), 2000);
    });
}"#;

const TTFB_JS: &str = r#"() => {
    const nav = performance.getEntriesByType('navigation');
    if (!nav || nav.length === 0) return -1;
    return nav[0].responseStart;
}"#;

const TBT_JS: &str = r#"() => {
    return new Promise((resolve) => {
        let tbt = 0;
        new PerformanceObserver((list) => {
            for (const entry of list.getEntries()) {
                tbt += Math.max(0, entry.duration - 50);
            }
            resolve(tbt);
        }).observe({ type: 'longtask', buffered: true });
        setTimeout(() => resolve(tbt), 5000);
    });
}"#;

// --- Public entry point ---

/// Measures all 5 Core Web Vitals for an already-navigated chromiumoxide Page.
/// The page must have completed navigation before this is called.
/// Returns exactly 5 AnalysisResult entries in order: LCP, FCP, CLS, TTFB, TBT.
pub async fn analyze_vitals(page: &Page) -> Vec<AnalysisResult> {
    vec![
        measure_lcp(page).await,
        measure_fcp(page).await,
        measure_cls(page).await,
        measure_ttfb(page).await,
        measure_tbt(page).await,
    ]
}

// --- Private async measure_* functions ---

async fn measure_lcp(page: &Page) -> AnalysisResult {
    let val = eval_f64(page, LCP_JS).await;
    classify_lcp(val)
}

async fn measure_fcp(page: &Page) -> AnalysisResult {
    let val = eval_f64(page, FCP_JS).await;
    classify_fcp(val)
}

async fn measure_cls(page: &Page) -> AnalysisResult {
    let val = eval_f64(page, CLS_JS).await;
    classify_cls(val)
}

async fn measure_ttfb(page: &Page) -> AnalysisResult {
    let val = eval_f64(page, TTFB_JS).await;
    classify_ttfb(val)
}

async fn measure_tbt(page: &Page) -> AnalysisResult {
    let val = eval_f64(page, TBT_JS).await;
    classify_tbt(val)
}

// --- Shared CDP evaluator helper ---

/// Evaluate a JavaScript expression in the page, returning the result as f64.
/// Returns -1.0 on any error (CDP failure, parse error, JS exception).
async fn eval_f64(page: &Page, js: &str) -> f64 {
    match page.evaluate(js).await {
        Ok(result) => result.into_value::<f64>().unwrap_or(-1.0),
        Err(e) => {
            tracing::warn!("CDP evaluation failed: {}", e);
            -1.0
        }
    }
}

// --- pub(crate) classify_* functions ---

pub(crate) fn classify_lcp(lcp_ms: f64) -> AnalysisResult {
    if lcp_ms < 0.0 {
        return AnalysisResult {
            check: "perf-lcp",
            status: Status::Fail,
            message: "LCP could not be measured (no LCP candidate found or page timed out)"
                .to_string(),
            recommendation:
                "Ensure the page renders a visible content element (image, text block, or video)."
                    .to_string(),
        };
    }
    let lcp_s = lcp_ms / 1000.0;
    let (status, message, recommendation) = if lcp_s <= 2.5 {
        (
            Status::Pass,
            format!("LCP {:.2}s — good (≤2.5s)", lcp_s),
            "No action needed. LCP is within Google's good threshold.".to_string(),
        )
    } else if lcp_s <= 4.0 {
        (
            Status::Warn,
            format!("LCP {:.2}s — needs improvement (2.5s–4s)", lcp_s),
            "Optimize server response time, eliminate render-blocking resources, and preload the LCP element."
                .to_string(),
        )
    } else {
        (
            Status::Fail,
            format!("LCP {:.2}s — poor (>4s)", lcp_s),
            "Critical: reduce LCP by optimizing images, server response time, and critical rendering path. Target ≤2.5s."
                .to_string(),
        )
    };
    AnalysisResult {
        check: "perf-lcp",
        status,
        message,
        recommendation,
    }
}

pub(crate) fn classify_fcp(fcp_ms: f64) -> AnalysisResult {
    if fcp_ms < 0.0 {
        return AnalysisResult {
            check: "perf-fcp",
            status: Status::Fail,
            message: "FCP could not be measured".to_string(),
            recommendation:
                "Ensure the page renders visible content and the first-contentful-paint entry is available."
                    .to_string(),
        };
    }
    let fcp_s = fcp_ms / 1000.0;
    let (status, message, recommendation) = if fcp_ms <= 1800.0 {
        (
            Status::Pass,
            format!("FCP {:.2}s — good (≤1.8s)", fcp_s),
            "No action needed.".to_string(),
        )
    } else if fcp_ms <= 3000.0 {
        (
            Status::Warn,
            format!("FCP {:.2}s — needs improvement (1.8s–3s)", fcp_s),
            "Reduce render-blocking CSS and JavaScript to improve first paint time.".to_string(),
        )
    } else {
        (
            Status::Fail,
            format!("FCP {:.2}s — poor (>3s)", fcp_s),
            "Eliminate render-blocking resources and reduce server response time. Target ≤1.8s."
                .to_string(),
        )
    };
    AnalysisResult {
        check: "perf-fcp",
        status,
        message,
        recommendation,
    }
}

pub(crate) fn classify_cls(cls: f64) -> AnalysisResult {
    if cls < 0.0 {
        return AnalysisResult {
            check: "perf-cls",
            status: Status::Fail,
            message: "CLS could not be measured".to_string(),
            recommendation:
                "Ensure layout-shift PerformanceObserver entries are available.".to_string(),
        };
    }
    let (status, message, recommendation) = if cls <= 0.1 {
        (
            Status::Pass,
            format!("CLS {:.3} — good (≤0.1)", cls),
            "No action needed.".to_string(),
        )
    } else if cls <= 0.25 {
        (
            Status::Warn,
            format!("CLS {:.3} — needs improvement (0.1–0.25)", cls),
            "Add size attributes to images and embeds; avoid inserting content above existing content."
                .to_string(),
        )
    } else {
        (
            Status::Fail,
            format!("CLS {:.3} — poor (>0.25)", cls),
            "Significant layout instability detected. Reserve space for dynamic content and avoid late-loading ads above the fold."
                .to_string(),
        )
    };
    AnalysisResult {
        check: "perf-cls",
        status,
        message,
        recommendation,
    }
}

pub(crate) fn classify_ttfb(ttfb_ms: f64) -> AnalysisResult {
    if ttfb_ms < 0.0 {
        return AnalysisResult {
            check: "perf-ttfb",
            status: Status::Fail,
            message: "TTFB could not be measured".to_string(),
            recommendation:
                "Ensure Navigation Timing API is available and the page completed a network request."
                    .to_string(),
        };
    }
    let ttfb_s = ttfb_ms / 1000.0;
    let (status, message, recommendation) = if ttfb_ms <= 800.0 {
        (
            Status::Pass,
            format!("TTFB {:.0}ms — good (≤800ms)", ttfb_ms),
            "No action needed.".to_string(),
        )
    } else if ttfb_ms <= 1800.0 {
        (
            Status::Warn,
            format!("TTFB {:.0}ms — needs improvement (800ms–1.8s)", ttfb_ms),
            "Optimize server response time, enable caching, and consider a CDN.".to_string(),
        )
    } else {
        (
            Status::Fail,
            format!("TTFB {:.1}s — poor (>1.8s)", ttfb_s),
            "Server is too slow. Optimize database queries, add server-side caching, or use a CDN. Target ≤800ms."
                .to_string(),
        )
    };
    AnalysisResult {
        check: "perf-ttfb",
        status,
        message,
        recommendation,
    }
}

pub(crate) fn classify_tbt(tbt_ms: f64) -> AnalysisResult {
    if tbt_ms < 0.0 {
        return AnalysisResult {
            check: "perf-tbt",
            status: Status::Fail,
            message: "TBT could not be measured".to_string(),
            recommendation:
                "Ensure longtask PerformanceObserver entries are available in the browser context."
                    .to_string(),
        };
    }
    let (status, message, recommendation) = if tbt_ms <= 200.0 {
        (
            Status::Pass,
            format!("TBT {:.0}ms — good (≤200ms)", tbt_ms),
            "No action needed.".to_string(),
        )
    } else if tbt_ms <= 600.0 {
        (
            Status::Warn,
            format!("TBT {:.0}ms — needs improvement (200ms–600ms)", tbt_ms),
            "Break up long JavaScript tasks into smaller async chunks using setTimeout or requestIdleCallback."
                .to_string(),
        )
    } else {
        (
            Status::Fail,
            format!("TBT {:.0}ms — poor (>600ms)", tbt_ms),
            "Excessive main-thread blocking detected. Defer or split large JavaScript bundles. Target ≤200ms."
                .to_string(),
        )
    };
    AnalysisResult {
        check: "perf-tbt",
        status,
        message,
        recommendation,
    }
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
