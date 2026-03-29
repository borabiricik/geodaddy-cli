---
phase: 07
slug: research-and-implement-missing-geo-metrics-llms-txt-ai-crawler-directives-citation-signals-entity-coverage-conversational-query-optimization
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-03-29
---

# Phase 07 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in Rust test framework) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 07-01-01 | 01 | 1 | D-05, D-06 | unit | `cargo test --lib geo_llms` | ❌ W0 | ⬜ pending |
| 07-01-02 | 01 | 1 | D-07, D-08 | unit | `cargo test --lib geo_directives` | ❌ W0 | ⬜ pending |
| 07-02-01 | 02 | 1 | D-09, D-10 | unit | `cargo test --lib geo_citations` | ❌ W0 | ⬜ pending |
| 07-03-01 | 03 | 1 | D-11 | unit | `cargo test --lib geo_entities` | ❌ W0 | ⬜ pending |
| 07-04-01 | 04 | 1 | D-12 | unit | `cargo test --lib geo_query` | ❌ W0 | ⬜ pending |
| 07-05-01 | 05 | 2 | D-13-D-17 | unit | `cargo test --lib geo_freshness` | ❌ W0 | ⬜ pending |
| 07-06-01 | 06 | 2 | All | integration | `cargo test --test integration` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `src/analyzers/geo_llms.rs` — test stubs for llms.txt detection
- [ ] `src/analyzers/geo_directives.rs` — test stubs for AI directive meta tags/headers
- [ ] `src/analyzers/geo_citations.rs` — test stubs for citation signal detection
- [ ] `src/analyzers/geo_entities.rs` — test stubs for entity coverage checks
- [ ] `src/analyzers/geo_query.rs` — test stubs for conversational query optimization
- [ ] `src/analyzers/geo_freshness.rs` — test stubs for freshness signals + FAQ quality + HowTo schema

*Existing test infrastructure (cargo test) covers all framework needs.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| llms.txt fetch from live site | D-05 | Requires network access | Run `geodaddy https://example.com` and verify llms.txt check in output |

*All other phase behaviors have automated verification via unit tests with HTML fixtures.*

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
