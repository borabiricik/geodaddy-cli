---
status: partial
phase: 05-core-web-vitals-measurement-lcp-fcp-cls-ttfb-tbt-and-performance-metrics-analyzer
source: [05-VERIFICATION.md]
started: 2026-03-23T00:00:00.000Z
updated: 2026-03-23T00:00:00.000Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Live --vitals measurement
expected: Run `cargo build --release && ./target/release/geodaddy https://example.com --vitals` — JSON output contains `"performance"` key with non-null value in `categories`, and `results` array includes 5 entries with check IDs `perf-lcp`, `perf-fcp`, `perf-cls`, `perf-ttfb`, `perf-tbt`. Requires Chromium (~150MB download on first use).
result: [pending]

### 2. --vitals + --enable-js combined
expected: Run `./target/release/geodaddy https://example.com --vitals --enable-js` — no crash, no panic. Two independent chromiumoxide browser instances can coexist (or the implementation reuses one gracefully).
result: [pending]

### 3. REQUIREMENTS.md back-fill
expected: PERF-01 through PERF-08 are referenced in PLAN frontmatter and ROADMAP.md but absent from `.planning/REQUIREMENTS.md`. Either add the 8 requirement definitions to REQUIREMENTS.md, or document that Phase 5 requirements are captured only in CONTEXT.md (D-01 through D-16). This is a documentation gap, not an implementation gap.
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
