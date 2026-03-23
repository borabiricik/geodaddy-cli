---
phase: quick-260323-vxm
plan: "01"
subsystem: ci-cd
tags: [github-actions, ci, release, cross-compilation, rust]
dependency_graph:
  requires: []
  provides: [ci-workflow, release-workflow]
  affects: []
tech_stack:
  added:
    - dtolnay/rust-toolchain@stable
    - Swatinem/rust-cache@v2
    - softprops/action-gh-release@v2
    - cross (cargo install from cross-rs/cross)
  patterns:
    - matrix strategy with include for heterogeneous build targets
    - use_cross flag per matrix row to switch between cross and native cargo
key_files:
  created:
    - .github/workflows/ci.yml
    - .github/workflows/release.yml
  modified: []
decisions:
  - "Use cross crate (not musl toolchain setup) for Linux musl targets — simpler, no manual toolchain installation"
  - "macos-13 for x86_64-apple-darwin, macos-14 for aarch64-apple-darwin — matches GitHub runner hardware"
  - "softprops/action-gh-release@v2 (not v1) — v2 supports newer token permission model"
  - "permissions: contents: write at workflow level — required for release creation/upload"
  - "Plain cargo test in CI — #[ignore] on test_vitals_flag_accepted already excludes Chromium-dependent test"
metrics:
  duration_minutes: 2
  completed_date: "2026-03-23"
  tasks_completed: 2
  files_created: 2
  files_modified: 0
---

# Phase quick-260323-vxm Plan 01: GitHub Actions CI and Release Pipeline Summary

**One-liner:** Two-workflow GitHub Actions setup — 3-OS CI on push/PR and 5-target cross-platform release pipeline on v* tags using cross crate for Linux musl builds.

## What Was Built

### CI Workflow (`.github/workflows/ci.yml`)

Runs `cargo test` on every push to main and every pull request targeting main. Matrix covers ubuntu-latest, macos-latest, and windows-latest. Uses `dtolnay/rust-toolchain@stable` and `Swatinem/rust-cache@v2` for caching. No special flags needed — `#[ignore]` on `test_vitals_flag_accepted` already prevents Chromium download in CI.

### Release Workflow (`.github/workflows/release.yml`)

Triggers on `push: tags: v*`. Builds 5 targets in parallel:

| Target | Runner | Method | Archive |
|--------|--------|--------|---------|
| x86_64-apple-darwin | macos-13 | native cargo | .tar.gz |
| aarch64-apple-darwin | macos-14 | native cargo | .tar.gz |
| x86_64-unknown-linux-musl | ubuntu-latest | cross crate | .tar.gz |
| aarch64-unknown-linux-musl | ubuntu-latest | cross crate | .tar.gz |
| x86_64-pc-windows-msvc | windows-latest | native cargo | .zip |

Archive naming: `geodaddy-{version}-{target}.tar.gz` / `.zip` (e.g. `geodaddy-v0.1.0-x86_64-apple-darwin.tar.gz`). Unix tarballs contain the `geodaddy` binary; Windows zip contains `geodaddy.exe`. All artifacts uploaded to GitHub Release via `softprops/action-gh-release@v2`.

## Decisions Made

1. **cross crate for Linux musl** — avoids manual musl toolchain setup; Docker-based cross-compilation is more reliable in CI
2. **macos-13 / macos-14 runner split** — GitHub's macos-13 is x86_64 hardware; macos-14 is arm64 (M1), enabling native compilation for both Apple targets
3. **softprops/action-gh-release@v2** — v2 is required for the newer `permissions: contents: write` token model
4. **Plain `cargo test`** — `#[ignore]` attribute on `test_vitals_flag_accepted` is sufficient; no extra `--skip` flags needed

## Deviations from Plan

None — plan executed exactly as written.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create CI workflow | 0720648 | .github/workflows/ci.yml |
| 2 | Create release workflow | c92fe5b | .github/workflows/release.yml |

## Known Stubs

None.

## Self-Check: PASSED

- .github/workflows/ci.yml: FOUND
- .github/workflows/release.yml: FOUND
- commit 0720648: FOUND
- commit c92fe5b: FOUND
