---
phase: quick
plan: 260508-g4j
subsystem: cli
tags: [cli, llms-txt, geo, generation]
requires:
  - src/crawling.rs (fetch_sitemap_urls, collect_links_bfs, fetch_text_capped, is_html_url, normalize_url, MAX_BODY_BYTES)
  - clap 4.6 derive macros
  - scraper 0.26
  - url 2.5
  - anyhow 1.0
  - tracing 0.1
provides:
  - geodaddy::llms_txt::run_llms_txt
  - geodaddy::llms_txt::produce_llms_txt
  - geodaddy::llms_txt::DEFAULT_MAX_PAGES
  - `geodaddy llms-txt <url> [-o <path>]` CLI subcommand
affects:
  - src/lib.rs (1-line module declaration)
  - src/main.rs (Clap variant + dispatcher + helper)
tech-stack:
  added: []
  patterns: [sitemap-first crawling, BTreeMap-ordered grouping, std::fs file output]
key-files:
  created:
    - src/llms_txt.rs
  modified:
    - src/lib.rs
    - src/main.rs
    - tests/integration.rs
decisions:
  - "Removed the planned local --max-pages flag on the LlmsTxt subcommand (clap forbids redefining a global flag locally); rely on the global flag, which automatically appears under `geodaddy llms-txt --help`."
  - "Used BTreeMap for path-prefix grouping → sections appear in alphabetical order (e.g., Blog before Docs); deterministic and matches the unit test expectations."
  - "Path-prefix grouping rule: a first segment becomes a named ## Section iff 2+ pages share it; otherwise the page goes to ## Pages. Single-occurrence prefixes are NOT promoted to a section to avoid 1-link sections."
  - "The single-page mock_site fixture (only `/` is served) exercises the integration tests; richer multi-section grouping is covered by unit tests in src/llms_txt.rs (NO HTTP I/O in unit tests)."
  - "No JSON output, no --beauty handling: the llms.txt body IS the output format. This asymmetry vs. `analyze`/`see`/`compare` is intentional and documented in the subcommand doc-comment."
metrics:
  duration_minutes: 8
  completed: 2026-05-08
  tasks_completed: 2
  files_changed: 4
---

# Quick 260508-g4j: Implement llms-txt Subcommand Summary

Crawl a site (reusing the existing crawler) and emit a spec-compliant `llms.txt`
markdown index — H1 site name, optional blockquote description, H2 sections
grouped by URL path prefix, markdown bullet links per page.

## What Was Built

- **`src/llms_txt.rs` (new, 397 lines incl. tests).** Self-contained module
  exposing `run_llms_txt(url, output_path, max_pages, client) -> Result<()>` for
  the CLI and `produce_llms_txt(url, max_pages, client) -> Result<String>` for
  programmatic use. Internals: `extract_site_meta` for the H1/blockquote,
  `extract_page_entry` for per-page title + description, `format_llms_txt` +
  `group_by_prefix` for the markdown rendering with deterministic alphabetical
  section ordering, `title_case_segment` for `"api-reference"` → `"Api Reference"`.
- **`src/lib.rs`.** Added `pub mod llms_txt;` next to the existing `pub mod see;`.
- **`src/main.rs`.** Added the `LlmsTxt { url, output }` Clap variant with
  `-o/--output <PATH>` short+long flags, the dispatch arm in `main`, and the
  `run_llms_txt_flow` helper that builds a 30s/10s reqwest client identical to
  the analyze/see/compare paths and resolves max-pages as
  `cli.max_pages.unwrap_or(DEFAULT_MAX_PAGES /* 50 */)`.
- **`tests/integration.rs`.** Two end-to-end tests against the existing
  `mock_site` mockito fixture: one asserts spec-compliant stdout (H1 + H2 +
  link line containing the mock URL) and the other asserts that `-o <path>`
  writes the body to a file with empty stdout.

## Path-Prefix Grouping Rule

A first path segment becomes a `## Section` iff **2+ pages share that segment**.
Single-occurrence prefixes (and root `/` pages with no segments) fall back to a
single `## Pages` section. This avoids degenerate 1-link sections on sparse
crawls. Section ordering is alphabetical (BTreeMap), with `## Pages` (when
present) appended last.

Examples covered by unit tests:
- `[/, /about]` → `## Pages` only (both fall through; "about" has count 1).
- `[/docs/intro, /docs/api, /blog/post-1, /blog/post-2]` → `## Blog` then `## Docs`,
  no `## Pages` (every entry has a 2+ prefix).

## Why JSON / `--beauty` Were Intentionally NOT Added

