# Phase 2: Core Analysis Engine - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-23
**Phase:** 02-core-analysis-engine
**Areas discussed:** Result item shape, Scoring formula, Code architecture, Broken link check scope

---

## Result Item Shape

**Question:** What should a single metric result look like in results[]?

**Options presented:**
- A) Minimal — `{ check, status, message }`
- B) Structured with recommendation — `{ check, status, message, recommendation }`
- C) Rich/machine-readable — `{ check, status, message, recommendation, details: {} }`

**Selected:** B — structured with recommendation

**Rationale:** Satisfies SCORE-04 without over-engineering. No `details` object in v1.

---

## Scoring Formula

**Question:** How do pass/fail/warn counts convert to a 0-100 score?

**Options presented:**
- A) Simple pass ratio — equal weight per check
- B) Weighted by category — fixed category weights (e.g., 40/40/20)
- C) Severity-weighted — each check has critical/warning/info severity

**Selected:** C — severity-weighted

**Follow-up:** Does `warn` behave differently from `fail`?
**Answer:** Yes — `fail` loses full points, `warn` loses half points.

---

## Code Architecture

**Question:** How should analyzer code be organized as phase 2 adds 12+ checks?

**Options presented:**
- A) Single growing main.rs
- B) Flat modules — analyzers/technical.rs, analyzers/content.rs, scoring.rs
- C) Trait-based Analyzer system

**Selected:** B — flat modules, no trait abstraction

---

## Broken Link Check Scope

**Question:** What should TECH-01 broken link detection cover in a single-URL crawl?

**Options presented:**
- A) All links on the page — fire async HEAD requests to every `<a href>`
- B) Internal links only — same-origin links only
- C) Defer to phase 4 — stub result in phase 2, full implementation with site-wide crawl

**Selected:** C — defer to phase 4, emit a warn stub in phase 2

---

*Log generated: 2026-03-23*
