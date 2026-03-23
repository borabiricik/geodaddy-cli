---
phase: quick-260323-vxm
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - .github/workflows/ci.yml
  - .github/workflows/release.yml
autonomous: true
requirements: []
must_haves:
  truths:
    - "Pushing a v* tag triggers a GitHub Actions release that builds binaries for all 5 targets"
    - "Every PR and push to main runs tests on Linux, macOS, and Windows"
    - "Release artifacts are packaged as .tar.gz (Unix) and .zip (Windows) and attached to the GitHub release"
    - "CI workflow runs `cargo test -- --skip test_vitals_flag_accepted` to avoid Chromium download in CI"
  artifacts:
    - path: ".github/workflows/ci.yml"
      provides: "CI workflow running tests on PRs and main pushes"
    - path: ".github/workflows/release.yml"
      provides: "Release workflow building and packaging cross-platform binaries on v* tags"
  key_links:
    - from: ".github/workflows/release.yml"
      to: "github.com release page"
      via: "softprops/action-gh-release"
      pattern: "softprops/action-gh-release"
---

<objective>
Set up two GitHub Actions workflows: a CI workflow (tests on every PR/push) and a release workflow (cross-platform builds triggered by v* tags).

Purpose: Enable automated testing and binary distribution for geodaddy across macOS (arm64 + x86_64), Linux (x86_64 + arm64), and Windows (x86_64).
Output: `.github/workflows/ci.yml` and `.github/workflows/release.yml`
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@Cargo.toml
</context>

<tasks>

<task type="auto">
  <name>Task 1: Create CI workflow</name>
  <files>.github/workflows/ci.yml</files>
  <action>
Create `.github/workflows/ci.yml` that runs `cargo test` on every push and pull_request targeting main.

Matrix: ubuntu-latest, macos-latest, windows-latest.

CRITICAL: Skip Chromium-dependent tests using `cargo test -- --skip test_vitals_flag_accepted` (per STATE.md decision: that test is marked `#[ignore]`, but use `--skip` as belt-and-braces). Alternatively, use `cargo test -- --include-ignored=false` pattern. The simplest correct approach: `cargo test` runs fine because `#[ignore]` already excludes it by default. Use plain `cargo test` — no extra flags needed, `#[ignore]` tests are skipped automatically.

Workflow structure:
```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    name: Test (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Run tests
        run: cargo test
```

Use `dtolnay/rust-toolchain@stable` (not the deprecated `actions-rs` actions). Use `Swatinem/rust-cache@v2` for build caching. No special chromiumoxide env vars needed — `#[ignore]` tests are excluded by default.
  </action>
  <verify>
    <automated>test -f /Users/borabiricik/Desktop/Repos/hobby/geodaddy/cli/.github/workflows/ci.yml && echo "ci.yml exists"</automated>
  </verify>
  <done>ci.yml exists with a 3-OS matrix running `cargo test` on push/PR to main</done>
</task>

<task type="auto">
  <name>Task 2: Create release workflow</name>
  <files>.github/workflows/release.yml</files>
  <action>
Create `.github/workflows/release.yml` that triggers on `push: tags: ['v*']`, builds geodaddy for 5 targets, packages artifacts, and uploads them to a GitHub release.

**Target matrix and runner strategy:**

| Target | Runner | Method | Archive |
|--------|--------|--------|---------|
| x86_64-apple-darwin | macos-13 | native cargo | .tar.gz |
| aarch64-apple-darwin | macos-14 | native cargo | .tar.gz |
| x86_64-unknown-linux-musl | ubuntu-latest | cross crate | .tar.gz |
| aarch64-unknown-linux-musl | ubuntu-latest | cross crate | .tar.gz |
| x86_64-pc-windows-msvc | windows-latest | native cargo | .zip |

**Why musl for Linux:** Better portability (static binary, no glibc version dependency). Use `cross` crate for Linux targets to avoid setting up musl toolchain manually.

**Why cross for aarch64-linux:** GitHub free tier has no native arm64 Linux runners. `cross` uses Docker-based cross-compilation.

Workflow structure:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