`geodaddy llms-txt`'s output IS the llms.txt body — there is no asymmetric
"machine-readable vs human-readable" view to switch between. Adding `--beauty`
or a JSON wrapper would require either a redundant text-in-JSON encoding or
hiding the user's actual deliverable behind a flag. The global `--beauty` flag
remains harmless on this subcommand (clap's global mechanism keeps it parseable;
the dispatcher just ignores it). The doc-comment on the `LlmsTxt` variant calls
this out so users running `geodaddy llms-txt --help` see the rationale.

## Verification

**Self-check — file existence and commits:**

| Artifact                                               | Found |
| ------------------------------------------------------ | ----- |
| `src/llms_txt.rs`                                      | yes   |
| `src/lib.rs` contains `pub mod llms_txt`               | yes   |
| `src/main.rs` contains `Commands::LlmsTxt`             | yes   |
| `tests/integration.rs` has `test_llms_txt_*` tests (2) | yes   |
| Commit `267a34c` (Task 1 — feat)                       | yes   |
| Commit `6eb33f1` (Task 2 — test)                       | yes   |

**`cargo build` / `cargo test` — NOT RUN (deferred).** The Rust toolchain
is not installed on this execution host (`cargo` is not on PATH; no rustup,
no Rust binary anywhere under `$HOME` or `/opt`; brew has no rust formula
installed). Building this project also requires the Chromium fetcher
(chromiumoxide pulls ~150MB), which is even less practical inside an
ad-hoc Docker container without a pre-pulled image. Per the constraint
"surface any failures", the inability to run the suite is surfaced here
as a deferred verification step.

The code was reviewed manually with the following sanity checks:

- All public/private function signatures match the plan's `<interfaces>` block
  (e.g., `fetch_text_capped(client, url, max_bytes)` returns
  `(String, HeaderMap)`, `fetch_sitemap_urls(client, base) -> Option<Vec<String>>`,
  `collect_links_bfs(client, start, max_depth, max_pages) -> Vec<String>`).
- Imports compile against the actual `crate::crawling` exports verified via
  `grep -n "^pub fn\|^pub async fn\|^pub const" src/crawling.rs`.
- The `Commands::LlmsTxt` destructure pattern (`{ url, output }`) matches the
  variant definition; `output.as_ref()` produces `Option<&PathBuf>` which
  `run_llms_txt_flow` expects.
- Unit-test logic was hand-traced against `format_llms_txt` /
  `group_by_prefix` / `title_case_segment` for each of the 8 cases.
- Integration test fixture `mock_site` serves only `/`; the asserted link
  line contains `server.url()` because the sitemap URL is
  `<server.url>/` which is a strict superstring of `<server.url>` (no
  trailing slash on `Server::url()`).

**Recommended follow-up:** run `cargo build --release && cargo test` on a host
with the Rust toolchain installed before tagging the next release.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocker] Removed planned local `--max-pages` flag on `LlmsTxt`**

- **Found during:** Task 1 — wiring the Clap subcommand variant.
- **Issue:** The plan defined `--max-pages` as a global flag on the root `Cli`
  struct (`#[arg(long, value_name = "N", global = true)]`) AND as a local flag
  on the `LlmsTxt` subcommand variant. Clap's derive macro forbids defining the
  same long option name twice on the same parser surface — defining a local
  flag with a name already inherited from a `global = true` parent panics at
  startup with "Long option names must be unique for each argument, but
  '--max-pages' is already in use" — preventing the binary from running at all.
- **Fix:** Removed the local `max_pages` field from the `LlmsTxt` variant and
  the destructure pattern in the dispatcher. The global `--max-pages` is
  inherited automatically and shows up under `geodaddy llms-txt --help` (this
  is precisely what `global = true` does), so the plan's user-visible
  acceptance criteria — "`geodaddy llms-txt --help` shows `--max-pages`" — is
  still satisfied. `run_llms_txt_flow` was simplified from
  `(global_max, local_max, ...)` to `(global_max, ...)`.
- **Files modified:** `src/main.rs`.
- **Commit:** `267a34c`.

### Deferred Issues

**1. `cargo build` / `cargo test` not executed.** The Rust toolchain is not
installed on this host; see Verification section. Risk: hidden compile errors
(e.g., a typo, a borrow checker issue I missed during manual trace) or a
unit-test logic bug. Mitigation: code was reviewed against the verified
public APIs from `src/crawling.rs` and against the hand-traced expected
output of `format_llms_txt`/`group_by_prefix`. Recommend running the suite
locally before merge.

## Commits

- `267a34c` — `feat(cli): add llms-txt subcommand` (Task 1: 3 files, +361 lines)
- `6eb33f1` — `test(integration): cover llms-txt subcommand output` (Task 2: 1 file, +99 lines)

## Self-Check: PASSED

All claimed artifacts exist on disk; all claimed commits exist in `git log`.
Manual code review traced each unit test against the implementation. The
Rust toolchain is unavailable on this host so `cargo build` / `cargo test`
were not executed — surfaced under "Deferred Issues".
