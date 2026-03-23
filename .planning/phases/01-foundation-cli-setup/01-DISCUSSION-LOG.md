# Phase 1: Foundation & CLI Setup - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-03-23
**Phase:** 01-foundation-cli-setup
**Areas discussed:** CLI command structure, JSON report schema shape, robots.txt enforcement mode, Project scaffold structure

---

## CLI Command Structure

**Question:** How should users invoke geodaddy?

**Options presented:**
- A) Flat interface — `geodaddy <url>`
- B) Subcommand scaffold — `geodaddy analyze <url>`
- C) Subcommand aliased — canonical subcommand + shorthand

**Selected:** A — Flat interface

---

## JSON Report Schema Shape

**Question:** What should the phase-1 JSON wrapper look like?

**Options presented:**
- A) Minimal envelope — `schema_version`, `url`, `crawled_at`, empty `results: []`
- B) Full shape with stubs — top-level includes `score: null`, `categories: {}`
- C) Page-centric structure — top-level wraps `pages: [{ url, results: [] }]` array

**Selected:** C — Page-centric structure

**Rationale:** Phase 4 adds multi-page crawling. Starting page-centric avoids schema changes later. Single-URL output returns `pages` with one item.

---

## robots.txt Enforcement Mode

**Question:** When robots.txt disallows the target URL, what should happen?

**Options presented:**
- A) Hard fail — non-zero exit, error to stderr, no JSON output
- B) Soft warn — crawl proceeds, `robots_blocked: true` in JSON at page level
- C) Flag-gated override — default hard fail, `--ignore-robots` to bypass

**Selected:** B — Soft warn

**Rationale:** Non-blocking behavior works better for CI/CD pipelines and local dev. User still sees the signal in JSON output.

---

## Project Scaffold Structure

**Question:** Cargo workspace from day 1, or single crate at root?

**Options presented:**
- A) Cargo workspace — root `Cargo.toml` as workspace, `cli/` as member crate
- B) Single crate at root for now — restructure later when needed

**User input:** web/backend stack is TBD and will not be Rust — Cargo workspace adds no value.

**Selected:** B variant — Single Rust crate in `cli/` directory (not at root, not a workspace member). `web/` will be a separate non-Rust project alongside it.

---

*Log generated: 2026-03-23*