jobs:
  build:
    name: Build ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - target: x86_64-apple-darwin
            os: macos-13
            archive: tar.gz
          - target: aarch64-apple-darwin
            os: macos-14
            archive: tar.gz
          - target: x86_64-unknown-linux-musl
            os: ubuntu-latest
            archive: tar.gz
            use_cross: true
          - target: aarch64-unknown-linux-musl
            os: ubuntu-latest
            archive: tar.gz
            use_cross: true
          - target: x86_64-pc-windows-msvc
            os: windows-latest
            archive: zip
    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - uses: Swatinem/rust-cache@v2
        with:
          key: ${{ matrix.target }}

      - name: Install cross
        if: matrix.use_cross
        run: cargo install cross --git https://github.com/cross-rs/cross

      - name: Build (cross)
        if: matrix.use_cross
        run: cross build --release --target ${{ matrix.target }}

      - name: Build (native)
        if: ${{ !matrix.use_cross }}
        run: cargo build --release --target ${{ matrix.target }}

      - name: Package (Unix .tar.gz)
        if: matrix.archive == 'tar.gz'
        run: |
          BIN=target/${{ matrix.target }}/release/geodaddy
          ARCHIVE=geodaddy-${{ github.ref_name }}-${{ matrix.target }}.tar.gz
          tar -czf "$ARCHIVE" -C "$(dirname "$BIN")" "$(basename "$BIN")"
          echo "ASSET=$ARCHIVE" >> "$GITHUB_ENV"

      - name: Package (Windows .zip)
        if: matrix.archive == 'zip'
        shell: pwsh
        run: |
          $bin = "target\${{ matrix.target }}\release\geodaddy.exe"
          $archive = "geodaddy-${{ github.ref_name }}-${{ matrix.target }}.zip"
          Compress-Archive -Path $bin -DestinationPath $archive
          "ASSET=$archive" | Out-File -FilePath $env:GITHUB_ENV -Append

      - name: Upload to GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          files: ${{ env.ASSET }}
```

Key details:
- `permissions: contents: write` is required for `softprops/action-gh-release` to create/update releases
- `github.ref_name` resolves to the tag name (e.g. `v0.1.0`) — use in archive filename for clarity
- `Swatinem/rust-cache@v2` with `key: ${{ matrix.target }}` keeps caches separate per target
- Do NOT install `cross` via `cargo install` on non-cross jobs (the `if: matrix.use_cross` guard handles this)
- The release is created automatically by `softprops/action-gh-release` if it does not exist yet
  </action>
  <verify>
    <automated>test -f /Users/borabiricik/Desktop/Repos/hobby/geodaddy/cli/.github/workflows/release.yml && echo "release.yml exists"</automated>
  </verify>
  <done>release.yml exists with a 5-target matrix, cross-compilation for Linux musl targets, platform-appropriate packaging (.tar.gz / .zip), and upload to GitHub Release on v* tag push</done>
</task>

</tasks>

<verification>
- Both workflow files exist under `.github/workflows/`
- ci.yml has a 3-OS matrix (ubuntu-latest, macos-latest, windows-latest) triggered on push/PR to main
- release.yml has a 5-target matrix triggered on v* tags with correct runner assignments
- Linux targets use `cross`, macOS and Windows use native cargo
- Archive formats are correct: .tar.gz for Unix, .zip for Windows
- `softprops/action-gh-release@v2` is used (not v1 — v2 supports newer token permissions)
- `permissions: contents: write` is declared at job or workflow level
</verification>

<success_criteria>
- Pushing a v0.1.0 tag to GitHub triggers 5 parallel build jobs that produce:
  - geodaddy-v0.1.0-x86_64-apple-darwin.tar.gz
  - geodaddy-v0.1.0-aarch64-apple-darwin.tar.gz
  - geodaddy-v0.1.0-x86_64-unknown-linux-musl.tar.gz
  - geodaddy-v0.1.0-aarch64-unknown-linux-musl.tar.gz
  - geodaddy-v0.1.0-x86_64-pc-windows-msvc.zip
- All 5 artifacts are attached to the GitHub release page
- PRs to main run `cargo test` on Linux, macOS, and Windows without downloading Chromium
</success_criteria>

<output>
After completion, create `.planning/quick/260323-vxm-set-up-github-actions-release-pipeline-f/260323-vxm-SUMMARY.md`
</output>
