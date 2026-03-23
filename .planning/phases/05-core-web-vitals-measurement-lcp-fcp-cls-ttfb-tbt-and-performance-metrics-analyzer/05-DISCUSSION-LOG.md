# Phase 5: Core Web Vitals Measurement - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-23
**Phase:** 05-core-web-vitals-measurement-lcp-fcp-cls-ttfb-tbt-and-performance-metrics-analyzer
**Areas discussed:** Triggering mechanism, Scoring category, Metrics & severity, Thresholds

---

## Triggering Mechanism

### How should Core Web Vitals measurement be triggered?

| Option | Description | Selected |
|--------|-------------|----------|
| New --vitals flag | Separate opt-in flag. Default stays fast (reqwest-only). --enable-js and --vitals independent. | ✓ |
| Fold into --enable-js | When --enable-js passed, also capture vitals. Simpler CLI surface. | |
| Always measure | Every run captures CWV via chromiumoxide. No flag, but every crawl is slower. | |

**User's choice:** New --vitals flag
**Notes:** None

### Should --vitals imply --enable-js automatically?

| Option | Description | Selected |
|--------|-------------|----------|
| Independent | --vitals measures via CDP; --enable-js controls JS fallback rendering. Neither implies the other. | ✓ |
| --vitals implies --enable-js | Already launching Chromium, so --enable-js activates automatically. | |

**User's choice:** Independent
**Notes:** Both flags can be combined but remain explicit and separate.

---

## Scoring Category

### New 'performance' category vs. fold into 'technical'?

| Option | Description | Selected |
|--------|-------------|----------|
| New 'performance' category | Add perf: f64 to CategoryScores. Overall = (tech + cont + geo + perf) / 4. | ✓ |
| Fold into 'technical' | perf- prefix routes to tech category. No schema change. | |

**User's choice:** New 'performance' category
**Notes:** None

### When --vitals is NOT passed, how should performance appear in JSON?

| Option | Description | Selected |
|--------|-------------|----------|
| Omit from JSON entirely | performance key absent when not measured. | |
| Always present, null | performance: null when --vitals not passed. Consistent schema. | ✓ |
| Always present, 100.0 | Consistent with geo pattern but may mislead. | |

**User's choice:** Always present, null when not measured
**Notes:** Keeps schema shape consistent; null clearly signals "not measured."

---

## Metrics & Severity

### Which metrics to include?

| Option | Description | Selected |
|--------|-------------|----------|
| LCP — Largest Contentful Paint | Google's primary UX signal. Critical severity. | ✓ |
| FCP — First Contentful Paint | Visibility signal. Warning severity. | ✓ |
| CLS — Cumulative Layout Shift | Visual stability. Warning severity. | ✓ |
| TTFB — Time to First Byte | Server response time. Warning severity. | ✓ |

**User's choice:** All four above selected.
**Notes:** TBT also included (separate question).

### Include TBT (Total Blocking Time)?

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — include TBT | Measures main thread blocking. Warning severity. | ✓ |
| No — skip TBT | Can be added later; CDP extraction is trickier. | |

**User's choice:** Yes — include TBT
**Notes:** All 5 metrics included (LCP, FCP, CLS, TTFB, TBT).

### Severity assignment in severity_points()?

| Option | Description | Selected |
|--------|-------------|----------|
| LCP critical (10pts), rest warning (5pts) | LCP is primary signal; others secondary. Mirrors tech-meta-title tiering. | ✓ |
| All critical (10pts each) | Treats all metrics equally. More punishing. | |
| All warning (5pts each) | Performance metrics are informational. More lenient. | |

**User's choice:** LCP critical (10pts), rest warning (5pts)
**Notes:** None

---

## Thresholds

### Which threshold standard to use?

| Option | Description | Selected |
|--------|-------------|----------|
| Google's official CWV thresholds | LCP ≤2.5s/≤4s/>4s, FCP ≤1.8s/≤3s/>3s, CLS ≤0.1/≤0.25/>0.25, TTFB ≤800ms/≤1800ms/>1800ms, TBT ≤200ms/≤600ms/>600ms | ✓ |
| Stricter thresholds | LCP ≤1.5s pass, FCP ≤1s pass. Best-in-class targeting. | |

**User's choice:** Google's official CWV thresholds
**Notes:** Standard industry thresholds, well-known to developers.

### Per-page measurement in multi-page crawl?

| Option | Description | Selected |
|--------|-------------|----------|
| Measure each page independently | Each page in pages[] gets own CWV. Aggregate = average. | ✓ |
| Measure start URL only | Faster but misleading for multi-page reports. | |

**User's choice:** Measure each page independently
**Notes:** None

---

## Claude's Discretion

- TBT extraction method via CDP (PerformanceTiming vs longtask entries)
- Browser instance reuse when --vitals and --enable-js both active
- Measurement failure handling (timeout/CDP error)
- Order of vitals measurement relative to HTML extraction

## Deferred Ideas

None — discussion stayed within phase scope.
